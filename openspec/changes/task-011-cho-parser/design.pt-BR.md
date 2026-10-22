# Design

## Context

Implementa `score_metadata.rs` para a TASK-011: o dialeto ChordPro estendido de `docs/architecture.md#score-metadata-chords-tablature-and-lyrics` e do playbook `chordpro-format`. O módulo é somente leitura na v1 e não depende do Tauri; os eventos `score_loaded`/`score_parse_error` que levam o resultado chegam com a TASK-012.

## Goals / Non-Goals

- Objetivo: transformar o texto `.cho` em um `Score` (metadados, acordes com tempo, linhas de letra, notas de tablatura com tempo) ou em um `ScoreParseError` com a linha (a partir de 1) e uma mensagem.
- Fora do escopo: emitir eventos ou chamar o parser a partir da `Session` (TASK-012).
- Fora do escopo: escrever o `.cho` (a edição de acordes fica para depois da v1).

## Decisions

- **Tablatura é uma pauta ASCII.** O exemplo de tablatura da arquitetura é uma pauta de seis linhas (`E|--5h7--|`), enquanto a gramática escreve uma nota como `fret (op fret | "~")* "." string`. As duas valem ao mesmo tempo: cada sequência de caracteres diferentes de `-` numa linha da pauta é uma `note`, e o seu `.string` é a linha onde ela está (linha de cima = corda 1). Um `|` dentro da pauta é barra de compasso e separa notas como o `-`. Um bloco precisa de exatamente seis linhas de mesma largura; a coluna de uma nota é contada depois do `<nome>|` inicial.
- **Uma âncora é tomada pela próxima linha.** `{t:}` deixa uma âncora pendente que a próxima linha de acorde/letra ou `{start_of_tab}` consome, esteja ela na mesma linha ou numa seguinte. Um `[acorde]` sem âncora pendente é erro, porque o seu `startSec` não teria como ser resolvido.
- **Âncoras avançam e ficam dentro do projeto.** Uma âncora anterior à anterior, ou depois da duração do projeto (o maior `stems[].durationSec`, passado por quem chama), é um `score_parse_error`. Qualquer uma das duas geraria um acorde com `endSec` antes do `startSec`.
- **Valores de diretiva estritos.** `{tempo}`, `{time}`, `{tuning}` (exatamente seis nomes) e `{define}` (exatamente seis trastes, 0–24 ou `x`/`X`/`N`) são validados; um valor malformado é erro na sua linha. Diretivas desconhecidas, e as conhecidas que o StemLoft não usa (`{c:}`, `{capo}`), são ignoradas.
- **Erros, nunca panics.** Todo helper devolve `Result`; números usam parse com verificação, então um minuto enorme em `{t:}` é erro, não overflow. Um arquivo que não pode ser lido é informado como linha 0.
- **Entrega com contrato primeiro.** O orquestrador escreveu os testes de contrato, a gramática de tablatura e a resolução; o implementer Haiku preencheu os sete helpers de diretiva/linha contra os testes congelados (verde em 2 execuções do check).

## Risks / Trade-offs

- Uma frase de tablatura maior que uma pauta precisa do seu próprio `{t:}` + bloco; não há bloco com várias pautas.
- As entradas de traste do `{define}` aceitam zero à esquerda (`03`), e uma palavra-chave repetida fica com o último valor; as duas coisas são inofensivas e ficaram tolerantes.
