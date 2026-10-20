# Tasks

## 1. Acceptance criteria

- [x] 1.1 Toda escrita passa por arquivo temporário + `rename`, nunca uma escrita direta no caminho final
- [x] 1.2 Escritas acontecem em um checkpoint com debounce, não a cada `Command` (veja nota de design (`docs/architecture.md#design-note-when-project-persistence-writes`))
- [x] 1.3 Os três casos de `schemaVersion` têm teste cobrindo
- [x] 1.4 Um crash simulado no meio de uma escrita deixa o arquivo anterior intacto (teste)
- [x] 1.5 `cargo check`/`cargo clippy` limpos (via `remote-build-offload`)
- [x] 1.6 Revisado pelo `review-rust-core`
- [x] 1.7 Testes unitários cobrem a limpeza quando a escrita falha, escritas simultâneas, uma mudança feita durante uma escrita, e uma migração quebrada ou que não devolve objeto
