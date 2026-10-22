# Design

## Context

Implementa `waveform.rs` para a TASK-007, seguindo `docs/architecture.md#design-note-waveform-cache-on-disk`: picos calculados uma vez por stem, guardados em cache ao lado do projeto pela escrita atômica e recalculados quando o stem muda. Calcular os picos é responsabilidade do `Audio Engine`; a escrita é do `Project Persistence`.

## Goals / Non-Goals

- Objetivo: calcular os picos de um stem, guardá-los em cache, servi-los do cache quando ele é válido e invalidá-los.
- Fora do escopo: chamar isto a partir da Session na importação ou na abertura, registrar `stems[].waveformCache` e emitir `waveform_ready`. Essa ligação pertence ao trabalho de eventos e é o critério 1.6 da TASK-010.

## Decisions

- **Resolução e definição de pico.** 100 picos por segundo: o frame `i` pertence ao bucket `i × 100 / taxa de amostragem`, então os picos acompanham o `durationSec` em qualquer taxa (um número fixo de `taxa / 100` frames por bucket deriva a 22050 Hz). Um pico é o maior valor absoluto de amostra entre todos os canais no seu bucket, em escala 0–1. O último bucket, mais curto, conta.
- **Decodificação manual, reaproveitando o leitor de cabeçalho do importador.** Os stems de um projeto são sempre WAV (TASK-003), então o módulo lê as amostras direto do chunk `data` com `read_wav_header`, em blocos de ~64 KiB: 8 bits sem sinal, 16/24/32 bits com sinal, float de 32/64 bits. Qualquer outra profundidade é erro.
- **Arquivo de cache que se valida sozinho.** `<id do stem>.waveform` guarda um pequeno cabeçalho (magic, versão, tamanho e data de modificação da origem, picos por segundo, quantidade) e os picos como f32. Qualquer coisa que não confira, inclusive lixo, uma versão antiga, um stem substituído no disco ou um pico fora de 0–1, é recalculada, nunca vira erro. O cabeçalho é conferido antes de o corpo ser lido.
- **Invalidação.** `invalidate` apaga o cache de um stem (ausente não é erro), para o fluxo de reimportação; um stem substituído fora do app é pego pela conferência de tamanho/mtime.
- **O cache é só uma otimização.** Se ele não puder ser escrito, os picos calculados ainda são devolvidos e a próxima abertura os calcula de novo.
- **Entrega com contrato primeiro.** O orquestrador escreveu os testes de contrato; o implementer Haiku preencheu o módulo e parou num teste falhando, relatando que o teste esperava um bucket onde a regra dá dois (dois frames a 100 Hz). O orquestrador corrigiu esse teste (200 Hz) em vez do código, e, depois da revisão `review-rust-core`, acrescentou a regra do cache que não pode ser escrito, limites de bucket exatos, um teste de desatualização só por mtime, a leitura do cache começando pelo cabeçalho e uma conferência de faixa nos picos em cache.

## Risks / Trade-offs

- A data de modificação de um arquivo pode continuar igual depois de uma regravação de mesmo tamanho em sistemas de arquivos com pouca resolução; essa substituição manteria os picos antigos até uma reimportação invalidá-los.
- Os picos são calculados na thread de quem chama; quem chama (TASK-010) precisa manter isso fora da thread de tempo real e, de preferência, fora da fila de Commands para stems longos.
