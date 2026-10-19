# Tasks

## 1. Acceptance criteria

- [ ] 1.1 `audio_error` e `score_parse_error` abrem o modal com a causa e a mensagem do evento
- [ ] 1.2 Fechar o modal não pausa nem desfaz nada que já esteja rodando no núcleo
- [ ] 1.3 Vitest cobre o modal abrindo com a causa/mensagem corretas para os dois eventos de erro, e o dismiss não tocando em nenhum outro estado
- [ ] 1.4 `npm run build` (tsc) e `prettier --check` limpos (via `remote-build-offload`)
- [ ] 1.5 Revisado pelo `review-angular-shell`
