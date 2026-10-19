---
name: atomic-persistence
description: Disk-write pattern for Project Persistence in StemLoft — write-to-temp-file + rename with no exceptions, when to write (a debounced checkpoint, not on every Command), and cache invalidation. Use when adding any new disk write (project .json, waveform cache, future .cho) or when touching when/how the project is saved.
---

# Persistence: atomic writes and checkpoints

## The rule, no exceptions

**Every** disk write made by `Project Persistence` is atomic: it writes to a
temporary file in the same directory as the destination, then `rename`s it
to the final path. `rename` is atomic at the file-system level — the
previous file is never left truncated or partially written if the app
crashes mid-write. Worst case possible: losing the write in progress, never
corrupting what already existed. This applies to the project's `.json`, to
the waveform cache, and will apply to the `.cho` once chord writing lands
(post-v1). See the
[full design note](../../../docs/architecture.md#design-note-when-project-persistence-writes).

If you're writing a `File::write`/`std::fs::write` directly to the final
path of something `Project Persistence` owns, stop — that's the antipattern
this decision exists to prevent.

## When to write (not "on every change")

`Stem Mixer` generates `Commands` on every tick of a fader being dragged —
writing on every one of them would be wasted I/O and a risk of
concurrent/partial writes. The pattern:

- A state change marks the project "dirty," it doesn't trigger an immediate
  write.
- The actual write happens at **checkpoints**: an inactivity debounce (a few
  ms with no new `Commands`) or definitive events (pausing playback, closing
  the project).
- **Reading stays immediate** — only the write is batched.

If you're adding a Command that changes persisted state, don't invent your
own checkpoint for it — use the existing dirty-flag + debounce mechanism. A
checkpoint per feature is the same mistake as "a `Mutex` per feature" in the
core: it fragments a mechanism that should be a single one.

## Waveform cache (a concrete application of the pattern)

- Computed once per stem (decoding the whole thing is expensive — perf on
  weak hardware is a project requirement, not a nice-to-have).
- Written to disk alongside the project, the `stems[].waveformCache` field
  in the `.json`, with the same atomic write as any other file.
- **Invalidated if the stem is re-imported/replaced** — if you're touching
  the re-import flow and not discarding the old cache, that's a bug: the
  displayed waveform would drift out of sync with the actual audio.
- See the [full design note](../../../docs/architecture.md#design-note-waveform-cache-on-disk).

## Versioning the project's `.json`

The project's `.json` does **not** have the tolerance for unknown fields
that the `.cho` has (unknown ChordPro directives are just ignored by other
readers). That's why `schemaVersion` is read before anything else:

- Same version → loads directly.
- Lower version → applies registered migrations in sequence (each one knows
  how to transform `N` → `N+1`) before exposing the project to the rest of
  the core.
- Higher version than supported → an explicit error ("project saved by a
  newer version of the app"), **never** a silent attempt at a partial read.

If you're adding a new field to the `.json` schema, ask: does this break
reading a project saved by an earlier app version? If so, it needs a
registered migration — it's not optional, even if it looks like "just one
more field." See the
[full schema and example](../../../docs/architecture.md#project-versioning).

## Quick checklist

- [ ] Does the write go through write-to-temp-file + rename, with no
      exceptions?
- [ ] Is the write batched into a checkpoint (dirty flag + debounce or a
      definitive event), not fired on every `Command`?
- [ ] If this invalidates an existing cache (waveform, and others in the
      future), is the invalidation code in the right place
      (re-import/replacement)?
- [ ] If the `.json` schema changed in an incompatible way, is there a
      registered `N → N+1` migration?
