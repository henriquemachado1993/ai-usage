# AI Usage

Widget flutuante para Windows e macOS que mostra quanto dos limites de **Claude**, **Codex** e **GitHub Copilot** você já usou.

- Detecta automaticamente as contas dos CLIs logados (Claude Code, Codex, `gh`).
- Permite adicionar contas extras (outro diretório de config ou token do GitHub).
- Barra de progresso por conta; passe o mouse para ver todas as janelas e horários de reset.
- Sempre no topo, arrastável, redimensionável, com ícone na bandeja.

## Requisitos
Node 20+, Rust stable. No Windows, WebView2 (já incluso no Windows 11). No macOS, Xcode Command Line Tools.

## Desenvolvimento
```bash
npm install
npm run tauri dev
```

## Build
```bash
npm run tauri build
```

Documentação: [docs/PRD.md](docs/PRD.md) e [docs/MEMORY.md](docs/MEMORY.md).
