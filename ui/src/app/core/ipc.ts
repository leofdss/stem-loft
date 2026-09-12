import { Injectable } from '@angular/core';

// Ambient `window.__TAURI__` typing (see tauri-global.d.ts) -- picked up
// automatically by the TypeScript program, nothing to import.

/**
 * `Commands` -- Angular → Rust. A thin wrapper around
 * `window.__TAURI__.core.invoke`, one method per `Command` the Rust core
 * exposes (see `src-tauri/src/ipc/commands.rs` and
 * docs/architecture.md#communication-bridge--tauri-ipc).
 *
 * This service has no queue, dedup, or "is it in progress?" logic of its
 * own -- it fires the `Command` immediately and returns the promise; that
 * intelligence lives entirely in the Rust core (see
 * docs/architecture.md#design-note-commands-queue-in-the-rust-core). A
 * component tracks its own "pending" Signal around a call if it needs to
 * disable a button until the response arrives -- that's presentation of
 * state, not logic, and doesn't belong here either.
 */
@Injectable({ providedIn: 'root' })
export class Ipc {
  isProjectOpen(): Promise<boolean> {
    return window.__TAURI__.core.invoke('is_project_open');
  }

  importStems(filePaths: string[]): Promise<void> {
    return window.__TAURI__.core.invoke('import_stems', { filePaths });
  }

  setMarkers(startSec: number, endSec: number): Promise<void> {
    return window.__TAURI__.core.invoke('set_markers', { startSec, endSec });
  }

  play(): Promise<void> {
    return window.__TAURI__.core.invoke('play');
  }

  pause(): Promise<void> {
    return window.__TAURI__.core.invoke('pause');
  }

  stop(): Promise<void> {
    return window.__TAURI__.core.invoke('stop');
  }

  setStemVolume(stemId: string, volume: number): Promise<void> {
    return window.__TAURI__.core.invoke('set_stem_volume', { stemId, volume });
  }

  setStemMute(stemId: string, muted: boolean): Promise<void> {
    return window.__TAURI__.core.invoke('set_stem_mute', { stemId, muted });
  }

  setStemSolo(stemId: string, solo: boolean): Promise<void> {
    return window.__TAURI__.core.invoke('set_stem_solo', { stemId, solo });
  }
}
