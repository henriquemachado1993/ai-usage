# AI Usage — instruções para agentes

Widget desktop (Tauri 2 + React + TypeScript) que mostra o consumo de limites de Claude, Codex e GitHub Copilot.

## Leia antes de trabalhar
- [docs/PRD.md](docs/PRD.md) — escopo, requisitos e o que está fora da v1.
- [docs/MEMORY.md](docs/MEMORY.md) — decisões, endpoints, caminhos de credenciais e armadilhas. Atualize-o ao tomar uma decisão nova ou descobrir algo não óbvio.

## Estrutura
- `src-tauri/src/providers/` — um módulo por provedor (`claude.rs`, `codex.rs`, `copilot.rs`), com testes de parsing.
- `src-tauri/src/poller.rs` — detecção de contas e atualização periódica; emite `usage-updated` e `refreshing`.
- `src-tauri/src/commands.rs` — commands chamados pelo frontend.
- `src-tauri/src/dock.rs` — snap na borda/canto e recolher/expandir a faixa lateral.
- `src/components/` — UI do widget; `src/lib/api.ts` encapsula `invoke`/`listen`.
- `src/dev/mockTauri.ts` — IPC simulado para ver a UI no navegador com `npm run dev`.

## Comandos
- `npm run tauri dev` — app em modo desenvolvimento.
- `cd src-tauri && cargo test` — testes do backend.
- `npx tsc --noEmit` — checagem de tipos do frontend.
- `npm run tauri build` — instaladores da plataforma atual.

## Convenções
- Evitar comentários no código; só quando realmente necessário.
- Textos da UI em português.
- Ícones de UI só com Tabler Icons (`@tabler/icons-react`).
- Tokens manuais só no keyring. Única escrita permitida em arquivo de outra ferramenta: a renovação do token do Claude no próprio `.credentials.json` (ver `docs/MEMORY.md`).
- Novo provedor: criar módulo em `providers/`, registrar em `providers/mod.rs`, adicionar ao `ProviderKind` (Rust e `src/lib/types.ts`) e documentar endpoint em `docs/MEMORY.md`.
