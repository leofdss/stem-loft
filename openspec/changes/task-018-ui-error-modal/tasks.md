# Tasks

## 1. Acceptance criteria

- [ ] 1.1 `audio_error` and `score_parse_error` open the modal with the event's cause and message
- [ ] 1.2 Dismissing the modal doesn't pause or undo anything already running in the core
- [ ] 1.3 Vitest covers the modal opening with the right cause/message for both error events, and dismiss not touching any other state
- [ ] 1.4 `npm run build` (tsc) and `prettier --check` clean (via `remote-build-offload`)
- [ ] 1.5 Reviewed by `review-angular-shell`
