//! `Score Metadata Manager` -- parses the `.cho` (extended ChordPro) file
//! referenced by the project: chords, tablature, lyrics, metronome, tuning.
//! See docs/architecture.md#score-metadata-chords-tablature-and-lyrics and
//! the `chordpro-format` skill for the full grammar this will implement
//! (one chord per line, `{t: m:ss.cc}` time anchors, tablature technique
//! notation).

use std::path::Path;

pub struct ScoreMetadataManager;

impl ScoreMetadataManager {
    pub fn new() -> Self {
        Self
    }

    /// Parses the `.cho` file at `path`. Once implemented, a successful
    /// parse backs the `score_loaded` event (chords, tablature, lyrics,
    /// tempo/time/tuning); a parse failure backs `score_parse_error` (the
    /// `.cho` line and a message) -- the rest of the app keeps working even
    /// if this fails, see
    /// docs/architecture.md#design-note-error-presentation-modal. Not yet
    /// implemented: see the `chordpro-format` skill for the grammar.
    pub fn load_score(&self, _path: &Path) -> Result<(), String> {
        Err("load_score: not yet implemented".into())
    }
}
