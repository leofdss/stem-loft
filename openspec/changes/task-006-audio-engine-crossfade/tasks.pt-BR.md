# Tasks

## 1. Acceptance criteria

- [ ] 1.1 O buffer do início do loop está pronto antes de o callback chegar em `endSec`
- [ ] 1.2 O crossfade é matemática pura sobre amostras já decodificadas dentro do callback — sem nova alocação/I/O
- [ ] 1.3 Testes unitários cobrem a matemática do crossfade (curva de fade) sobre buffers de amostra sintéticos, independente do `cpal`
- [ ] 1.4 Um humano escuta a fronteira do loop em pelo menos um projeto real e confirma ausência de cliques/artefatos
- [ ] 1.5 `cargo check`/`cargo clippy` limpos (via `remote-build-offload`)
- [ ] 1.6 Revisado pelo `review-rust-core`
