# Tasks

## 1. Acceptance criteria

- [x] 1.1 Every grammar rule in the `chordpro-format` skill (one chord per line, `{t:}` anchors, tablature grammar, `startSec`/`endSec` resolution) has a passing unit test
- [x] 1.2 An unknown directive is ignored, not fatal
- [x] 1.3 `score_parse_error` carries the offending line and message without halting the rest of the app
- [x] 1.4 `cargo check`/`cargo clippy` clean (via `remote-build-offload`)
- [x] 1.5 Reviewed by `review-rust-core`
