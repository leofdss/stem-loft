# Ambiente de desenvolvimento (Distrobox)

> **Nota de manutenção:** este documento tem uma versão em inglês em
> [`README.md`](./README.md). Sempre que um dos dois for atualizado,
> atualize o outro na mesma alteração — não deixe as duas versões divergirem.

Esta pasta guarda as duas peças do ambiente de dev do StemLoft: o
`Containerfile` (o que fica *dentro* da imagem) e o `distrobox.ini` (como o
container é *criado* a partir dela).

## Início rápido

Requer [Distrobox](https://distrobox.it/) e Podman no host. A partir da
raiz do repositório:

```bash
distrobox assemble create --file distrobox/distrobox.ini
distrobox enter stemloft
```

O primeiro comando baixa a imagem publicada e cria o container `stemloft`;
é seguro executá-lo de novo — se o container já existir, ele avisa e não
faz nada. Para recriá-lo do zero (para buscar uma imagem mais nova, por
exemplo), adicione `--replace`; só o container é descartado, já que todo o
trabalho vive no `HOME` do host, que é compartilhado.

Dentro do container, o repositório está no mesmo caminho que no host, e
`git`, `claude` e `code` já estão prontos para uso com as próprias
credenciais do host.

## A imagem

O `Containerfile` desta pasta define a imagem de desenvolvimento do
StemLoft: Arch Linux (`quay.io/toolbx/arch-toolbox`, mantida pelo projeto
Toolbx — irmão do Distrobox, mesma família de ferramentas) com Python,
Node.js + fnm, Rust, git, Claude Code, VS Code, e fish/starship para um
shell interativo.

Runtime alvo: **Podman**. Publicação: **GitHub Container Registry
(ghcr.io)**, pública, construída via GitHub Actions
(`.github/workflows/build-dev-image.yml`).

## Por que publicar a imagem

1. **Auditoria externa** — qualquer pessoa fora do projeto pode ler o
   `Containerfile`, ver exatamente o que entra na imagem, e reproduzir o
   build (`podman build -f distrobox/Containerfile .`) sem depender de
   nada pré-empacotado por terceiros.
2. **Confiabilidade de longo prazo** — o build roda semanalmente (Arch é
   rolling release), então a imagem publicada carrega patches de segurança
   recentes em vez de congelar os pacotes que existiam no dia em que
   alguém lembrou de reconstruí-la.

## Decisões relevantes no Containerfile

- **Base `quay.io/toolbx/arch-toolbox`, não `archlinux:base-devel`.** É a
  imagem Arch mantida pelo próprio ecossistema Toolbx/Distrobox — já vem
  com `base-devel`, `git` e `sudo` prontos. Mesmo assim, o `Containerfile`
  ainda precisa rodar `pacman-key --init && pacman-key --populate
  archlinux` antes do primeiro `pacman -Syu`: essa imagem base não vem com
  a chave mestra de assinatura local gerada, e sem ela o hook de
  sincronização do `archlinux-keyring` falha ("no secret key available to
  sign with") durante a atualização desse pacote.
- **Nada em `/home`.** O Distrobox monta o `HOME` do host por cima do
  `HOME` do container — qualquer coisa instalada em `/home/<user>` durante
  o build fica invisível (ou pior, silenciosamente sobrescrita) assim que
  o container roda de verdade. Todo pacote vai para `/usr` (via
  `pacman`/`npm` como root) ou `/opt` (VS Code).
- **Rust via o pacote do repositório (`rust`), não `rustup`.** Por padrão
  o `rustup` vive em `$HOME/.cargo`/`$HOME/.rustup`, o que cairia na mesma
  armadilha do ponto acima. O pacote `rust` do Arch é atualizado com
  frequência (rolling release), o que já cobre a política do projeto de
  manter toolchains sempre na versão estável mais recente (a mesma lógica
  aplicada a Tauri/Angular, veja
  `docs/architecture.md#stack-and-platform`). Se algum dia for necessário
  um target que o pacote do sistema não cobre (ex.: Android), o `rustup`
  pode ser instalado depois, por usuário, dentro do próprio `HOME`
  compartilhado — nesse caso, "vive no HOME" é o comportamento esperado,
  não um acidente de build de imagem.
- **`fnm` e o `nodejs`/`npm` do sistema, os dois.** O `nodejs`/`npm` do
  repositório dão uma versão de sistema sempre pronta; o `fnm` (também do
  repositório) fica disponível para fixar uma versão de Node por projeto
  sempre que isso for necessário algum dia. O diretório de versões do
  `fnm` (`~/.local/share/fnm` por padrão) fica no `HOME` de propósito —
  esse é o comportamento nativo da ferramenta, e os dados pertencem ao
  usuário, não à imagem.
- **Claude Code via `npm install -g` como root, durante o build.** Não há
  pacote oficial nos repositórios do Arch; instalado como root durante o
  build, o que o coloca em `/usr/lib/node_modules` + `/usr/bin/claude` —
  fora do `HOME`.
- **VS Code via o tarball oficial da Microsoft, não o AUR.** Também não
  tem pacote oficial nos repositórios do Arch. A alternativa mais comum
  seria o AUR (`visual-studio-code-bin`), mas isso amarraria a imagem à
  manutenção de um PKGBUILD de terceiros — contra o objetivo de uma imagem
  auditável que "funciona a longo prazo". Em vez disso, o `Containerfile`
  baixa o tarball genérico publicado pela própria Microsoft e o extrai em
  `/opt/vscode`, com um symlink em `/usr/local/bin/code`.
- **Autenticação (git, Claude Code) não recebe tratamento especial aqui.**
  `~/.gitconfig`, `~/.git-credentials`, `~/.claude/`, etc. já vivem no
  `HOME` — como o Distrobox compartilha o `HOME` com o host por padrão,
  fazer login uma vez no host continua funcionando mesmo depois de
  recriar o container. Não há nada para configurar na imagem para isso
  funcionar.
- **Nenhum `USER`/criação de usuário na imagem.** O Distrobox cria o
  usuário do container (mesmo UID/GID/nome do host) e concede sudo no
  momento em que o container é criado, não no momento de build da imagem
  — por isso o pacote `sudo` é instalado, mas nenhuma conta de usuário é
  criada aqui.

## Construindo e testando localmente

```bash
podman build -t stemloft-dev -f distrobox/Containerfile .
podman run --rm -it stemloft-dev bash -lc \
  'git --version && python --version && node --version && fnm --version && rustc --version && claude --version && code --version'
```

## Publicação (GitHub Actions)

O workflow constrói e publica em `ghcr.io/leofdss/stemloft-dev` a cada
push que altera o `Containerfile`, semanalmente (segundas, 06:00 UTC), e
sob demanda (`workflow_dispatch`). Ele já rodou: a imagem está publicada e
pode ser baixada publicamente, com a tag `latest` mais uma tag datada
`YYYYMMDD-<short sha>` por build (útil para fixar um build específico no
`distrobox.ini` quando necessário).

Pacotes no GHCR nascem privados por padrão, mesmo em um repositório
público — essa troca de visibilidade foi um passo manual único na primeira
publicação (aba *Package settings* do pacote `stemloft-dev`, em
`github.com/leofdss/stem-loft/pkgs/container/stemloft-dev`) e já foi
feito. Só volta a ser necessário se o pacote algum dia for apagado e
republicado.

## O container (`distrobox.ini`)

`distrobox.ini` é um manifesto do
[`distrobox assemble`](https://distrobox.it/usage/distrobox-assemble/):
ele declara o container no repositório em vez de deixar isso para uma
longa linha de `distrobox create` digitada à mão, então o ambiente fica
revisável em um diff e idêntico para todo mundo que roda o comando — o
mesmo raciocínio por trás de publicar a imagem.

Decisões relevantes nele:

- **Nome do container: `stemloft`** (o cabeçalho da seção), então o
  comando para entrar é `distrobox enter stemloft`.
- **`pull=true`.** Sempre checa o registro por um `latest` mais novo ao
  criar o container — do contrário, recriá-lo reaproveitaria
  silenciosamente uma cópia local com meses de idade e anularia o
  rebuild semanal.
- **`init=false`** (sem systemd). Nada aqui é um serviço, e `--init` faz o
  Distrobox *deixar de* montar o `XDG_RUNTIME_DIR` do host — que é
  exatamente onde ficam os sockets de Wayland, PipeWire e PulseAudio de
  que a WebView do Tauri e o motor de áudio precisam.
- **`entry=false`.** Sem entrada `.desktop` no menu de aplicativos do
  host: isso é um container de desenvolvimento acessado por terminal, não
  um app. Um app GUI que *deva* aparecer no menu do host pode ser
  exportado por usuário, de dentro do container, com
  `distrobox-export --app code`.
- **`nvidia=false`.** O StemLoft é um app de áudio — a WebView renderiza
  pela pilha regular GTK/WebKit e nada no core precisa da GPU. Vale
  revisitar só se a separação automática de stems algum dia rodar
  localmente em vez de por uma API externa.
- **Sem volumes extras, deliberadamente.** O Distrobox já monta, sem
  nenhuma configuração: o `HOME` do host (então o repositório, o
  `~/.gitconfig` e o `~/.claude` são os mesmos arquivos dentro e fora — o
  compartilhamento do qual o `Containerfile` depende), `/dev` e `/sys`
  (para que `/dev/snd` alcance ALSA/cpal) junto com os grupos
  suplementares do host (`audio`, `video`, …), os sockets do
  `XDG_RUNTIME_DIR`, e todo o filesystem do host sob `/run/host` para
  qualquer coisa fora do `HOME` (stems em outro ponto de montagem,
  digamos). Uma linha `volume=` só é necessária para um caminho que
  precise aparecer no *mesmo* caminho dentro do container que no host.
- **Sem `additional_packages`.** Toda ferramenta pertence ao
  `Containerfile`, onde faz parte da imagem publicada e auditável.
  Instalar a partir do manifesto tornaria o container de cada
  desenvolvedor silenciosamente diferente da imagem que todo mundo mais
  baixa.

Para checar no que o manifesto se expande sem criar nada:

```bash
distrobox assemble create --dry-run --file distrobox/distrobox.ini
```

## Rodando um container a partir de uma imagem construída localmente

O `distrobox.ini` aponta para o `ghcr.io/leofdss/stemloft-dev:latest`
publicado. Para rodar o container a partir de um build local em vez
disso — para testar uma mudança no `Containerfile` antes de fazer push —
construa-a como em
[Construindo e testando localmente](#construindo-e-testando-localmente),
depois edite o manifesto para usar essa imagem e pular o registro:

```ini
image=localhost/stemloft-dev:latest
pull=false
```

Crie-o com `--replace` para trocar um container `stemloft` existente por
um construído na imagem local, e reverta essas duas linhas antes de
commitar.
