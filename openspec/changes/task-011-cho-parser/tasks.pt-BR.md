# Tasks

## 1. Acceptance criteria

- [x] 1.1 Toda regra de gramática da skill `chordpro-format` (um acorde por linha, âncoras `{t:}`, gramática de tablatura, resolução de `startSec`/`endSec`) tem um teste unitário passando
- [x] 1.2 Uma diretiva desconhecida é ignorada, não fatal
- [x] 1.3 `score_parse_error` carrega a linha e a mensagem do problema sem travar o resto do app
- [x] 1.4 `cargo check`/`cargo clippy` limpos (via `remote-build-offload`)
- [x] 1.5 Revisado pelo `review-rust-core`
