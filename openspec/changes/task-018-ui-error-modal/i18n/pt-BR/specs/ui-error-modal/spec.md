## Purpose

Mostra num diálogo modal os erros informados pelo núcleo.

## ADDED Requirements

### Requirement: Erros abrem um modal
audio_error e score_parse_error SHALL abrir o modal mostrando a causa e a mensagem do evento.

#### Scenario: Erro de áudio
- **WHEN** chega um evento audio_error
- **THEN** o modal abre com a causa e a mensagem dele

### Requirement: Fechar não muda mais nada
Fechar o modal MUST NOT pausar, parar nem desfazer nada que esteja rodando no núcleo.

#### Scenario: Fechar durante a reprodução
- **WHEN** o usuário fecha o modal enquanto uma música toca
- **THEN** a reprodução continua sem mudança
