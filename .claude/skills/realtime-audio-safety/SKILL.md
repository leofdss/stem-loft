---
name: realtime-audio-safety
description: Safety checklist for any change touching the Audio Engine or the cpal real-time callback in StemLoft — what may and may not run inside the callback, how crossfade/decoding/volume-mute-solo are resolved outside it. Use before editing any code near cpal, decoding, mixing, or the playback loop.
---

# Real-time thread safety (Audio Engine)

The `cpal` audio callback runs on a real-time thread: **nothing that
allocates, blocks on a lock, or does I/O may run inside it** — a single
underrun is already audible as a glitch. This is a physical constraint, not
a style preference, and it holds even when it seems convenient to violate it
"just this once." Source:
[architecture.md — Audio Engine and the real-time thread](../../../docs/architecture.md#design-note-audio-engine-and-the-real-time-thread).

## The rule in one sentence

**Inside the callback**: only reading from already-ready buffers (ring
buffer / double-buffer), mixing by multiplying/adding already-decoded
samples, and the loop's crossfade (which is also just math over already
decoded samples). Nothing else.

**Outside the callback, ahead of time**: file decoding, buffer allocation,
waveform computation, any disk I/O.

## Checklist before writing code in the `Audio Engine`

- [ ] **Does this read or write a file?** → it has to run outside the
      callback, on a decoding thread, delivering to the callback via `rtrb`
      (SPSC, wait-free) — never disk directly inside the callback.
- [ ] **Does this allocate (`Vec::new()`, `String::new()`, `Box::new()`, a
      non-trivial clone)?** → it can't be on the callback's path.
      Pre-allocate outside and reuse.
- [ ] **Does this `lock()` a `Mutex`?** → it can't be in the callback.
      Volume, mute, solo arrive via an atomic variable or a double-buffer
      published by the `Command` handler outside the callback — never via a
      `Mutex` the callback might have to wait on. See the
      [concurrency model design note](../../../docs/architecture.md#design-note-concurrency-model-for-shared-state)
      to understand the boundary between the two mechanisms (the general
      state's `Mutex` vs. the callback's atomics — different things, don't
      confuse them).
- [ ] **Is this crossfade logic at the loop boundary?** → it needs the
      *start* of the loop (from `startSec`) already in a separate buffer,
      ready *before* the callback reaches the end of the loop — prepared
      outside the callback as soon as the loop is set or restarted. The
      callback only reads the two buffers (the one ending + the one
      starting) and blends by multiplying/adding. Exact duration and curve
      (linear/equal-power) are a free implementation detail — not a pending
      architecture decision. See the
      [full design note](../../../docs/architecture.md#design-note-crossfade-at-the-loop-boundary).
- [ ] **Does this touch channels/sample format?** → every buffer that
      reaches the mixing callback is already stereo (mono→stereo upmixing
      happens at import time, not on playback) and has already been
      validated as having the same sample rate across every stem in the
      project (rejected at import, no silent resampling). `Audio Engine`
      never needs to handle mono or a mismatched sample rate — if your code
      is handling those cases inside the engine, the bug is in import, not
      here.

## Crates, and why they were chosen (don't reopen the decision)

| Responsibility | Crate | Reason (don't re-evaluate without measuring) |
|---|---|---|
| Decoding | `symphonia` | Pure Rust, no native dependency — same code on all 4 platforms (Linux/Windows/macOS/Android) |
| Decode→callback buffer | `rtrb` | SPSC wait-free, an API narrow enough to not slip into a use that violates real-time constraints (`ringbuf` is more general-purpose, not chosen for that reason) |
| Audio output | `cpal` | ALSA/WASAPI/CoreAudio/Oboe behind the same API |

Android point of attention: `cpal`'s Oboe backend needs the JVM
handle/context that Tauri mobile exposes — this is initialization wiring,
not domain logic; don't let this detail leak into the `Audio Engine` itself.

## If you're not sure whether something "counts" as real-time code

The separation is by **thread**, not by file or module — the `Audio
Engine` has code on both sides. Ask: "does this run inside the closure
`cpal` calls on every audio buffer, or does it run before that, preparing
data for it to read later?" If you don't know, assume it's inside and apply
the constraint — the cost of erring on the safe side is just a bit more
pre-computed buffer; the cost of erring the other way is an audible glitch
in production.
