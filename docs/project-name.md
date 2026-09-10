# Project name

## Why we moved away from "Stem Player"

"Stem Player" only describes a fraction of what the app does — playing back stems. It doesn't describe what the app actually is in practice: the main feature (loop repeats via time markers), the per-stem mixer (volume/mute/solo), the chord/tablature view synced with the timeline (see
[Stack and platform](architecture.md#stack-and-platform) and
[Score metadata](architecture.md#score-metadata-chords-tablature-and-lyrics)),
or the automatic stem separation planned for the future
([Current status and future work](architecture.md#current-status-and-future-work)). It's a practice tool for musicians — the name should reflect that, not just the file format it consumes.

## Search criteria

Every search below was informal web-search due diligence (name + "app"/
"software"/"github"/"trademark") — **this is not legal advice**. There's no
direct access to databases like USPTO TESS or the WIPO Global Brand Database
through this method; what was checked for was practical conflict: another
software product in the same category (music practice apps, looping,
chords/tablature) using the same name or a phonetically very close one.

Central finding of the research: this niche (music practice, looping,
chords/tablature) is extremely saturated with obvious descriptive names —
practically every common musical-vocabulary term is already used by some
real product. More abstract/coined names (in the style of Blender, Inkscape,
Audacity — which don't literally describe the tool's function) had far less
conflict.

## Names ruled out

| Name | Conflict found |
|---|---|
| **Woodshed** | Real direct competitor: **WoodShed Music** (woodshedmusic.studio) — same pitch (import a song, loop the hard part, slow it down, isolate/mute an instrument, play along). There's also **Woodshedder** and **Woodshedding**, two other music-practice apps with the same root. |
| **Vamp** | A brand already established for nearly 20 years in the free-audio-software community: the **Vamp** plugin format/API (Queen Mary University of London), integrated into Sonic Visualiser and into Audacity itself — the same community this project targets. |
| **LoopShed** | No identical product found, but it inherits the "Shed" suffix from the already-crowded family (Woodshed/Woodshedder/Woodshedding). |
| **PracticeDeck** | An app with this exact name already exists (sport-shooting practice) — different category, but the generic name is already taken. |
| **Rondo** | There's a "Rondo Songbook App" — close enough a category (lyrics/chord sheets) to cause confusion. |
| **Fretwork** | A real guitar app named exactly "Fretwork" (getfretwork.com), plus a famous music ensemble with the same name. |
| **Reprise** | **Reprise, Inc.** holds active USPTO trademark registrations (REPRISE, REPRISE REPLAY, REPRISE REPLICATE, REPRISE REVEAL), on top of the well-known "Reprise License Manager" — the strongest formal legal risk found in the whole search. |
| **Capstan** | Celemony (the same company behind Melodyne) sells an audio software product called "Capstan" — direct competition within the same category. |
| **Luthier** | "Luthier Lab" already exists (instrument-building apps) — an adjacent musical category, and a heavily recycled term in this space. |
| **Cifra** | **Cifra Club** is the dominant Brazilian brand for exactly chords/tablature — the worst conflict of all: same category, same market (Brazilian Portuguese). |
| **Cadenzo / Cadenza** | Several real software products named Cadenza (a music-accompaniment app) and a "Cadenzo" (cadenzo.de, German music software). |
| **Cifrado** | No music-space competitor found, but the word collides semantically with "encryption" (it's the term used by encryption software) — hurts the name's searchability, even with no legal risk. |
| **Da Capo** | Too close to **Capo** (supermegaultragroovy.com) — a real, established app (macOS/iOS) with a positioning nearly identical to this project's: slows audio down, isolates instruments, detects chords, loops by measure. |
| **Ostinato** | Already the name of an established open-source project (a network traffic generator/analyzer, 14+ years old), outside the music domain but within the open-source tooling world — a risk of confusing this project's own developer community. |
| **Trastan** | No conflicting software/app found — it comes from "traste" (fret, in Portuguese). Ruled out only for sounding phonetically close to "Tristan"/"Trojan," not for a real conflict. |

Side finding from the research, not about naming: there are at least two
direct competitors to this project — **[Riffloop](https://riffloop.app/)**
(splits into stems, mutes a part, A-B loop, play along) and
**[Capo](https://supermegaultragroovy.com/products/capo/)** (slows down,
isolates an instrument, detects chords, loops by measure). This doesn't
block anything — this project's scope (local stems + a synced `.cho` file,
Rust core, no cloud dependency) is different — but it shows the niche has
serious competition.

## Decision: StemLoft

Across the searches performed, "StemLoft" didn't come up associated with
any existing software/app. The user independently confirmed finding no
results on [GitHub](https://github.com/search) or on
[USPTO TESS](https://www.uspto.gov/trademarks/search) either.

Why this name, beyond being free:

- It keeps **"Stem"** — it doesn't drop the connection to what the app
  technically works with (audio stems), and it doesn't require a full
  identity overhaul.
- **"Loft"** evokes a creative workspace (a studio/atelier) instead of
  literally describing a function — the same kind of "abstract" naming that
  Blender, Inkscape, and Audacity use, unlike the purely descriptive names
  (loop, shed, deck, cifra) that collided in nearly every attempt.
- Simple pronunciation and spelling in any language, with no awkward
  translation into Portuguese.

## Caveat

This research (both what was done here and what the user did independently)
reduces the risk of an obvious conflict, but it doesn't replace a formal
check before any step with real-world consequences — publishing to an app
store, registering a domain or trademark, or third-party commercial use.
Neither of us is a lawyer.

## Rename applied

"Stem Player" was replaced with "StemLoft" throughout the repository's text:
`README.md` (title), `docs/architecture.md`, `docs/Architecture.canvas`,
the agents and skills under `.claude/`, and the Distrobox development image
(`distrobox/Containerfile`, `distrobox/README.md`,
`.github/workflows/build-dev-image.yml` — `stem-player-dev` became
`stemloft-dev`).

One item was left out on purpose, since it affects the active working
environment (open editor, active terminal session) rather than just
versioned content: renaming the local repository folder (`stem-player` →
`stemloft` or similar). That needs to be done manually, outside a session
with files open in it.

## Full project translation to American English

Following the same widen-the-contributor-pool goal, the entire project —
this document included — was translated from Brazilian Portuguese to
American English (see the commit that introduced this line for the full
file list). File names that were themselves Portuguese words were also
renamed: `docs/doc.md` → `docs/architecture.md`, this file
(`docs/nome-do-projeto.md` → `docs/project-name.md`), and
`docs/Arquitetura.canvas` → `docs/Architecture.canvas`.
