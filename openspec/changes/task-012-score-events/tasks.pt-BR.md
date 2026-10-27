# Tasks

## 1. Acceptance criteria

- [x] 1.1 `score_loaded` dispara uma vez por parse com `tempo`/`time`/`tuning` do cabeçalho
- [x] 1.2 O payload em tempo real a cada tick carrega só o acorde/batida ativo, não a partitura inteira (veja Real-time visual state (`docs/architecture.md#real-time-visual-state-angular`))
- [x] 1.3 Testes unitários cobrem a seleção da fatia de acorde/batida para alguns valores de `positionSec` contra uma partitura pequena e fixa
- [x] 1.4 `cargo check`/`cargo clippy` limpos (via `remote-build-offload`)
- [x] 1.5 Revisado pelo `review-rust-core`
