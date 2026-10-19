# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Uma mudança de volume/mute/solo escreve em um atomic/double-buffer que o callback lê — nunca um `Mutex` no qual ele possa bloquear
- [ ] 1.2 Uma mudança no mixer marca o projeto como sujo para o checkpoint com debounce (TASK-002), não uma escrita a cada tick
- [ ] 1.3 Testes unitários cobrem as transições de estado de volume/mute/solo e a flag de sujeira sendo marcada numa mudança
- [ ] 1.4 `cargo check`/`cargo clippy` limpos (via `remote-build-offload`)
- [ ] 1.5 Revisado pelo `review-rust-core`
