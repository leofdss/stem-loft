# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Arrastar um fader não dispara uma escrita de persistência a cada tick (verificado contra o debounce da TASK-002/TASK-008)
- [ ] 1.2 Vitest cobre o fader disparando o `Command` do mixer sem nenhuma lógica extra própria do cliente (fila, debounce ou dedup)
- [ ] 1.3 `npm run build` (tsc) e `prettier --check` limpos (via `remote-build-offload`)
- [ ] 1.4 Revisado pelo `review-angular-shell`
