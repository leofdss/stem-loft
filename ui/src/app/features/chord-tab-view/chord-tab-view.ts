import { Component } from '@angular/core';

/**
 * Chords, tablature, and lyrics parsed from the project's `.cho`, synced
 * with playback. `score` is optional on a project -- the main feature
 * (loop via markers) doesn't depend on one, so this component must handle
 * "no score" without breaking anything else on screen. See
 * docs/architecture.md#real-time-visual-state-angular.
 *
 * Still an empty-state placeholder: nothing emits `score_loaded` yet
 * (`Score Metadata Manager::load_score` is a stub, see
 * `src-tauri/src/core/score_metadata.rs`). When it's implemented, this
 * component subscribes to `score_loaded` (static, once) and derives
 * `ChordBeatStream` (`activeChord`/`activeBeat`) from `playback_progress`
 * on every tick -- never re-requesting the static score data per tick, see
 * docs/architecture.md#real-time-visual-state-angular.
 */
@Component({
  selector: 'stemloft-chord-tab-view',
  templateUrl: './chord-tab-view.html',
  styleUrl: './chord-tab-view.css',
})
export class ChordTabView {}
