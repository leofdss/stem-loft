//! `Events` -- Rust → Angular. A multi-producer notification bus (`Audio
//! Engine` and `Score Metadata Manager` both publish on it) -- see the
//! [events table](../../../docs/architecture.md#events-table).
//! [`TauriEvents`] is the Tauri side of `core::events::EventSink`: domain
//! modules publish through that trait, and this emits the event to the
//! WebView. So far only `Score Metadata Manager` publishes (`score_loaded`,
//! `score_parse_error`); the other payload shapes below wait for their
//! producer (`Audio Engine`, TASK-010).
//!
//! `#![allow(dead_code)]`: the payloads and names nobody emits yet are
//! unused until their producer exists. Remove it once the last one is.
#![allow(dead_code)]

use tauri::{AppHandle, Emitter};

use crate::core::events::EventSink;
use crate::core::score_metadata::{self, Score};

use serde::Serialize;

/// Event names as emitted over the Tauri IPC bridge -- kept as constants so
/// the (future) Rust producer and the Angular consumer's subscription never
/// drift apart on the literal string.
pub mod event_name {
    pub const PLAYBACK_PROGRESS: &str = "playback_progress";
    pub const WAVEFORM_READY: &str = "waveform_ready";
    pub const TRANSPORT_STATE_CHANGED: &str = "transport_state_changed";
    pub const AUDIO_ERROR: &str = "audio_error";
    pub const SCORE_LOADED: &str = "score_loaded";
    pub const SCORE_PARSE_ERROR: &str = "score_parse_error";
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackProgress {
    pub position_sec: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WaveformReady {
    pub stem_id: String,
    /// Amplitude peaks, read from (or computed and written to) the disk
    /// cache -- see docs/architecture.md#design-note-waveform-cache-on-disk.
    pub peaks: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportStateChanged {
    Playing,
    Paused,
    Stopped,
}

/// Reported causes for `audio_error` -- not exhaustive, extend as real
/// failure modes are implemented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioErrorCause {
    DeviceUnavailable,
    DecodeFailed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioError {
    pub cause: AudioErrorCause,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreParseError {
    /// The `.cho` line where parsing failed.
    pub line: u32,
    pub message: String,
}

/// Emits the core's events to the WebView through Tauri.
pub struct TauriEvents {
    app: AppHandle,
}

impl TauriEvents {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    /// A failed emit (the window is gone) has nobody to tell: logged only.
    fn emit<P: Serialize + Clone>(&self, name: &str, payload: P) {
        if let Err(e) = self.app.emit(name, payload) {
            eprintln!("stemloft: couldn't emit {name}: {e}");
        }
    }
}

impl EventSink for TauriEvents {
    fn score_loaded(&self, score: &Score) {
        self.emit(event_name::SCORE_LOADED, score.clone());
    }

    fn score_parse_error(&self, error: &score_metadata::ScoreParseError) {
        self.emit(event_name::SCORE_PARSE_ERROR, ScoreParseError::from(error));
    }
}

impl From<&score_metadata::ScoreParseError> for ScoreParseError {
    fn from(error: &score_metadata::ScoreParseError) -> Self {
        ScoreParseError {
            line: error.line,
            message: error.message.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_parse_error_has_the_events_table_payload() {
        let core_error = score_metadata::ScoreParseError {
            line: 7,
            message: "two chords".into(),
        };
        assert_eq!(
            serde_json::to_value(ScoreParseError::from(&core_error)).unwrap(),
            serde_json::json!({ "line": 7, "message": "two chords" })
        );
    }
}
