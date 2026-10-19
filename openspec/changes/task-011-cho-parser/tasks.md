# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Every grammar rule in the `chordpro-format` skill (one chord per line, `{t:}` anchors, tablature grammar, `startSec`/`endSec` resolution) has a passing unit test
- [ ] 1.2 An unknown directive is ignored, not fatal
- [ ] 1.3 `score_parse_error` carries the offending line and message without halting the rest of the app
- [ ] 1.4 `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [ ] 1.5 Reviewed by `review-rust-core`
