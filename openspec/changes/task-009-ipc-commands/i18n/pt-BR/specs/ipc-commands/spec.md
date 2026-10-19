## Purpose

Define como a UI envia intenção ao núcleo: Commands enfileirados, ordenados e executados inteiramente pelo núcleo.

## ADDED Requirements

### Requirement: Fila de commands no núcleo
Os Commands SHALL ser enfileirados e ordenados só no núcleo; a UI MUST NOT enfileirá-los, deduplicá-los nem reordená-los.

#### Scenario: Dois commands rápidos
- **WHEN** a UI envia dois Commands em sequência rápida
- **THEN** o núcleo executa os dois, na ordem em que chegaram

### Requirement: Handlers só despacham
Cada handler de Command SHALL entregar o Command ao módulo dono daquele domínio, sem lógica de coordenação própria.

#### Scenario: Definir marcadores
- **WHEN** chega um Command set_markers
- **THEN** o handler dele o repassa ao módulo de marcadores de loop e devolve o resultado desse módulo
