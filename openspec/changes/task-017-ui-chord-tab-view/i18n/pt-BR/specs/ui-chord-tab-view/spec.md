## Purpose

Exibição somente leitura de acordes, tablaturas e letras em sincronia com a reprodução.

## ADDED Requirements

### Requirement: Exibição sincronizada com a reprodução
A tela SHALL destacar o acorde e o tempo do recorte de tempo real mais recente.

#### Scenario: Acorde muda
- **WHEN** o recorte de tempo real passa para um novo acorde
- **THEN** a tela destaca esse acorde

### Requirement: Erros de interpretação ficam nesta tela
Um score_parse_error SHALL colocar só esta tela num estado de erro; o resto da interface MUST continuar funcionando.

#### Scenario: Partitura quebrada
- **WHEN** chega um score_parse_error
- **THEN** esta tela mostra o erro enquanto transporte, timeline e mixer continuam funcionando
