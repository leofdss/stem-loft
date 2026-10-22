# Design

## Context

Implements `stem_importer.rs` for TASK-003, following `docs/architecture.md#consistency-across-a-projects-stems`: one sample rate per project, every stem stereo, `durationSec` read from the header. The `import_stems` Command still returns "not yet implemented": knowing which project is open and saving the new entries is wired by TASK-009 on top of `StemImporter::import`.

## Goals / Non-Goals

- Goal: validate stem files, copy them into the project folder (mono upmixed to stereo) and return their `StemEntry`s.
- Non-goal: adding the entries to the open project and marking it dirty (TASK-009).
- Non-goal: formats other than WAV, and computing waveforms (TASK-007).

## Decisions

- **WAV only, read by its own RIFF walker.** `symphonia` (already a dependency, with only `wav`/`pcm` enabled) would decode; the importer only needs the `fmt ` and `data` chunks, so a small chunk walker reads them and nothing else. Integer PCM and IEEE float are accepted, plain or `WAVE_FORMAT_EXTENSIBLE`; unknown chunks are skipped, pad bytes honored.
- **Duration from the header.** `durationSec = (data length / block align) / sample rate`, so no sample is decoded.
- **Upmix copies bytes, never converts.** Each mono frame's bytes are written twice under a canonical 44-byte stereo header, streamed in ~64 KiB chunks, so the result is bit-identical on both channels at any bit depth. A stereo stem is copied byte for byte.
- **More than two channels is an error.** Unlike mono there's no lossless, unambiguous fold to stereo, so it's rejected (`TooManyChannels`), like a sample-rate mismatch.
- **All or nothing.** Every header (and the rate of the project's first existing stem) is checked before any file is written; if a write fails, the files that call already wrote are removed.
- **Ids from file names, never overwriting.** The id is a slug of the file name (`Lead Vocals.wav` → `lead-vocals`), suffixed `-2`, `-3`, ... when it clashes with a stem of the project, another file of the same import, or any file already in the project folder.
- **Atomic, streaming writes.** Stems go through the new `ProjectPersistence::atomic_write_with`, the streaming form of `atomic_write` (temp file + `rename`, temp removed on any failure), so a multi-minute stem never sits whole in memory.
- **Contract-first delivery.** The orchestrator wrote the contract tests, the WAV fixtures (`src-tauri/tests/fixtures/stems/generate.py`) and the streaming write; the Haiku implementer filled in the module against the frozen tests (green in 1 check run); the never-overwrite rule and its test came from the orchestrator's review, and the `review-rust-core` review moved the upmix size check into validation and required a full 40-byte extensible `fmt `.

## Risks / Trade-offs

- A mono WAV whose stereo copy wouldn't fit RIFF's 32-bit sizes (over ~2 GiB of samples) is rejected during validation, before anything is written.
- The upmixed float file has a plain 16-byte `fmt ` and no `fact` chunk; common readers accept it, a strict one might not.
- Rolling back a failed import is best effort: a file that can't be removed stays, and later imports pick another id.
- A trailing partial frame in a mono `data` chunk is dropped by the upmix.
