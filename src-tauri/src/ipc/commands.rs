//! `Commands` -- Angular → Rust. Every handler here only translates the
//! Tauri call into a `Session` method (via `AppState`) and returns its result -- no queue,
//! ordering, or dedup logic on this side (or on Angular's, see
//! docs/architecture.md#design-note-commands-queue-in-the-rust-core). See
//! the `add-ipc-contract` skill before adding a new one.
//!
//! TODO(scaffold): the FIFO `Commands` queue itself isn't implemented yet
//! -- it lands with TASK-009, see the TODO on `core::session`'s module doc
//! comment.

use std::path::PathBuf;

use tauri::State;

use crate::core::app_state::AppState;

#[tauri::command]
pub fn is_project_open(state: State<'_, AppState>) -> bool {
    state.session.is_project_open()
}

#[tauri::command]
pub fn import_stems(state: State<'_, AppState>, file_paths: Vec<PathBuf>) -> Result<(), String> {
    state.session.import_stems(file_paths)
}

#[tauri::command]
pub fn set_markers(state: State<'_, AppState>, start_sec: f64, end_sec: f64) -> Result<(), String> {
    state.session.set_markers(start_sec, end_sec)
}

#[tauri::command]
pub fn play(state: State<'_, AppState>) -> Result<(), String> {
    state.session.play()
}

#[tauri::command]
pub fn pause(state: State<'_, AppState>) -> Result<(), String> {
    state.session.pause()
}

#[tauri::command]
pub fn stop(state: State<'_, AppState>) -> Result<(), String> {
    state.session.stop()
}

#[tauri::command]
pub fn set_stem_volume(
    state: State<'_, AppState>,
    stem_id: String,
    volume: f32,
) -> Result<(), String> {
    state.session.set_stem_volume(&stem_id, volume)
}

#[tauri::command]
pub fn set_stem_mute(
    state: State<'_, AppState>,
    stem_id: String,
    muted: bool,
) -> Result<(), String> {
    state.session.set_stem_mute(&stem_id, muted)
}

#[tauri::command]
pub fn set_stem_solo(
    state: State<'_, AppState>,
    stem_id: String,
    solo: bool,
) -> Result<(), String> {
    state.session.set_stem_solo(&stem_id, solo)
}
