//! `Audio Engine` -- decodes, mixes, and plays back the stems; applies the
//! marked loop and the volume/mute/solo state; emits playback/waveform/error
//! events (once wired). See
//! docs/architecture.md#design-note-audio-engine-and-the-real-time-thread.
//!
//! Read the `realtime-audio-safety` skill in full before writing the real
//! `play`/`pause`/`stop` implementations below -- the short version: the
//! `cpal` callback this module will eventually own may never allocate,
//! lock a `Mutex`, or touch disk. Decoding (`symphonia`) happens ahead of
//! time on its own thread, handed to the callback through an `rtrb` ring
//! buffer; volume/mute/solo reach the callback through atomics/a
//! double-buffer, never through a lock.

use std::sync::Arc;

use super::loop_manager::LoopManager;

pub struct AudioEngine {
    // Not read anywhere yet -- `play` below doesn't start real playback,
    // so there's no callback yet that needs the active loop bounds. Kept
    // explicit in the constructor because the real implementation reads it
    // on every callback (see the module doc comment).
    #[allow(dead_code)]
    loop_manager: Arc<LoopManager>,
}

impl AudioEngine {
    pub fn new(loop_manager: Arc<LoopManager>) -> Self {
        Self { loop_manager }
    }

    /// Starts playback of the current project's stems, honoring the active
    /// loop and mixer state. Not yet implemented -- see the
    /// `realtime-audio-safety` skill before writing this: stems need to be
    /// decoded ahead of time into an `rtrb` ring buffer the `cpal` callback
    /// reads from, and the loop's start needs to be pre-loaded into its own
    /// buffer before the callback reaches `end_sec`, for the crossfade (see
    /// docs/architecture.md#design-note-crossfade-at-the-loop-boundary).
    pub fn play(&self) -> Result<(), String> {
        Err("play: not yet implemented".into())
    }

    pub fn pause(&self) -> Result<(), String> {
        Err("pause: not yet implemented".into())
    }

    pub fn stop(&self) -> Result<(), String> {
        Err("stop: not yet implemented".into())
    }

    /// Still to implement: published as an atomic/double-buffer value the
    /// `cpal` callback reads -- never behind a `Mutex` the callback might
    /// have to wait on.
    pub fn set_stem_volume(&self, _stem_id: &str, _volume: f32) -> Result<(), String> {
        Err("set_stem_volume: not yet implemented".into())
    }

    pub fn set_stem_mute(&self, _stem_id: &str, _muted: bool) -> Result<(), String> {
        Err("set_stem_mute: not yet implemented".into())
    }

    pub fn set_stem_solo(&self, _stem_id: &str, _solo: bool) -> Result<(), String> {
        Err("set_stem_solo: not yet implemented".into())
    }
}
