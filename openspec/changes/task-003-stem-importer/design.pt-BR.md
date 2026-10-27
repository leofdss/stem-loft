# Design

## Context

Implementa `stem_importer.rs` para a TASK-003, seguindo `docs/architecture.md#consistency-across-a-projects-stems`: uma taxa de amostragem por projeto, todo stem estéreo, `durationSec` lido do cabeçalho. O Command `import_stems` ainda devolve "not yet implemented": saber qual projeto está aberto e salvar as novas entradas fica com a TASK-009, em cima de `StemImporter::import`.

## Goals / Non-Goals

- Objetivo: validar os arquivos de stem, copiá-los para a pasta do projeto (mono convertido para estéreo) e devolver os seus `StemEntry`s.
- Fora do escopo: adicionar as entradas ao projeto aberto e marcá-lo como sujo (TASK-009).
- Fora do escopo: formatos além de WAV, e calcular waveforms (TASK-007).

## Decisions

- **Só WAV, lido por um leitor RIFF próprio.** O `symphonia` (já dependência, só com `wav`/`pcm` ligados) decodificaria; o importador só precisa dos chunks `fmt ` e `data`, então um pequeno leitor de chunks lê esses e mais nada. PCM inteiro e float IEEE são aceitos, simples ou `WAVE_FORMAT_EXTENSIBLE`; chunks desconhecidos são pulados e os bytes de padding respeitados.
- **Duração vinda do cabeçalho.** `durationSec = (tamanho de data / block align) / taxa de amostragem`, então nenhuma amostra é decodificada.
- **O upmix copia bytes, nunca converte.** Os bytes de cada frame mono são escritos duas vezes sob um cabeçalho estéreo canônico de 44 bytes, em blocos de ~64 KiB, então o resultado é idêntico bit a bit nos dois canais em qualquer profundidade. Um stem estéreo é copiado byte a byte.
- **Mais de dois canais é erro.** Diferente do mono, não existe redução sem perda e sem ambiguidade para estéreo, então ele é rejeitado (`TooManyChannels`), como uma taxa de amostragem diferente.
- **Tudo ou nada.** Todo cabeçalho (e a taxa do primeiro stem existente do projeto) é conferido antes de qualquer arquivo ser escrito; se uma escrita falha, os arquivos que aquela chamada já escreveu são removidos.
- **Ids a partir do nome do arquivo, sem sobrescrever.** O id é um slug do nome do arquivo (`Lead Vocals.wav` → `lead-vocals`), com sufixo `-2`, `-3`, ... quando colide com um stem do projeto, outro arquivo da mesma importação ou qualquer arquivo já presente na pasta do projeto.
- **Escritas atômicas em streaming.** Os stems passam pelo novo `ProjectPersistence::atomic_write_with`, a forma em streaming do `atomic_write` (arquivo temporário + `rename`, temporário removido em qualquer falha), então um stem de vários minutos nunca fica inteiro na memória.
- **Entrega com contrato primeiro.** O orquestrador escreveu os testes de contrato, as fixtures WAV (`src-tauri/tests/fixtures/stems/generate.py`) e a escrita em streaming; o implementer Haiku preencheu o módulo contra os testes congelados (verde em 1 execução do check); a regra de nunca sobrescrever e o seu teste vieram da revisão do orquestrador, e a revisão `review-rust-core` levou a checagem de tamanho do upmix para a validação e passou a exigir um `fmt ` extensible completo de 40 bytes.

## Risks / Trade-offs

- Um WAV mono cuja cópia estéreo não caberia nos tamanhos de 32 bits do RIFF (mais de ~2 GiB de amostras) é rejeitado na validação, antes de qualquer escrita.
- O arquivo float convertido tem um `fmt ` simples de 16 bytes e nenhum chunk `fact`; leitores comuns aceitam, um leitor estrito talvez não.
- Desfazer uma importação que falhou é melhor esforço: um arquivo que não pode ser removido fica, e importações seguintes escolhem outro id.
- Um frame parcial no fim de um chunk `data` mono é descartado pelo upmix.
