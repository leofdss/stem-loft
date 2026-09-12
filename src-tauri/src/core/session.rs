//! `Session/State Manager` -- the thin router. It holds only which project
//! is currently open and dispatches each `Command` to the domain module
//! that owns it; it never accumulates business rules of its own. See
//! docs/architecture.md#design-note-session-as-a-thin-router.
//!
//! If you're adding a handler here that does more than forward to a domain
//! module -- coordinating more than one module with its own logic -- that
//! logic belongs in a new module, not here. See the `add-ipc-contract`
//! skill before wiring a new `Command`.
//!
//! TODO(scaffold): the `Commands` queue itself
//! (docs/architecture.md#design-note-commands-queue-in-the-rust-core) isn't
//! implemented yet -- Tauri currently dispatches each invoked command
//! directly, concurrently. That's why this struct still needs its own
//! small lock around `open_project`, and why the domain modules below hold
//! their own locks too: nothing yet serializes these calls FIFO the way
//! the design note describes.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::audio_engine::AudioEngine;
use super::loop_manager::LoopManager;
use super::persistence::ProjectPersistence;
use super::score_metadata::ScoreMetadataManager;
use super::stem_importer::StemImporter;

/// The only state `Session` owns directly: which project is open right now.
#[derive(Default)]
struct OpenProject {
    path: Option<PathBuf>,
}

pub struct Session {
    stem_importer: Arc<StemImporter>,
    loop_manager: Arc<LoopManager>,
    audio_engine: Arc<AudioEngine>,
    // Not dispatched to by any Command yet (no handler needs to load/save a
    // project or a score before Stem Importer/Audio Engine have something
    // real to persist) -- kept here because Session already needs to hold
    // the reference once project load/save is wired in.
    #[allow(dead_code)]
    persistence: Arc<ProjectPersistence>,
    #[allow(dead_code)]
    score_metadata: Arc<ScoreMetadataManager>,
    open_project: Mutex<OpenProject>,
}

impl Session {
    /// Manual dependency injection: every `Arc` is constructed by the
    /// caller (see `run()` in `lib.rs`) and handed in explicitly here, once
    /// -- no container, no service locator.
    pub fn new(
        stem_importer: Arc<StemImporter>,
        loop_manager: Arc<LoopManager>,
        audio_engine: Arc<AudioEngine>,
        persistence: Arc<ProjectPersistence>,
        score_metadata: Arc<ScoreMetadataManager>,
    ) -> Self {
        Self {
            stem_importer,
            loop_manager,
            audio_engine,
            persistence,
            score_metadata,
            open_project: Mutex::new(OpenProject::default()),
        }
    }

    pub fn is_project_open(&self) -> bool {
        self.open_project
            .lock()
            .expect("Session.open_project mutex poisoned")
            .path
            .is_some()
    }

    // --- Dispatch to the owning module. Each of these only forwards --
    // see the module doc comment above for why nothing else belongs here. ---

    pub fn import_stems(&self, file_paths: Vec<PathBuf>) -> Result<(), String> {
        self.stem_importer.import_stems(file_paths)
    }

    pub fn set_markers(&self, start_sec: f64, end_sec: f64) -> Result<(), String> {
        self.loop_manager.set_markers(start_sec, end_sec)
    }

    pub fn play(&self) -> Result<(), String> {
        self.audio_engine.play()
    }

    pub fn pause(&self) -> Result<(), String> {
        self.audio_engine.pause()
    }

    pub fn stop(&self) -> Result<(), String> {
        self.audio_engine.stop()
    }

    pub fn set_stem_volume(&self, stem_id: &str, volume: f32) -> Result<(), String> {
        self.audio_engine.set_stem_volume(stem_id, volume)
    }

    pub fn set_stem_mute(&self, stem_id: &str, muted: bool) -> Result<(), String> {
        self.audio_engine.set_stem_mute(stem_id, muted)
    }

    pub fn set_stem_solo(&self, stem_id: &str, solo: bool) -> Result<(), String> {
        self.audio_engine.set_stem_solo(stem_id, solo)
    }
}
