# Tasks

## 1. Acceptance criteria

- [x] 1.1 `AppState` corresponde à nota de design do modelo de concorrência (`docs/architecture.md#design-note-concurrency-model-for-shared-state`) — sem channels, sem actor
- [x] 1.2 `Session` guarda apenas "qual projeto está aberto"; todo handler de `Command` delega ao módulo dono daquele domínio, conforme Session as a thin router (`docs/architecture.md#design-note-session-as-a-thin-router`)
- [x] 1.3 Testes unitários cobrem o `Session` despachando um `Command` para o módulo correto (um módulo falso registra a chamada), sem retestar a lógica própria daquele módulo
- [x] 1.4 `cargo check`/`cargo clippy` limpos (via `remote-build-offload`)
- [x] 1.5 Revisado pelo `review-rust-core`
