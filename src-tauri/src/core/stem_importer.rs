//! `Stem Importer` -- receives stem files (local, or later the output of
//! automatic separation), enforces consistency across a project's stems,
//! and writes them into the project. See
//! docs/architecture.md#consistency-across-a-projects-stems.

use std::path::PathBuf;
use std::sync::Arc;

use super::persistence::ProjectPersistence;

pub struct StemImporter {
    // Not called by anything yet -- `import_stems` below is still a stub.
    // Kept here (rather than added when needed) because the constructor's
    // signature is part of this module's contract from the start: importing
    // ends with `Project Persistence` writing the updated project, and that
    // dependency is explicit from day one, not bolted on later.
    #[allow(dead_code)]
    persistence: Arc<ProjectPersistence>,
}

impl StemImporter {
    pub fn new(persistence: Arc<ProjectPersistence>) -> Self {
        Self { persistence }
    }

    /// Imports one or more stem files into the current project.
    ///
    /// Still to implement, in this order (see
    /// docs/architecture.md#consistency-across-a-projects-stems):
    /// 1. Read each file's sample rate and reject the whole import with an
    ///    explicit error if it differs from stems already in the project --
    ///    never silently resample.
    /// 2. Upmix a mono stem to stereo (duplicate the channel to L/R) --
    ///    at import time, not on every playback.
    /// 3. Read `durationSec` from the file header only (no full decode) and
    ///    store it on the stem entry.
    /// 4. Write the stem into the project's folder and hand the updated
    ///    project to `Project Persistence` (atomic write -- see the
    ///    `atomic-persistence` skill).
    pub fn import_stems(&self, _file_paths: Vec<PathBuf>) -> Result<(), String> {
        Err("import_stems: not yet implemented".into())
    }
}
