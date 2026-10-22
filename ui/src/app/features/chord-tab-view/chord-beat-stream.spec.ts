import { Score } from '../../core/score';
import { chordBeatAt } from './chord-beat-stream';

// The contract for TASK-012's chord/beat slice
// (openspec/changes/task-012-score-events). Written before the
// implementation, by a different author than it -- never loosen one of
// these to make it pass (AGENTS.md, testing policy).

/** The architecture's chords example, as the core serializes it: 80 BPM, 4/4. */
const SCORE: Score = {
  title: 'Study in G Major',
  artist: null,
  key: 'G',
  tempo: 80,
  time: { numerator: 4, denominator: 4 },
  tuning: ['E', 'A', 'D', 'G', 'B', 'E'],
  gridOriginSec: 0,
  chords: [
    { name: 'G', startSec: 0, endSec: 3, diagram: { baseFret: 1, frets: [3, 2, 0, 0, 0, 3] } },
    { name: 'C', startSec: 3, endSec: 6, diagram: { baseFret: 1, frets: ['x', 3, 2, 0, 1, 0] } },
    { name: 'D', startSec: 6, endSec: 9, diagram: null },
    { name: 'G', startSec: 9, endSec: 12, diagram: { baseFret: 1, frets: [3, 2, 0, 0, 0, 3] } },
  ],
  lines: [],
  tabs: [],
};

describe('chordBeatAt', () => {
  it('returns the position it was asked about', () => {
    expect(chordBeatAt(SCORE, 4.2).positionSec).toBe(4.2);
  });

  it('picks the chord whose window holds the position', () => {
    expect(chordBeatAt(SCORE, 0).activeChord?.name).toBe('G');
    expect(chordBeatAt(SCORE, 2.99).activeChord?.name).toBe('G');
    expect(chordBeatAt(SCORE, 3).activeChord).toEqual(SCORE.chords[1]);
    expect(chordBeatAt(SCORE, 7.5).activeChord).toEqual({
      name: 'D',
      startSec: 6,
      endSec: 9,
      diagram: null,
    });
    expect(chordBeatAt(SCORE, 11.9).activeChord?.startSec).toBe(9);
  });

  it('has no chord before the first one or after the last one ends', () => {
    const late: Score = { ...SCORE, chords: SCORE.chords.slice(1) };
    expect(chordBeatAt(late, 1).activeChord).toBeNull();
    expect(chordBeatAt(SCORE, 12).activeChord).toBeNull();
  });

  it('counts beats and bars from the grid origin at the score tempo', () => {
    // 80 BPM: one beat = 0.75 s, one 4/4 bar = 3 s.
    expect(chordBeatAt(SCORE, 0).activeBeat).toEqual({
      bar: 1,
      beatInBar: 1,
      startSec: 0,
      endSec: 0.75,
    });
    expect(chordBeatAt(SCORE, 2.3).activeBeat).toEqual({
      bar: 1,
      beatInBar: 4,
      startSec: 2.25,
      endSec: 3,
    });
    // Exactly on a boundary belongs to the beat that starts there.
    expect(chordBeatAt(SCORE, 3).activeBeat).toEqual({
      bar: 2,
      beatInBar: 1,
      startSec: 3,
      endSec: 3.75,
    });
    expect(chordBeatAt(SCORE, 6.75).activeBeat?.bar).toBe(3);
    expect(chordBeatAt(SCORE, 6.75).activeBeat?.beatInBar).toBe(2);
  });

  it('uses the time signature numerator for the bar length', () => {
    const waltz: Score = { ...SCORE, tempo: 60, time: { numerator: 3, denominator: 4 } };
    expect(chordBeatAt(waltz, 3.5).activeBeat).toEqual({
      bar: 2,
      beatInBar: 1,
      startSec: 3,
      endSec: 4,
    });
  });

  it('has no beat before the grid origin or without a full bar grid', () => {
    const intro: Score = { ...SCORE, gridOriginSec: 2.5 };
    expect(chordBeatAt(intro, 1).activeBeat).toBeNull();
    expect(chordBeatAt(intro, 2.5).activeBeat).toEqual({
      bar: 1,
      beatInBar: 1,
      startSec: 2.5,
      endSec: 3.25,
    });

    expect(chordBeatAt({ ...SCORE, tempo: null }, 1).activeBeat).toBeNull();
    expect(chordBeatAt({ ...SCORE, time: null }, 1).activeBeat).toBeNull();
    expect(chordBeatAt({ ...SCORE, gridOriginSec: null }, 1).activeBeat).toBeNull();
  });

  it('never returns more than the slice for one tick', () => {
    expect(Object.keys(chordBeatAt(SCORE, 1)).sort()).toEqual([
      'activeBeat',
      'activeChord',
      'positionSec',
    ]);
  });
});
