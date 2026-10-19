## Purpose

Controles de volume, mute e solo por stem.

## ADDED Requirements

### Requirement: Faders enviam Commands do mixer direto
Cada mudança de controle SHALL enviar o Command do mixer sem fila, debounce nem deduplicação no cliente.

#### Scenario: Movendo um fader
- **WHEN** o usuário move um fader de volume
- **THEN** um Command do mixer é enviado com o novo valor

### Requirement: Nenhuma escrita em disco por movimento
Arrastar um fader MUST NOT causar uma escrita do projeto por movimento; a persistência segue o checkpoint com debounce do núcleo.

#### Scenario: Arraste longo
- **WHEN** um fader é arrastado por toda a sua faixa
- **THEN** o projeto é gravado no máximo uma vez por checkpoint
