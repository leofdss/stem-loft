# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Renders chords/tablature/lyrics synced to the playback position
- [ ] 1.2 `score_parse_error` puts only this view into an error state — the rest of the screen keeps working
- [ ] 1.3 Vitest covers chord/beat highlighting from a `ChordBeatStream`-shaped fixture, and the isolated error state on `score_parse_error`
- [ ] 1.4 `npm run build` (tsc) and `prettier --check` clean (via `remote-build-offload`)
- [ ] 1.5 Reviewed by `review-angular-shell`
