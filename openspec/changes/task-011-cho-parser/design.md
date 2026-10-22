# Design

## Context

Implements `score_metadata.rs` for TASK-011: the extended ChordPro dialect in `docs/architecture.md#score-metadata-chords-tablature-and-lyrics` and the `chordpro-format` playbook. The module is read-only in v1 and has no Tauri dependency; the `score_loaded`/`score_parse_error` events that carry its result land with TASK-012.

## Goals / Non-Goals

- Goal: turn `.cho` text into a `Score` (metadata, timed chords, lyric lines, timed tab notes) or a `ScoreParseError` with the 1-based line and a message.
- Non-goal: emitting events or calling the parser from `Session` (TASK-012).
- Non-goal: writing the `.cho` (chord editing is after v1).

## Decisions

- **Tablature is an ASCII staff.** The architecture's tab example is a six-line staff (`E|--5h7--|`), while its grammar writes a note as `fret (op fret | "~")* "." string`. Both hold at once: each run of non-`-` characters on a staff line is one `note`, and its `.string` is the line it sits on (top line = string 1). A `|` inside the staff is a bar line and separates notes like `-` does. A block needs exactly six lines of equal width; a note's column is counted after the leading `<name>|`.
- **An anchor is claimed by the next line.** `{t:}` sets a pending anchor that the next chord/lyrics line or `{start_of_tab}` consumes, whether it follows on the same line or a later one. A `[chord]` with no pending anchor is an error, since its `startSec` couldn't be resolved.
- **Anchors move forward and stay inside the project.** An anchor earlier than the previous one, or past the project's duration (the largest `stems[].durationSec`, passed in by the caller), is a `score_parse_error`. Either would produce a chord whose `endSec` is before its `startSec`.
- **Strict directive values.** `{tempo}`, `{time}`, `{tuning}` (exactly six names) and `{define}` (exactly six frets, 0–24 or `x`/`X`/`N`) are validated; a malformed value is an error on its line. Unknown directives, plus known ones StemLoft doesn't use (`{c:}`, `{capo}`), are ignored.
- **Errors, never panics.** Every helper returns `Result`; numbers use checked parsing, so a huge minute in `{t:}` is an error, not an overflow. A file that can't be read is reported as line 0.
- **Contract-first delivery.** The orchestrator wrote the contract tests, the tab grammar and the resolution; the Haiku implementer filled in the seven directive/line helpers against the frozen tests (green in 2 check runs).

## Risks / Trade-offs

- A tab phrase longer than one staff needs its own `{t:}` + block; there's no multi-system block.
- `{define}` fret entries accept a leading zero (`03`), and a repeated keyword keeps the last value; both are harmless and left lenient.
