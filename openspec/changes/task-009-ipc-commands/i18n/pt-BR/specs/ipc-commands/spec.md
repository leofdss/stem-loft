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

### Requirement: Commands precisam de um projeto aberto
Todo Command que age sobre um projeto SHALL falhar com "no project is open" quando nenhum está aberto, sem chamar o seu módulo; create_project e open_project SHALL abrir um, e close_project SHALL fechá-lo.

#### Scenario: Importar sem nada aberto
- **WHEN** import_stems chega e nenhum projeto está aberto
- **THEN** ele falha com "no project is open" e o Stem Importer não é chamado

#### Scenario: Criar nunca sobrescreve
- **WHEN** create_project aponta para uma pasta que já tem um arquivo de projeto
- **THEN** ele falha, o arquivo existente fica intacto e o projeto atual continua aberto

### Requirement: Mudanças são salvas em checkpoints
Um Command bem-sucedido que muda estado SHALL marcar o projeto aberto como sujo; o núcleo SHALL escrevê-lo depois do debounce, e pausar, fechar, abrir ou criar outro projeto e sair do app SHALL escrevê-lo na hora; um snapshot mais antigo MUST NOT sobrescrever um mais novo no disco.

#### Scenario: Arrastar um fader
- **WHEN** vários Commands set_stem_volume chegam dentro do debounce
- **THEN** o arquivo do projeto é escrito uma vez, com o último volume

#### Scenario: Reabrir o projeto aberto
- **WHEN** open_project aponta para o projeto que já está aberto com mudanças não salvas
- **THEN** essas mudanças são salvas antes e o projeto reaberto as inclui
