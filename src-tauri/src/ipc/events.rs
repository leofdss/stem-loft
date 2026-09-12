//! `Events` -- Rust → Angular. A multi-producer notification bus (`Audio
//! Engine` and `Score Metadata Manager` both publish on it) -- see the
//! [events table](../../../docs/architecture.md#events-table). Not emitted
//! by anything yet in this scaffold: these are the payload shapes the real
//! producers will send once each one exists (`Audio Engine::play` and
//! friends, `ScoreMetadataManager::load_score`).
//!
//! `score_loaded`'s payload (parsed chords, tablature, lyrics, plus
//! tempo/time/tuning) isn't modeled here yet -- it lands together with
//! `ScoreMetadataManager::load_score`, see
//! docs/architecture.md#score-metadata-chords-tablature-and-lyrics and the
//! `chordpro-format` skill.
//!
//! `#![allow(dead_code)]`: every item below is unused until its producer
//! exists -- see the module doc above. Remove this once the first one
//! (`event_name::PLAYBACK_PROGRESS`, most likely) is actually emitted.
#![allow(dead_code)]

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
