//! Logic Core -- see docs/architecture.md#logic-core--rust. Every module
//! here is a plain struct with an explicit `new(...)` constructor (manual
//! dependency injection, no DI container/service locator) and has no
//! dependency on Tauri or Angular API details to make a domain decision --
//! see docs/architecture.md#design-note-patterns-familiar-to-people-coming-from-typescript
//! and the `rust-domain-module` skill.

pub mod audio_engine;
pub mod loop_manager;
pub mod persistence;
pub mod score_metadata;
pub mod session;
pub mod stem_importer;
