# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Renders `waveform_ready` peaks and the `playback_progress` position
- [ ] 1.2 Marker/loop selection sends `set_markers` and waits for the response before moving the marker on screen
- [ ] 1.3 Vitest covers rendering from injected `waveform_ready`/`playback_progress` fixtures, and the marker not moving before the `set_markers` response
- [ ] 1.4 `npm run build` (tsc) and `prettier --check` clean (via `remote-build-offload`)
- [ ] 1.5 Reviewed by `review-angular-shell`
