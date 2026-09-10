---
name: add-ipc-contract
description: Roadmap for adding or changing a Command (Angular → Rust) or Event (Rust → Angular) in StemLoft, respecting the decisions already settled in docs/architecture.md (queue in the core, Session as a thin router, Signals with no NgRx, no optimistic updates). Use whenever exposing a new UI action, a new state to report, or asking "where does this Command/Event belong."
---

# Adding a Command or Event

This is the most common entry point for work in StemLoft: almost every new
feature involves adding a `Command` (Angular → Rust) and/or an `Event` (Rust
→ Angular). The design decisions that govern this are already settled —
this skill exists so you don't have to re-decide anything, just follow the
roadmap.

Source of truth: [`docs/architecture.md`](../../../docs/architecture.md), the
"Communication Bridge — Tauri IPC" section and the design notes linked to
it. If anything here diverges from `architecture.md`, **`architecture.md`
wins**, and you should update this skill — not the other way around.

## Before you start: where the logic lives

**Never** put decision logic in Angular. Angular only presents state and
sends intent — no queue, dedup, ordering, or business rule on its side
([design note](../../../docs/architecture.md#design-note-commands-queue-in-the-rust-core)).

**Never** put new logic in `Session/State Manager`. The Session is a thin
router — it only knows "which project is open" and dispatches to the module
that owns that domain
([design note](../../../docs/architecture.md#design-note-session-as-a-thin-router)).
Question to decide where the handler lives: **which domain module owns this
information?** (`Audio Engine`, `Loop Manager`, `Stem Importer`, `Project
Persistence`, `Score Metadata Manager`). If the answer is "more than one,
and something needs to coordinate them with its own logic," that's a sign a
new module is missing — not that the Session should grow.

## Step by step: adding a Command

1. **Choose the Command's name** in English, imperative-verb style
   consistent with the existing ones (`import_stems`, `set_markers`).
2. **The handler lives in the owning domain module**, not in the Session —
   the Session only dispatches if the handler needs to know which project is
   active.
3. **Don't implement your own queue, dedup, or "is it processing?" logic on
   the Rust side for this specific Command** — that's already core-wide
   behavior (FIFO processing of the Commands queue), not something each
   handler reimplements.
4. **The Command's response is the state confirmation.** Angular doesn't
   apply the change before the response arrives — no optimistic updates.
   The only immediate feedback on click is the control becoming disabled
   until that specific Command's response arrives (preventing a double
   click) — that's the UI reflecting "in progress," not queue logic.
5. If the Command touches project state that needs to survive a restart,
   check the `atomic-persistence` skill — don't write to disk directly from
   the handler.
6. If the Command touches `Audio Engine` or the `cpal` callback, stop and
   load the `realtime-audio-safety` skill before coding.

## Step by step: adding an Event

1. Check the [events table](../../../docs/architecture.md#events-table) —
   confirm there isn't already an event covering the same data.
2. **Separate static data from data that changes every tick.** This is the
   same lesson that produced `waveform_ready` separate from
   `playback_progress`: a payload computed once (waveform, parsed score)
   shouldn't be re-dispatched on every progress event — that's redundant
   IPC. If your new Event looks like "the same thing again with one more
   field," it probably should instead be a field derived in Angular from an
   Event that already exists (see `ChordBeatStream` in
   [Real-time visual state](../../../docs/architecture.md#real-time-visual-state-angular)
   as an example: recomputed on the client by cross-referencing
   `positionSec` with already-loaded data, not a new Event per tick).
3. **Errors are dedicated Events**, not an error field inside a
   success event — follow the `audio_error`/`score_parse_error` pattern:
   cause + message, consumed by the UI as an error state localized to the
   affected component, never halting the rest of the app.
4. **Error presentation is always a modal** — not a toast, not a persistent
   banner
   ([design note](../../../docs/architecture.md#design-note-error-presentation-modal)).
   The modal only covers presentation: it doesn't pause or undo anything
   already running in the core.
5. Document the Event in `architecture.md`'s events table (producer,
   essential payload, consumer) — it's the single inventory of every Event,
   keep it in sync.

## Self-check checklist before considering it done

Reread the diff and answer "no" to all of these, or go back and fix it:

- [ ] Is there any queue, dedup, "last response" cache, or decision logic on
      the Angular side? → move it to Rust.
- [ ] Did `Session Manager` gain a new business rule (not just dispatch)? →
      move it to the right domain module.
- [ ] Does the UI apply the new state before the Command's response
      confirms it? → remove the optimistic update.
- [ ] Is there a new Event carrying data that's already derivable from an
      existing Event + state already loaded on the client? → derive it in
      Angular instead of dispatching it again.
- [ ] Is an error being shown as a toast/banner, or mixed into a success
      event? → separate it into an error Event + modal.
- [ ] Did `architecture.md`'s events table fall out of sync? → update it.

If any answer is "yes," fix it before requesting review — don't leave for
the `rust-core-reviewer`/`angular-shell-reviewer` agent something you can
solve yourself just by following this checklist.
