# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Dispara o `Command` de importação imediatamente, sem fila própria no cliente ou atualização otimista
- [ ] 1.2 O controle que disparou fica desabilitado até a resposta daquele `Command` chegar
- [ ] 1.3 Vitest cobre: o `Command` dispara na ação do usuário, e o controle fica desabilitado até a resposta chegar
- [ ] 1.4 `npm run build` (tsc) e `prettier --check` limpos (via `remote-build-offload`)
- [ ] 1.5 Revisado pelo `review-angular-shell`
