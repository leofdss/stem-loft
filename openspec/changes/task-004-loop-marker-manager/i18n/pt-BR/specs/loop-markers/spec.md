## Purpose

Guarda os marcadores de início e fim do trecho em loop, validados contra a duração do projeto.

## ADDED Requirements

### Requirement: Marcadores dentro da duração do projeto
O núcleo MUST rejeitar marcadores de loop cujo endSec passe da duração do projeto (o durationSec do stem mais longo), e MUST NOT ajustá-los silenciosamente.

#### Scenario: Marcadores dentro da música
- **WHEN** marcadores com endSec dentro da duração do projeto são definidos
- **THEN** eles são aceitos e viram o loop atual

#### Scenario: Fim além da música
- **WHEN** marcadores com endSec além da duração do projeto são definidos
- **THEN** eles são rejeitados com um erro e os marcadores anteriores continuam valendo

### Requirement: Marcadores guiam o loop na reprodução
Os marcadores atuais SHALL ficar expostos para o audio engine, que faz o loop da reprodução entre eles.

#### Scenario: Novos marcadores durante a reprodução
- **WHEN** os marcadores mudam enquanto a música toca
- **THEN** a próxima volta do loop usa os novos marcadores

### Requirement: Loops bem formados
O núcleo MUST rejeitar um loop cujos limites não são números finitos, cujo início é negativo ou não vem antes do fim, ou definido enquanto a duração do projeto é desconhecida, deixando o loop atual como está.

#### Scenario: Início depois do fim
- **WHEN** marcadores com startSec igual ou depois de endSec são definidos
- **THEN** eles são rejeitados e os marcadores anteriores continuam valendo

### Requirement: O loop acompanha o projeto aberto
Abrir um projeto SHALL restaurar o loop salvo nele quando ele ainda cabe na duração do projeto e descartá-lo caso contrário, nunca recortando; criar ou fechar um projeto SHALL limpar o loop atual.

#### Scenario: Reabrir um projeto
- **WHEN** um projeto salvo com um loop é aberto
- **THEN** esse loop é o loop atual
