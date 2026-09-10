---
name: chordpro-format
description: Gramática e regras de validação do dialeto ChordPro estendido (.cho) usado pelo Gerenciador de Metadados de Partitura do StemLoft — âncoras {t:}, um acorde por linha, gramática da tablatura, resolução de startSec/endSec. Use ao implementar ou testar o parser do .cho, ou ao criar/editar arquivos .cho de exemplo/teste.
---

# Formato `.cho` (ChordPro estendido)

O StemLoft usa [ChordPro](https://www.chordpro.org/) padrão mais duas
extensões próprias e uma restrição de dialeto. Fonte completa, com exemplos:
[doc.md — Metadados de partitura](../../../docs/doc.md#metadados-de-partitura-acordes-tablatura-e-letra).
Este skill é o resumo operacional pra implementar/testar o parser sem ter
que reconstituir as regras a cada vez.

## As duas extensões

- **`{tuning: E A D G B E}`** — afinação, corda grave→aguda, mesma ordem que
  `frets` em `{define}`.
- **`{t: minuto:segundo.centesimo}`** — âncora de tempo absoluto, início de
  linha. Gramática exata: `minuto` 1+ dígitos sem zero à esquerda
  obrigatório; `segundo` sempre 2 dígitos (00–59); `centesimo` sempre 2
  dígitos (00–99). Válido: `0:00.00`, `1:05.30`, `12:40.00`. Inválido:
  `00.5`, `1:5.3` (faltam dígitos).

Diretivas desconhecidas (incluindo essas duas, pra qualquer leitor ChordPro
que não é o StemLoft) são ignoradas, não erro — é o mecanismo de extensão
esperado do formato. Não trate `{tuning}`/`{t}` como obrigatórias pra um
arquivo ser "válido" fora do nosso parser.

## Regra do dialeto: no máximo um acorde por linha

`{t:}` só ancora o *início* da linha — um segundo `[acorde]` na mesma linha
não teria horário próprio. **O parser deve rejeitar (`score_parse_error`)**
uma linha com mais de um `[acorde]`. Isso é validação obrigatória, não
sugestão de estilo.

## Gramática da tablatura (`{start_of_tab}` … `{end_of_tab}`)

```
item        := nota ("-" nota)* | "-"        ; "-" sozinho = descanso
nota        := traste (op traste | "~")* "." corda
op          := "h" | "p" | "b" | "r" | "/" | "\"
traste      := digito digito?                ; 0-24, sem zero à esquerda
corda       := "1" | "2" | "3" | "4" | "5" | "6"
digito      := "0".."9"
```

`~` (vibrato) é o único operador sem traste-alvo — não é seguido de dígito.
Exemplo válido: `8~~b10r8` = `traste=8`, dois `~` em sequência, depois `b`
`10` `r` `8`.

**Regras de validação que o parser deve rejeitar/avisar:**

| Regra | Motivo |
|---|---|
| `r` só é válido depois de pelo menos um `b` na mesma nota | Release sem bend anterior não tem o que soltar |
| Duas notas do mesmo `item` (separadas por `-`) não podem apontar pra mesma corda | Fisicamente uma corda só soa uma altura por vez |
| `traste` fora de 0–24 | Limite físico do braço |
| `corda` fora de 1–6 | Afinação de 6 cordas |

Tabela de técnicas (referência de significado, não de gramática):

| Símbolo | Técnica | Exemplo |
|---|---|---|
| `h` | Hammer-on | `5h7` |
| `p` | Pull-off | `7p5` |
| `b` | Bend | `7b9` |
| `r` | Release | `7b9r7` |
| `/` | Slide ascendente | `5/7` |
| `\` | Slide descendente | `7\5` |
| `~` | Vibrato | `8~` |

## Resolução de `startSec`/`endSec`

- `startSec` de um acorde = `{t:}` da linha onde o `[acorde]` aparece.
- `endSec` = `{t:}` do **próximo evento que muda o que está soando** (próximo
  `[acorde]` ou próximo `{start_of_tab}`). Uma linha de letra **sem**
  colchete não encerra o acorde atual — só avança o texto exibido.
- **Último acorde do arquivo**: não existe "próximo evento" — `endSec` =
  maior `stems[].durationSec` do projeto, lido do `.json`, **não**
  decodificado na hora. Pra fechar antes do fim real, o autor do `.cho`
  adiciona uma linha final só com `{t:}` + `[acorde]` (pode repetir o mesmo
  nome).
- **Origem da grade de compasso** (compasso 1, batida 1) = o **primeiro
  `{t:}` do arquivo**, não necessariamente o segundo 0 do áudio. Trecho antes
  disso (intro/contagem) fica fora da numeração de compassos — isso é
  esperado, não bug.
- **Tempo dentro de `{start_of_tab}`**: uma única âncora `{t:}` no início do
  bloco; posição de cada nota é proporcional à coluna do caractere na linha:
  `notaSec = startSec + (coluna / totalDeColunas) × (endSec − startSec)`,
  usando o mesmo `endSec` (próximo evento) e o comprimento da linha (todas as
  6 cordas têm o mesmo número de colunas). Não precisa de âncora por nota —
  se seu código está procurando `{t:}` dentro de um bloco de tab por nota,
  está resolvendo isso do jeito errado.

## Checklist ao implementar/alterar o parser

- [ ] Linha com mais de um `[acorde]` → `score_parse_error`, não a primeira
      ocorrência silenciosamente aceita.
- [ ] `{t:}` mal formado (dígitos faltando em segundo/centésimo) →
      `score_parse_error` com a linha do arquivo onde ocorreu — a mensagem
      de erro precisa apontar a linha, não só "arquivo inválido".
- [ ] Erro de parsing do `.cho` **não trava o resto do app** — stems, loop,
      waveform continuam normais; só `Visualização de Acordes/Tablatura` fica
      em estado de erro. Se seu código propaga o erro pra algo que afeta
      playback, isso é bug.
- [ ] Diretiva desconhecida é ignorada, não rejeitada.
- [ ] `endSec` do último acorde vem de `stems[].durationSec` (já persistido,
      sem decodificar áudio) — não do Motor de Áudio diretamente.
