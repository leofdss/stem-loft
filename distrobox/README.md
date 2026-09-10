# Imagem de desenvolvimento (Distrobox)

`Containerfile` desta pasta define a imagem de desenvolvimento do Stem
Player: Arch Linux (`quay.io/toolbx/arch-toolbox`, mantida pelo projeto
Toolbx — irmão do Distrobox, mesma família de ferramentas) com Python,
Node.js + fnm, Rust, git, Claude Code e VS Code.

Runtime alvo: **Podman**. Publicação: **GitHub Container Registry (ghcr.io)**,
pública, buildada via GitHub Actions (`.github/workflows/build-dev-image.yml`).

## Por que publicar a imagem

1. **Auditoria externa** — qualquer pessoa de fora do projeto pode ler o
   `Containerfile`, ver exatamente o que entra na imagem e reproduzir o build
   (`podman build -f distrobox/Containerfile .`) sem depender de nada
   pré-empacotado por terceiros.
2. **Garantia de funcionamento a longo prazo** — o build roda semanalmente
   (Arch é rolling release), então a imagem publicada carrega patches de
   segurança recentes em vez de congelar pacotes do dia em que alguém lembrou
   de rebuildar.

## Decisões relevantes do Containerfile

- **Base `quay.io/toolbx/arch-toolbox`, não `archlinux:base-devel`.** É a
  imagem Arch mantida pelo próprio ecossistema Toolbx/Distrobox — já vem com
  `base-devel`, `git` e `sudo` prontos. Mesmo assim o `Containerfile` ainda
  precisa rodar `pacman-key --init && pacman-key --populate archlinux` antes
  do primeiro `pacman -Syu`: essa imagem base não traz a chave mestra de
  assinatura local gerada, e sem ela o hook de sincronização do
  `archlinux-keyring` falha ("no secret key available to sign with") durante
  a atualização desse pacote.
- **Nada em `/home`.** O Distrobox monta o `HOME` do host por cima do `HOME`
  do container — qualquer coisa instalada em `/home/<user>` durante o build
  fica invisível (ou pior, silenciosamente sobrescrita) quando o container
  roda de verdade. Todo pacote vai para `/usr` (via `pacman`/`npm` como root)
  ou `/opt` (VS Code).
- **Rust via pacote do repo (`rust`), não `rustup`.** `rustup` por padrão
  vive em `$HOME/.cargo`/`$HOME/.rustup`, o que cairia na mesma armadilha do
  ponto acima. O pacote `rust` do Arch é atualizado com frequência (rolling
  release), o que já cobre a política do projeto de manter toolchains
  sempre na versão estável mais recente (mesma lógica aplicada a
  Tauri/Angular, ver `docs/doc.md#stack-e-plataforma`). Se um dia for preciso
  compilar para um target que o pacote do sistema não cobre (ex.: Android),
  `rustup` pode ser instalado depois, por usuário, dentro do próprio `HOME`
  compartilhado — nesse caso o comportamento "vive em HOME" é o esperado, não
  um acidente de build de imagem.
- **`fnm` + `nodejs`/`npm` do sistema, os dois.** `nodejs`/`npm` do repo dão
  uma versão de sistema sempre pronta pra uso; `fnm` (também do repo) fica
  disponível pra fixar uma versão de Node por projeto quando algum dia for
  necessário. O diretório de versões do `fnm` (`~/.local/share/fnm` por
  padrão) fica em `HOME` de propósito — é o comportamento nativo da
  ferramenta e o dado é do usuário, não da imagem.
- **Claude Code via `npm install -g` como root, dentro do build.** Sem
  pacote oficial no repo do Arch; instalado como root durante o build, o que
  o coloca em `/usr/lib/node_modules` + `/usr/bin/claude` — fora de `HOME`.
- **VS Code via tarball oficial da Microsoft, não AUR.** Também sem pacote
  oficial no repo do Arch. A alternativa mais comum seria AUR
  (`visual-studio-code-bin`), mas isso amarra a imagem à manutenção de um
  PKGBUILD de terceiros — contra o objetivo de "funcionar a longo prazo" de
  forma auditável. Em vez disso, o `Containerfile` baixa o tarball genérico
  publicado pela própria Microsoft e extrai para `/opt/vscode`, com symlink
  em `/usr/local/bin/code`.
- **Autenticação (git, Claude Code) não recebe tratamento especial aqui.**
  `~/.gitconfig`, `~/.git-credentials`, `~/.claude/`, etc. já vivem em `HOME`
  — como o Distrobox compartilha `HOME` com o host por padrão, login feito
  uma vez no host continua valendo mesmo recriando o container. Não há nada
  a configurar na imagem para isso funcionar.
- **Nenhum `USER`/criação de usuário na imagem.** O Distrobox cria o usuário
  do container (mesmo UID/GID/nome do host) e concede sudo na hora de criar
  o container, não no build da imagem — por isso o pacote `sudo` está
  instalado, mas nenhuma conta de usuário é criada aqui.

## Build e teste local

```bash
podman build -t stem-player-dev -f distrobox/Containerfile .
podman run --rm -it stem-player-dev bash -lc \
  'git --version && python --version && node --version && fnm --version && rustc --version && claude --version && code --version'
```

## Publicação (GitHub Actions)

O workflow builda e publica em `ghcr.io/<owner>/stem-player-dev` a cada push
que mude o `Containerfile`, semanalmente (segundas, 06:00 UTC) e sob demanda
(`workflow_dispatch`). Ele só roda depois que este repositório existir no
GitHub e o push for feito para lá — hoje o projeto ainda é só local.

**Passo manual único, na primeira publicação:** pacotes no GHCR nascem
privados por padrão, mesmo em repositório público. Depois do primeiro build
publicado, é preciso ir em *Package settings* do pacote `stem-player-dev` (em
`github.com/<owner>/stem-player-dev/pkgs/container/stem-player-dev`, aba
*Package settings*) e trocar a visibilidade para *Public* — só nessa vez.

## Próximo passo

Este `Containerfile` resolve só a **imagem**. A configuração do Distrobox em
si (arquivo de `assemble`, montagens extras, nome do container) é o próximo
passo, depois que a imagem estiver publicada.
