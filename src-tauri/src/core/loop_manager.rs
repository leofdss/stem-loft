//! `Loop & Marker Manager` -- holds the start/end marker for the looped
//! section and feeds it to the `Audio Engine`. See the "Playback with
//! marker loop" flow in docs/architecture.md.

use std::sync::Mutex;

/// The active loop's bounds, in seconds.
#[derive(Debug, Clone, Copy, Default)]
pub struct LoopMarkers {
    pub start_sec: f64,
    pub end_sec: f64,
}

pub struct LoopManager {
    // Its own small lock, not the giant lock some other design would put
    // around the whole app: `Audio Engine` will need to read these bounds
    // independently of whatever calls into `Session` (see
    // docs/architecture.md#design-note-concurrency-model-for-shared-state --
    // "a single Arc<Mutex<AppState>> (or at most one Mutex per module)").
    markers: Mutex<LoopMarkers>,
}

impl LoopManager {
    pub fn new() -> Self {
        Self {
            markers: Mutex::new(LoopMarkers::default()),
        }
    }

    pub fn markers(&self) -> LoopMarkers {
        *self
            .markers
            .lock()
            .expect("LoopManager.markers mutex poisoned")
    }

    /// Sets the loop's start/end markers.
    ///
    /// Still to implement: validate `end_sec` against the project's
    /// duration (the largest `stems[].durationSec` among the project's
    /// stems) -- a marker beyond that must be rejected with an explicit
    /// error, never silently truncated. See
    /// docs/architecture.md#consistency-across-a-projects-stems. That
    /// validation needs the project's stems, which `Stem Importer` doesn't
    /// expose yet.
    pub fn set_markers(&self, _start_sec: f64, _end_sec: f64) -> Result<(), String> {
        Err("set_markers: not yet implemented (needs the project's duration to validate against)"
            .into())
    }
}
