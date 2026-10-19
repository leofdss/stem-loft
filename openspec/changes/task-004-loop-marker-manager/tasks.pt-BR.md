# Tasks

## 1. Acceptance criteria

- [ ] 1.1 `endSec` além da duração do projeto (o `durationSec` do stem mais longo) é rejeitado, não truncado silenciosamente
- [ ] 1.2 Os marcadores atuais são expostos ao `Audio Engine` para a reprodução em loop
- [ ] 1.3 Testes unitários cobrem: um marcador dentro da duração é aceito, `endSec` além da duração é rejeitado
- [ ] 1.4 `cargo check`/`cargo clippy` limpos (via `remote-build-offload`)
- [ ] 1.5 Revisado pelo `review-rust-core`
