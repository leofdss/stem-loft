# StemLoft

> **Nota de manutenção:** este documento tem uma versão em inglês em
> [`README.md`](./README.md). Sempre que um dos dois for atualizado,
> atualize o outro na mesma alteração — não deixe as duas versões divergirem.

Uma ferramenta de prática para músicos, construída em torno de repetir em
loop um trecho de uma música e mixar seus stems enquanto você toca junto.

> 🚧 **Em desenvolvimento ativo — ainda não utilizável.** O esqueleto Tauri
> v2 + Angular já está montado (`src-tauri/`, `ui/`) — ele compila e as duas
> camadas estão conectadas — mas todo módulo de domínio no núcleo Rust ainda
> é um stub (`import_stems`, `play`, `set_markers`, e o resto retornam "not
> yet implemented"). Veja [TODO.md](TODO.md) para o que falta antes de o app
> fazer algo de fato.

## Para músicos (quando estiver utilizável)

StemLoft é voltado para **músicos iniciantes** aprendendo uma música a
partir de seus stems — as faixas isoladas de cada instrumento (guitarra,
baixo, bateria, vocais...). A ideia central: marcar um trecho na linha do
tempo e o StemLoft o repete continuamente, no andamento original da
gravação, para que você possa praticar sua parte repetidamente, tocando
junto com ou no lugar de um instrumento da gravação original.

O que isso vai permitir:

- **Repetir um trecho difícil em loop** usando um marcador de início/fim —
  sem precisar rebobinar manualmente.
- **Mixar cada stem você mesmo**: volume, mudo e solo por instrumento — por
  exemplo, silenciar a parte que você está aprendendo e tocar sobre o resto
  da banda.
- **Ver os acordes e a tablatura sincronizados com a reprodução**, a partir
  de um arquivo de partitura em texto simples que você também pode ler e
  editar fora do app.
- **(Planejado)** enviar uma faixa completa e tê-la automaticamente
  separada em stems, em vez de precisar deles já pré-separados.

Nada disso está utilizável ainda — veja o aviso de status acima.

## Para contribuidores

### O que é isso

App desktop construído sobre **Tauri v2**: um núcleo em **Rust** (áudio,
estado, persistência) em [`src-tauri/`](src-tauri), embutido com uma camada
de apresentação em **Angular** em [`ui/`](ui) rodando dentro da WebView do
Tauri. Veja [`docs/architecture.md`](docs/architecture.md) para o design
completo — componentes, fluxo de dados e o raciocínio por trás de cada
decisão.

### Por quê

O projeto minimiza deliberadamente a superfície de ataque na cadeia de
suprimentos (supply chain): a camada Angular usa **apenas recursos nativos
do framework**, nenhum pacote npm de terceiros, e o Rust é a única fonte de
verdade para estado e lógica — o Angular apenas apresenta o que o Rust
reporta e envia a intenção do usuário, nunca decidindo ou mantendo lógica
própria. Detalhes e justificativa em
[`docs/architecture.md#stack-and-platform`](docs/architecture.md#stack-and-platform).

### Stack técnica

- **Tauri v2** — shell desktop e ponte de IPC (`Commands`/`Events`) entre as duas camadas.
- **Rust** — o núcleo: motor de áudio (`symphonia`, `rtrb`, `cpal`), estado, persistência e o parser de `.cho` (ChordPro estendido).
- **Angular** — apenas apresentação, Signals nativos para estado, sem pacotes de terceiros.

### Ambiente de desenvolvimento

O repositório fornece um container de desenvolvimento Arch Linux
reproduzível (Distrobox/Podman) com Rust, Node, Python, Claude Code e as
dependências de build do Tauri já instaladas. Com Distrobox e Podman no
host:

```bash
distrobox assemble create --file distrobox/distrobox.ini
distrobox enter stemloft
```

Detalhes e o raciocínio por trás da imagem em
[`distrobox/README.md`](distrobox/README.md). Uma vez dentro do container,
a partir da raiz do repositório:

```bash
cargo tauri dev
```

(a CLI do Tauri é instalada como um binário `cargo`, não como um pacote
npm — veja
[`docs/architecture.md#stack-and-platform`](docs/architecture.md#stack-and-platform)
para entender por que o lado Angular evita dependências npm além do que o
próprio Angular traz). Isso inicia o servidor de dev do Angular e abre a
janela do app — hoje, as telas de estado vazio que o esqueleto já traz,
já que nenhuma lógica de domínio do núcleo Rust está implementada ainda.

### Documentação

- [`docs/architecture.md`](docs/architecture.md) — arquitetura completa: componentes, contrato de IPC, o formato `.cho`, fluxos de dados.
- [`docs/project-name.md`](docs/project-name.md) — por que o projeto se chama StemLoft.
- [`TODO.md`](TODO.md) — decisões pendentes e o que falta antes da implementação.
- [`distrobox/README.md`](distrobox/README.md) — ambiente de desenvolvimento Arch Linux reproduzível (Distrobox/Podman): o que tem na imagem (`Containerfile`) e como o container é criado a partir dela (`distrobox.ini`).

### Contribuindo

Contribuições são bem-vindas. Comece por [`TODO.md`](TODO.md) para o que
está em aberto no momento, e [`docs/architecture.md`](docs/architecture.md)
para as decisões já definidas — mudanças devem seguir esse design, ou
atualizar o documento junto com o código caso o alterem deliberadamente. O
projeto é GPL-3.0-or-later (veja [Licença](#licença) abaixo), então tudo
que for contribuído permanece aberto.

Este repositório também fornece agentes, skills e hooks do Claude Code
(`.claude/`) que codificam essas regras para que um assistente de IA
trabalhando no código as siga automaticamente — por exemplo, um hook
`PostToolUse` incentiva o agente a manter este README sincronizado sempre
que `docs/architecture.md`, `TODO.md`, ou a documentação do ambiente de
dev mudam.

## Licença

Copyright (C) 2026 Leonardo Farias de Souza Silva

Distribuído sob a [GNU General Public License v3.0 ou posterior](./LICENSE)
— a mesma família de licenças usada por Inkscape, GIMP, Blender e Audacity:
qualquer pessoa pode usar, estudar, modificar e redistribuir o código (uso
comercial incluído), mas todo trabalho derivado distribuído deve permanecer
sob a mesma licença, com o código-fonte disponível — ninguém pode pegar
este projeto, fechá-lo e vender uma versão proprietária.
