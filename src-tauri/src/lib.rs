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

use core::app_state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            ipc::commands::is_project_open,
            ipc::commands::import_stems,
            ipc::commands::set_markers,
            ipc::commands::play,
            ipc::commands::pause,
            ipc::commands::stop,
            ipc::commands::set_stem_volume,
            ipc::commands::set_stem_mute,
            ipc::commands::set_stem_solo,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the StemLoft application");
}
