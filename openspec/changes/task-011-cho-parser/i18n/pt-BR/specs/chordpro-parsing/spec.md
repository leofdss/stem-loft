## Purpose

Interpreta o arquivo ChordPro estendido (.cho) de um projeto em acordes, tablaturas e letras com tempo.

## ADDED Requirements

### Requirement: Gramática ChordPro estendida
O parser SHALL aceitar o dialeto do playbook chordpro-format: um acorde por linha, âncoras de tempo {t:}, a gramática de tablatura e a resolução de startSec/endSec.

#### Scenario: Partitura válida
- **WHEN** um arquivo .cho válido é interpretado
- **THEN** todo acorde tem startSec e endSec resolvidos

### Requirement: Diretivas desconhecidas são ignoradas
O parser SHALL pular uma diretiva que não conhece e continuar a interpretação.

#### Scenario: Diretiva desconhecida
- **WHEN** um arquivo contém uma diretiva que o dialeto não define
- **THEN** o resto do arquivo é interpretado normalmente

### Requirement: Erros de interpretação ficam isolados
Com entrada inválida, o parser SHALL informar score_parse_error com a linha problemática e uma mensagem, e o resto da aplicação MUST continuar funcionando.

#### Scenario: Linha quebrada
- **WHEN** um arquivo .cho tem uma linha malformada
- **THEN** o score_parse_error aponta essa linha e a reprodução continua funcionando
