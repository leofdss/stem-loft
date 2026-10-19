# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Dragging a fader doesn't trigger a persistence write per tick (verified against TASK-002/TASK-008's debounce)
- [ ] 1.2 Vitest covers the fader firing the mixer `Command` with no extra client-side logic (queue, debounce, or dedup) of its own
- [ ] 1.3 `npm run build` (tsc) and `prettier --check` clean (via `remote-build-offload`)
- [ ] 1.4 Reviewed by `review-angular-shell`
