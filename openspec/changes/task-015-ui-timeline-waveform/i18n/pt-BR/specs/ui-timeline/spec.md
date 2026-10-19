## Purpose

Mostra as formas de onda dos stems, a posição de reprodução e os marcadores de loop numa timeline.

## ADDED Requirements

### Requirement: Exibição da forma de onda e da posição
A timeline SHALL exibir os picos do waveform_ready e o cursor de reprodução do playback_progress.

#### Scenario: Picos chegam
- **WHEN** chega um evento waveform_ready de um stem
- **THEN** a forma de onda daquele stem é desenhada a partir dos picos

### Requirement: Marcadores só se movem após confirmação
Selecionar um loop SHALL enviar set_markers, e os marcadores SHALL se mover na tela só depois da resposta do núcleo.

#### Scenario: Marcadores aceitos
- **WHEN** o núcleo aceita o set_markers
- **THEN** os marcadores vão para as novas posições

#### Scenario: Marcadores rejeitados
- **WHEN** o núcleo rejeita o set_markers
- **THEN** os marcadores ficam onde estavam
