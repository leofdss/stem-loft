---
name: implement-against-frozen-tests
description: Delegation method for StemLoft — an orchestrator (stronger model or person) writes the contract tests and a skeleton, commits them, and hands one bounded unit to a cheaper implementer (a smaller model running as a sub-agent with repo tools, or a person) that fills it in until the frozen tests pass, without ever touching them. Use when splitting a task so a cheaper model does the implementation, when acting as that implementer, or when asked "can a smaller model do this part?"
---

# Implement against frozen tests

The default way to hand implementation work to a cheaper model. It exists
because of what was measured: a model left to write both the code and its
tests "fixes" correct code to match its own wrong expectation; vague tasks
don't compile; and a near-green draft still hides bugs the tests don't
cover. So the roles are split, and the split is checked by a script, not
trusted.

Who plays which role is a choice per task, not per tool: the
**implementer** is typically a smaller hosted model running as a sub-agent
(in Claude Code: the `implementer` agent, Haiku), but can be a person or a
local model (`scripts/llm_task.py`, experimental — see AGENTS.md).

## What can be delegated

Only a **bounded unit with a mechanical judge**: a module, a function or a
small group of functions whose behavior the contract tests pin down, in an
area where a green test means correct code. Never: anything inside the
`cpal` real-time callback, the `.cho` grammar's edge cases, anything marked
`human`/`both` in `openspec/board.toml`, reviews, or writing the tests
themselves (AGENTS.md, "Choosing who does the work").

## Orchestrator: before delegating

1. **Write the contract tests** in the file's `#[cfg(test)]` module (or the
   `.spec.ts` for Angular) from the change's `tasks.md` and delta spec.
   Validate them: they must compile against the skeleton and fail for the
   right reason. When feasible, prove them against a throwaway reference
   implementation you don't commit.
2. **Write the skeleton**: every type, attribute and signature final, bodies
   `todo!()`, a one-line comment per body stating its rule. Precision is the
   biggest lever measured — a bare spec doesn't compile; a skeleton does.
3. **Commit the contract** (`test: contract for TASK-NNN <unit>`). This
   commit is what `scripts/check_frozen.py` compares against.
4. **Write the brief** (template below). One unit per brief; split big
   tasks into several briefs rather than one large one.
5. **Pick where the check runs** (`remote-build-offload` playbook). If the
   remote build host is reachable, the brief's check command goes through
   it — the implementer runs it after every edit, so that's where most of
   the CPU goes:
   `distrobox/remote/sync.sh && distrobox/remote/run.sh --dir src-tauri cargo test -q --lib <module>`.
   Otherwise the plain local command. At the end of the session,
   `distrobox/remote/power.sh down build`.

### Brief template

```text
Task: TASK-NNN, unit <name>, in <file>.
Fill in every todo!() above the `#[cfg(test)]` marker so this passes:
  <check command, e.g. cd src-tauri && cargo test -q --lib core::persistence,
   or via the remote build host: distrobox/remote/sync.sh && distrobox/remote/run.sh --dir src-tauri cargo test -q --lib core::persistence>
Frozen: everything from `#[cfg(test)]` on. Contract commit: <sha>.
Read first: .agents/skills/<area playbook>/SKILL.md, <spec path>.
Allowed dependencies: <list>. Iteration budget: <N> check runs.
Report: the format in .agents/skills/implement-against-frozen-tests/SKILL.md.
```

## Implementer: the rules

- Read the brief, the area playbook it names, the skeleton and the tests.
- Edit **only** the code above the frozen marker, in the file the brief
  names. Never edit, delete, rename, skip (`#[ignore]`, `.skip()`) or
  "adjust" a test — if a test looks wrong, say so in the report and stop.
- After each change, run the brief's check command. Read the first error,
  fix that, run again.
- Don't add dependencies the brief doesn't list, don't touch other files,
  don't commit, don't edit `openspec/board.toml`.
- If the same failure survives two attempts, change approach (re-read the
  failing assertion's exact expectation) instead of re-sending the same fix.
- Stop when the check is green, or when the iteration budget is spent.

### Report format

```text
Status: GREEN | NOT GREEN (x/y tests passing)
Check runs used: N
What I implemented: <one line per function>
Remaining failures: <test name — the assertion — my best hypothesis>
Concerns for the reviewer: <anything the tests don't cover that looked risky>
```

## Orchestrator: after the implementer returns

1. `python3 scripts/check_frozen.py <file> --ref <contract sha>` — exit 0 or
   the work is rejected, whatever the test results say.
2. Run the check yourself; don't trust the report's status line.
3. Finish what's left (the last tests are where cheaper models stall).
4. Review the whole unit with the area's review playbook
   (`review-rust-core` / `review-angular-shell`) — a green contract is not a
   correct module. Every bug the review finds gets a new contract test.
5. `scripts/verify.sh`, then commit (`feat: ...`).
6. Record the outcome in the change's notes or PR description: green or
   not, check runs, what had to be fixed — that's the data for deciding
   what to delegate next.
