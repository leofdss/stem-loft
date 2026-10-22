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
//! What Session does besides forwarding is exactly what its design note
//! allows: it owns the open project (its `.json` path and its current
//! state), hands the owning module what it needs from it, records a
//! successful change there, and tells `Project Persistence` the project is
//! dirty. *When* that gets written is `Project Persistence`'s decision
//! ([`Session::checkpoint`] is called by a background ticker and on
//! definitive events). Commands reach Session one at a time, in arrival
//! order, through `core::command_queue`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use super::audio_engine::AudioEngine;
use super::loop_manager::LoopManager;
use super::persistence::{LoopRange, MixerSettings, Project, ProjectPersistence, StemEntry};
use super::score_metadata::ScoreMetadataManager;
use super::stem_importer::{ImportError, StemImporter};

/// The project file's name inside a project folder.
pub const PROJECT_FILE: &str = "project.json";

/// The error every project-scoped `Command` returns when nothing is open.
pub const NO_PROJECT_OPEN: &str = "no project is open";

/// The `Commands` Session forwards to `Stem Importer`.
pub trait StemImporterCommands: Send + Sync {
    fn import(
        &self,
        project_dir: &Path,
        existing: &[StemEntry],
        files: &[PathBuf],
    ) -> Result<Vec<StemEntry>, ImportError>;
}

/// The `Commands` Session forwards to `Loop & Marker Manager`, plus putting
/// a project's saved loop back when it's opened.
pub trait LoopManagerCommands: Send + Sync {
    fn set_markers(
        &self,
        start_sec: f64,
        end_sec: f64,
        project_duration_sec: f64,
    ) -> Result<(), String>;
    fn restore(&self, markers: Option<LoopRange>);
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
    fn import(
        &self,
        project_dir: &Path,
        existing: &[StemEntry],
        files: &[PathBuf],
    ) -> Result<Vec<StemEntry>, ImportError> {
        StemImporter::import(self, project_dir, existing, files)
    }
}

impl LoopManagerCommands for LoopManager {
    fn set_markers(
        &self,
        start_sec: f64,
        end_sec: f64,
        project_duration_sec: f64,
    ) -> Result<(), String> {
        LoopManager::set_markers(self, start_sec, end_sec, project_duration_sec)
    }

    fn restore(&self, markers: Option<LoopRange>) {
        LoopManager::restore(self, markers)
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

/// The only state `Session` owns directly: the project that's open right
/// now -- where its `.json` lives, and its current state.
struct OpenProject {
    path: PathBuf,
    project: Project,
}

impl OpenProject {
    /// The folder holding the project file and its stems.
    fn dir(&self) -> &Path {
        self.path.parent().unwrap_or_else(|| Path::new("."))
    }
}

pub struct Session {
    stem_importer: Arc<dyn StemImporterCommands>,
    loop_manager: Arc<dyn LoopManagerCommands>,
    audio_engine: Arc<dyn AudioEngineCommands>,
    persistence: Arc<ProjectPersistence>,
    // Not dispatched to by any Command yet (score loading lands with
    // TASK-012) -- held concretely, with no trait, until one does.
    #[allow(dead_code)]
    score_metadata: Arc<ScoreMetadataManager>,
    // Session's one `Mutex` -- see
    // docs/architecture.md#design-note-concurrency-model-for-shared-state.
    // Commands only change it one at a time (the queue), so the only other
    // party is the checkpoint ticker.
    open_project: Mutex<Option<OpenProject>>,
    // Serializes whole checkpoints (snapshot, write, mark clean): without
    // it, a slow ticker write of an older snapshot could land after a
    // newer definitive one and leave the stale file on disk, marked clean.
    // Taken before `open_project`, never while holding it.
    checkpoint_writing: Mutex<()>,
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
            open_project: Mutex::new(None),
            checkpoint_writing: Mutex::new(()),
        }
    }

    /// `OpenProject` is plain data and every change to it is a single
    /// assignment, so a poisoned lock still holds a usable value.
    fn open(&self) -> MutexGuard<'_, Option<OpenProject>> {
        self.open_project
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn is_project_open(&self) -> bool {
        self.open().is_some()
    }

    /// Writes the open project if `Project Persistence` says it's time
    /// (dirty, and the debounce elapsed or `definitive`). `Ok(false)` when
    /// nothing was written, including when no project is open. The token
    /// and the snapshot are taken together under Session's lock, and the
    /// write happens after releasing it -- so a change made meanwhile is
    /// never marked as saved. Two checkpoints never overlap.
    pub fn checkpoint(&self, now: Instant, definitive: bool) -> io::Result<bool> {
        let _writing = self
            .checkpoint_writing
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let (path, snapshot, token) = {
            let open = self.open();
            let Some(open) = open.as_ref() else {
                return Ok(false);
            };
            let Some(token) = self.persistence.begin_checkpoint(now, definitive) else {
                return Ok(false);
            };
            (open.path.clone(), open.project.clone(), token)
        };
        self.persistence.write_checkpoint(&path, &snapshot, token)?;
        Ok(true)
    }

    /// `Err(NO_PROJECT_OPEN)` unless a project is open. Takes the lock only
    /// for the check; the caller then calls its module without holding it.
    fn require_open(&self) -> Result<(), String> {
        if self.is_project_open() {
            Ok(())
        } else {
            Err(NO_PROJECT_OPEN.to_string())
        }
    }

    /// Applies `change` to the open project and marks it dirty, in one
    /// locked section. Errors with `NO_PROJECT_OPEN` if nothing is open.
    fn record_change(&self, change: impl FnOnce(&mut Project)) -> Result<(), String> {
        let mut open = self.open();
        let open = open.as_mut().ok_or_else(|| NO_PROJECT_OPEN.to_string())?;
        change(&mut open.project);
        self.persistence.mark_dirty(Instant::now());
        Ok(())
    }

    /// Records a mixer change for `stem_id`, starting from the defaults if
    /// the stem has no entry yet.
    fn record_mixer(
        &self,
        stem_id: &str,
        change: impl FnOnce(&mut MixerSettings),
    ) -> Result<(), String> {
        self.record_change(|project| change(project.mixer.entry(stem_id.to_owned()).or_default()))
    }

    /// Definitive checkpoint of the open project (if any), then forget it.
    /// The project stays open if the write fails. A no-op when nothing is open.
    fn close_current(&self) -> Result<(), String> {
        if !self.is_project_open() {
            return Ok(());
        }
        self.checkpoint(Instant::now(), true)
            .map_err(|e| e.to_string())?;
        *self.open() = None;
        self.loop_manager.restore(None);
        Ok(())
    }

    // --- Project lifecycle -------------------------------------------------

    /// Creates `dir` (and any missing parent) holding a new, empty project
    /// named `name` -- a fresh UUID v4 `id`, `CURRENT_SCHEMA_VERSION`, no
    /// stems, no score, no loop, empty mixer -- written right away to
    /// `dir/PROJECT_FILE` with `ProjectPersistence::save_project`, then
    /// opened (closing the current project first, as [`Self::close_project`]
    /// does). Refuses, writing nothing and keeping the current project open,
    /// when `dir/PROJECT_FILE` already exists. Returns the new project.
    pub fn create_project(&self, dir: &Path, name: &str) -> Result<Project, String> {
        let path = dir.join(PROJECT_FILE);
        if path.exists() {
            return Err(format!("a project already exists at {}", path.display()));
        }
        // Save the current project before creating anything: if that fails,
        // no new file is left behind and the current project stays open.
        self.checkpoint(Instant::now(), true)
            .map_err(|e| e.to_string())?;
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let project = Project::new_empty(name);
        self.persistence
            .save_project(&path, &project)
            .map_err(|e| e.to_string())?;
        *self.open() = Some(OpenProject {
            path,
            project: project.clone(),
        });
        self.loop_manager.restore(None);
        Ok(project)
    }

    /// Saves the current project (definitive checkpoint), loads the
    /// project file at `path` with `ProjectPersistence::load_project` (its
    /// error's `Display` is the message), then opens it in place of the
    /// current one. When saving or loading fails, the current project stays
    /// open. Returns the loaded project.
    pub fn open_project(&self, path: &Path) -> Result<Project, String> {
        // Save the current project before reading: reopening the same file
        // must load what's in memory now, not an older version on disk.
        self.checkpoint(Instant::now(), true)
            .map_err(|e| e.to_string())?;
        let project = self
            .persistence
            .load_project(path)
            .map_err(|e| e.to_string())?;
        self.close_current()?;
        *self.open() = Some(OpenProject {
            path: path.to_path_buf(),
            project: project.clone(),
        });
        self.loop_manager.restore(project.loop_range);
        Ok(project)
    }

    /// Writes the open project if it has unsaved changes (a definitive
    /// checkpoint at `Instant::now()`), then closes it. If the write fails
    /// the project stays open and the error is returned.
    /// [`NO_PROJECT_OPEN`] when nothing is open.
    pub fn close_project(&self) -> Result<(), String> {
        self.require_open()?;
        self.close_current()
    }

    // --- Dispatch to the owning module --------------------------------------
    // Every one of these needs an open project: without one they return
    // NO_PROJECT_OPEN and the module is never called. A module's error is
    // returned unchanged and changes nothing. A successful state change is
    // recorded on the open project and marks it dirty
    // (`ProjectPersistence::mark_dirty(Instant::now())`).

    /// Hands the project folder, the project's current stems and `files`
    /// to `Stem Importer`; appends the returned entries to the project's
    /// `stems`, marks it dirty and returns them. An `ImportError` comes
    /// back as its `Display` text.
    pub fn import_stems(&self, files: Vec<PathBuf>) -> Result<Vec<StemEntry>, String> {
        let (dir, existing) = {
            let open = self.open();
            let open = open.as_ref().ok_or_else(|| NO_PROJECT_OPEN.to_string())?;
            (open.dir().to_path_buf(), open.project.stems.clone())
        };
        let new_stems = self
            .stem_importer
            .import(&dir, &existing, &files)
            .map_err(|e| e.to_string())?;
        if !new_stems.is_empty() {
            self.record_change(|project| project.stems.extend(new_stems.iter().cloned()))?;
        }
        Ok(new_stems)
    }

    /// Forwards to `Loop & Marker Manager` with the project's duration (its
    /// longest stem) to validate against; on success sets the project's
    /// `loop` to `{ start_sec, end_sec }` and marks it dirty. Opening a
    /// project restores its saved loop there; creating or closing one
    /// clears it.
    pub fn set_markers(&self, start_sec: f64, end_sec: f64) -> Result<(), String> {
        let duration = {
            let open = self.open();
            let open = open.as_ref().ok_or_else(|| NO_PROJECT_OPEN.to_string())?;
            open.project.duration_sec()
        };
        self.loop_manager
            .set_markers(start_sec, end_sec, duration)?;
        self.record_change(|project| {
            project.loop_range = Some(LoopRange { start_sec, end_sec });
        })
    }

    /// Forwards to `Audio Engine`. Transport state isn't saved in the
    /// project, so nothing is marked dirty.
    pub fn play(&self) -> Result<(), String> {
        self.require_open()?;
        self.audio_engine.play()
    }

    /// Forwards to `Audio Engine`; on success, pausing is a definitive
    /// event: a definitive checkpoint at `Instant::now()`. If that write
    /// fails the pause still succeeded; the project stays dirty and the
    /// ticker retries.
    pub fn pause(&self) -> Result<(), String> {
        self.require_open()?;
        self.audio_engine.pause()?;
        // Playback did pause, so that's the answer. A failed save leaves
        // the project dirty for the ticker to retry; it doesn't turn a
        // successful pause into an error.
        let _ = self.checkpoint(Instant::now(), true);
        Ok(())
    }

    /// Forwards to `Audio Engine`; nothing is marked dirty.
    pub fn stop(&self) -> Result<(), String> {
        self.require_open()?;
        self.audio_engine.stop()
    }

    /// Forwards to `Audio Engine`; on success records `volume` in the
    /// project's `mixer[stem_id]` (a stem with no entry yet starts from
    /// `{ volume: 1.0, mute: false, solo: false }`) and marks it dirty.
    pub fn set_stem_volume(&self, stem_id: &str, volume: f32) -> Result<(), String> {
        self.require_open()?;
        self.audio_engine.set_stem_volume(stem_id, volume)?;
        self.record_mixer(stem_id, |m| m.volume = volume)
    }

    /// Like [`Self::set_stem_volume`], for `mute`.
    pub fn set_stem_mute(&self, stem_id: &str, muted: bool) -> Result<(), String> {
        self.require_open()?;
        self.audio_engine.set_stem_mute(stem_id, muted)?;
        self.record_mixer(stem_id, |m| m.mute = muted)
    }

    /// Like [`Self::set_stem_volume`], for `solo`.
    pub fn set_stem_solo(&self, stem_id: &str, solo: bool) -> Result<(), String> {
        self.require_open()?;
        self.audio_engine.set_stem_solo(stem_id, solo)?;
        self.record_mixer(stem_id, |m| m.solo = solo)
    }
}

#[cfg(test)]
mod tests {
    // The contract for TASK-009 (openspec/changes/task-009-ipc-commands).
    // Written before the implementation, by a different author than it --
    // never loosen one of these to make it pass (AGENTS.md, testing policy).
    //
    // These prove Session forwards each `Command` to the right module with
    // the arguments untouched, returns that module's result unchanged, and
    // records successful changes on the open project for `Project
    // Persistence` to write. Each module's own behavior is tested in its
    // own file -- not here.

    use super::*;
    use crate::core::persistence::CURRENT_SCHEMA_VERSION;
    use std::time::Duration;

    /// One fake for the loop manager and the audio engine: every call is
    /// appended to `calls` as a readable string, and returns `result`.
    struct FakeModule {
        calls: Mutex<Vec<String>>,
        /// `LoopManagerCommands::restore` calls, kept apart from `calls` so
        /// a dispatch assertion doesn't depend on project lifecycle.
        restored: Mutex<Vec<Option<LoopRange>>>,
        result: Result<(), String>,
    }

    impl FakeModule {
        fn returning(result: Result<(), String>) -> Arc<Self> {
            Arc::new(Self {
                calls: Mutex::new(Vec::new()),
                restored: Mutex::new(Vec::new()),
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

    impl LoopManagerCommands for FakeModule {
        fn set_markers(
            &self,
            start_sec: f64,
            end_sec: f64,
            project_duration_sec: f64,
        ) -> Result<(), String> {
            self.record(format!(
                "set_markers({start_sec}, {end_sec}, {project_duration_sec})"
            ))
        }

        fn restore(&self, markers: Option<LoopRange>) {
            self.restored.lock().unwrap().push(markers);
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

    /// Records what it was handed; on success returns one entry per file
    /// (id = file stem, 2.5 s long), on failure an `Unsupported` error.
    /// (project dir, ids of the existing stems, files to import)
    type ImportCall = (PathBuf, Vec<String>, Vec<PathBuf>);

    struct FakeImporter {
        calls: Mutex<Vec<ImportCall>>,
        fail: bool,
    }

    impl StemImporterCommands for FakeImporter {
        fn import(
            &self,
            project_dir: &Path,
            existing: &[StemEntry],
            files: &[PathBuf],
        ) -> Result<Vec<StemEntry>, ImportError> {
            self.calls.lock().unwrap().push((
                project_dir.to_path_buf(),
                existing.iter().map(|s| s.id.clone()).collect(),
                files.to_vec(),
            ));
            if self.fail {
                return Err(ImportError::Unsupported {
                    file: files[0].clone(),
                    message: "fake failure".into(),
                });
            }
            Ok(files
                .iter()
                .map(|f| {
                    let id = f.file_stem().unwrap().to_string_lossy().into_owned();
                    StemEntry {
                        file: format!("{id}.wav"),
                        id,
                        duration_sec: 2.5,
                        waveform_cache: None,
                    }
                })
                .collect())
        }
    }

    /// A Session wired to separate fakes and a real `Project Persistence`
    /// with a one-hour debounce, so only a definitive checkpoint (or a
    /// `now` an hour later) writes.
    struct Fixture {
        session: Session,
        importer: Arc<FakeImporter>,
        loop_manager: Arc<FakeModule>,
        audio_engine: Arc<FakeModule>,
        dir: PathBuf,
    }

    const DEBOUNCE: Duration = Duration::from_secs(3600);

    fn fixture_with(result: Result<(), String>, import_fails: bool) -> Fixture {
        let importer = Arc::new(FakeImporter {
            calls: Mutex::new(Vec::new()),
            fail: import_fails,
        });
        let loop_manager = FakeModule::returning(result.clone());
        let audio_engine = FakeModule::returning(result);
        let session = Session::new(
            importer.clone(),
            loop_manager.clone(),
            audio_engine.clone(),
            Arc::new(ProjectPersistence::with_config(
                CURRENT_SCHEMA_VERSION,
                vec![],
                DEBOUNCE,
            )),
            Arc::new(ScoreMetadataManager::new()),
        );
        let dir = std::env::temp_dir().join(format!(
            "stemloft-session-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        Fixture {
            session,
            importer,
            loop_manager,
            audio_engine,
            dir,
        }
    }

    fn fixture(result: Result<(), String>) -> Fixture {
        fixture_with(result, false)
    }

    /// A fixture with a freshly created project "Song" open in `dir/song`.
    fn with_project(result: Result<(), String>) -> Fixture {
        let f = fixture(result);
        f.session
            .create_project(&f.dir.join("song"), "Song")
            .unwrap();
        f
    }

    fn saved(f: &Fixture) -> Project {
        let path = f.dir.join("song").join(PROJECT_FILE);
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.dir).ok();
        }
    }

    // ---- project lifecycle ---------------------------------------------

    #[test]
    fn no_project_is_open_on_a_new_session() {
        assert!(!fixture(Ok(())).session.is_project_open());
    }

    #[test]
    fn create_project_writes_an_empty_project_and_opens_it() {
        let f = fixture(Ok(()));
        let project = f
            .session
            .create_project(&f.dir.join("song"), "Song")
            .unwrap();

        assert_eq!(project.name, "Song");
        assert_eq!(project.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(project.stems.is_empty() && project.mixer.is_empty());
        assert_eq!(project.loop_range, None);
        assert!(uuid::Uuid::parse_str(&project.id).is_ok(), "{}", project.id);
        assert!(f.session.is_project_open());
        assert_eq!(saved(&f), project);
    }

    #[test]
    fn create_project_never_overwrites_an_existing_project() {
        let f = with_project(Ok(()));
        let first = saved(&f);

        let err = f.session.create_project(&f.dir.join("song"), "Other");

        assert!(err.is_err());
        assert_eq!(saved(&f), first);
        assert!(f.session.is_project_open());
    }

    #[test]
    fn open_project_loads_the_file_and_a_bad_path_changes_nothing() {
        let f = with_project(Ok(()));
        let path = f.dir.join("song").join(PROJECT_FILE);
        f.session.close_project().unwrap();

        assert!(f.session.open_project(&f.dir.join("missing.json")).is_err());
        assert!(!f.session.is_project_open());

        let project = f.session.open_project(&path).unwrap();
        assert_eq!(project.name, "Song");
        assert!(f.session.is_project_open());
    }

    #[test]
    fn close_project_saves_unsaved_changes_first() {
        let f = with_project(Ok(()));
        f.session.set_markers(1.0, 4.0).unwrap();
        assert_eq!(
            saved(&f).loop_range,
            None,
            "nothing written before the debounce"
        );

        f.session.close_project().unwrap();

        assert_eq!(
            saved(&f).loop_range,
            Some(LoopRange {
                start_sec: 1.0,
                end_sec: 4.0
            })
        );
        assert!(!f.session.is_project_open());
        assert_eq!(f.session.close_project(), Err(NO_PROJECT_OPEN.into()));
    }

    #[test]
    fn opening_another_project_saves_the_current_one_first() {
        let f = with_project(Ok(()));
        let other = f.dir.join("other");
        f.session.create_project(&other, "Other").unwrap();
        f.session.close_project().unwrap();
        f.session
            .open_project(&f.dir.join("song").join(PROJECT_FILE))
            .unwrap();
        f.session.set_markers(2.0, 3.0).unwrap();

        let project = f.session.open_project(&other.join(PROJECT_FILE)).unwrap();

        assert_eq!(project.name, "Other");
        assert!(saved(&f).loop_range.is_some());
    }

    // ---- commands need an open project ---------------------------------

    #[test]
    fn project_commands_without_an_open_project_fail_and_call_nothing() {
        let f = fixture(Ok(()));
        let no = Err::<(), String>(NO_PROJECT_OPEN.into());

        assert_eq!(
            f.session.import_stems(vec![PathBuf::from("/a/bass.wav")]),
            Err(NO_PROJECT_OPEN.into())
        );
        assert_eq!(f.session.set_markers(0.0, 1.0), no);
        assert_eq!(f.session.play(), no);
        assert_eq!(f.session.pause(), no);
        assert_eq!(f.session.stop(), no);
        assert_eq!(f.session.set_stem_volume("bass", 0.5), no);
        assert_eq!(f.session.set_stem_mute("bass", true), no);
        assert_eq!(f.session.set_stem_solo("bass", true), no);

        assert!(f.importer.calls.lock().unwrap().is_empty());
        assert!(f.loop_manager.calls().is_empty());
        assert!(f.audio_engine.calls().is_empty());
    }

    // ---- dispatch ------------------------------------------------------

    #[test]
    fn import_stems_hands_the_importer_the_project_and_records_the_new_stems() {
        let f = with_project(Ok(()));
        let first = f
            .session
            .import_stems(vec![PathBuf::from("/a/bass.wav")])
            .unwrap();
        let second = f
            .session
            .import_stems(vec![
                PathBuf::from("/a/drums.wav"),
                PathBuf::from("/a/keys.wav"),
            ])
            .unwrap();

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 2);
        let calls = f.importer.calls.lock().unwrap().clone();
        assert_eq!(calls[0].0, f.dir.join("song"));
        assert!(calls[0].1.is_empty());
        assert_eq!(calls[1].1, vec!["bass"]);
        assert_eq!(
            calls[1].2,
            vec![PathBuf::from("/a/drums.wav"), PathBuf::from("/a/keys.wav")]
        );
        assert!(f.loop_manager.calls().is_empty());
        assert!(f.audio_engine.calls().is_empty());

        assert!(f.session.checkpoint(Instant::now(), true).unwrap());
        let ids: Vec<String> = saved(&f).stems.into_iter().map(|s| s.id).collect();
        assert_eq!(ids, vec!["bass", "drums", "keys"]);
    }

    #[test]
    fn a_failed_import_returns_the_importers_message_and_changes_nothing() {
        let f = fixture_with(Ok(()), true);
        f.session
            .create_project(&f.dir.join("song"), "Song")
            .unwrap();

        let err = f
            .session
            .import_stems(vec![PathBuf::from("/a/bass.mp3")])
            .unwrap_err();

        assert_eq!(
            err,
            ImportError::Unsupported {
                file: PathBuf::from("/a/bass.mp3"),
                message: "fake failure".into()
            }
            .to_string()
        );
        assert!(
            !f.session.checkpoint(Instant::now(), true).unwrap(),
            "not dirty"
        );
    }

    #[test]
    fn set_markers_dispatches_only_to_loop_manager_and_records_the_loop() {
        let f = with_project(Ok(()));

        assert_eq!(f.session.set_markers(1.5, 8.25), Ok(()));

        assert_eq!(f.loop_manager.calls(), vec!["set_markers(1.5, 8.25, 0)"]);
        assert!(f.audio_engine.calls().is_empty());
        f.session.checkpoint(Instant::now(), true).unwrap();
        assert_eq!(
            saved(&f).loop_range,
            Some(LoopRange {
                start_sec: 1.5,
                end_sec: 8.25
            })
        );
    }

    #[test]
    fn set_markers_validates_against_the_longest_stem() {
        let f = with_project(Ok(()));
        f.session
            .import_stems(vec![PathBuf::from("/a/bass.wav")])
            .unwrap();

        f.session.set_markers(0.5, 2.0).unwrap();

        // The fake importer makes every stem 2.5 s long.
        assert_eq!(f.loop_manager.calls(), vec!["set_markers(0.5, 2, 2.5)"]);
    }

    #[test]
    fn opening_a_project_restores_its_loop_and_closing_clears_it() {
        let f = with_project(Ok(()));
        f.session.set_markers(1.0, 2.0).unwrap();
        f.session.close_project().unwrap();
        f.session
            .open_project(&f.dir.join("song").join(PROJECT_FILE))
            .unwrap();
        f.session.close_project().unwrap();

        assert_eq!(
            *f.loop_manager.restored.lock().unwrap(),
            vec![
                None, // created
                None, // closed
                Some(LoopRange {
                    start_sec: 1.0,
                    end_sec: 2.0
                }), // reopened
                None, // closed
            ]
        );
    }

    #[test]
    fn transport_and_mixer_commands_dispatch_only_to_audio_engine() {
        let f = with_project(Ok(()));

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
        assert!(f.importer.calls.lock().unwrap().is_empty());
        assert!(f.loop_manager.calls().is_empty());
    }

    #[test]
    fn mixer_changes_are_recorded_per_stem_from_the_defaults() {
        let f = with_project(Ok(()));
        f.session.set_stem_volume("bass", 0.5).unwrap();
        f.session.set_stem_mute("bass", true).unwrap();
        f.session.set_stem_solo("drums", true).unwrap();

        f.session.checkpoint(Instant::now(), true).unwrap();

        let mixer = saved(&f).mixer;
        let expected: std::collections::BTreeMap<String, MixerSettings> = [
            (
                "bass".to_string(),
                MixerSettings {
                    volume: 0.5,
                    mute: true,
                    solo: false,
                },
            ),
            (
                "drums".to_string(),
                MixerSettings {
                    volume: 1.0,
                    mute: false,
                    solo: true,
                },
            ),
        ]
        .into();
        assert_eq!(mixer, expected);
    }

    #[test]
    fn a_module_error_is_returned_unchanged_and_records_nothing() {
        let f = with_project(Err("boom".into()));

        assert_eq!(f.session.set_markers(0.0, 1.0), Err("boom".into()));
        assert_eq!(f.session.play(), Err("boom".into()));
        assert_eq!(f.session.pause(), Err("boom".into()));
        assert_eq!(f.session.stop(), Err("boom".into()));
        assert_eq!(f.session.set_stem_volume("bass", 1.0), Err("boom".into()));
        assert_eq!(f.session.set_stem_mute("bass", true), Err("boom".into()));
        assert_eq!(f.session.set_stem_solo("bass", true), Err("boom".into()));

        assert!(
            !f.session.checkpoint(Instant::now(), true).unwrap(),
            "not dirty"
        );
    }

    // ---- checkpoints ---------------------------------------------------

    #[test]
    fn changes_are_written_after_the_debounce_not_per_command() {
        let f = with_project(Ok(()));
        let t0 = Instant::now();
        f.session.set_stem_volume("bass", 0.1).unwrap();
        f.session.set_stem_volume("bass", 0.2).unwrap();

        assert!(!f.session.checkpoint(t0, false).unwrap());
        assert!(saved(&f).mixer.is_empty());

        assert!(f
            .session
            .checkpoint(t0 + DEBOUNCE + DEBOUNCE, false)
            .unwrap());
        assert_eq!(saved(&f).mixer["bass"].volume, 0.2);
        assert!(
            !f.session.checkpoint(t0 + DEBOUNCE * 3, false).unwrap(),
            "clean now"
        );
    }

    #[test]
    fn pausing_writes_unsaved_changes_right_away() {
        let f = with_project(Ok(()));
        f.session.set_markers(0.5, 2.0).unwrap();

        f.session.pause().unwrap();

        assert!(saved(&f).loop_range.is_some());
    }

    #[test]
    fn transport_commands_do_not_mark_the_project_dirty() {
        let f = with_project(Ok(()));
        f.session.play().unwrap();
        f.session.stop().unwrap();
        assert!(!f.session.checkpoint(Instant::now(), true).unwrap());
    }

    #[test]
    fn concurrent_checkpoints_never_leave_an_older_snapshot_on_disk() {
        // Added after review: the ticker and a definitive checkpoint
        // racing used to be able to rename an older snapshot last.
        let f = Arc::new(with_project(Ok(())));
        let ticker = {
            let f = Arc::clone(&f);
            std::thread::spawn(move || {
                for _ in 0..300 {
                    let _ = f.session.checkpoint(Instant::now() + DEBOUNCE * 2, false);
                }
            })
        };
        for i in 0..300 {
            f.session.set_stem_volume("bass", i as f32).unwrap();
            f.session.checkpoint(Instant::now(), true).unwrap();
        }
        ticker.join().unwrap();

        assert_eq!(saved(&f).mixer["bass"].volume, 299.0);
    }

    #[test]
    fn reopening_the_open_project_keeps_its_unsaved_changes() {
        // Added after review.
        let f = with_project(Ok(()));
        f.session.set_markers(1.0, 2.0).unwrap();

        let project = f
            .session
            .open_project(&f.dir.join("song").join(PROJECT_FILE))
            .unwrap();

        assert!(project.loop_range.is_some());
    }

    #[test]
    fn checkpoint_with_no_project_open_writes_nothing() {
        let f = fixture(Ok(()));
        assert!(!f.session.checkpoint(Instant::now(), true).unwrap());
    }
}
