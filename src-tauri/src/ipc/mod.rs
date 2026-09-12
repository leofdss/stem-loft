//! Communication Bridge -- Tauri IPC. See
//! docs/architecture.md#communication-bridge--tauri-ipc. `commands` is
//! Angular → Rust, `events` is Rust → Angular. This is the one layer
//! allowed to know about Tauri's API; no `core` module may depend on it
//! (docs/architecture.md#design-note-patterns-familiar-to-people-coming-from-typescript,
//! "no domain module depends on Tauri or Angular API details").

pub mod commands;
pub mod events;
