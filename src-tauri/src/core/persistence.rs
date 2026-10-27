//! `Project Persistence` -- serializes/reads the project's `.json`, plus the
//! waveform cache and (eventually) the `.cho`. Every disk write goes
//! through [`ProjectPersistence::atomic_write`]: write to a temp file in the
//! same directory, then `rename` into place -- no exceptions. Writes happen
//! at checkpoints ([`SaveScheduler`]: dirty flag + debounce, or a definitive
//! event), never once per `Command`. See
//! docs/architecture.md#design-note-when-project-persistence-writes,
//! docs/architecture.md#project-versioning and the `atomic-persistence`
//! playbook.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// The `schemaVersion` this build writes and reads without migrating.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;
/// Inactivity window before a dirty project is written ("a few ms with no
/// new `Commands`" -- long enough to absorb a fader drag).
pub const DEFAULT_DEBOUNCE: Duration = Duration::from_millis(300);

/// One registered `N -> N+1` migration: `migrations[i]` turns a project at
/// version `i + 1` into version `i + 2`. The loader bumps `schemaVersion`
/// after each one, so a migration only reshapes the data.
pub type Migration = fn(serde_json::Value) -> Result<serde_json::Value, String>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub stems: Vec<StemEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<String>,
    #[serde(rename = "loop", default, skip_serializing_if = "Option::is_none")]
    pub loop_range: Option<LoopRange>,
    #[serde(default)]
    pub mixer: BTreeMap<String, MixerSettings>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StemEntry {
    pub id: String,
    pub file: String,
    pub duration_sec: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waveform_cache: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoopRange {
    pub start_sec: f64,
    pub end_sec: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MixerSettings {
    pub volume: f32,
    pub mute: bool,
    pub solo: bool,
}

/// A stem nobody has touched yet: full volume, not muted, not soloed.
impl Default for MixerSettings {
    fn default() -> Self {
        MixerSettings {
            volume: 1.0,
            mute: false,
            solo: false,
        }
    }
}

impl Project {
    /// The project's duration: its longest stem's `durationSec`, 0 with no
    /// stems (docs/architecture.md#consistency-across-a-projects-stems).
    pub fn duration_sec(&self) -> f64 {
        self.stems
            .iter()
            .map(|s| s.duration_sec)
            .fold(0.0, f64::max)
    }

    /// A new, empty project: a fresh UUID v4 `id`, the current schema, no
    /// stems, score, loop or mixer settings.
    pub fn new_empty(name: &str) -> Self {
        Project {
            schema_version: CURRENT_SCHEMA_VERSION,
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_owned(),
            stems: Vec::new(),
            score: None,
            loop_range: None,
            mixer: BTreeMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    Malformed(String),
    NewerVersion { found: u32, supported: u32 },
    Migration { from: u32, message: String },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io(e) => write!(f, "IO error: {e}"),
            LoadError::Malformed(e) => write!(f, "malformed project file: {e}"),
            LoadError::NewerVersion { found, supported } => {
                write!(
                    f,
                    "project saved by a newer version of the app (schema {found}; this version reads up to {supported})"
                )
            }
            LoadError::Migration { from, message } => {
                write!(f, "migration from v{from} failed: {message}")
            }
        }
    }
}

impl std::error::Error for LoadError {}

/// Decides *when* a dirty project is written: after `debounce` with no new
/// change, or right away on a definitive event (pause, close). Pure state,
/// no clock and no thread -- callers pass `now`, which keeps it testable.
pub struct SaveScheduler {
    debounce: Duration,
    last_change: Option<Instant>,
    /// Bumped on every change. The write token is this number, not the
    /// change's `Instant`: two changes can share an `Instant` (one `now`
    /// reused, a coarse clock), two generations never do.
    generation: u64,
}

impl SaveScheduler {
    pub fn new(debounce: Duration) -> Self {
        SaveScheduler {
            debounce,
            last_change: None,
            generation: 0,
        }
    }

    pub fn mark_dirty(&mut self, now: Instant) {
        self.last_change = Some(now);
        self.generation += 1;
    }

    pub fn is_dirty(&self) -> bool {
        self.last_change.is_some()
    }

    /// `Some(token)` when a write should happen now: dirty, and either a
    /// definitive event or `debounce` elapsed since the last change.
    pub fn begin_write(&self, now: Instant, definitive: bool) -> Option<u64> {
        self.last_change
            .filter(|&last| definitive || now.saturating_duration_since(last) >= self.debounce)
            .map(|_| self.generation)
    }

    /// Clears the dirty flag only if no change happened since `token` was
    /// handed out -- a change that arrives mid-write still needs saving.
    pub fn finish_write(&mut self, token: u64) {
        if self.generation == token {
            self.last_change = None;
        }
    }
}

pub struct ProjectPersistence {
    /// Module's one `Mutex` (docs/architecture.md, concurrency model): held
    /// only to read/update the scheduler, never across disk I/O.
    scheduler: Mutex<SaveScheduler>,
    schema_version: u32,
    migrations: Vec<Migration>,
}

/// Distinguishes concurrent writes to the same path from one process.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

impl ProjectPersistence {
    pub fn new() -> Self {
        ProjectPersistence::with_config(CURRENT_SCHEMA_VERSION, vec![], DEFAULT_DEBOUNCE)
    }

    /// Manual DI for tests and future schema bumps. Panics on a registry
    /// that can't reach `schema_version` -- a programming error caught at
    /// startup, not a half-migrated project at load time.
    pub fn with_config(
        schema_version: u32,
        migrations: Vec<Migration>,
        debounce: Duration,
    ) -> Self {
        assert!(schema_version >= 1, "schema versions start at 1");
        assert_eq!(
            migrations.len() as u32,
            schema_version - 1,
            "need exactly one migration per step from 1 to {schema_version}"
        );
        ProjectPersistence {
            schema_version,
            migrations,
            scheduler: Mutex::new(SaveScheduler::new(debounce)),
        }
    }

    /// Reads `schemaVersion` before anything else: the same version loads
    /// directly, a lower one runs every registered migration in order, a
    /// higher one is [`LoadError::NewerVersion`]. Never writes the file and
    /// never returns a partial project.
    pub fn load_project(&self, path: &Path) -> Result<Project, LoadError> {
        let contents = fs::read_to_string(path).map_err(LoadError::Io)?;
        let value: serde_json::Value = serde_json::from_str(&contents)
            .map_err(|e| LoadError::Malformed(format!("not valid JSON: {e}")))?;

        let schema_version = value["schemaVersion"]
            .as_u64()
            .and_then(|v| u32::try_from(v).ok())
            .filter(|&v| v >= 1)
            .ok_or_else(|| {
                LoadError::Malformed("schemaVersion missing or not an integer >= 1".into())
            })?;

        if schema_version > self.schema_version {
            return Err(LoadError::NewerVersion {
                found: schema_version,
                supported: self.schema_version,
            });
        }

        let mut value = value;
        for v in schema_version..self.schema_version {
            let migration = self.migrations[(v - 1) as usize];
            value =
                migration(value).map_err(|message| LoadError::Migration { from: v, message })?;
            let object = value.as_object_mut().ok_or_else(|| LoadError::Migration {
                from: v,
                message: "migration returned something other than a JSON object".into(),
            })?;
            object.insert("schemaVersion".into(), (v + 1).into());
        }

        serde_json::from_value(value)
            .map_err(|e| LoadError::Malformed(format!("project file has wrong shape: {e}")))
    }

    /// Writes `project` now, through [`Self::atomic_write`]. Prefer
    /// [`Self::checkpoint`] for state changes; this is the primitive under it.
    pub fn save_project(&self, path: &Path, project: &Project) -> io::Result<()> {
        let contents = serde_json::to_vec_pretty(project)
            .map_err(|e| io::Error::other(format!("failed to serialize project: {e}")))?;
        self.atomic_write(path, &contents)
    }

    /// Records that persisted state changed at `now`. Cheap: no I/O.
    pub fn mark_dirty(&self, now: Instant) {
        self.scheduler().mark_dirty(now);
    }

    /// The scheduler holds only plain values that stay valid after any
    /// panic, so a poisoned lock is safe to keep using.
    fn scheduler(&self) -> MutexGuard<'_, SaveScheduler> {
        self.scheduler
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Writes `project` if it's dirty and the debounce has elapsed (or
    /// `definitive` is set, e.g. pause/close); `Ok(true)` when it wrote. A
    /// failed write stays dirty for the next try. Callers pass a snapshot
    /// taken outside any lock of their own.
    pub fn checkpoint(
        &self,
        path: &Path,
        project: &Project,
        now: Instant,
        definitive: bool,
    ) -> io::Result<bool> {
        let Some(token) = self.begin_checkpoint(now, definitive) else {
            return Ok(false);
        };
        self.write_checkpoint(path, project, token)?;
        Ok(true)
    }

    /// First half of [`Self::checkpoint`], for a caller that keeps the
    /// project behind its own lock: call it *under* that lock, together with
    /// taking the snapshot, so the token covers exactly the changes in the
    /// snapshot. `None` when there's nothing to write yet.
    pub fn begin_checkpoint(&self, now: Instant, definitive: bool) -> Option<u64> {
        self.scheduler().begin_write(now, definitive)
    }

    /// Second half: writes the snapshot (after the caller released its
    /// lock) and clears the dirty flag unless a change came in since
    /// `token`. A failed write stays dirty.
    pub fn write_checkpoint(&self, path: &Path, project: &Project, token: u64) -> io::Result<()> {
        // The scheduler lock is released here: a change that arrives during
        // the write is recorded and keeps the project dirty.
        self.save_project(path, project)?;
        self.scheduler().finish_write(token);
        Ok(())
    }

    /// Writes `contents` to `path` atomically: a unique temp file in the same
    /// directory, synced, then `rename`d into place. A crash or error at any
    /// point leaves the previous file intact -- worst case the write in
    /// progress is lost, never an existing file. Every write this module
    /// makes goes through here (the `atomic-persistence` playbook).
    pub fn atomic_write(&self, path: &Path, contents: &[u8]) -> io::Result<()> {
        self.atomic_write_with_hook(path, contents, || Ok(()))
    }

    /// [`Self::atomic_write`] for contents too big to hold in memory (an
    /// imported stem): `write` streams them into the temp file. Same
    /// guarantee -- if `write` fails, the temp file is removed and `path` is
    /// left as it was.
    pub fn atomic_write_with(
        &self,
        path: &Path,
        write: impl FnOnce(&mut fs::File) -> io::Result<()>,
    ) -> io::Result<()> {
        self.atomic_write_streaming(path, write, || Ok(()))
    }

    fn atomic_write_with_hook(
        &self,
        path: &Path,
        contents: &[u8],
        before_rename: impl FnOnce() -> io::Result<()>,
    ) -> io::Result<()> {
        self.atomic_write_streaming(path, |file| file.write_all(contents), before_rename)
    }

    fn atomic_write_streaming(
        &self,
        path: &Path,
        write: impl FnOnce(&mut fs::File) -> io::Result<()>,
        before_rename: impl FnOnce() -> io::Result<()>,
    ) -> io::Result<()> {
        let dir = path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "path has no parent directory")
        })?;
        let file_name = path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no file name"))?;
        // Same directory as the destination: `rename` is only atomic within
        // one file system.
        let temp_path = dir.join(format!(
            ".{}.tmp-{}-{}",
            file_name.to_string_lossy(),
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));

        let result = (|| {
            let mut file = fs::File::create(&temp_path)?;
            write(&mut file)?;
            file.sync_all()?;
            drop(file);
            before_rename()?;
            fs::rename(&temp_path, path)
        })();
        if result.is_err() {
            // Whatever step failed, never leave a temp file behind.
            let _ = fs::remove_file(&temp_path);
        }
        result
    }
}

impl Default for ProjectPersistence {
    fn default() -> Self {
        ProjectPersistence::new()
    }
}

#[cfg(test)]
mod tests {
    // The contract for TASK-002 (openspec/changes/task-002-project-persistence).
    // Written before the implementation, by a different author than it --
    // never loosen one of these to make it pass (AGENTS.md, testing policy).
    use super::*;
    use std::time::{Duration, Instant};

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "stemloft-test-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn other_files(dir: &Path, keep: &str) -> Vec<String> {
        fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name != keep)
            .collect()
    }

    fn sample_project() -> Project {
        Project {
            schema_version: CURRENT_SCHEMA_VERSION,
            id: "b3a1e6c2-8f21-4d9a-9c3e-1a2b3c4d5e6f".into(),
            name: "Study in G Major".into(),
            stems: vec![StemEntry {
                id: "guitar".into(),
                file: "guitar.wav".into(),
                duration_sec: 187.42,
                waveform_cache: None,
            }],
            score: None,
            loop_range: None,
            mixer: BTreeMap::new(),
        }
    }

    /// The example from docs/architecture.md#project-versioning, verbatim.
    const ARCHITECTURE_EXAMPLE: &str = r#"{
      "schemaVersion": 1,
      "id": "b3a1e6c2-8f21-4d9a-9c3e-1a2b3c4d5e6f",
      "name": "Study in G Major",
      "stems": [
        { "id": "guitar", "file": "guitar.wav", "durationSec": 187.42, "waveformCache": "guitar.waveform" },
        { "id": "vocals", "file": "vocals.wav", "durationSec": 187.42, "waveformCache": "vocals.waveform" },
        { "id": "bass", "file": "bass.wav", "durationSec": 186.90, "waveformCache": "bass.waveform" },
        { "id": "drums", "file": "drums.wav", "durationSec": 187.42, "waveformCache": "drums.waveform" }
      ],
      "score": "study-in-g-major.cho",
      "loop": { "startSec": 0.0, "endSec": 12.0 },
      "mixer": {
        "guitar": { "volume": 0.9, "mute": false, "solo": true },
        "vocals": { "volume": 0.7, "mute": false, "solo": false }
      }
    }"#;

    // ---- atomic writes -------------------------------------------------

    #[test]
    fn atomic_write_creates_the_file_with_the_given_contents() {
        let dir = temp_dir("create");
        let path = dir.join("project.json");

        ProjectPersistence::new()
            .atomic_write(&path, b"{\"schemaVersion\":1}")
            .unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"{\"schemaVersion\":1}");
        assert!(other_files(&dir, "project.json").is_empty());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn atomic_write_overwrites_an_existing_file() {
        let dir = temp_dir("overwrite");
        let path = dir.join("project.json");
        let persistence = ProjectPersistence::new();

        persistence.atomic_write(&path, b"old").unwrap();
        persistence.atomic_write(&path, b"new").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"new");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_crash_between_temp_write_and_rename_leaves_the_previous_file_intact() {
        let dir = temp_dir("crash");
        let path = dir.join("project.json");
        let persistence = ProjectPersistence::new();
        persistence.atomic_write(&path, b"previous").unwrap();

        let mut hook_ran = false;
        let result = persistence.atomic_write_with_hook(&path, b"next", || {
            hook_ran = true;
            Err(io::Error::other("simulated crash before rename"))
        });

        assert!(hook_ran, "the hook must run after the temp file is written");
        assert!(result.is_err());
        assert_eq!(fs::read(&path).unwrap(), b"previous");
        assert!(
            other_files(&dir, "project.json").is_empty(),
            "a failed write must clean up its temp file"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_change_between_begin_and_write_checkpoint_keeps_the_project_dirty() {
        let dir = temp_dir("split-checkpoint");
        let path = dir.join("project.json");
        let persistence = ProjectPersistence::new();
        let t0 = Instant::now();
        persistence.mark_dirty(t0);

        let token = persistence.begin_checkpoint(t0, true).unwrap();
        persistence.mark_dirty(t0); // arrives after the snapshot was taken
        persistence
            .write_checkpoint(&path, &sample_project(), token)
            .unwrap();

        assert!(path.exists());
        assert!(persistence.begin_checkpoint(t0, true).is_some());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_project_duration_is_its_longest_stem() {
        let mut project = Project::new_empty("x");
        assert_eq!(project.duration_sec(), 0.0);
        for (id, duration_sec) in [("bass", 186.9), ("guitar", 187.42), ("drums", 12.0)] {
            project.stems.push(StemEntry {
                id: id.into(),
                file: format!("{id}.wav"),
                duration_sec,
                waveform_cache: None,
            });
        }
        assert_eq!(project.duration_sec(), 187.42);
    }

    #[test]
    fn atomic_write_with_streams_into_place() {
        let dir = temp_dir("stream");
        let path = dir.join("bass.wav");

        ProjectPersistence::new()
            .atomic_write_with(&path, |file| {
                file.write_all(b"RIFF")?;
                file.write_all(b"....")
            })
            .unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"RIFF....");
        assert!(other_files(&dir, "bass.wav").is_empty());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_failed_streaming_write_leaves_the_previous_file_and_no_temp() {
        let dir = temp_dir("stream-fail");
        let path = dir.join("bass.wav");
        let persistence = ProjectPersistence::new();
        persistence.atomic_write(&path, b"previous").unwrap();

        let result = persistence.atomic_write_with(&path, |file| {
            file.write_all(b"half a stem")?;
            Err(io::Error::other("source file vanished mid-copy"))
        });

        assert!(result.is_err());
        assert_eq!(fs::read(&path).unwrap(), b"previous");
        assert!(other_files(&dir, "bass.wav").is_empty());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_hook_sees_the_new_contents_only_in_a_temp_file() {
        let dir = temp_dir("hook-view");
        let path = dir.join("project.json");
        let persistence = ProjectPersistence::new();
        persistence.atomic_write(&path, b"previous").unwrap();

        let dir_for_hook = dir.clone();
        persistence
            .atomic_write_with_hook(&path, b"next", || {
                assert_eq!(
                    fs::read(dir_for_hook.join("project.json")).unwrap(),
                    b"previous"
                );
                let temps = other_files(&dir_for_hook, "project.json");
                assert_eq!(temps.len(), 1, "exactly one temp file: {temps:?}");
                assert_eq!(fs::read(dir_for_hook.join(&temps[0])).unwrap(), b"next");
                Ok(())
            })
            .unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"next");
        assert!(other_files(&dir, "project.json").is_empty());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn concurrent_writes_to_the_same_file_never_collide() {
        let dir = temp_dir("concurrent");
        let path = dir.join("project.json");
        let persistence = std::sync::Arc::new(ProjectPersistence::new());

        let handles: Vec<_> = (0..8)
            .map(|i| {
                let (persistence, path) = (std::sync::Arc::clone(&persistence), path.clone());
                std::thread::spawn(move || {
                    persistence.atomic_write(&path, format!("write {i}").as_bytes())
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap().unwrap();
        }

        assert!(fs::read_to_string(&path).unwrap().starts_with("write "));
        assert!(other_files(&dir, "project.json").is_empty());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn io_errors_keep_their_kind() {
        let dir = temp_dir("error-kind");
        let missing_dir = dir.join("no-such-dir").join("project.json");
        let error = ProjectPersistence::new()
            .save_project(&missing_dir, &sample_project())
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::NotFound, "{error}");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    #[should_panic(expected = "one migration per step")]
    fn a_migration_registry_that_cant_reach_the_version_is_rejected_at_startup() {
        ProjectPersistence::with_config(3, vec![v1_to_v2], Duration::from_millis(10));
    }

    // ---- save / load ---------------------------------------------------

    #[test]
    fn a_saved_project_loads_back_unchanged() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("project.json");
        let persistence = ProjectPersistence::new();
        let mut project = sample_project();
        project.score = Some("song.cho".into());
        project.loop_range = Some(LoopRange {
            start_sec: 1.5,
            end_sec: 9.25,
        });
        project.mixer.insert(
            "guitar".into(),
            MixerSettings {
                volume: 0.5,
                mute: true,
                solo: false,
            },
        );

        persistence.save_project(&path, &project).unwrap();

        assert_eq!(persistence.load_project(&path).unwrap(), project);
        assert!(other_files(&dir, "project.json").is_empty());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_architecture_example_loads() {
        let dir = temp_dir("example");
        let path = dir.join("project.json");
        fs::write(&path, ARCHITECTURE_EXAMPLE).unwrap();

        let project = ProjectPersistence::new().load_project(&path).unwrap();

        assert_eq!(project.schema_version, 1);
        assert_eq!(project.name, "Study in G Major");
        assert_eq!(project.stems.len(), 4);
        assert_eq!(project.stems[2].id, "bass");
        assert_eq!(project.stems[2].duration_sec, 186.90);
        assert_eq!(
            project.stems[0].waveform_cache.as_deref(),
            Some("guitar.waveform")
        );
        assert_eq!(project.score.as_deref(), Some("study-in-g-major.cho"));
        assert_eq!(
            project.loop_range,
            Some(LoopRange {
                start_sec: 0.0,
                end_sec: 12.0
            })
        );
        assert_eq!(
            project.mixer["guitar"],
            MixerSettings {
                volume: 0.9,
                mute: false,
                solo: true
            }
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn optional_fields_may_be_absent_and_are_omitted_on_save() {
        let dir = temp_dir("optional");
        let path = dir.join("project.json");
        fs::write(
            &path,
            r#"{"schemaVersion":1,"id":"x","name":"n","stems":[{"id":"a","file":"a.wav","durationSec":1.0}]}"#,
        )
        .unwrap();
        let persistence = ProjectPersistence::new();

        let project = persistence.load_project(&path).unwrap();
        assert_eq!(project.score, None);
        assert_eq!(project.loop_range, None);
        assert!(project.mixer.is_empty());
        assert_eq!(project.stems[0].waveform_cache, None);

        persistence.save_project(&path, &project).unwrap();
        let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert!(
            saved.get("score").is_none(),
            "absent score must not be written: {saved}"
        );
        assert!(
            saved.get("loop").is_none(),
            "absent loop must not be written: {saved}"
        );
        assert!(saved["stems"][0].get("waveformCache").is_none(), "{saved}");
        assert_eq!(saved["schemaVersion"], 1);
        assert_eq!(saved["stems"][0]["durationSec"], 1.0);
        fs::remove_dir_all(&dir).ok();
    }

    // ---- schemaVersion: the three cases --------------------------------

    #[test]
    fn same_schema_version_loads_directly_without_migrations() {
        let dir = temp_dir("same-version");
        let path = dir.join("project.json");
        fs::write(&path, ARCHITECTURE_EXAMPLE).unwrap();
        fn must_not_run(_: serde_json::Value) -> Result<serde_json::Value, String> {
            panic!("no migration may run for a file already at the current version")
        }
        let persistence =
            ProjectPersistence::with_config(2, vec![must_not_run], Duration::from_millis(10));
        fs::write(
            &path,
            ARCHITECTURE_EXAMPLE.replace("\"schemaVersion\": 1", "\"schemaVersion\": 2"),
        )
        .unwrap();

        let project = persistence.load_project(&path).unwrap();

        assert_eq!(project.schema_version, 2);
        assert_eq!(project.stems.len(), 4);
        fs::remove_dir_all(&dir).ok();
    }

    fn v1_to_v2(mut value: serde_json::Value) -> Result<serde_json::Value, String> {
        // v1 called the field "title".
        let title = value["title"]
            .as_str()
            .ok_or("v1 project without title")?
            .to_owned();
        let object = value.as_object_mut().unwrap();
        object.remove("title");
        object.insert("name".into(), format!("{title} v2").into());
        assert_eq!(
            object["schemaVersion"], 1,
            "migration 1->2 must see version 1"
        );
        Ok(value)
    }

    fn v2_to_v3(mut value: serde_json::Value) -> Result<serde_json::Value, String> {
        assert_eq!(
            value["schemaVersion"], 2,
            "the loader bumps schemaVersion after each migration"
        );
        let name = value["name"].as_str().unwrap().to_owned();
        value["name"] = format!("{name} v3").into();
        Ok(value)
    }

    #[test]
    fn a_lower_schema_version_runs_every_migration_in_order() {
        let dir = temp_dir("migrate");
        let path = dir.join("project.json");
        let v1 = r#"{"schemaVersion":1,"id":"x","title":"Song","stems":[]}"#;
        fs::write(&path, v1).unwrap();
        let persistence =
            ProjectPersistence::with_config(3, vec![v1_to_v2, v2_to_v3], Duration::from_millis(10));

        let project = persistence.load_project(&path).unwrap();

        assert_eq!(project.schema_version, 3);
        assert_eq!(project.name, "Song v2 v3");
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            v1,
            "loading must never rewrite the file"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn migrations_start_from_the_file_version_not_from_one() {
        let dir = temp_dir("migrate-partial");
        let path = dir.join("project.json");
        fs::write(
            &path,
            r#"{"schemaVersion":2,"id":"x","name":"Song","stems":[]}"#,
        )
        .unwrap();
        let persistence =
            ProjectPersistence::with_config(3, vec![v1_to_v2, v2_to_v3], Duration::from_millis(10));

        let project = persistence.load_project(&path).unwrap();

        assert_eq!(project.name, "Song v3");
        assert_eq!(project.schema_version, 3);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_higher_schema_version_is_an_explicit_error_and_the_file_is_untouched() {
        let dir = temp_dir("newer");
        let path = dir.join("project.json");
        let newer = ARCHITECTURE_EXAMPLE.replace("\"schemaVersion\": 1", "\"schemaVersion\": 7");
        fs::write(&path, &newer).unwrap();

        let error = ProjectPersistence::new().load_project(&path).unwrap_err();

        match &error {
            LoadError::NewerVersion { found, supported } => {
                assert_eq!((*found, *supported), (7, CURRENT_SCHEMA_VERSION));
            }
            other => panic!("expected NewerVersion, got {other:?}"),
        }
        assert!(
            error.to_string().contains("newer version of the app"),
            "message must tell the user why: {error}"
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), newer);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_failing_migration_reports_the_version_it_started_from() {
        let dir = temp_dir("migrate-fail");
        let path = dir.join("project.json");
        fs::write(
            &path,
            r#"{"schemaVersion":1,"id":"x","name":"no title here","stems":[]}"#,
        )
        .unwrap();
        let persistence =
            ProjectPersistence::with_config(3, vec![v1_to_v2, v2_to_v3], Duration::from_millis(10));

        match persistence.load_project(&path).unwrap_err() {
            LoadError::Migration { from, message } => {
                assert_eq!(from, 1);
                assert!(message.contains("without title"), "{message}");
            }
            other => panic!("expected Migration, got {other:?}"),
        }
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_migration_returning_a_non_object_is_an_error_not_a_panic() {
        let dir = temp_dir("migrate-non-object");
        let path = dir.join("project.json");
        fs::write(
            &path,
            r#"{"schemaVersion":1,"id":"x","name":"n","stems":[]}"#,
        )
        .unwrap();
        fn to_array(_: serde_json::Value) -> Result<serde_json::Value, String> {
            Ok(serde_json::json!([1, 2, 3]))
        }
        let persistence =
            ProjectPersistence::with_config(2, vec![to_array], Duration::from_millis(10));

        match persistence.load_project(&path) {
            Err(LoadError::Migration { from: 1, .. }) => {}
            other => panic!("expected Migration from 1, got {other:?}"),
        }
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_or_invalid_schema_version_and_bad_json_are_malformed() {
        let dir = temp_dir("malformed");
        let path = dir.join("project.json");
        let persistence = ProjectPersistence::new();
        for contents in [
            r#"{"id":"x","name":"n","stems":[]}"#,
            r#"{"schemaVersion":"1","id":"x","name":"n","stems":[]}"#,
            r#"{"schemaVersion":0,"id":"x","name":"n","stems":[]}"#,
            r#"{"schemaVersion":1,"id":"x","stems":[]}"#,
            "not json at all",
            "[1, 2, 3]",
        ] {
            fs::write(&path, contents).unwrap();
            match persistence.load_project(&path) {
                Err(LoadError::Malformed(_)) => {}
                other => panic!("{contents:?}: expected Malformed, got {other:?}"),
            }
        }
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_missing_file_is_an_io_error() {
        let dir = temp_dir("missing");
        match ProjectPersistence::new().load_project(&dir.join("nope.json")) {
            Err(LoadError::Io(error)) => assert_eq!(error.kind(), io::ErrorKind::NotFound),
            other => panic!("expected Io, got {other:?}"),
        }
        fs::remove_dir_all(&dir).ok();
    }

    // ---- when to write: dirty flag + debounce / definitive events -------

    const DEBOUNCE: Duration = Duration::from_millis(300);

    #[test]
    fn a_clean_scheduler_never_asks_for_a_write() {
        let scheduler = SaveScheduler::new(DEBOUNCE);
        let now = Instant::now();
        assert!(!scheduler.is_dirty());
        assert_eq!(scheduler.begin_write(now + DEBOUNCE * 10, false), None);
        assert_eq!(
            scheduler.begin_write(now, true),
            None,
            "a definitive event with nothing dirty writes nothing"
        );
    }

    #[test]
    fn a_write_is_due_only_after_the_debounce_with_no_new_change() {
        let mut scheduler = SaveScheduler::new(DEBOUNCE);
        let t0 = Instant::now();
        scheduler.mark_dirty(t0);

        assert!(scheduler.is_dirty());
        assert_eq!(
            scheduler.begin_write(t0 + DEBOUNCE - Duration::from_millis(1), false),
            None
        );
        assert!(scheduler.begin_write(t0 + DEBOUNCE, false).is_some());
    }

    #[test]
    fn every_new_change_restarts_the_debounce() {
        let mut scheduler = SaveScheduler::new(DEBOUNCE);
        let t0 = Instant::now();
        scheduler.mark_dirty(t0);
        scheduler.mark_dirty(t0 + Duration::from_millis(200));

        assert_eq!(scheduler.begin_write(t0 + DEBOUNCE, false), None);
        assert!(scheduler
            .begin_write(t0 + Duration::from_millis(200) + DEBOUNCE, false)
            .is_some());
    }

    #[test]
    fn a_definitive_event_writes_immediately_when_dirty() {
        let mut scheduler = SaveScheduler::new(DEBOUNCE);
        let t0 = Instant::now();
        scheduler.mark_dirty(t0);
        assert!(scheduler.begin_write(t0, true).is_some());
    }

    #[test]
    fn finishing_a_write_cleans_only_if_nothing_changed_meanwhile() {
        let mut scheduler = SaveScheduler::new(DEBOUNCE);
        let t0 = Instant::now();
        scheduler.mark_dirty(t0);
        let token = scheduler.begin_write(t0 + DEBOUNCE, false).unwrap();

        scheduler.mark_dirty(t0 + DEBOUNCE + Duration::from_millis(1)); // arrives mid-write
        scheduler.finish_write(token);
        assert!(
            scheduler.is_dirty(),
            "the change made during the write still needs saving"
        );

        let later = t0 + DEBOUNCE * 3;
        let token = scheduler.begin_write(later, false).unwrap();
        scheduler.finish_write(token);
        assert!(!scheduler.is_dirty());
    }

    #[test]
    fn a_change_with_the_same_instant_as_the_write_still_stays_dirty() {
        let mut scheduler = SaveScheduler::new(DEBOUNCE);
        let t = Instant::now();
        scheduler.mark_dirty(t);
        let token = scheduler.begin_write(t, true).unwrap();

        scheduler.mark_dirty(t); // same `now` reused by the caller, mid-write
        scheduler.finish_write(token);

        assert!(
            scheduler.is_dirty(),
            "a change made during the write must not be lost"
        );
    }

    #[test]
    fn a_burst_of_changes_is_written_once() {
        let dir = temp_dir("burst");
        let path = dir.join("project.json");
        let persistence = ProjectPersistence::with_config(CURRENT_SCHEMA_VERSION, vec![], DEBOUNCE);
        let mut project = sample_project();
        let t0 = Instant::now();

        let mut writes = 0;
        for tick in 0..50u32 {
            let now = t0 + Duration::from_millis(u64::from(tick) * 5);
            project.name = format!("fader tick {tick}");
            persistence.mark_dirty(now);
            if persistence.checkpoint(&path, &project, now, false).unwrap() {
                writes += 1;
            }
        }
        assert_eq!(writes, 0, "nothing is written while changes keep coming");
        assert!(!path.exists());

        let quiet = t0 + Duration::from_millis(49 * 5) + DEBOUNCE;
        assert!(persistence
            .checkpoint(&path, &project, quiet, false)
            .unwrap());
        assert!(!persistence
            .checkpoint(&path, &project, quiet + DEBOUNCE, false)
            .unwrap());
        assert!(!persistence
            .checkpoint(&path, &project, quiet + DEBOUNCE, true)
            .unwrap());

        assert_eq!(
            persistence.load_project(&path).unwrap().name,
            "fader tick 49"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn closing_the_project_flushes_pending_changes_immediately() {
        let dir = temp_dir("definitive");
        let path = dir.join("project.json");
        let persistence = ProjectPersistence::new();
        let now = Instant::now();

        persistence.mark_dirty(now);
        assert!(persistence
            .checkpoint(&path, &sample_project(), now, true)
            .unwrap());
        assert!(path.exists());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_failed_checkpoint_write_stays_dirty_for_the_next_try() {
        let dir = temp_dir("checkpoint-fail");
        let unwritable = dir.join("no-such-dir").join("project.json");
        let persistence = ProjectPersistence::new();
        let now = Instant::now();

        persistence.mark_dirty(now);
        assert!(persistence
            .checkpoint(&unwritable, &sample_project(), now, true)
            .is_err());

        let path = dir.join("project.json");
        assert!(persistence
            .checkpoint(&path, &sample_project(), now, true)
            .unwrap());
        fs::remove_dir_all(&dir).ok();
    }
}
