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

### Requirement: Partitura publicada quando o seu projeto abre
Abrir um projeto que indica uma partitura SHALL carregá-la a partir da pasta do projeto e publicar exatamente um score_loaded ou um score_parse_error; uma partitura quebrada MUST NOT fazer a abertura falhar, e um carregamento que falhou SHALL limpar a partitura que a UI mostra.

#### Scenario: Partitura quebrada na abertura
- **WHEN** um projeto cujo .cho tem uma linha malformada é aberto
- **THEN** o projeto abre, um score_parse_error aponta a linha, e nenhuma partitura anterior fica na tela

### Requirement: Grade de tempos a partir do cabeçalho da partitura
O tempo ativo SHALL ser contado a partir da primeira âncora de tempo da partitura a 60 / tempo segundos por tempo, com o numerador da fórmula de compasso de tempos por compasso, e SHALL estar ausente antes dessa âncora ou quando faltar tempo, fórmula ou âncora.

#### Scenario: Segundo compasso
- **WHEN** uma partitura de 80 BPM em 4/4 começando em 0 s está em 3 s
- **THEN** o tempo ativo é o compasso 2, tempo 1, de 3 s a 3,75 s
