//! `Loop & Marker Manager` -- holds the start/end marker for the looped
//! section and feeds it to the `Audio Engine`. See the "Playback with
//! marker loop" flow in docs/architecture.md and
//! docs/architecture.md#consistency-across-a-projects-stems: a marker past
//! the project's duration is rejected, never silently clamped.

use std::sync::{Mutex, MutexGuard};

use super::persistence::LoopRange;

pub struct LoopManager {
    // Its own small lock, not the giant lock some other design would put
    // around the whole app: `Audio Engine` reads these bounds independently
    // of whatever calls into `Session` (see
    // docs/architecture.md#design-note-concurrency-model-for-shared-state --
    // "a single Arc<Mutex<AppState>> (or at most one Mutex per module)").
    // `None` until a loop is set or restored. The `cpal` callback never
    // reads this lock: the engine copies the bounds out ahead of time.
    markers: Mutex<Option<LoopRange>>,
}

impl LoopManager {
    pub fn new() -> Self {
        Self {
            markers: Mutex::new(None),
        }
    }

    /// Plain data, valid after any panic: a poisoned lock is safe to keep
    /// using.
    fn lock(&self) -> MutexGuard<'_, Option<LoopRange>> {
        self.markers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The active loop, for `Audio Engine` to play between: `None` when no
    /// loop is set.
    pub fn markers(&self) -> Option<LoopRange> {
        *self.lock()
    }

    /// Sets the loop to `start_sec..end_sec`. `project_duration_sec` is the
    /// project's duration (its longest stem's `durationSec`). Rejected,
    /// leaving the current loop unchanged, with an explicit message when:
    /// either bound isn't a finite number; `start_sec` is negative;
    /// `start_sec` isn't before `end_sec`; or `end_sec` is past
    /// `project_duration_sec` -- that message names both `end_sec` and the
    /// duration. Never clamps.
    pub fn set_markers(
        &self,
        start_sec: f64,
        end_sec: f64,
        project_duration_sec: f64,
    ) -> Result<(), String> {
        todo!("validate, then store Some(LoopRange)")
    }

    /// Puts back a loop saved in the project (validated when it was set),
    /// or clears it with `None` -- on opening, creating or closing a
    /// project.
    pub fn restore(&self, markers: Option<LoopRange>) {
        todo!("replace the stored loop as is")
    }
}

impl Default for LoopManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    // The contract for TASK-004 (openspec/changes/task-004-loop-marker-manager).
    // Written before the implementation, by a different author than it --
    // never loosen one of these to make it pass (AGENTS.md, testing policy).
    use super::*;

    fn range(start_sec: f64, end_sec: f64) -> Option<LoopRange> {
        Some(LoopRange { start_sec, end_sec })
    }

    #[test]
    fn no_loop_until_one_is_set() {
        assert_eq!(LoopManager::new().markers(), None);
    }

    #[test]
    fn markers_within_the_duration_are_accepted_and_exposed() {
        let manager = LoopManager::new();

        assert_eq!(manager.set_markers(1.5, 8.25, 187.42), Ok(()));
        assert_eq!(manager.markers(), range(1.5, 8.25));

        // Up to the very end, and from the very start, is fine.
        assert_eq!(manager.set_markers(0.0, 187.42, 187.42), Ok(()));
        assert_eq!(manager.markers(), range(0.0, 187.42));
    }

    #[test]
    fn an_end_past_the_duration_is_rejected_not_clamped() {
        let manager = LoopManager::new();
        manager.set_markers(1.0, 2.0, 10.0).unwrap();

        let err = manager.set_markers(5.0, 10.5, 10.0).unwrap_err();

        assert!(err.contains("10.5") && err.contains("10"), "{err}");
        assert_eq!(manager.markers(), range(1.0, 2.0), "unchanged, not clamped");
    }

    #[test]
    fn malformed_ranges_are_rejected() {
        let manager = LoopManager::new();
        for (start, end) in [
            (3.0, 3.0),
            (4.0, 3.0),
            (-0.5, 3.0),
            (f64::NAN, 3.0),
            (0.0, f64::NAN),
            (0.0, f64::INFINITY),
            (f64::NEG_INFINITY, 1.0),
        ] {
            assert!(
                manager.set_markers(start, end, 10.0).is_err(),
                "accepted {start}..{end}"
            );
        }
        assert_eq!(manager.markers(), None);
    }

    #[test]
    fn a_project_with_no_stems_has_no_room_for_a_loop() {
        assert!(LoopManager::new().set_markers(0.0, 1.0, 0.0).is_err());
    }

    #[test]
    fn restore_puts_back_or_clears_the_loop_without_validating() {
        let manager = LoopManager::new();

        manager.restore(range(2.0, 4.0));
        assert_eq!(manager.markers(), range(2.0, 4.0));

        manager.restore(None);
        assert_eq!(manager.markers(), None);
    }
}
