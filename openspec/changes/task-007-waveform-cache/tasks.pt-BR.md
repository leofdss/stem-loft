# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Picos são calculados uma vez (na importação, ou na primeira abertura se não houver cache) e escritos pelo caminho de escrita atômica
- [ ] 1.2 O cache é invalidado quando o stem correspondente é reimportado/substituído
- [ ] 1.3 `waveform_ready` serve os picos do cache em disco quando presente, sem redecodificar
- [ ] 1.4 Testes unitários cobrem: picos calculados no primeiro acesso, o cache em disco reaproveitado (não recalculado) na próxima abertura, e invalidado após reimportação
- [ ] 1.5 `cargo check`/`cargo clippy` limpos (via `remote-build-offload`)
- [ ] 1.6 Revisado pelo `review-rust-core`
