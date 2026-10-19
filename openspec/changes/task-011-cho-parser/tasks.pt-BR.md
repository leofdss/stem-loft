# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Toda regra de gramática da skill `chordpro-format` (um acorde por linha, âncoras `{t:}`, gramática de tablatura, resolução de `startSec`/`endSec`) tem um teste unitário passando
- [ ] 1.2 Uma diretiva desconhecida é ignorada, não fatal
- [ ] 1.3 `score_parse_error` carrega a linha e a mensagem do problema sem travar o resto do app
- [ ] 1.4 `cargo check`/`cargo clippy` limpos (via `remote-build-offload`)
- [ ] 1.5 Revisado pelo `review-rust-core`
