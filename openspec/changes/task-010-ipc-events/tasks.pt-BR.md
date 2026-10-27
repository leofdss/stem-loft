# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Cada evento corresponde exatamente à tabela de eventos (`docs/architecture.md#events-table`) (produtor, payload, consumidores)
- [ ] 1.2 `audio_error` é de fato alcançável em teste manual (ex.: desconectar o dispositivo de saída)
- [ ] 1.3 Testes unitários cobrem o formato do payload de cada evento contra a tabela de eventos (`docs/architecture.md#events-table`)
- [ ] 1.4 `cargo check`/`cargo clippy` limpos (via `remote-build-offload`)
- [ ] 1.5 Revisado pelo `review-rust-core`
- [ ] 1.6 A Session usa o `WaveformCache`: picos calculados na importação (ou na primeira abertura quando não há cache), `stems[].waveformCache` registrado, o cache invalidado na reimportação e o `waveform_ready` servido a partir dele, tudo fora da thread de tempo real (adiado da TASK-007)
