# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Toda escrita passa por arquivo temporário + `rename`, nunca uma escrita direta no caminho final
- [ ] 1.2 Escritas acontecem em um checkpoint com debounce, não a cada `Command` (veja nota de design (`docs/architecture.md#design-note-when-project-persistence-writes`))
- [ ] 1.3 Os três casos de `schemaVersion` têm teste cobrindo
- [ ] 1.4 Um crash simulado no meio de uma escrita deixa o arquivo anterior intacto (teste)
- [ ] 1.5 `cargo check`/`cargo clippy` limpos (via `remote-build-offload`)
- [ ] 1.6 Revisado pelo `review-rust-core`
