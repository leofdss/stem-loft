# Design

## Context

Implements `waveform.rs` for TASK-007, following `docs/architecture.md#design-note-waveform-cache-on-disk`: peaks computed once per stem, cached next to the project through the atomic write, recomputed when the stem changes. The `Audio Engine` owns computing peaks; `Project Persistence` owns the write.

## Goals / Non-Goals

- Goal: compute a stem's peaks, cache them, serve them from the cache when it's valid, and invalidate them.
- Non-goal: calling this from Session on import or on open, recording `stems[].waveformCache`, and emitting `waveform_ready`. That wiring belongs with the events work and is criterion 1.6 of TASK-010.

## Decisions

- **Resolution and peak definition.** 100 peaks per second (10 ms buckets); a peak is the largest absolute sample across all channels in its bucket, scaled to 0–1. The last, shorter bucket counts.
- **Decoding by hand, reusing the importer's header reader.** Stems in a project are always WAV (TASK-003), so the module reads samples straight from the `data` chunk with `read_wav_header`, in ~64 KiB chunks: 8-bit unsigned, 16/24/32-bit signed, 32/64-bit float. Any other depth is an error.
- **Self-validating cache file.** `<stem id>.waveform` holds a small header (magic, version, the source's size and modification time, peaks per second, count) and the peaks as f32. Anything that doesn't match, including garbage, an old version or a stem file replaced on disk, is recomputed, never an error.
- **Invalidation.** `invalidate` deletes a stem's cache (missing is fine), for the re-import flow; a stem replaced outside the app is caught by the size/mtime check.
- **The cache is only an optimization.** If it can't be written, the computed peaks are still returned and the next open computes them again.
- **Contract-first delivery.** The orchestrator wrote the contract tests; the Haiku implementer filled in the module and stopped on one failing test, reporting that the test expected one bucket where the rule gives two (two frames at 100 Hz). The orchestrator fixed that test (200 Hz) instead of the code, and added the unwritable-cache rule after review.

## Risks / Trade-offs

- A file's modification time can stay the same across a same-size rewrite on coarse file systems; such a replacement would keep the old peaks until re-import invalidates them.
- Peaks are computed on the calling thread; the caller (TASK-010) must keep that off the real-time thread and, ideally, off the Command queue for long stems.
