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
//! Session reaches each module it dispatches to through a small trait
//! (`StemImporterCommands`, `LoopManagerCommands`, `AudioEngineCommands`
//! below) instead of the concrete struct -- the Rust equivalent of a
//! Nest.js controller depending on a service's interface. The one reason
//! these traits exist is so the tests at the bottom of this file can hand
//! Session a fake module that records the call; there is exactly one real
//! implementation of each, the domain module itself.
//!
//! TODO(scaffold): the `Commands` queue itself
//! (docs/architecture.md#design-note-commands-queue-in-the-rust-core) isn't
//! implemented yet -- it lands with TASK-009. Until then Tauri dispatches
//! each invoked command directly, concurrently, which is why this struct
//! still needs its own small lock around `open_project`, and why the
//! domain modules hold their own locks too.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::audio_engine::AudioEngine;
use super::loop_manager::LoopManager;
use super::persistence::ProjectPersistence;
use super::score_metadata::ScoreMetadataManager;
use super::stem_importer::StemImporter;

/// The `Commands` Session forwards to `Stem Importer`.
pub trait StemImporterCommands: Send + Sync {
    fn import_stems(&self, file_paths: Vec<PathBuf>) -> Result<(), String>;
}

/// The `Commands` Session forwards to `Loop & Marker Manager`.
pub trait LoopManagerCommands: Send + Sync {
    fn set_markers(&self, start_sec: f64, end_sec: f64) -> Result<(), String>;
}

/// The `Commands` Session forwards to `Audio Engine` (transport + mixer).
pub trait AudioEngineCommands: Send + Sync {
    fn play(&self) -> Result<(), String>;
    fn pause(&self) -> Result<(), String>;
    fn stop(&self) -> Result<(), String>;
    fn set_stem_volume(&self, stem_id: &str, volume: f32) -> Result<(), String>;
    fn set_stem_mute(&self, stem_id: &str, muted: bool) -> Result<(), String>;
    fn set_stem_solo(&self, stem_id: &str, solo: bool) -> Result<(), String>;
}

// The real implementations: each one only calls the module's own method of
// the same name -- the logic lives in the module, never in these impls.

impl StemImporterCommands for StemImporter {
    fn import_stems(&self, file_paths: Vec<PathBuf>) -> Result<(), String> {
        StemImporter::import_stems(self, file_paths)
    }
}

impl LoopManagerCommands for LoopManager {
    fn set_markers(&self, start_sec: f64, end_sec: f64) -> Result<(), String> {
        LoopManager::set_markers(self, start_sec, end_sec)
    }
}

impl AudioEngineCommands for AudioEngine {
    fn play(&self) -> Result<(), String> {
        AudioEngine::play(self)
    }

    fn pause(&self) -> Result<(), String> {
        AudioEngine::pause(self)
    }

    fn stop(&self) -> Result<(), String> {
        AudioEngine::stop(self)
    }

    fn set_stem_volume(&self, stem_id: &str, volume: f32) -> Result<(), String> {
        AudioEngine::set_stem_volume(self, stem_id, volume)
    }

    fn set_stem_mute(&self, stem_id: &str, muted: bool) -> Result<(), String> {
        AudioEngine::set_stem_mute(self, stem_id, muted)
    }

    fn set_stem_solo(&self, stem_id: &str, solo: bool) -> Result<(), String> {
        AudioEngine::set_stem_solo(self, stem_id, solo)
    }
}

/// The only state `Session` owns directly: which project is open right now.
#[derive(Default)]
struct OpenProject {
    path: Option<PathBuf>,
}

pub struct Session {
    stem_importer: Arc<dyn StemImporterCommands>,
    loop_manager: Arc<dyn LoopManagerCommands>,
    audio_engine: Arc<dyn AudioEngineCommands>,
    // Not dispatched to by any Command yet (project load/save lands with
    // TASK-002, score loading with TASK-012) -- held concretely, with no
    // trait, until a Command actually dispatches to them.
    #[allow(dead_code)]
    persistence: Arc<ProjectPersistence>,
    #[allow(dead_code)]
    score_metadata: Arc<ScoreMetadataManager>,
    // Session's one `Mutex` -- see
    // docs/architecture.md#design-note-concurrency-model-for-shared-state.
    open_project: Mutex<OpenProject>,
}

impl Session {
    /// Manual dependency injection: every `Arc` is constructed by the
    /// caller (see `AppState::new` in `core::app_state`) and handed in
    /// explicitly here, once -- no container, no service locator.
    pub fn new(
        stem_importer: Arc<dyn StemImporterCommands>,
        loop_manager: Arc<dyn LoopManagerCommands>,
        audio_engine: Arc<dyn AudioEngineCommands>,
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

#[cfg(test)]
mod tests {
    //! These tests only prove Session forwards each `Command` to the right
    //! module, with the arguments untouched, and returns that module's
    //! result unchanged. Each module's own behavior is tested in its own
    //! file -- not here.

    use super::*;

    /// One fake for all three traits: every call is appended to `calls` as
    /// a readable string, and every call returns `result`.
    struct FakeModule {
        calls: Mutex<Vec<String>>,
        result: Result<(), String>,
    }

    impl FakeModule {
        fn returning(result: Result<(), String>) -> Arc<Self> {
            Arc::new(Self {
                calls: Mutex::new(Vec::new()),
                result,
            })
        }

        fn record(&self, call: String) -> Result<(), String> {
            self.calls.lock().unwrap().push(call);
            self.result.clone()
        }

        fn calls(&self) -> Vec<String> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl StemImporterCommands for FakeModule {
        fn import_stems(&self, file_paths: Vec<PathBuf>) -> Result<(), String> {
            self.record(format!("import_stems({file_paths:?})"))
        }
    }

    impl LoopManagerCommands for FakeModule {
        fn set_markers(&self, start_sec: f64, end_sec: f64) -> Result<(), String> {
            self.record(format!("set_markers({start_sec}, {end_sec})"))
        }
    }

    impl AudioEngineCommands for FakeModule {
        fn play(&self) -> Result<(), String> {
            self.record("play()".into())
        }

        fn pause(&self) -> Result<(), String> {
            self.record("pause()".into())
        }

        fn stop(&self) -> Result<(), String> {
            self.record("stop()".into())
        }

        fn set_stem_volume(&self, stem_id: &str, volume: f32) -> Result<(), String> {
            self.record(format!("set_stem_volume({stem_id}, {volume})"))
        }

        fn set_stem_mute(&self, stem_id: &str, muted: bool) -> Result<(), String> {
            self.record(format!("set_stem_mute({stem_id}, {muted})"))
        }

        fn set_stem_solo(&self, stem_id: &str, solo: bool) -> Result<(), String> {
            self.record(format!("set_stem_solo({stem_id}, {solo})"))
        }
    }

    /// A Session wired to three separate fakes, so each test can assert
    /// that only the owning module was called.
    struct Fixture {
        session: Session,
        stem_importer: Arc<FakeModule>,
        loop_manager: Arc<FakeModule>,
        audio_engine: Arc<FakeModule>,
    }

    fn fixture(result: Result<(), String>) -> Fixture {
        let stem_importer = FakeModule::returning(result.clone());
        let loop_manager = FakeModule::returning(result.clone());
        let audio_engine = FakeModule::returning(result);
        let session = Session::new(
            stem_importer.clone(),
            loop_manager.clone(),
            audio_engine.clone(),
            Arc::new(ProjectPersistence::new()),
            Arc::new(ScoreMetadataManager::new()),
        );
        Fixture {
            session,
            stem_importer,
            loop_manager,
            audio_engine,
        }
    }

    #[test]
    fn no_project_is_open_on_a_new_session() {
        assert!(!fixture(Ok(())).session.is_project_open());
    }

    #[test]
    fn import_stems_dispatches_only_to_stem_importer() {
        let f = fixture(Ok(()));
        let paths = vec![PathBuf::from("/a/bass.wav"), PathBuf::from("/a/drums.wav")];

        assert_eq!(f.session.import_stems(paths), Ok(()));

        assert_eq!(
            f.stem_importer.calls(),
            vec![r#"import_stems(["/a/bass.wav", "/a/drums.wav"])"#]
        );
        assert!(f.loop_manager.calls().is_empty());
        assert!(f.audio_engine.calls().is_empty());
    }

    #[test]
    fn set_markers_dispatches_only_to_loop_manager() {
        let f = fixture(Ok(()));

        assert_eq!(f.session.set_markers(1.5, 8.25), Ok(()));

        assert_eq!(f.loop_manager.calls(), vec!["set_markers(1.5, 8.25)"]);
        assert!(f.stem_importer.calls().is_empty());
        assert!(f.audio_engine.calls().is_empty());
    }

    #[test]
    fn transport_and_mixer_commands_dispatch_only_to_audio_engine() {
        let f = fixture(Ok(()));

        assert_eq!(f.session.play(), Ok(()));
        assert_eq!(f.session.pause(), Ok(()));
        assert_eq!(f.session.stop(), Ok(()));
        assert_eq!(f.session.set_stem_volume("bass", 0.5), Ok(()));
        assert_eq!(f.session.set_stem_mute("drums", true), Ok(()));
        assert_eq!(f.session.set_stem_solo("vocals", false), Ok(()));

        assert_eq!(
            f.audio_engine.calls(),
            vec![
                "play()",
                "pause()",
                "stop()",
                "set_stem_volume(bass, 0.5)",
                "set_stem_mute(drums, true)",
                "set_stem_solo(vocals, false)",
            ]
        );
        assert!(f.stem_importer.calls().is_empty());
        assert!(f.loop_manager.calls().is_empty());
    }

    #[test]
    fn a_module_error_is_returned_unchanged() {
        let f = fixture(Err("boom".into()));

        assert_eq!(f.session.import_stems(vec![]), Err("boom".into()));
        assert_eq!(f.session.set_markers(0.0, 1.0), Err("boom".into()));
        assert_eq!(f.session.play(), Err("boom".into()));
        assert_eq!(f.session.set_stem_volume("bass", 1.0), Err("boom".into()));
    }
}
