# Tasks

## 1. Acceptance criteria

- [ ] 1.1 Renderiza os picos de `waveform_ready` e a posição de `playback_progress`
- [ ] 1.2 A seleção de marcador/loop envia `set_markers` e espera a resposta antes de mover o marcador na tela
- [ ] 1.3 Vitest cobre a renderização a partir de fixtures injetadas de `waveform_ready`/`playback_progress`, e o marcador não se mover antes da resposta de `set_markers`
- [ ] 1.4 `npm run build` (tsc) e `prettier --check` limpos (via `remote-build-offload`)
- [ ] 1.5 Revisado pelo `review-angular-shell`
