//! The core's side of the `Events` bus (Rust → Angular): what a domain
//! module calls to publish something. The IPC layer implements it with
//! Tauri (`ipc::events::TauriEvents`); no `core` module knows about Tauri
//! (docs/architecture.md#communication-bridge--tauri-ipc). One method per
//! event in the [events table](../../../docs/architecture.md#events-table),
//! added as each producer is implemented.

use super::score_metadata::{Score, ScoreParseError};

pub trait EventSink: Send + Sync {
    /// `score_loaded` -- published by `Score Metadata Manager`.
    fn score_loaded(&self, score: &Score);
    /// `score_parse_error` -- published by `Score Metadata Manager`.
    fn score_parse_error(&self, error: &ScoreParseError);
}

/// Records every event, for tests of the modules that publish them.
#[cfg(test)]
#[derive(Default)]
pub struct RecordedEvents {
    pub events: std::sync::Mutex<Vec<String>>,
}

#[cfg(test)]
impl EventSink for RecordedEvents {
    fn score_loaded(&self, score: &Score) {
        self.events.lock().unwrap().push(format!(
            "score_loaded(chords={}, tempo={:?})",
            score.chords.len(),
            score.tempo
        ));
    }

    fn score_parse_error(&self, error: &ScoreParseError) {
        self.events
            .lock()
            .unwrap()
            .push(format!("score_parse_error({error})"));
    }
}
