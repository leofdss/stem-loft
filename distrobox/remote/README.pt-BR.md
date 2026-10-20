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
   envia sua árvore de trabalho para o host remoto e roda o
   `scripts/verify.sh` lá — o passe completo de verificação do repositório — então um tipo errado ou
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
4. `python3 scripts/check_remote_image.py --fix` a partir da sua máquina
   local — faz o container `stemloft` remoto rodar **exatamente a mesma
   imagem do seu container local** (baixada pelo digest, mantendo todas as
   outras configurações do `distrobox.ini`). A referência é o container
   local, não o `latest`, já que `latest` aponta para uma imagem diferente
   em cada máquina assim que uma delas baixa um build mais novo. O
   `check.sh` roda o mesmo script (sem `--fix`) antes de cada execução e
   para se houver divergência, para que uma imagem remota desatualizada
   não dê resultados diferentes dos que a sua máquina daria. Sempre que
   recriar o seu container local, repita este passo.

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

distrobox/remote/check.sh             # sync + scripts/verify.sh (o passe completo), tudo de uma vez
```

`sync.sh` exclui `.git`, `target/`, `node_modules/`, `dist/`, `.angular/`
— só o código-fonte trafega pela rede; artefatos de build são regenerados
remotamente e nunca voltam automaticamente por padrão (nada aqui copia o
`target/` de volta, diferente do `cargo-remote` — o objetivo é feedback
rápido sobre erros/avisos/resultados de teste, não rodar localmente o
binário construído remotamente, já que os handles de ALSA/janela que ele
precisaria não estão lá).

## Modelos locais

O mesmo host pode servir modelos de linguagem locais na GPU dele, para que
qualquer pessoa ou agente de IA possa passar uma tarefa verificável
mecanicamente para um modelo que não custa nada por chamada e nunca sai da
tailnet. Ele roda o [Ollama](https://ollama.com/) como um serviço Podman
Quadlet sem root (`stemloft-ollama`), com a API presa só ao IP do Tailscale
do host e os modelos em nuvem desligados. A GPU é detectada pela versão
ROCm do Ollama (AMD); para outro fabricante, mude a tag da imagem e os
dispositivos em [`ollama/ollama.container.in`](ollama/ollama.container.in).

```bash
distrobox/remote/ollama/install.sh        # install/update the service + pull the models
distrobox/remote/power.sh up ollama       # start it for a work session
python3 scripts/local_llm.py --status     # is it up, which models are pulled/loaded
distrobox/remote/power.sh down            # done: stop ollama and the build container
```

**Nada roda enquanto ninguém trabalha.** O host de build costuma ser a
máquina pessoal de alguém, usada para outras coisas também, então o
serviço nunca sobe no boot (o `ollama.container.in` não tem seção
`[Install]`), descarrega um modelo ocioso depois de 5 minutos, e o
`check.sh` desliga o container de build quando o passe termina. Quem
encerra uma sessão de trabalho — uma pessoa ou um agente de IA — roda
`distrobox/remote/power.sh down`; o `power.sh status` mostra o que está
rodando e quanto de VRAM está em uso.

Depois, adicione ao seu `host.local` a URL que o `install.sh` imprime
(`export STEMLOFT_OLLAMA_URL=http://<ip-do-tailscale>:11434`). Quem chama
pede um **papel**, nunca um modelo — os papéis ficam em
[`ollama/models.toml`](ollama/models.toml), cujos padrões cabem numa GPU
com cerca de 8 GB de VRAM (ajuste para a sua e rode o `install.sh` de
novo):

| Papel | Modelo padrão | Use para |
|---|---|---|
| `quick` | `qwen3:8b` | mensagens de commit, resumos de PR, classificar um diff |
| `translate` | `qwen3:8b` (o mesmo modelo, sem recarregar) | rascunhos em pt-BR via `scripts/translate_md.py` |
| `code` | `qwen2.5-coder:7b` | edições mecânicas validadas por `cargo check`/`tsc`/testes, por exemplo pelo `scripts/llm_task.py` |
| `heavy` | `qwen3-coder:30b` | sessões mais longas, com contexto maior (um MoE que pode transbordar da VRAM para a RAM) |

Regras de projeto que os scripts garantem, porque um modelo não as segue
de forma confiável só porque foi pedido:

- **O modelo nunca vê a estrutura que não pode mudar.** O
  `scripts/translate_md.py` envia só trechos de prosa (JSON com schema) e
  rejeita um trecho cujo código inline ou `SHALL`/`MUST` mudou; o
  `scripts/llm_task.py` congela os testes (`--freeze-from`) e remove os
  números de linha que um modelo devolve.
- **O mesmo modelo nunca escreve o código e o teste que o julga** — senão
  ele pode "corrigir" código certo para bater com a própria expectativa
  errada.
- **Verificações travadas são encerradas**, não esperadas: o `llm_task.py`
  roda a verificação no próprio grupo de processos, com timeout.
- **A saída é um rascunho.** Um script, um modelo mais forte ou uma pessoa
  confere antes de ela entrar (veja "Choosing who does the work" no
  [`AGENTS.md`](../../AGENTS.md)).

Guarde as medições do seu próprio host — velocidades, quais modelos cabem,
execuções de laboratório — em `distrobox/remote/lab.local.md` (ignorado
pelo git, como o `host.local`): elas descrevem uma máquina, não o projeto.

## O que isso deliberadamente não faz

- **Não roda a janela de `cargo tauri dev` de verdade remotamente.** A
  reprodução de áudio (cpal/ALSA) e a janela da WebView precisam estar na
  máquina em que você está sentado. Isso transfere a compilação, não a
  execução.
- **Não sincroniza de volta automaticamente.** Se um `cargo check` remoto
  mudar o `Cargo.lock` (uma dependência nova), copie-o de volta manualmente
  — essa é uma escolha deliberada que alguém deve revisar, não algo para
  sincronizar às cegas.
