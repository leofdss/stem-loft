# Nome do projeto

> **Nota de manutenção:** este documento tem uma versão em inglês em
> [`project-name.md`](./project-name.md). Sempre que um dos dois for
> atualizado, atualize o outro na mesma alteração — não deixe as duas
> versões divergirem.

## Por que saímos de "Stem Player"

"Stem Player" descreve só uma fração do que o app faz — reproduzir stems.
Não descreve o que o app é de fato na prática: a funcionalidade principal
(repetição de trechos via marcadores de tempo), o mixer por stem
(volume/mudo/solo), a visualização de cifra/tablatura sincronizada com a
linha do tempo (ver
[Stack e plataforma](architecture.md#stack-and-platform) e
[Metadados de partitura](architecture.md#score-metadata-chords-tablature-and-lyrics)),
ou a separação automática de stems planejada para o futuro
([Status atual e trabalho futuro](architecture.md#current-status-and-future-work)).
É uma ferramenta de prática para músicos — o nome deveria refletir isso, não
apenas o formato de arquivo que consome.

## Critérios de busca

Cada busca abaixo foi uma verificação informal por busca na web (nome +
"app"/"software"/"github"/"trademark") — **isto não é aconselhamento
jurídico**. Não há acesso direto a bases como USPTO TESS ou a WIPO Global
Brand Database por esse método; o que foi verificado foi conflito prático:
outro produto de software na mesma categoria (apps de prática musical,
loop, cifras/tablatura) usando o mesmo nome ou um foneticamente muito
próximo.

Achado central da pesquisa: esse nicho (prática musical, loop,
cifras/tablatura) está extremamente saturado de nomes descritivos óbvios —
praticamente todo termo comum do vocabulário musical já é usado por algum
produto real. Nomes mais abstratos/inventados (no estilo de Blender,
Inkscape, Audacity — que não descrevem literalmente a função da
ferramenta) tiveram bem menos conflito.

## Nomes descartados

| Nome | Conflito encontrado |
|---|---|
| **Woodshed** | Concorrente direto real: **WoodShed Music** (woodshedmusic.studio) — mesma proposta (importar uma música, repetir a parte difícil, desacelerar, isolar/mutar um instrumento, tocar junto). Há também **Woodshedder** e **Woodshedding**, outros dois apps de prática musical com a mesma raiz. |
| **Vamp** | Uma marca já estabelecida há quase 20 anos na comunidade de software de áudio livre: o formato/API de plugin **Vamp** (Queen Mary University of London), integrado ao Sonic Visualiser e ao próprio Audacity — a mesma comunidade que este projeto tem como alvo. |
| **LoopShed** | Nenhum produto idêntico encontrado, mas herda o sufixo "Shed" da família já saturada (Woodshed/Woodshedder/Woodshedding). |
| **PracticeDeck** | Já existe um app com esse nome exato (prática de tiro esportivo) — categoria diferente, mas o nome genérico já está ocupado. |
| **Rondo** | Existe um "Rondo Songbook App" — categoria próxima o suficiente (letras/cifras) para causar confusão. |
| **Fretwork** | Um app de guitarra real chamado exatamente "Fretwork" (getfretwork.com), além de um conjunto musical famoso com o mesmo nome. |
| **Reprise** | A **Reprise, Inc.** detém registros de marca ativos no USPTO (REPRISE, REPRISE REPLAY, REPRISE REPLICATE, REPRISE REVEAL), além do já conhecido "Reprise License Manager" — o risco jurídico formal mais forte encontrado em toda a busca. |
| **Capstan** | A Celemony (a mesma empresa por trás do Melodyne) vende um produto de software de áudio chamado "Capstan" — concorrência direta na mesma categoria. |
| **Luthier** | Já existe "Luthier Lab" (apps de construção de instrumentos) — categoria musical adjacente, e um termo muito reciclado nesse espaço. |
| **Cifra** | **Cifra Club** é a marca brasileira dominante para exatamente cifras/tablatura — o pior conflito de todos: mesma categoria, mesmo mercado (português brasileiro). |
| **Cadenzo / Cadenza** | Vários produtos de software reais chamados Cadenza (um app de acompanhamento musical) e um "Cadenzo" (cadenzo.de, software musical alemão). |
| **Cifrado** | Nenhum concorrente no espaço musical encontrado, mas a palavra colide semanticamente com "criptografia" (é o termo usado por software de criptografia) — prejudica a capacidade de busca do nome, mesmo sem risco jurídico. |
| **Da Capo** | Muito próximo de **Capo** (supermegaultragroovy.com) — um app real e estabelecido (macOS/iOS) com posicionamento quase idêntico ao deste projeto: desacelera áudio, isola instrumentos, detecta cifras, repete por compasso. |
| **Ostinato** | Já é o nome de um projeto open-source estabelecido (um gerador/analisador de tráfego de rede, com mais de 14 anos), fora do domínio musical mas dentro do mundo de ferramentas open-source — um risco de confundir a própria comunidade de desenvolvedores deste projeto. |
| **Trastan** | Nenhum software/app conflitante encontrado — vem de "traste" (fret, em inglês). Descartado apenas por soar foneticamente próximo de "Tristan"/"Trojan", não por um conflito real. |

Achado paralelo da pesquisa, não relacionado a nomeação: há pelo menos dois
concorrentes diretos deste projeto — **[Riffloop](https://riffloop.app/)**
(separa em stems, muta uma parte, loop A-B, tocar junto) e
**[Capo](https://supermegaultragroovy.com/products/capo/)** (desacelera,
isola um instrumento, detecta cifras, repete por compasso). Isso não
bloqueia nada — o escopo deste projeto (stems locais + um arquivo `.cho`
sincronizado, núcleo em Rust, sem dependência de nuvem) é diferente — mas
mostra que o nicho tem concorrência séria.

## Decisão: StemLoft

Nas buscas realizadas, "StemLoft" não apareceu associado a nenhum
software/app existente. O usuário confirmou de forma independente não ter
encontrado resultados no [GitHub](https://github.com/search) nem no
[USPTO TESS](https://www.uspto.gov/trademarks/search).

Por que esse nome, além de estar livre:

- Mantém **"Stem"** — não abandona a conexão com o que o app tecnicamente
  trabalha (stems de áudio), e não exige uma reformulação completa de
  identidade.
- **"Loft"** evoca um espaço de trabalho criativo (um estúdio/ateliê) em
  vez de descrever literalmente uma função — o mesmo tipo de nomeação
  "abstrata" que Blender, Inkscape e Audacity usam, diferente dos nomes
  puramente descritivos (loop, shed, deck, cifra) que colidiram em quase
  toda tentativa.
- Pronúncia e escrita simples em qualquer idioma, sem tradução estranha
  para o português.

## Ressalva

Esta pesquisa (tanto o que foi feito aqui quanto o que o usuário fez de
forma independente) reduz o risco de um conflito óbvio, mas não substitui
uma verificação formal antes de qualquer passo com consequências reais —
publicar em uma loja de aplicativos, registrar um domínio ou marca, ou uso
comercial por terceiros. Nenhum de nós é advogado.

## Renomeação aplicada

"Stem Player" foi substituído por "StemLoft" em todo o texto do
repositório: `README.md` (título), `docs/architecture.md`,
`docs/Architecture.canvas`, os agentes e skills sob `.claude/`, e a imagem
de desenvolvimento do Distrobox (`distrobox/Containerfile`,
`distrobox/README.md`, `.github/workflows/build-dev-image.yml` —
`stem-player-dev` virou `stemloft-dev`).

Um item foi deixado de fora de propósito, por afetar o ambiente de trabalho
ativo (editor aberto, sessão de terminal ativa) em vez de apenas o
conteúdo versionado: renomear a pasta local do repositório (`stem-player`
→ `stemloft` ou similar). Isso precisa ser feito manualmente, fora de uma
sessão com arquivos abertos nela.

## Tradução completa do projeto para o inglês americano

Seguindo o mesmo objetivo de ampliar o alcance de contribuidores, o projeto
inteiro — este documento incluído — foi traduzido do português brasileiro
para o inglês americano (ver o commit que introduziu esta linha para a
lista completa de arquivos). Nomes de arquivo que eram eles mesmos palavras
em português também foram renomeados: `docs/doc.md` → `docs/architecture.md`,
este arquivo (`docs/nome-do-projeto.md` → `docs/project-name.md`), e
`docs/Arquitetura.canvas` → `docs/Architecture.canvas`.
