# Nome do projeto

## Por que trocar "Stem Player"

"Stem Player" descreve só uma fração do que o app faz — reproduzir stems.
Não descreve o que é, na prática, a funcionalidade principal (loop de
repetição por marcadores temporais), nem o mixer por stem (volume/mute/solo),
nem a visualização de acordes/tablatura sincronizada com a timeline (ver
[Stack e plataforma](doc.md#stack-e-plataforma) e
[Metadados de partitura](doc.md#metadados-de-partitura-acordes-tablatura-e-letra)),
nem a separação automática de stems planejada para o futuro
([Estado atual e trabalho futuro](doc.md#estado-atual-e-trabalho-futuro)). É
uma ferramenta de estudo para músicos — o nome deveria refletir isso, não só
o formato de arquivo que ela consome.

## Critério de busca

Toda pesquisa abaixo foi checagem informal por busca na web (nome + "app"/
"software"/"github"/"trademark") — **não é parecer jurídico**. Não há acesso
direto a bases como USPTO TESS ou WIPO Global Brand Database por essa via; o
que se buscou foi conflito prático: outro produto de software na mesma
categoria (apps de prática musical, loop, acordes/tablatura) usando o mesmo
nome ou um nome foneticamente muito próximo.

Constatação central da pesquisa: esse nicho (prática musical, loop,
acordes/tablatura) está extremamente saturado de nomes descritivos óbvios —
praticamente todo termo do vocabulário musical comum já é usado por algum
produto real. Nomes mais abstratos/cunhados (no estilo Blender, Inkscape,
Audacity — que não descrevem literalmente a função da ferramenta) tiveram
bem menos conflito.

## Nomes descartados

| Nome | Conflito encontrado |
|---|---|
| **Woodshed** | Concorrente direto real: **WoodShed Music** (woodshedmusic.studio) — mesma proposta (importar música, loop de trecho difícil, desacelerar, isolar/mutar instrumento, tocar junto). Também existem **Woodshedder** e **Woodshedding**, outros dois apps de prática musical com o mesmo radical. |
| **Vamp** | Marca já estabelecida há quase 20 anos na comunidade de áudio livre: formato/API de plugins **Vamp** (Queen Mary University of London), integrado ao Sonic Visualiser e ao próprio **Audacity** — a mesma comunidade que este projeto mira. |
| **LoopShed** | Sem produto idêntico encontrado, mas herda o sufixo "Shed" da família já lotada (Woodshed/Woodshedder/Woodshedding). |
| **PracticeDeck** | Já existe um app com esse nome (treino de tiro esportivo) — categoria diferente, mas nome genérico já ocupado. |
| **Rondo** | Existe um "Rondo Songbook App" — categoria próxima o suficiente (letras/cifras) pra gerar confusão. |
| **Fretwork** | App de guitarra real chamado exatamente "Fretwork" (getfretwork.com) + um conjunto musical famoso com o mesmo nome. |
| **Reprise** | **Reprise, Inc.** tem marcas registradas ativas no USPTO (REPRISE, REPRISE REPLAY, REPRISE REPLICATE, REPRISE REVEAL), além do conhecido "Reprise License Manager" — maior risco jurídico formal encontrado em toda a pesquisa. |
| **Capstan** | A Celemony (mesma empresa da Melodyne) vende um software de áudio chamado "Capstan" — concorrência direta de categoria. |
| **Luthier** | "Luthier Lab" já existe (apps de construção de instrumentos) — categoria musical adjacente, termo bastante reciclado nesse universo. |
| **Cifra** | **Cifra Club** é a marca dominante no Brasil para exatamente acordes/tablaturas — pior conflito de todos: mesma categoria, mesmo mercado (PT-BR). |
| **Cadenzo / Cadenza** | Vários softwares reais chamados Cadenza (app de acompanhamento musical) e um "Cadenzo" (cadenzo.de, software de música alemão). |
| **Cifrado** | Sem concorrente de música encontrado, mas a palavra colide semanticamente com "encriptação" (é o termo usado por softwares de criptografia) — prejudica a capacidade de busca do nome, mesmo sem risco legal. |
| **Da Capo** | Próximo demais de **Capo** (supermegaultragroovy.com) — app real, consolidado (macOS/iOS), com posicionamento quase idêntico ao deste projeto: desacelera, isola instrumento, detecta acordes, faz loop por compasso. |
| **Ostinato** | Já é o nome de um projeto open source estabelecido (gerador/analisador de tráfego de rede, 14+ anos), fora do domínio musical mas dentro do universo de ferramentas open source — risco de confundir a própria comunidade de desenvolvedores. |
| **Trastan** | Nenhum software/app conflitante encontrado — vem de "traste" (fret, em PT). Descartado só por soar foneticamente parecido com "Tristan"/"Trojan", não por conflito real. |

Achado colateral da pesquisa, não é sobre nome: existem pelo menos dois
concorrentes diretos deste projeto — **[Riffloop](https://riffloop.app/)**
(separa em stems, muta parte, loop A-B, toca junto) e
**[Capo](https://supermegaultragroovy.com/products/capo/)** (desacelera,
isola instrumento, detecta acordes, loop por compasso). Não bloqueia nada —
o recorte deste projeto (stems locais + `.cho` sincronizado, núcleo Rust,
sem dependência de nuvem) é diferente — mas mostra que o nicho tem
concorrência de peso.

## Decisão: StemLoft

Nas buscas realizadas, "StemLoft" não apareceu associado a nenhum
software/app existente. O próprio usuário confirmou, de forma independente,
não encontrar resultados no [GitHub](https://github.com/search) nem no
[USPTO TESS](https://www.uspto.gov/trademarks/search).

Por que esse nome, além de estar livre:

- Mantém **"Stem"** — não descarta a conexão com o que o app tecnicamente
  manipula (stems de áudio) nem exige uma reformulação total de identidade.
- **"Loft"** evoca um espaço de trabalho criativo (estúdio/ateliê) em vez de
  descrever literalmente uma função — o mesmo tipo de nome "abstrato" que
  Blender, Inkscape e Audacity usam, ao contrário dos nomes puramente
  descritivos (loop, shed, deck, cifra) que colidiram em praticamente toda
  tentativa.
- Pronúncia e escrita simples em qualquer idioma, sem tradução estranha para
  o português.

## Ressalva

Esta pesquisa (a feita aqui e a feita pelo usuário) reduz o risco de
conflito óbvio, mas não substitui uma checagem formal antes de qualquer
passo com consequência prática — publicar em loja de app, registrar domínio
ou marca, ou uso comercial de terceiros. Nenhuma das duas é advogado.

## Renomeação aplicada

"Stem Player" foi trocado por "StemLoft" em todo o texto do repositório:
`README.md` (título), `docs/doc.md`, `docs/Arquitetura.canvas`, agents e
skills em `.claude/`, e a imagem de desenvolvimento do Distrobox
(`distrobox/Containerfile`, `distrobox/README.md`,
`.github/workflows/build-dev-image.yml` — `stem-player-dev` virou
`stemloft-dev`).

Uma pendência ficou de fora de propósito, por afetar o ambiente de trabalho
em uso (IDE aberto, sessão de terminal ativa) em vez de só conteúdo
versionado: renomear a pasta/repositório local (`stem-player` →
`stemloft` ou similar). Isso precisa ser feito manualmente, fora de uma
sessão com arquivos abertos nela.
