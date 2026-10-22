# Tasks

## 1. Acceptance criteria

- [x] 1.1 Um stem com taxa de amostragem diferente dos já importados é rejeitado com erro explícito, não resampleado silenciosamente
- [x] 1.2 Um stem mono é convertido para estéreo na importação — `Audio Engine` nunca vê um buffer mono
- [x] 1.3 `durationSec` é lido do cabeçalho e armazenado conforme Consistência entre os stems de um projeto (`docs/architecture.md#consistency-across-a-projects-stems`)
- [x] 1.4 Testes unitários cobrem: um descompasso de taxa de amostragem é rejeitado, um stem mono é convertido para estéreo, e `durationSec` é lido corretamente, cada um contra um arquivo de fixture pequeno e real
- [x] 1.5 `cargo check`/`cargo clippy` limpos (via `remote-build-offload`)
- [x] 1.6 Revisado pelo `review-rust-core`
