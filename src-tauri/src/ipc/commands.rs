//! `Commands` -- Angular → Rust. Every handler here only translates the
//! Tauri call into a `Session` method and returns its result -- no queue,
//! ordering, or dedup logic on this side (or on Angular's, see
//! docs/architecture.md#design-note-commands-queue-in-the-rust-core). See
//! the `add-ipc-contract` skill before adding a new one.
//!
//! TODO(scaffold): the FIFO `Commands` queue itself isn't implemented yet
//! -- see the TODO on `core::session`'s module doc comment.

use std::path::PathBuf;

use tauri::State;

use crate::core::session::Session;

#[tauri::command]
pub fn is_project_open(session: State<'_, Session>) -> bool {
    session.is_project_open()
}

#[tauri::command]
pub fn import_stems(session: State<'_, Session>, file_paths: Vec<PathBuf>) -> Result<(), String> {
    session.import_stems(file_paths)
}

#[tauri::command]
pub fn set_markers(
    session: State<'_, Session>,
    start_sec: f64,
    end_sec: f64,
) -> Result<(), String> {
    session.set_markers(start_sec, end_sec)
}

#[tauri::command]
pub fn play(session: State<'_, Session>) -> Result<(), String> {
    session.play()
}

#[tauri::command]
pub fn pause(session: State<'_, Session>) -> Result<(), String> {
    session.pause()
}

#[tauri::command]
pub fn stop(session: State<'_, Session>) -> Result<(), String> {
    session.stop()
}

#[tauri::command]
pub fn set_stem_volume(
    session: State<'_, Session>,
    stem_id: String,
    volume: f32,
) -> Result<(), String> {
    session.set_stem_volume(&stem_id, volume)
}

#[tauri::command]
pub fn set_stem_mute(
    session: State<'_, Session>,
    stem_id: String,
    muted: bool,
) -> Result<(), String> {
    session.set_stem_mute(&stem_id, muted)
}

#[tauri::command]
pub fn set_stem_solo(
    session: State<'_, Session>,
    stem_id: String,
    solo: bool,
) -> Result<(), String> {
    session.set_stem_solo(&stem_id, solo)
}
