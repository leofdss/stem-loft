# Claude Code adapter

@AGENTS.md

Everything above comes from `AGENTS.md`, the tool-neutral guide. This file
only adds how Claude Code is wired to it — never a rule of its own (see
"Agent-agnostic rule" in AGENTS.md):

- Playbooks: `.claude/skills` is a symlink to `.agents/skills`, so they show
  up as skills.
- Hooks (`.claude/settings.json`): `hooks/on_edit.py` runs
  `scripts/check_changed.py` after every Write/Edit; `hooks/bash_guard.py`
  runs `scripts/guard_command.py` before every Bash call.
- Subagents (`.claude/agents/`): `rust-core-reviewer` and
  `angular-shell-reviewer` apply the `review-rust-core` /
  `review-angular-shell` playbooks; `implementer` (Haiku) is the cheaper
  implementer of the `implement-against-frozen-tests` playbook;
  `canvas-to-mermaid` runs that playbook on a lighter model.
