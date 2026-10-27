/**
 * `score_loaded`'s payload: the parsed `.cho`, exactly as
 * `Score` in src-tauri/src/core/score_metadata.rs serializes it (camelCase,
 * `None` as `null`). Static data -- it arrives once per parse, never per
 * playback tick (docs/architecture.md#real-time-visual-state-angular).
 */
export interface Score {
  readonly title: string | null;
  readonly artist: string | null;
  readonly key: string | null;
  readonly tempo: number | null;
  readonly time: TimeSignature | null;
  /** Lowest string to highest. */
  readonly tuning: readonly string[] | null;
  /** Bar 1, beat 1: the file's first `{t:}`. */
  readonly gridOriginSec: number | null;
  readonly chords: readonly TimedChord[];
  readonly lines: readonly LyricLine[];
  readonly tabs: readonly TabBlock[];
}

export interface TimeSignature {
  readonly numerator: number;
  readonly denominator: number;
}

/** Fingering diagram for a chord (from the `.cho`'s `{define}`). */
export interface ChordDiagram {
  readonly baseFret: number;
  /** From the lowest to the highest string, 6 positions; "x" = muted string. */
  readonly frets: readonly (number | 'x')[];
}

export interface TimedChord {
  readonly name: string;
  readonly startSec: number;
  readonly endSec: number;
  readonly diagram: ChordDiagram | null;
}

export interface LyricLine {
  readonly startSec: number | null;
  readonly chord: string | null;
  readonly chordColumn: number | null;
  readonly text: string;
}

export interface TabBlock {
  readonly startSec: number;
  readonly endSec: number;
  readonly lines: readonly string[];
  readonly notes: readonly TabNote[];
}

export interface TabNote {
  readonly string: number;
  readonly column: number;
  readonly startSec: number;
  readonly steps: readonly TabStep[];
}

export type TabStep =
  | {
      readonly kind: 'fret' | 'hammerOn' | 'pullOff' | 'bend' | 'release' | 'slideUp' | 'slideDown';
      readonly fret: number;
    }
  | { readonly kind: 'vibrato' };
