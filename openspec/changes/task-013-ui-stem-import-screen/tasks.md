# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Fires the import `Command` immediately, with no client-side queue or optimistic update
- [ ] 1.2 The triggering control stays disabled until that `Command`'s response arrives
- [ ] 1.3 Vitest covers: the `Command` fires on user action, and the control disables until the response arrives
- [ ] 1.4 `npm run build` (tsc) and `prettier --check` clean (via `remote-build-offload`)
- [ ] 1.5 Reviewed by `review-angular-shell`
