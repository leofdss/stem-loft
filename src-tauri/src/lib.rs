//! StemLoft's Rust core, wired into a Tauri v2 application. See
//! docs/architecture.md for the full design; this file is the composition
//! root -- it assembles every domain module by hand (manual dependency
//! injection, no DI container, see the `rust-domain-module` skill) and
//! registers the `Commands` Tauri exposes to the Angular WebView.
//!
//! `#[cfg_attr(mobile, tauri::mobile_entry_point)]` is what lets this same
//! `run()` also be the entry point on Android -- see
//! docs/architecture.md#design-note-audio-crates-cross-platform for the
//! `cpal`/Oboe wiring point that will matter there.

mod core;
mod ipc;

use std::sync::Arc;

use core::audio_engine::AudioEngine;
use core::loop_manager::LoopManager;
use core::persistence::ProjectPersistence;
use core::score_metadata::ScoreMetadataManager;
use core::session::Session;
use core::stem_importer::StemImporter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(build_session())
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

/// Manual dependency injection: each domain module is constructed
/// explicitly and handed only the `Arc`s it needs, once, here -- the same
/// mental model as a Nest.js module's providers array, minus the container.
fn build_session() -> Session {
    let persistence = Arc::new(ProjectPersistence::new());
    let stem_importer = Arc::new(StemImporter::new(Arc::clone(&persistence)));
    let loop_manager = Arc::new(LoopManager::new());
    let audio_engine = Arc::new(AudioEngine::new(Arc::clone(&loop_manager)));
    let score_metadata = Arc::new(ScoreMetadataManager::new());

    Session::new(
        stem_importer,
        loop_manager,
        audio_engine,
        persistence,
        score_metadata,
    )
}
