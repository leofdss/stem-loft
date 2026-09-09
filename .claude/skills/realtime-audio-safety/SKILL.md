---
name: realtime-audio-safety
description: Checklist de segurança para qualquer mudança que toque o Motor de Áudio ou o callback de tempo real do cpal no Stem Player — o que pode e não pode rodar dentro do callback, como crossfade/decodificação/volume-mute-solo são resolvidos fora dele. Use antes de editar qualquer código perto de cpal, decodificação, mixagem, ou do loop de reprodução.
---

# Segurança da thread de tempo real (Motor de Áudio)

O callback de áudio do `cpal` roda em uma thread de tempo real: **nada que
aloque, bloqueie em lock ou faça I/O pode rodar dentro dele** — um único
underrun já é audível como glitch. Isso é uma restrição física, não uma
preferência de estilo, e vale mesmo quando parecer conveniente violá-la "só
essa vez". Fonte:
[doc.md — Motor de Áudio e a thread de tempo real](../../../docs/doc.md#nota-de-design-motor-de-áudio-e-a-thread-de-tempo-real).

## A regra em uma frase

**Dentro do callback**: só leitura de buffers já prontos (ring buffer /
double-buffer), mixagem por multiplicação/soma de amostras já decodificadas,
e o crossfade do loop (que também é só matemática sobre amostras já
decodificadas). Nada mais.

**Fora do callback, adiantado**: decodificação de arquivo, alocação de
buffer, cálculo de waveform, qualquer I/O de disco.

## Checklist antes de escrever código no `Motor de Áudio`

- [ ] **Isso lê ou escreve arquivo?** → tem que rodar fora do callback, numa
      thread de decodificação, entregando pro callback via `rtrb` (SPSC,
      wait-free) — nunca disco direto dentro do callback.
- [ ] **Isso aloca (`Vec::new()`, `String::new()`, `Box::new()`, clone de
      algo não trivial)?** → não pode estar no caminho do callback. Pré-aloque
      fora e reuse.
- [ ] **Isso faz `lock()` de um `Mutex`?** → não pode estar no callback. Volume,
      mute, solo chegam via variável atômica ou double-buffer publicada pelo
      handler de `Command` fora do callback — nunca via `Mutex` que o callback
      possa ficar esperando. Ver
      [nota de design do modelo de concorrência](../../../docs/doc.md#nota-de-design-modelo-de-concorrência-do-estado-compartilhado)
      pra entender a fronteira entre os dois mecanismos (`Mutex` do estado
      geral vs. atômicos do callback — são coisas diferentes, não confundir).
- [ ] **É lógica de crossfade no limite do loop?** → precisa que o *início*
      do loop (a partir de `startSec`) já esteja num buffer separado, pronto
      *antes* do callback chegar no fim do loop — preparado fora do callback
      assim que o loop é definido ou reiniciado. O callback só lê os dois
      buffers (o que termina + o que começa) e mistura por multiplicação/soma.
      Duração exata e curva (linear/equal-power) são detalhe de implementação
      livre — não é decisão de arquitetura pendente. Ver
      [nota de design completa](../../../docs/doc.md#nota-de-design-crossfade-no-limite-do-loop).
- [ ] **Mexe em canais/formato de sample?** → todo buffer que chega ao
      callback de mixagem já é estéreo (upmix mono→estéreo acontece na
      importação, não no playback) e já foi validado como mesmo sample rate
      entre todos os stems do projeto (rejeição na importação, sem resample
      silencioso). O `Motor de Áudio` nunca precisa lidar com mono nem com
      sample rate divergente — se seu código está tratando esses casos dentro
      do motor, o bug está na importação, não aqui.

## Crates e por que elas foram escolhidas (não reabra a decisão)

| Responsabilidade | Crate | Motivo (não reavaliar sem medir) |
|---|---|---|
| Decodificação | `symphonia` | Puro Rust, sem dependência nativa — mesmo código nas 4 plataformas (Linux/Windows/macOS/Android) |
| Buffer decodificação→callback | `rtrb` | SPSC wait-free, API estreita o suficiente pra não escorregar num uso que viole tempo real (`ringbuf` é mais genérico, não escolhido por isso) |
| Saída de áudio | `cpal` | ALSA/WASAPI/CoreAudio/Oboe atrás da mesma API |

Ponto de atenção Android: o backend Oboe do `cpal` precisa do handle
JVM/contexto que o Tauri mobile expõe — isso é wiring de inicialização, não
lógica de domínio; não deixe esse detalhe vazar pra dentro do `Motor de
Áudio` em si.

## Se você não tem certeza se algo "conta" como código de tempo real

A separação é por **thread**, não por arquivo ou módulo — o `Motor de
Áudio` tem código dos dois lados. Pergunte: "isso roda dentro da closure que
o `cpal` chama a cada buffer de áudio, ou roda antes disso, preparando dado
pra ela ler depois?". Se não souber, assuma que está dentro e aplique a
restrição — o custo de errar pro lado seguro é só um pouco mais de buffer
pré-calculado; o custo de errar pro outro lado é um glitch audível em
produção.
