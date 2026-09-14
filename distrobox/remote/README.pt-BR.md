# Offload de build remoto

> **Nota de manutenção:** este documento tem uma versão em inglês em
> [`README.md`](./README.md). Sempre que um dos dois for atualizado, atualize
> o outro também na mesma alteração — não deixe as duas versões divergirem.

Opcional: se você tem uma segunda máquina Linux mais rápida e sempre ligada
na mesma rede Tailscale/LAN, com [Distrobox](https://distrobox.it/) e Podman
instalados, você pode transferir para ela as partes do loop de
desenvolvimento que consomem CPU — `cargo check`/`clippy`/`test`/`build`/
`llvm-cov` para o núcleo em Rust, `npm run build`/`test` para a UI em
Angular — enquanto a própria janela do Tauri (e a reprodução de áudio, que
precisa de ALSA/PipeWire local) continua rodando na sua própria máquina.

## Fluxo do dia a dia

Nada muda em como você edita código — você continua trabalhando nos
arquivos localmente, no seu próprio editor. O que isso adiciona é uma
segunda trilha, separada, para um momento diferente do loop:

| | Roda onde | Para que serve | Como |
|---|---|---|---|
| **Rodar o app de verdade** (janela, áudio) | Local | Ver/ouvir o que você construiu | `cargo tauri dev`, como sempre |
| **Só checar se compila / os testes passam** | Host remoto | Feedback rápido, sem gastar CPU local | `distrobox/remote/check.sh` |

`cargo tauri dev` precisa continuar local — ele abre uma janela na sua tela
e toca áudio pela sua placa de som, nenhum dos dois o host remoto tem
acesso. Mas na maior parte do tempo você não está testando a janela nem o
áudio, você está perguntando "isso compila? eu quebrei algum teste?" — isso
é CPU pura, sem necessidade de GUI, e é essa a parte que isso transfere para
os núcleos do host remoto em vez dos da sua própria máquina.

Um loop típico:

1. Edite código localmente, como de costume.
2. Antes de subir o app de verdade, rode `distrobox/remote/check.sh`. Ele
   envia sua árvore de trabalho para o host remoto e roda `cargo
   check`/`clippy`/`test` mais `npm run build`/`test` lá — um tipo errado ou
   um teste quebrado (Rust ou Angular) aparece em bem menos de um minuto,
   sem sua máquina compilar `webkit2gtk` localmente só para descobrir isso.
3. Quando estiver tudo limpo (ou sempre que quiser realmente ver a
   mudança), entre no container local e rode `cargo tauri dev` como antes.

Para pular a etapa de "lembrei de sincronizar?", deixe um terminal rodando
`distrobox/remote/sync.sh --watch` em segundo plano — ele fica enviando
suas edições a cada 2s, então `distrobox/remote/run.sh --dir src-tauri
cargo check` está sempre checando o código atual.

## Por que isso existe

Compilar `src-tauri` traz `webkit2gtk`, `tao`, `muda`, `soup3` e o resto da
pilha GTK/WebKit — um `cargo check` a frio sozinho leva dezenas de segundos
mesmo em hardware decente, e um ciclo completo de `cargo build`/`cargo test`
é pior. Se uma máquina mais forte está ociosa na rede, não há motivo para a
ventoinha da máquina local girar por causa disso.

## Trabalhos anteriores

Essa não é uma ideia nova — é o mesmo padrão que algumas ferramentas já
existentes formalizam:

- [`cargo-remote`](https://github.com/sgeisler/cargo-remote) — um
  subcomando do `cargo` que envia um projeto Rust via rsync para um host
  remoto por SSH, roda o `cargo` lá e copia o `target/` de volta. Este
  diretório é uma reimplementação pequena, consciente de Tauri/Angular,
  dessa mesma ideia (`sync.sh` + `run.sh` em vez de um único comando `cargo
  remote`), de modo a poder acionar as duas metades do projeto (`src-tauri`
  e `ui`) e reaproveitar o próprio `distrobox.ini`/`Containerfile` do
  projeto em vez de assumir um toolchain bare-metal no host remoto.
- [Discussão do Tauri #6357, "Developing a Tauri app over
  SSH"](https://github.com/tauri-apps/tauri/discussions/6357) — alguém com
  um notebook fraco e um desktop potente esbarrou exatamente na mesma
  versão do problema para apps com GUI: o conteúdo da WebView (o servidor
  de dev do Angular/vite) pode ser servido remotamente por ser só HTTP,
  enquanto o shell nativo (janela + áudio) ainda precisa rodar localmente.
- [`sccache` da Mozilla](https://github.com/mozilla/sccache) — a versão
  "industrial" de "terceirizar a compilação para outras máquinas", usada
  no próprio build do Firefox. Vale a pena migrar para ele em vez disso se
  a equipe algum dia crescer a ponto de valer a pena um cache distribuído
  compartilhado em vez de um único host remoto dedicado.
- [Mutagen](https://mutagen.io/) — uma alternativa mais robusta ao loop de
  rsync por polling do `sync.sh`, caso sincronização contínua bidirecional
  de baixa latência (em vez de um loop só de envio) passe a valer a pena
  como dependência extra.

## Configuração

1. No host remoto: Distrobox + Podman instalados, `~/Projects/` com
   permissão de escrita, e espaço em disco suficiente para um target dir de
   Rust + node_modules (redirecione `CARGO_TARGET_DIR` via `[build]
   target-dir` do `~/.cargo/config.toml` e o cache do npm via `cache=` do
   `~/.npmrc` se o disco raiz estiver apertado).
2. `distrobox assemble create --file distrobox/distrobox.ini` no host
   remoto, a partir de uma cópia sincronizada deste repositório — cria o
   mesmo container `stemloft` documentado em `distrobox/README.md`, só que
   naquela máquina.
3. `cp distrobox/remote/host.local.example distrobox/remote/host.local` e
   defina `STEMLOFT_REMOTE_HOST` (`usuario@host`) e `STEMLOFT_REMOTE_PATH`
   (onde o projeto fica nesse host, ex.: `~/Projects/stem-loft`) —
   ignorado pelo git, é por desenvolvedor, não compartilhado.

## Uso

```bash
distrobox/remote/sync.sh              # envio único da árvore de trabalho
distrobox/remote/sync.sh --watch      # continua enviando a cada 2s enquanto você edita

distrobox/remote/run.sh --dir src-tauri cargo check
distrobox/remote/run.sh --dir src-tauri cargo clippy --all-targets
distrobox/remote/run.sh --dir src-tauri cargo test
distrobox/remote/run.sh --dir src-tauri cargo llvm-cov --summary-only
distrobox/remote/run.sh --dir ui npm run build
distrobox/remote/run.sh --dir ui npm run test -- --watch=false

distrobox/remote/check.sh             # sync + check + clippy + test + build + testes do Angular, tudo de uma vez
```

`sync.sh` exclui `.git`, `target/`, `node_modules/`, `dist/`, `.angular/`
— só o código-fonte trafega pela rede; artefatos de build são regenerados
remotamente e nunca voltam automaticamente por padrão (nada aqui copia o
`target/` de volta, diferente do `cargo-remote` — o objetivo é feedback
rápido sobre erros/avisos/resultados de teste, não rodar localmente o
binário construído remotamente, já que os handles de ALSA/janela que ele
precisaria não estão lá).

## O que isso deliberadamente não faz

- **Não roda a janela de `cargo tauri dev` de verdade remotamente.** A
  reprodução de áudio (cpal/ALSA) e a janela da WebView precisam estar na
  máquina em que você está sentado. Isso transfere a compilação, não a
  execução.
- **Não sincroniza de volta automaticamente.** Se um `cargo check` remoto
  mudar o `Cargo.lock` (uma dependência nova), copie-o de volta manualmente
  — essa é uma escolha deliberada que alguém deve revisar, não algo para
  sincronizar às cegas.
