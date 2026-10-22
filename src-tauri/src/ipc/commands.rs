//! `Commands` -- Angular → Rust. Every handler here only translates the
//! Tauri call into a `Session` method and returns its result -- no queue,
//! ordering, or dedup logic of its own (or on Angular's side, see
//! docs/architecture.md#design-note-commands-queue-in-the-rust-core). See
//! the `add-ipc-contract` skill before adding a new one.
//!
//! Each handler is `async` and runs its `Session` call on a blocking
//! thread through [`queued`], inside `AppState.queue`: the core's FIFO
//! `Commands` queue. So a slow `Command` (copying stems) never blocks the
//! window's main thread, and two `Commands` never run at the same time or
//! out of the order they reached the core.

use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::core::app_state::AppState;
use crate::core::persistence::{Project, StemEntry};
use crate::core::session::Session;

/// Runs `command` against the `Session` through the core's queue, off the
/// main thread.
async fn queued<T: Send + 'static>(
    app: AppHandle,
    command: impl FnOnce(&Session) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        state.queue.run(|| command(&state.session))
    })
    .await
    .map_err(|e| format!("the command failed unexpectedly: {e}"))?
}

#[tauri::command]
pub async fn is_project_open(app: AppHandle) -> Result<bool, String> {
    queued(app, |session| Ok(session.is_project_open())).await
}

#[tauri::command]
pub async fn create_project(app: AppHandle, dir: PathBuf, name: String) -> Result<Project, String> {
    queued(app, move |session| session.create_project(&dir, &name)).await
}

#[tauri::command]
pub async fn open_project(app: AppHandle, path: PathBuf) -> Result<Project, String> {
    queued(app, move |session| session.open_project(&path)).await
}

#[tauri::command]
pub async fn close_project(app: AppHandle) -> Result<(), String> {
    queued(app, |session| session.close_project()).await
}

#[tauri::command]
pub async fn import_stems(
    app: AppHandle,
    file_paths: Vec<PathBuf>,
) -> Result<Vec<StemEntry>, String> {
    queued(app, move |session| session.import_stems(file_paths)).await
}

#[tauri::command]
pub async fn set_markers(app: AppHandle, start_sec: f64, end_sec: f64) -> Result<(), String> {
    queued(app, move |session| session.set_markers(start_sec, end_sec)).await
}

#[tauri::command]
pub async fn play(app: AppHandle) -> Result<(), String> {
    queued(app, |session| session.play()).await
}

#[tauri::command]
pub async fn pause(app: AppHandle) -> Result<(), String> {
    queued(app, |session| session.pause()).await
}

#[tauri::command]
pub async fn stop(app: AppHandle) -> Result<(), String> {
    queued(app, |session| session.stop()).await
}

#[tauri::command]
pub async fn set_stem_volume(app: AppHandle, stem_id: String, volume: f32) -> Result<(), String> {
    queued(app, move |session| {
        session.set_stem_volume(&stem_id, volume)
    })
    .await
}

#[tauri::command]
pub async fn set_stem_mute(app: AppHandle, stem_id: String, muted: bool) -> Result<(), String> {
    queued(app, move |session| session.set_stem_mute(&stem_id, muted)).await
}

#[tauri::command]
pub async fn set_stem_solo(app: AppHandle, stem_id: String, solo: bool) -> Result<(), String> {
    queued(app, move |session| session.set_stem_solo(&stem_id, solo)).await
}
