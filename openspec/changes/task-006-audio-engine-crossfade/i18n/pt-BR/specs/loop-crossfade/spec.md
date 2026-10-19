## Purpose

Faz a volta do fim do loop para o início ser suave, com um crossfade curto em vez de um corte seco.

## ADDED Requirements

### Requirement: Crossfade na borda do loop
Quando a reprodução chega ao endSec, o engine SHALL fazer um crossfade de poucos milissegundos para o startSec, em vez de cortar abruptamente.

#### Scenario: Chegando ao fim do loop
- **WHEN** a reprodução chega ao endSec
- **THEN** ela continua a partir do startSec sem estalo audível

### Requirement: Início do loop preparado com antecedência
O áudio do startSec MUST estar preparado antes de a reprodução chegar ao endSec, para que o crossfade não precise de decodificação, alocação nem I/O no callback.

#### Scenario: Aproximando-se do fim do loop
- **WHEN** a reprodução se aproxima do endSec
- **THEN** o áudio do início do loop já está decodificado e esperando no seu próprio buffer
