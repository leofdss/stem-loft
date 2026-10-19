## Purpose

Toca os stems de um projeto no dispositivo de saída de áudio de forma contínua e sem falhas.

## ADDED Requirements

### Requirement: Reprodução contínua
O engine SHALL tocar os stems do projeto, mixados, no dispositivo de saída selecionado, sem cortes audíveis.

#### Scenario: Tocando um projeto
- **WHEN** o usuário inicia a reprodução num dispositivo de saída real
- **THEN** todos os stems tocam sincronizados, sem estalos nem buracos

### Requirement: Segurança do callback de tempo real
O callback de saída de áudio MUST NOT alocar memória, travar um lock nem fazer I/O; ele SHALL só ler áudio preparado fora dele.

#### Scenario: A decodificação acontece em outro lugar
- **WHEN** o callback precisa do próximo bloco de amostras
- **THEN** ele as lê de um buffer que a etapa de decodificação já preencheu
