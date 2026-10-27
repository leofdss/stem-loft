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

### Requirement: Um acorde por linha
O parser MUST rejeitar uma linha com mais de um [acorde] com score_parse_error apontando essa linha, e MUST NOT manter só o primeiro acorde.

#### Scenario: Dois acordes numa linha
- **WHEN** uma linha diz `[G]Twinkle twinkle [C]little star`
- **THEN** a interpretação falha com score_parse_error nessa linha, dizendo que o dialeto permite um acorde por linha

### Requirement: Âncoras de tempo estritas
Uma âncora {t:} SHALL ser `minuto:segundo.centésimo` com segundo de dois dígitos (00–59) e centésimo de dois dígitos; ela SHALL NOT ser anterior à âncora anterior nem passar da duração do projeto, e um [acorde] SHALL ter uma âncora própria.

#### Scenario: Âncora malformada
- **WHEN** uma linha contém `{t: 1:5.3}`
- **THEN** a interpretação falha com score_parse_error nessa linha

#### Scenario: Âncora e linha juntas
- **WHEN** uma linha contém `{t: 0:01.00}[G]letra`
- **THEN** o acorde G começa em 1,0 s

### Requirement: Janelas dos acordes
O startSec de um acorde SHALL ser a âncora da sua linha e o endSec o início do próximo acorde ou bloco de tablatura; uma linha de letra sem acorde SHALL NOT encerrar o acorde atual; o último acorde SHALL terminar na duração do projeto, passada ao parser a partir do maior stems[].durationSec; a origem da grade de compassos SHALL ser a primeira âncora do arquivo.

#### Scenario: Linha de letra sob o mesmo acorde
- **WHEN** uma linha de acorde em 0 s é seguida por uma linha só de letra em 2 s e por uma linha de acorde em 4 s
- **THEN** o primeiro acorde vai de 0 a 4 s

#### Scenario: Último acorde
- **WHEN** o último acorde começa em 1 s num projeto cujo stem mais longo dura 187,42 s
- **THEN** esse acorde termina em 187,42 s

### Requirement: Pauta de tablatura
Um bloco de tablatura SHALL ter seis linhas de pauta de mesma largura, com a corda 1 em cima; cada sequência de caracteres diferentes de `-` e `|` SHALL ser uma nota na gramática do playbook, na corda daquela linha, com tempo dado pela sua coluna entre o início e o fim do bloco; um traste fora de 0–24, um zero à esquerda ou um release sem bend antes SHALL ser um score_parse_error nessa linha.

#### Scenario: Tempo da nota
- **WHEN** um bloco de 64 colunas vai de 0 a 6,4 s
- **THEN** uma nota na coluna 9 começa em 0,9 s

#### Scenario: Release sem bend
- **WHEN** uma linha da pauta contém `7r5`
- **THEN** a interpretação falha com score_parse_error nessa linha da pauta
