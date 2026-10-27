## Purpose

Carrega e salva o estado de um projeto em disco sem nunca deixar para trás um arquivo de projeto corrompido ou escrito pela metade.

## ADDED Requirements

### Requirement: Escritas atômicas do projeto
O núcleo MUST escrever o arquivo do projeto gravando um arquivo temporário no mesmo diretório e renomeando-o por cima do caminho final, e MUST NOT escrever diretamente no caminho final.

#### Scenario: Queda no meio da escrita
- **WHEN** o processo para enquanto um salvamento está em andamento
- **THEN** o arquivo de projeto salvo anteriormente continua intacto e carrega normalmente

#### Scenario: A escrita falha
- **WHEN** qualquer etapa de um salvamento falha, como o disco cheio
- **THEN** o salvamento informa o erro com a causa original, o arquivo anterior fica intacto e nenhum arquivo temporário sobra

### Requirement: Escritas simultâneas não colidem
Dois salvamentos do mesmo arquivo ao mesmo tempo SHALL terminar cada um por inteiro, deixando o arquivo com o conteúdo de um deles.

#### Scenario: Dois salvamentos ao mesmo tempo
- **WHEN** dois salvamentos do mesmo projeto rodam em paralelo
- **THEN** os dois dão certo e o arquivo fica com uma versão completa

### Requirement: Checkpoints com debounce
Uma mudança de estado SHALL marcar o projeto como sujo, e o núcleo SHALL gravá-lo num checkpoint com debounce, e não uma vez por Command; um evento definitivo, como pausar ou fechar, SHALL gravar na hora um projeto sujo.

#### Scenario: Rajada de mudanças
- **WHEN** vários Commands mudam o projeto dentro de uma mesma janela de debounce
- **THEN** o arquivo do projeto é gravado uma vez

#### Scenario: Fechando o projeto
- **WHEN** o projeto é fechado com mudanças não salvas
- **THEN** ele é gravado na hora, sem esperar o debounce

### Requirement: Mudanças feitas durante uma escrita são mantidas
Uma mudança que chega enquanto um checkpoint está gravando SHALL manter o projeto sujo, para que o próximo checkpoint a salve.

#### Scenario: Mudança durante uma escrita
- **WHEN** um Command muda o projeto enquanto a gravação de um checkpoint está em andamento
- **THEN** o projeto continua sujo depois dessa gravação e a mudança é salva no próximo checkpoint

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
- **THEN** o carregamento falha com um erro dizendo que ele foi salvo por uma versão mais nova do app, e o arquivo fica intocado

### Requirement: Projetos ilegíveis falham de forma explícita
O carregamento MUST terminar num erro explícito, nunca num projeto parcial nem numa queda, quando o arquivo não é JSON válido, não tem schemaVersion válido, ou uma migração falha ou produz algo que não é um projeto.

#### Scenario: Arquivo malformado
- **WHEN** um arquivo de projeto não é JSON ou não tem schemaVersion válido
- **THEN** o carregamento falha com um erro de projeto malformado

#### Scenario: Migração quebrada
- **WHEN** uma migração falha ou devolve algo que não é um objeto de projeto
- **THEN** o carregamento falha com um erro de migração que cita a versão de onde partiu, e o arquivo fica intocado
