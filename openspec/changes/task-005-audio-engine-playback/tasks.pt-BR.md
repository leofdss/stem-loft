# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Decodificação e qualquer I/O acontecem fora do callback do `cpal`; o callback só lê buffers já preparados
- [ ] 1.2 Nenhuma alocação, lock ou I/O dentro do callback (checado contra a checklist de `realtime-audio-safety`)
- [ ] 1.3 Testes unitários cobrem a entrega decodificação → ring buffer isoladamente, sem precisar de um dispositivo `cpal` real
- [ ] 1.4 Um humano confirma reprodução audível e sem glitches em pelo menos um dispositivo real
- [ ] 1.5 `cargo check`/`cargo clippy` limpos (via `remote-build-offload`)
- [ ] 1.6 Revisado pelo `review-rust-core`
