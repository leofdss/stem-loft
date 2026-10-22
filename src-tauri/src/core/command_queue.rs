//! The `Commands` queue -- see
//! docs/architecture.md#design-note-commands-queue-in-the-rust-core. Every
//! `Command` the UI sends runs through [`CommandQueue::run`]: one at a
//! time, in the order they reached the core (FIFO), whichever thread each
//! one arrives on. Angular never queues, orders or dedups anything; this is
//! where that guarantee lives.
//!
//! It's a ticket lock, not a worker thread fed by a channel: each call
//! takes the next ticket on arrival and waits until it's that ticket's turn
//! -- the "one `Mutex`, lock, change, release" model of
//! docs/architecture.md#design-note-concurrency-model-for-shared-state,
//! plus fairness. A plain `Mutex` would serialize too, but doesn't promise
//! that the first one to wait is the first one in.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard};

pub struct CommandQueue {
    /// The ticket the next arriving `Command` takes.
    next_ticket: AtomicU64,
    /// The ticket allowed to run now.
    serving: Mutex<u64>,
    turn: Condvar,
}

impl CommandQueue {
    pub fn new() -> Self {
        CommandQueue {
            next_ticket: AtomicU64::new(0),
            serving: Mutex::new(0),
            turn: Condvar::new(),
        }
    }

    /// Runs `command` once every `Command` that arrived before it has
    /// finished, and returns its result. The next one is let in even if
    /// `command` panics.
    pub fn run<T>(&self, command: impl FnOnce() -> T) -> T {
        let ticket = self.next_ticket.fetch_add(1, Ordering::SeqCst);
        let mut serving = self.serving();
        while *serving != ticket {
            serving = self
                .turn
                .wait(serving)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        drop(serving);

        // Hands the turn to the next ticket on the way out, panic or not.
        struct NextTurn<'a>(&'a CommandQueue);
        impl Drop for NextTurn<'_> {
            fn drop(&mut self) {
                *self.0.serving() += 1;
                self.0.turn.notify_all();
            }
        }
        let _next = NextTurn(self);
        command()
    }

    /// Only holds a counter, valid after any panic: a poisoned lock is safe
    /// to keep using.
    fn serving(&self) -> MutexGuard<'_, u64> {
        self.serving
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for CommandQueue {
    fn default() -> Self {
        CommandQueue::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    /// Blocks until `n` tickets have been handed out, so a test controls
    /// the exact arrival order of its threads.
    fn wait_for_tickets(queue: &CommandQueue, n: u64) {
        while queue.next_ticket.load(Ordering::SeqCst) < n {
            thread::yield_now();
        }
    }

    #[test]
    fn returns_the_commands_result() {
        assert_eq!(CommandQueue::new().run(|| 41 + 1), 42);
    }

    #[test]
    fn commands_run_in_arrival_order_even_when_threads_start_late() {
        let queue = Arc::new(CommandQueue::new());
        let order = Arc::new(Mutex::new(Vec::new()));
        let (release, gate) = std::sync::mpsc::channel::<()>();

        // The first command holds the queue until every other one queued up.
        let first = {
            let (queue, order) = (Arc::clone(&queue), Arc::clone(&order));
            thread::spawn(move || {
                queue.run(|| {
                    gate.recv().unwrap();
                    order.lock().unwrap().push(0);
                })
            })
        };
        wait_for_tickets(&queue, 1);

        let mut handles = Vec::new();
        for i in 1..=5 {
            let (worker_queue, order) = (Arc::clone(&queue), Arc::clone(&order));
            handles.push(thread::spawn(move || {
                worker_queue.run(|| order.lock().unwrap().push(i))
            }));
            wait_for_tickets(&queue, i as u64 + 1);
        }
        release.send(()).unwrap();
        first.join().unwrap();
        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(*order.lock().unwrap(), vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn never_two_commands_at_once() {
        let queue = Arc::new(CommandQueue::new());
        let running = Arc::new(AtomicUsize::new(0));
        let overlaps = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..8)
            .map(|_| {
                let (queue, running, overlaps) = (
                    Arc::clone(&queue),
                    Arc::clone(&running),
                    Arc::clone(&overlaps),
                );
                thread::spawn(move || {
                    for _ in 0..20 {
                        queue.run(|| {
                            if running.fetch_add(1, Ordering::SeqCst) != 0 {
                                overlaps.fetch_add(1, Ordering::SeqCst);
                            }
                            thread::sleep(Duration::from_micros(50));
                            running.fetch_sub(1, Ordering::SeqCst);
                        });
                    }
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(overlaps.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn a_panicking_command_does_not_block_the_next_one() {
        let queue = CommandQueue::new();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            queue.run(|| panic!("handler bug"))
        }));

        assert!(result.is_err());
        assert_eq!(queue.run(|| "still serving"), "still serving");
    }
}
