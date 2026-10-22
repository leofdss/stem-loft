import { Component } from '@angular/core';

/**
 * Chords, tablature, and lyrics parsed from the project's `.cho`, synced
 * with playback. `score` is optional on a project -- the main feature
 * (loop via markers) doesn't depend on one, so this component must handle
 * "no score" without breaking anything else on screen. See
 * docs/architecture.md#real-time-visual-state-angular.
 *
 * Still an empty-state placeholder. The pieces exist: `IpcEvents.score`
 * relays `score_loaded` (static, once per parse) and `chordBeatAt` derives
 * `ChordBeatStream` (`activeChord`/`activeBeat`) for one position. Once
 * `playback_progress` is emitted (TASK-010), this component recomputes it
 * on every tick -- never re-requesting the static score data per tick, see
 * docs/architecture.md#real-time-visual-state-angular.
 */
@Component({
  selector: 'stemloft-chord-tab-view',
  templateUrl: './chord-tab-view.html',
  styleUrl: './chord-tab-view.css',
})
export class ChordTabView {}
