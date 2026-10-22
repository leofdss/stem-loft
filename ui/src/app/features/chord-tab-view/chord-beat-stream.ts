import { ChordDiagram, Score } from '../../core/score';

/** Chord currently playing, already resolved with absolute timing. */
export interface ActiveChord {
  readonly name: string; // "G", "Am7", ...
  readonly startSec: number;
  readonly endSec: number;
  readonly diagram: ChordDiagram | null;
}

/** Current beat within the bar grid. */
export interface ActiveBeat {
  readonly bar: number; // bar, 1-based
  readonly beatInBar: number; // 1-based, up to {time}'s numerator
  readonly startSec: number;
  readonly endSec: number;
}

/**
 * What the Chord/Tab View highlights at one playback position -- the
 * `ChordBeatStream` of docs/architecture.md#real-time-visual-state-angular.
 * Recomputed in Angular on every `playback_progress` from the score that
 * arrived once via `score_loaded`: the core only emits the position, so no
 * tick ever carries the whole score.
 */
export interface ChordBeatStream {
  readonly positionSec: number;
  readonly activeChord: ActiveChord | null; // null = section with no chord
  readonly activeBeat: ActiveBeat | null; // null = no bar grid
}

/**
 * The chord and beat active at `positionSec` in `score`.
 *
 * - `activeChord`: the chord whose window holds the position --
 *   `startSec <= positionSec < endSec` -- with its `diagram` (null when the
 *   score has none for it); null when no chord's window holds it. If more
 *   than one does, the one that starts last.
 * - `activeBeat`: null unless the score has `tempo`, `time` and
 *   `gridOriginSec`, and `positionSec >= gridOriginSec` (an intro before
 *   the first `{t:}` has no bar number). Otherwise one beat lasts
 *   `60 / tempo` s; the beat index is `floor((positionSec - gridOriginSec)
 *   / beatSec)` (computed so that a position exactly on a beat boundary
 *   belongs to the beat that starts there); `bar = floor(index /
 *   numerator) + 1`, `beatInBar = index % numerator + 1`, and the beat's
 *   window is `gridOriginSec + index * beatSec` plus one `beatSec`.
 */
export function chordBeatAt(score: Score, positionSec: number): ChordBeatStream {
  // Chord whose window holds the position; on overlap, the one starting last.
  let activeChord: ActiveChord | null = null;
  for (const chord of score.chords) {
    if (chord.startSec <= positionSec && positionSec < chord.endSec) {
      if (activeChord === null || chord.startSec >= activeChord.startSec) {
        activeChord = {
          name: chord.name,
          startSec: chord.startSec,
          endSec: chord.endSec,
          diagram: chord.diagram ?? null,
        };
      }
    }
  }

  // Beat grid: needs tempo, time signature and origin, and a position past the origin.
  let activeBeat: ActiveBeat | null = null;
  const tempo = score.tempo;
  const time = score.time;
  const origin = score.gridOriginSec;
  if (tempo != null && time != null && origin != null && positionSec >= origin) {
    const beatSec = 60 / tempo;
    const index = Math.floor((positionSec - origin) / beatSec + 1e-9);
    const startSec = origin + index * beatSec;
    activeBeat = {
      bar: Math.floor(index / time.numerator) + 1,
      beatInBar: (index % time.numerator) + 1,
      startSec,
      endSec: startSec + beatSec,
    };
  }

  return { positionSec, activeChord, activeBeat };
}
