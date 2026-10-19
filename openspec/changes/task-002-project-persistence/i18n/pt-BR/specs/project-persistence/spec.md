## Purpose

Carrega e salva o estado de um projeto em disco sem nunca deixar para trás um arquivo de projeto corrompido ou escrito pela metade.

## ADDED Requirements

### Requirement: Escritas atômicas do projeto
O núcleo MUST escrever o arquivo do projeto gravando um arquivo temporário e renomeando-o por cima do caminho final, e MUST NOT escrever diretamente no caminho final.

#### Scenario: Queda no meio da escrita
- **WHEN** o processo para enquanto um salvamento está em andamento
- **THEN** o arquivo de projeto salvo anteriormente continua intacto e carrega normalmente

### Requirement: Checkpoints com debounce
Uma mudança de estado SHALL marcar o projeto como sujo, e o núcleo SHALL gravá-lo num checkpoint com debounce, e não uma vez por Command.

#### Scenario: Rajada de mudanças
- **WHEN** vários Commands mudam o projeto dentro de uma mesma janela de debounce
- **THEN** o arquivo do projeto é gravado uma vez

### Requirement: Tratamento de versão do schema
Ao carregar, o núcleo SHALL comparar o schemaVersion do arquivo com o atual: carregar direto uma versão igual, migrar uma menor e rejeitar uma maior com um erro explícito.

#### Scenario: Mesma versão
- **WHEN** um projeto com o schemaVersion atual é aberto
- **THEN** ele carrega sem migração

#### Scenario: Versão mais antiga
- **WHEN** um projeto com um schemaVersion menor é aberto
- **THEN** as migrações rodam e o projeto carrega na versão atual

#### Scenario: Versão mais nova
- **WHEN** um projeto com um schemaVersion maior é aberto
- **THEN** o carregamento falha com um erro explícito e o arquivo fica intocado
