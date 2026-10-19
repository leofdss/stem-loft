---
name: chordpro-format
description: Grammar and validation rules for the extended ChordPro dialect (.cho) used by StemLoft's Score Metadata Manager — {t:} anchors, one chord per line, tablature grammar, startSec/endSec resolution. Use when implementing or testing the .cho parser, or when creating/editing example/test .cho files.
---

# `.cho` format (extended ChordPro)

StemLoft uses standard [ChordPro](https://www.chordpro.org/) plus two
custom extensions and one dialect restriction. Full source, with examples:
[architecture.md — Score metadata](../../../docs/architecture.md#score-metadata-chords-tablature-and-lyrics).
This skill is the operational summary for implementing/testing the parser
without having to reconstruct the rules every time.

## The two extensions

- **`{tuning: E A D G B E}`** — tuning, lowest string→highest, same order as
  `frets` in `{define}`.
- **`{t: minute:second.hundredth}`** — absolute time anchor, start of line.
  Exact grammar: `minute` 1+ digits with no leading zero required; `second`
  always 2 digits (00–59); `hundredth` always 2 digits (00–99). Valid:
  `0:00.00`, `1:05.30`, `12:40.00`. Invalid: `00.5`, `1:5.3` (missing
  digits).

Unknown directives (including these two, for any ChordPro reader that isn't
StemLoft) are ignored, not an error — this is the format's expected
extension mechanism. Don't treat `{tuning}`/`{t}` as required for a file to
be "valid" outside our parser.

## Dialect rule: at most one chord per line

`{t:}` only anchors the *start* of the line — a second `[chord]` on the
same line wouldn't have its own timestamp. **The parser must reject
(`score_parse_error`)** a line with more than one `[chord]`. This is a
mandatory validation, not a style suggestion.

## Tablature grammar (`{start_of_tab}` … `{end_of_tab}`)

```
item        := note ("-" note)* | "-"        ; "-" alone = rest
note        := fret (op fret | "~")* "." string
op          := "h" | "p" | "b" | "r" | "/" | "\"
fret        := digit digit?                  ; 0-24, no leading zero
string      := "1" | "2" | "3" | "4" | "5" | "6"
digit       := "0".."9"
```

`~` (vibrato) is the only operator with no target fret — it isn't followed
by a digit. Valid example: `8~~b10r8` = `fret=8`, two `~` in a row, then
`b` `10` `r` `8`.

**Validation rules the parser must reject/warn on:**

| Rule | Reason |
|---|---|
| `r` is only valid after at least one `b` in the same note | A release with no preceding bend has nothing to release from |
| Two notes in the same `item` (separated by `-`) can't point to the same string | Physically, a string can only sound one pitch at a time |
| `fret` outside 0–24 | The neck's physical limit |
| `string` outside 1–6 | 6-string tuning |

Technique table (meaning reference, not grammar):

| Symbol | Technique | Example |
|---|---|---|
| `h` | Hammer-on | `5h7` |
| `p` | Pull-off | `7p5` |
| `b` | Bend | `7b9` |
| `r` | Release | `7b9r7` |
| `/` | Slide up | `5/7` |
| `\` | Slide down | `7\5` |
| `~` | Vibrato | `8~` |

## Resolving `startSec`/`endSec`

- `startSec` of a chord = the `{t:}` of the line where the `[chord]`
  appears.
- `endSec` = the `{t:}` of the **next event that changes what's sounding**
  (the next `[chord]` or the next `{start_of_tab}`). A lyric line **with
  no** bracket doesn't end the current chord — it only advances the
  displayed text.
- **Last chord in the file**: there's no "next event" — `endSec` = the
  project's largest `stems[].durationSec`, read from the `.json`, **not**
  decoded on the spot. To close it before the real end, the `.cho`'s author
  adds a final line with just `{t:}` + `[chord]` (can repeat the same
  name).
- **Bar-grid origin** (bar 1, beat 1) = the **first `{t:}` in the file**,
  not necessarily second 0 of the audio. The section before that
  (intro/count-in) falls outside the bar numbering — that's expected, not a
  bug.
- **Time inside `{start_of_tab}`**: a single `{t:}` anchor at the start of
  the block; each note's position is proportional to the character's column
  on the line:
  `noteSec = startSec + (column / totalColumns) × (endSec − startSec)`,
  using the same `endSec` (next event) and the line's length (all 6 strings
  have the same number of columns). No anchor needed per note — if your
  code is looking for a `{t:}` inside a tab block per note, it's solving
  this the wrong way.

## Checklist when implementing/changing the parser

- [ ] A line with more than one `[chord]` → `score_parse_error`, not the
      first occurrence silently accepted.
- [ ] A malformed `{t:}` (missing digits in second/hundredth) →
      `score_parse_error` with the file line where it occurred — the error
      message needs to point at the line, not just say "invalid file."
- [ ] A `.cho` parse error **doesn't halt the rest of the app** — stems,
      loop, waveform keep working normally; only `Chord/Tab View` sits in
      an error state. If your code propagates the error to something that
      affects playback, that's a bug.
- [ ] An unknown directive is ignored, not rejected.
- [ ] The last chord's `endSec` comes from `stems[].durationSec` (already
      persisted, no audio decoding) — not from the Audio Engine directly.
