# Tasks

## 1. Acceptance criteria

- [x] 1.1 Picos são calculados uma vez (na importação, ou na primeira abertura se não houver cache) e escritos pelo caminho de escrita atômica
- [x] 1.2 O cache é invalidado quando o stem correspondente é reimportado/substituído
- [x] 1.3 `waveform_ready` serve os picos do cache em disco quando presente, sem redecodificar
- [x] 1.4 Testes unitários cobrem: picos calculados no primeiro acesso, o cache em disco reaproveitado (não recalculado) na próxima abertura, e invalidado após reimportação
- [x] 1.5 `cargo check`/`cargo clippy` limpos (via `remote-build-offload`)
- [x] 1.6 Revisado pelo `review-rust-core`
