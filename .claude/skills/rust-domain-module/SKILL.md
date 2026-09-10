---
name: rust-domain-module
description: How to structure a new domain module (or a new struct within an existing module) in StemLoft's Rust core — a constructor with manual dependency injection, no DI container, no unnecessary generics/trait objects/macros. Use when creating a new Rust module, a new domain struct, or when deciding "is this too idiomatic?"
---

# Structuring a domain module in the Rust core

StemLoft's Rust core is written by people coming from TypeScript
(Angular/Nest.js), not from a Rust background. The golden rule, documented in
[`architecture.md`](../../../docs/architecture.md#design-note-patterns-familiar-to-people-coming-from-typescript):
**"understandable coming from Angular/Nest.js without learning advanced Rust
first"** outweighs "idiomatic by Rust community standards." This only gives
way if performance is **significantly** affected, and measured — not as the
default choice.

## The mental parallel (use it to decide, not just to explain)

- **Domain module ≈ Nest.js injectable service.** A `struct` with public
  methods and a `new(...)` that receives its dependencies explicitly — just
  like `@Injectable()` receiving them in its constructor, except with no
  container: the "injection" is passing the `Arc<...>` values by hand, once,
  when initializing `AppState`.
- **`Session/State Manager` ≈ Nest.js controller.** Receives, dispatches to
  the service (domain module), returns — never accumulates its own business
  rules. See the `add-ipc-contract` skill to decide where a new handler
  should live.

## Checklist when writing the struct

1. **Explicit constructor, no DI framework.**
   `pub fn new(dep_a: Arc<DepA>, dep_b: Arc<DepB>) -> Self`. No
   `#[derive(Inject)]`, service locator, or dynamic registration.
2. **Prefer explicit and repetitive over "clever."** Heavy generics, macros,
   excessive trait objects, elaborate lifetimes: only if both options solve
   the same problem with the same clarity AND the simple version is
   demonstrably too slow. When in doubt, write the boring version first.
3. **Concurrency: the model simplest to reason about, not the theoretically
   fastest.** A module's shared state sits behind a single
   `Arc<Mutex<...>>` (or, at most, one `Mutex` per module — never one per
   feature). `lock() → change what's needed → release the lock`. Only
   consider channels, actors, or finer-grained locking if profiling shows
   real contention — not before, not as speculative optimization. See the
   [full design note](../../../docs/architecture.md#design-note-concurrency-model-for-shared-state).
4. **The `cpal` real-time thread never touches this `Mutex`.** If the
   module you're writing has any relationship to the audio callback, stop
   and load the `realtime-audio-safety` skill — the concurrency rule above
   doesn't apply in there.
5. **`Session/State Manager` doesn't gain new logic.** If you're tempted to
   add a method to the Session that does more than "read/update which
   project is active and dispatch," that method belongs in the domain
   module that owns that information.
6. **No domain module depends on Tauri or Angular API details to decide
   anything.** The `Commands`/`Events` bridge is treated as replaceable —
   if your module imports something from `tauri::` outside the IPC layer,
   that's a responsibility leak.

## Antipatterns to avoid (documented reason, not personal taste)

| Antipattern | Why avoid it here |
|---|---|
| A trait object / `dyn Trait` for "future flexibility" with no concrete use today | Reading cost for people coming from TS, with no measured gain — explicit YAGNI in the docs |
| A channel (`mpsc`, actor) for common shared state | Already decided: a simple `Mutex` is the default until profiling proves real contention |
| A `Mutex` per feature/handler | Decided: at most one per module, preferably a single one (`AppState`) |
| A lock (`Mutex`, `RwLock`) accessed inside the `cpal` callback | Violates the real-time boundary — see the `realtime-audio-safety` skill |
| Cross-module coordination logic inside the Session | Turns it into a *god object* — extract it into a new module that owns that logic |

## When you're done

Run `cargo check` (the `PostToolUse` hook already does this automatically on
every `.rs` edit and hands you back the compiler error to fix yourself —
don't wait for the user to ask). If the hook doesn't fire (an old session, a
watcher that hasn't reloaded), run it manually before reporting the task as
done.
