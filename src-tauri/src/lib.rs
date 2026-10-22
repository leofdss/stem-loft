//! StemLoft's Rust core, wired into a Tauri v2 application. See
//! docs/architecture.md for the full design; this file is the composition
//! root -- it builds the `AppState` (whose `new` assembles every domain
//! module by hand, see `core::app_state`) and registers the `Commands`
//! Tauri exposes to the Angular WebView.
//!
//! `#[cfg_attr(mobile, tauri::mobile_entry_point)]` is what lets this same
//! `run()` also be the entry point on Android -- see
//! docs/architecture.md#design-note-audio-crates-cross-platform for the
//! `cpal`/Oboe wiring point that will matter there.

mod core;
mod ipc;

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use core::app_state::AppState;
use ipc::events::TauriEvents;
use tauri::Manager;

/// How often the background ticker asks `Project Persistence` whether a
/// dirty project is due for its debounced write.
const CHECKPOINT_TICK: Duration = Duration::from_millis(100);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Built here, not before the builder: the core publishes its
            // Events through the app's handle.
            let events = Arc::new(TauriEvents::new(app.handle().clone()));
            app.manage(AppState::new(events));

            // The debounced checkpoint: a background task that touches
            // Session's state through its lock, never through the Commands
            // queue (docs/architecture.md, concurrency model).
            let handle = app.handle().clone();
            thread::spawn(move || loop {
                thread::sleep(CHECKPOINT_TICK);
                let state = handle.state::<AppState>();
                if let Err(e) = state.session.checkpoint(Instant::now(), false) {
                    eprintln!("stemloft: saving the project failed, will retry: {e}");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::commands::is_project_open,
            ipc::commands::create_project,
            ipc::commands::open_project,
            ipc::commands::close_project,
            ipc::commands::import_stems,
            ipc::commands::set_markers,
            ipc::commands::play,
            ipc::commands::pause,
            ipc::commands::stop,
            ipc::commands::set_stem_volume,
            ipc::commands::set_stem_mute,
            ipc::commands::set_stem_solo,
        ])
        .build(tauri::generate_context!())
        .expect("error while building the StemLoft application")
        .run(|app, event| {
            // Quitting is a definitive event, like closing the project.
            if let tauri::RunEvent::Exit = event {
                let state = app.state::<AppState>();
                // Through the queue, so it runs after any Command still in
                // flight and saves what that Command changed.
                let saved = state
                    .queue
                    .run(|| state.session.checkpoint(Instant::now(), true));
                if let Err(e) = saved {
                    eprintln!("stemloft: saving the project on exit failed: {e}");
                }
            }
        });
}
