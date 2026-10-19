## Purpose

Entrega a partitura interpretada à UI e mantém o acorde e o tempo atuais acompanhando a reprodução.

## ADDED Requirements

### Requirement: Evento de partitura carregada
O núcleo SHALL emitir score_loaded uma vez por interpretação, com o andamento, a fórmula de compasso e a afinação do cabeçalho do arquivo.

#### Scenario: Partitura interpretada
- **WHEN** um arquivo .cho é interpretado com sucesso
- **THEN** um único evento score_loaded traz o andamento, o compasso e a afinação

### Requirement: Recorte de acordes em tempo real
Cada tick de tempo real SHALL trazer só o acorde e o tempo ativos na posição atual, nunca a partitura inteira.

#### Scenario: Posição dentro de um acorde
- **WHEN** a reprodução está num positionSec dentro do intervalo de um acorde
- **THEN** o payload do tick indica esse acorde e o tempo atual
