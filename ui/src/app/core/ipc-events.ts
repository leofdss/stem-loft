import { Injectable, signal } from '@angular/core';

/** `audio_error`'s payload -- see the events table in docs/architecture.md. */
export interface AudioErrorPayload {
  readonly cause: 'device_unavailable' | 'decode_failed';
  readonly message: string;
}

/** `score_parse_error`'s payload -- see the events table in docs/architecture.md. */
export interface ScoreParseErrorPayload {
  readonly line: number;
  readonly message: string;
}

/**
 * `Events` -- Rust → Angular. Subscribes once (from `App`, at startup) to
 * the two error events and exposes them as Signals `ErrorModal` renders.
 *
 * Not a second source of truth: this service only relays what the core
 * already decided and published -- it never derives or guesses state on
 * its own (see docs/architecture.md#stack-and-platform, "Angular ...
 * projection/cache of what Rust has already decided").
 *
 * `playback_progress`, `waveform_ready`, `transport_state_changed`, and
 * `score_loaded` aren't wired here yet -- nothing in the Rust core emits
 * them yet either (`Audio Engine`/`Score Metadata Manager` are still
 * stubs). Add them here, the same way, once their producer exists.
 */
@Injectable({ providedIn: 'root' })
export class IpcEvents {
  private readonly _audioError = signal<AudioErrorPayload | null>(null);
  private readonly _scoreParseError = signal<ScoreParseErrorPayload | null>(null);

  readonly audioError = this._audioError.asReadonly();
  readonly scoreParseError = this._scoreParseError.asReadonly();

  /**
   * Call once, from the app root, to start listening. A no-op outside a
   * Tauri WebView (a plain browser preview, or a unit test) -- `__TAURI__`
   * only exists once `app.withGlobalTauri` injects it at runtime.
   */
  async listen(): Promise<void> {
    if (typeof window === 'undefined' || !window.__TAURI__) {
      return;
    }

    await window.__TAURI__.event.listen<AudioErrorPayload>('audio_error', (event) => {
      this._audioError.set(event.payload);
    });
    await window.__TAURI__.event.listen<ScoreParseErrorPayload>(
      'score_parse_error',
      (event) => {
        this._scoreParseError.set(event.payload);
      },
    );
  }

  /** Presentation-only: dismissing the modal doesn't touch the core. */
  dismissAudioError(): void {
    this._audioError.set(null);
  }

  dismissScoreParseError(): void {
    this._scoreParseError.set(null);
  }
}
