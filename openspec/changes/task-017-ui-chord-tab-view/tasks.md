# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Renders chords/tablature/lyrics synced to the playback position
- [ ] 1.2 `score_parse_error` puts only this view into an error state — the rest of the screen keeps working
- [ ] 1.3 Vitest covers chord/beat highlighting from a `ChordBeatStream`-shaped fixture, and the isolated error state on `score_parse_error`
- [ ] 1.4 `npm run build` (tsc) and `prettier --check` clean (via `remote-build-offload`)
- [ ] 1.5 Reviewed by `review-angular-shell`
- [ ] 1.6 Opening, creating or closing a project clears the shown score, so a previous project's chords never linger when the new one has no score (the core publishes nothing for a project without a score; deferred from TASK-012)
