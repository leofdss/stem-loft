## Purpose

Fornece os picos da forma de onda de cada stem para exibição, sem decodificar o áudio de novo toda vez que um projeto abre.

## ADDED Requirements

### Requirement: Picos calculados uma vez
O núcleo SHALL calcular os picos de um stem na importação, ou na primeira abertura quando não há cache, e SHALL gravá-los em disco pelo caminho de escrita atômica.

#### Scenario: Ainda sem cache
- **WHEN** um projeto é aberto e um stem não tem picos em cache
- **THEN** os picos são calculados e o arquivo de cache é gravado

### Requirement: Reuso do cache
Quando existe um cache válido, o núcleo SHALL atender o waveform_ready a partir dele, sem decodificar o stem.

#### Scenario: Reabrindo um projeto
- **WHEN** um projeto cujos stems têm picos em cache é reaberto
- **THEN** o waveform_ready é atendido pelo cache e nada é decodificado

### Requirement: Invalidação na reimportação
Reimportar ou substituir um stem MUST invalidar os picos em cache daquele stem.

#### Scenario: Stem substituído
- **WHEN** um stem é substituído por um arquivo novo
- **THEN** os picos dele são recalculados em vez de lidos do cache antigo
