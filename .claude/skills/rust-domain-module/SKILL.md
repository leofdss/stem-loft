---
name: rust-domain-module
description: Como estruturar um módulo de domínio novo (ou uma struct nova dentro de um módulo existente) no núcleo Rust do StemLoft — construtor com injeção manual de dependências, sem DI container, sem generics/trait objects/macros desnecessários. Use ao criar um novo módulo Rust, uma nova struct de domínio, ou ao decidir "isso é idiomático demais?".
---

# Estruturar um módulo de domínio no núcleo Rust

O núcleo Rust do StemLoft é escrito por quem vem de TypeScript
(Angular/Nest.js), não de background Rust. A regra de ouro, documentada em
[`doc.md`](../../../docs/doc.md#nota-de-design-padrões-familiares-a-quem-vem-de-typescript):
**"dá pra entender vindo de Angular/Nest.js sem aprender Rust avançado
primeiro"** vale mais que "idiomático pros padrões da comunidade Rust". Isso
só cede se performance for **muito** afetada, medido — não como escolha
default.

## O paralelo mental (use-o para decidir, não só para explicar)

- **Módulo de domínio ≈ serviço injetável do Nest.js.** Uma `struct` com
  métodos públicos e um `new(...)` que recebe as dependências explicitamente
  — igual `@Injectable()` recebendo no construtor, só que sem container: a
  "injeção" é passar os `Arc<...>` à mão, uma vez, na inicialização do
  `AppState`.
- **`Gerenciador de Sessão / Estado` ≈ controller do Nest.js.** Recebe,
  despacha pro service (módulo de domínio), devolve — nunca acumula regra de
  negócio própria. Ver skill `add-ipc-contract` pra decidir onde um handler
  novo deve morar.

## Checklist ao escrever a struct

1. **Construtor explícito, sem DI framework.** `pub fn new(dep_a: Arc<DepA>, dep_b: Arc<DepB>) -> Self`.
   Nada de `#[derive(Inject)]`, service locator, ou registro dinâmico.
2. **Prefira explícito e repetitivo a "esperto".** Generics pesados, macros,
   trait objects em excesso, lifetimes elaborados: só se as duas opções
   resolverem o mesmo problema com a mesma clareza E a versão simples for
   comprovadamente lenta demais. Na dúvida, escreva a versão chata primeiro.
3. **Concorrência: o modelo mais simples de raciocinar, não o mais rápido em
   teoria.** Estado compartilhado do módulo vive atrás de um único
   `Arc<Mutex<...>>` (ou, no máximo, um `Mutex` por módulo — nunca um por
   feature). `lock() → muda o que precisa → solta o lock`. Só considere
   canais, actors ou granularidade maior de lock se profiling mostrar
   contenção real — não antes, não como otimização especulativa. Ver
   [nota de design completa](../../../docs/doc.md#nota-de-design-modelo-de-concorrência-do-estado-compartilhado).
4. **A thread de tempo real do `cpal` nunca toca esse `Mutex`.** Se o módulo
   que você está escrevendo tem qualquer relação com o callback de áudio,
   pare e carregue o skill `realtime-audio-safety` — a regra de
   concorrência acima não se aplica lá dentro.
5. **`Gerenciador de Sessão` não ganha lógica nova.** Se você está tentado a
   adicionar um método na Sessão que faz mais que "ler/atualizar qual
   projeto está ativo e despachar", esse método pertence ao módulo de
   domínio dono da informação.
6. **Nenhum módulo de domínio depende de detalhe de API do Tauri ou do
   Angular para decidir algo.** A ponte `Commands`/`Events` é tratada como
   substituível — se seu módulo importa algo de `tauri::` fora da camada de
   IPC, isso é vazamento de responsabilidade.

## Antipadrões a evitar (motivo documentado, não gosto pessoal)

| Antipadrão | Por quê evitar aqui |
|---|---|
| Trait object / `dyn Trait` para "flexibilidade futura" sem uso concreto hoje | Custo de leitura pra quem vem de TS sem ganho medido — YAGNI explícito no doc |
| Canal (`mpsc`, actor) para estado compartilhado comum | Já decidido: `Mutex` simples é o padrão até profiling provar contenção real |
| `Mutex` por feature/handler | Decidido: no máximo um por módulo, preferencialmente um só (`AppState`) |
| Lock (`Mutex`, `RwLock`) acessado dentro do callback `cpal` | Viola a fronteira de tempo real — ver skill `realtime-audio-safety` |
| Lógica de coordenação entre módulos dentro da Sessão | Vira *god object* — extraia pra um módulo novo dono dessa lógica |

## Ao terminar

Rode `cargo check` (o hook `PostToolUse` já faz isso automaticamente a cada
edição de `.rs` e devolve erro de compilação pra você corrigir sozinho — não
espere o usuário pedir). Se o hook não disparar (sessão antiga, watcher não
recarregado), rode manualmente antes de reportar a tarefa como concluída.
