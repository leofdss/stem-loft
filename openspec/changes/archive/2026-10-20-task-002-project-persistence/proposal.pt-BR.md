# TASK-002 — Project Persistence: carregar/salvar, escritas atômicas, schemaVersion

**Área:** Núcleo Rust — Project Persistence

## Why

Implementar `persistence.rs`: ler/escrever o `.json` do projeto conforme o schema (`docs/architecture.md#project-versioning`), escrita em arquivo temporário + rename sem exceções, e os três casos de `schemaVersion` (mesma versão carrega direto, versão menor roda migrações, versão maior gera erro explícito).

## What Changes

- **Escritas atômicas do projeto** — O núcleo MUST escrever o arquivo do projeto gravando um arquivo temporário no mesmo diretório e renomeando-o por cima do caminho final, e MUST NOT escrever diretamente no caminho final.
- **Escritas simultâneas não colidem** — Dois salvamentos do mesmo arquivo ao mesmo tempo SHALL terminar cada um por inteiro, deixando o arquivo com o conteúdo de um deles.
- **Checkpoints com debounce** — Uma mudança de estado SHALL marcar o projeto como sujo, e o núcleo SHALL gravá-lo num checkpoint com debounce, e não uma vez por Command; um evento definitivo, como pausar ou fechar, SHALL gravar na hora um projeto sujo.
- **Mudanças feitas durante uma escrita são mantidas** — Uma mudança que chega enquanto um checkpoint está gravando SHALL manter o projeto sujo, para que o próximo checkpoint a salve.
- **Tratamento de versão do schema** — Ao carregar, o núcleo SHALL comparar o schemaVersion do arquivo com o atual: carregar direto uma versão igual, migrar uma menor e rejeitar uma maior com um erro explícito.
- **Projetos ilegíveis falham de forma explícita** — O carregamento MUST terminar num erro explícito, nunca num projeto parcial nem numa queda, quando o arquivo não é JSON válido, não tem schemaVersion válido, ou uma migração falha ou produz algo que não é um projeto.

## Capabilities

### New Capabilities

- `project-persistence`: Carrega e salva o estado de um projeto em disco sem nunca deixar para trás um arquivo de projeto corrompido ou escrito pela metade.

### Modified Capabilities

Nenhuma.

## Impact

Núcleo Rust — Project Persistence. Status, branch, dependências e playbooks desta change ficam em `openspec/board.toml`.
