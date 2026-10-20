# Tasks

## 1. Acceptance criteria

- [ ] 1.1 A fila de `Commands` mora inteiramente no núcleo Rust; nada no Angular enfileira, deduplica ou reordena
- [ ] 1.2 Cada handler de `Command` apenas despacha para o módulo dono daquele domínio — sem lógica de coordenação no próprio handler
- [ ] 1.3 Testes unitários cobrem cada handler de `Command` despachando para seu módulo dono, independente do runtime do Tauri
- [ ] 1.4 `cargo check`/`cargo clippy` limpos (via `remote-build-offload`)
- [ ] 1.5 Revisado pelo `review-rust-core` e pelo `review-angular-shell` (pontos de chamada)
- [ ] 1.6 Todo `Command` que muda estado chama `ProjectPersistence::mark_dirty`, um checkpoint periódico grava depois do debounce, e pausar a reprodução ou fechar o projeto roda um checkpoint definitivo — o retrato do projeto é clonado sob o lock e gravado depois de soltá-lo (adiado da TASK-002)
