---
name: atomic-persistence
description: Padrão de escrita em disco da Persistência de Projetos no StemLoft — write-to-temp-file + rename sem exceção, quando gravar (checkpoint com debounce, não a cada Command), e invalidação de cache. Use ao adicionar qualquer escrita nova em disco (projeto .json, cache de waveform, .cho futuro) ou ao mexer em quando/como o projeto é salvo.
---

# Persistência: escrita atômica e checkpoints

## A regra sem exceção

**Toda** escrita em disco feita por `Persistência de Projetos` é atômica:
grava num arquivo temporário no mesmo diretório do destino, depois `rename`
pro caminho final. `rename` é atômico a nível de sistema de arquivos — o
arquivo anterior nunca fica truncado ou parcialmente escrito se o app
crashar no meio. Pior caso possível: perder a escrita em andamento, nunca
corromper o que já existia. Isso vale pro `.json` do projeto, pro cache de
waveform, e vai valer pro `.cho` quando a escrita de acordes entrar (pós-v1).
Ver
[nota de design completa](../../../docs/doc.md#nota-de-design-quando-persistência-de-projetos-grava).

Se você está escrevendo um `File::write`/`std::fs::write` direto no caminho
final de algo que `Persistência de Projetos` é dona, pare — isso é o
antipadrão que essa decisão existe pra evitar.

## Quando gravar (não é "a cada mudança")

`Mixer de Stems` gera `Commands` a cada tick de um fader sendo arrastado —
gravar a cada um seria I/O descartado e risco de escrita concorrente/parcial.
O padrão:

- Mudança de estado marca o projeto como "sujo" (dirty flag), não dispara
  escrita imediata.
- Escrita real acontece em **checkpoints**: debounce de inatividade (alguns
  ms sem novos `Commands`) ou eventos definitivos (pausar playback, fechar o
  projeto).
- **Leitura continua imediata** — só a escrita é agrupada.

Se você está adicionando um Command que muda estado persistido, não invente
um checkpoint próprio pra ele — use o mecanismo de dirty-flag + debounce que
já existe. Um checkpoint por feature é o mesmo erro que "um `Mutex` por
feature" no núcleo: fragmenta um mecanismo que devia ser único.

## Cache da waveform (aplicação concreta do padrão)

- Calculado uma vez por stem (decodificação inteira é cara — perf em
  hardware fraco é requisito do projeto, não nice-to-have).
- Gravado em disco junto do projeto, campo `stems[].waveformCache` no
  `.json`, com a mesma escrita atômica de qualquer outro arquivo.
- **Invalidado se o stem for reimportado/substituído** — se você está
  mexendo no fluxo de reimportação e não está descartando o cache antigo,
  isso é bug: a waveform exibida ficaria dessincronizada do áudio real.
- Ver [nota de design completa](../../../docs/doc.md#nota-de-design-cache-da-waveform-em-disco).

## Versionamento do `.json` do projeto

O `.json` do projeto **não** tem a tolerância a campos desconhecidos que o
`.cho` tem (diretivas ChordPro desconhecidas são só ignoradas por outros
leitores). Por isso `schemaVersion` é lido antes de qualquer outra coisa:

- Mesma versão → carrega direto.
- Versão menor → aplica migrações registradas em sequência (cada uma sabe
  transformar `N` → `N+1`) antes de expor o projeto ao resto do núcleo.
- Versão maior que a suportada → erro explícito ("projeto salvo por uma
  versão mais nova do app"), **nunca** uma tentativa silenciosa de leitura
  parcial.

Se você está adicionando um campo novo ao schema do `.json`, pergunte: isso
quebra a leitura de um projeto salvo por uma versão anterior do app? Se sim,
precisa de uma migração registrada — não é opcional, mesmo que pareça "só um
campo a mais". Ver
[schema completo e exemplo](../../../docs/doc.md#versionamento-do-projeto).

## Checklist rápido

- [ ] A escrita passa por write-to-temp-file + rename, sem exceção?
- [ ] A escrita está agrupada num checkpoint (dirty flag + debounce ou evento
      definitivo), não disparada a cada `Command`?
- [ ] Se isso invalida um cache existente (waveform, e no futuro outros),
      o código de invalidação está no lugar certo (reimportação/substituição)?
- [ ] Se o schema do `.json` mudou de forma incompatível, existe migração
      `N → N+1` registrada?
