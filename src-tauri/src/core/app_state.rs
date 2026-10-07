//! `AppState` -- the one object the app hands to the IPC layer, assembled
//! once at startup. See
//! docs/architecture.md#design-note-concurrency-model-for-shared-state.
//!
//! Of the two shapes that design note allows -- a single
//! `Arc<Mutex<AppState>>`, or at most one `Mutex` per module -- this uses
//! the second: `AppState` itself is never locked (it only holds the wiring,
//! which never changes after `new`), and each module keeps its own mutable
//! state behind its own single `Mutex` (`Session.open_project`,
//! `LoopManager.markers`, ...). No channels, no actor. A `Command` handler
//! does `lock() → change → release` on the module it's dispatched to, and
//! the `cpal` real-time thread never touches any of these locks (see the
//! `realtime-audio-safety` skill).

use std::sync::Arc;

use super::audio_engine::AudioEngine;
use super::loop_manager::LoopManager;
use super::persistence::ProjectPersistence;
use super::score_metadata::ScoreMetadataManager;
use super::session::Session;
use super::stem_importer::StemImporter;

pub struct AppState {
    pub session: Session,
}

impl AppState {
    /// Manual dependency injection: each domain module is constructed
    /// explicitly and handed only the `Arc`s it needs, once, here -- the
    /// same mental model as a Nest.js module's providers array, minus the
    /// container.
    pub fn new() -> Self {
        let persistence = Arc::new(ProjectPersistence::new());
        let stem_importer = Arc::new(StemImporter::new(Arc::clone(&persistence)));
        let loop_manager = Arc::new(LoopManager::new());
        let audio_engine = Arc::new(AudioEngine::new(Arc::clone(&loop_manager)));
        let score_metadata = Arc::new(ScoreMetadataManager::new());

        let session = Session::new(
            stem_importer,
            loop_manager,
            audio_engine,
            persistence,
            score_metadata,
        );

        Self { session }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_app_state_starts_with_no_project_open() {
        assert!(!AppState::new().session.is_project_open());
    }
}
