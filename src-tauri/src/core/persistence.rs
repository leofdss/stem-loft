//! `Project Persistence` -- serializes/reads the project's `.json`, plus the
//! waveform cache and (eventually) the `.cho`. Every disk write goes
//! through [`ProjectPersistence::atomic_write`]: write to a temp file in the
//! same directory, then `rename` into place -- no exceptions. See
//! docs/architecture.md#design-note-when-project-persistence-writes and the
//! `atomic-persistence` skill.

use std::fs;
use std::io;
use std::path::Path;

pub struct ProjectPersistence;

impl ProjectPersistence {
    pub fn new() -> Self {
        Self
    }

    /// Loads the project at `path`. Still to implement: read
    /// `schemaVersion` first -- same version loads directly, a lower one
    /// applies registered migrations in sequence, and a higher one is an
    /// explicit error, never a silent partial read. See
    /// docs/architecture.md#project-versioning.
    pub fn load_project(&self, _path: &Path) -> Result<(), String> {
        Err("load_project: not yet implemented".into())
    }

    /// Writes `contents` to `path`, atomically: a temp file in the same
    /// directory, then `rename`d into place. `rename` is atomic at the
    /// file-system level, so a crash mid-write never leaves `path`
    /// truncated or partially written -- worst case, the write in progress
    /// is lost, never a file that already existed.
    ///
    /// This is the one primitive every write `Project Persistence` makes
    /// must go through -- the project's `.json`, the waveform cache, and
    /// eventually the `.cho` -- see the `atomic-persistence` skill. It's
    /// real (not a stub) because it's simple, self-contained infrastructure
    /// with no domain decision in it.
    pub fn atomic_write(&self, path: &Path, contents: &[u8]) -> io::Result<()> {
        let dir = path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "path has no parent directory")
        })?;

        // The temp file must live in the *same* directory as the
        // destination -- `rename` is only guaranteed atomic within the same
        // file system/mount point.
        let tmp_name = format!(
            ".{}.tmp-{}",
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("stemloft-write"),
            std::process::id()
        );
        let tmp_path = dir.join(tmp_name);

        fs::write(&tmp_path, contents)?;
        fs::rename(&tmp_path, path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn atomic_write_creates_the_file_with_the_given_contents() {
        let dir = temp_dir("create");
        let path = dir.join("project.json");

        ProjectPersistence::new()
            .atomic_write(&path, b"{\"schemaVersion\":1}")
            .unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"{\"schemaVersion\":1}");

        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name() != "project.json")
            .collect();
        assert!(leftovers.is_empty(), "leftover files: {leftovers:?}");

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
}
