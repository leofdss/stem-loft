## Purpose

Define os eventos que o núcleo emite para que a UI apresente um estado que ela nunca decide sozinha.

## ADDED Requirements

### Requirement: Eventos seguem a tabela de eventos
O núcleo SHALL emitir playback_progress, waveform_ready, transport_state_changed e audio_error com o produtor, o payload e os consumidores da tabela de eventos da arquitetura.

#### Scenario: Reprodução em andamento
- **WHEN** a música está tocando
- **THEN** eventos playback_progress trazem a posição atual

### Requirement: Falhas de áudio são informadas
Quando a saída de áudio falha, o núcleo SHALL emitir audio_error com uma causa e uma mensagem.

#### Scenario: Dispositivo desconectado
- **WHEN** o dispositivo de saída é desconectado durante a reprodução
- **THEN** um evento audio_error chega à UI
