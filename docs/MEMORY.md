# MEMORY — decisões e aprendizados

Registro vivo do projeto. Agentes e pessoas devem consultar antes de mudar algo relevante e acrescentar entradas novas ao final de cada seção (com data).

## Decisões
- **2026-10-26** — Stack: Tauri 2 + React 19 + TypeScript + Vite. Motivo: binário pequeno e backend Rust para ler arquivos, Keychain e fazer HTTP sem CORS.
- **2026-10-26** — Toda coleta de dados acontece no backend (`src-tauri/src/poller.rs`); o frontend só escuta o evento `usage-updated` e chama commands.
- **2026-10-26** — Codex: não renovamos o token (o refresh rotaciona e desloga o CLI); expirado vira `expired`/fallback local.
- **2026-10-26** — Claude: renovamos o token quando vence (ou em 401) e gravamos o novo par de forma atômica no mesmo `.credentials.json`, preservando os demais campos — igual ao CLI, que relê o arquivo. Motivo: quando o Claude Code roda dentro do app desktop, o login é do app (`CLAUDE_CODE_SDK_HAS_HOST_AUTH_REFRESH`) e o arquivo fica parado. Endpoint `POST https://platform.claude.com/v1/oauth/token` (JSON com `grant_type`, `refresh_token`, `client_id` `9d1c250a-e61b-44d9-88ed-5944d1962f5e`, `scope`), confirmado no binário do Claude Code 2.1.309. O refresh token rotaciona a cada renovação. Credenciais do Keychain (macOS) não são renovadas.
- **2026-10-26** — Contas auto-detectadas têm id fixo `<provider>-auto`; ao remover, o id vai para `settings.dismissed` para não reaparecer até "Redetectar".
- **2026-10-26** — Tokens manuais ficam no keyring (serviço `com.aiusage.app`, usuário = id da conta). `accounts.json` e `settings.json` ficam no `app_config_dir` e não guardam segredos.
- **2026-10-26** — Redimensionamento por handles próprios (`ResizeHandles.tsx` + `startResizeDragging`) porque a janela não tem decoração; arrasto por `data-tauri-drag-region` no cabeçalho.
- **2026-10-26** — UI no estilo da referência "tokenkarma": tema escuro único, acento verde, linha compacta (ícone, avatar da conta, provedor, janela curta, "reseta em", %), rodapé com última atualização/versão/Sair.
- **2026-10-26** — Modo dock: ao soltar a janela a até 28px de uma borda/canto da área de trabalho ela gruda (`dock.rs`, debounce de 350ms no evento `Moved`). Recolher troca a janela por uma faixa de 60px ancorada na borda; o tamanho expandido fica em `settings.expandedWidth/Height`. O tamanho mínimo é aplicado pelo backend (não no `tauri.conf.json`) para permitir a faixa estreita.
- **2026-10-26** — Ícones de UI: Tabler Icons (`@tabler/icons-react`), por preferência do usuário (não usar lucide). Ícones de marca (Claude, Codex, GitHub Copilot) vêm do pacote MIT `@lobehub/icons-static-svg`, embutidos como paths em `ProviderIcon.tsx`.
- **2026-10-26** — `src/dev/mockTauri.ts` simula o IPC quando a UI roda no navegador (`npm run dev`, `?edge=left|right&collapsed`); só carrega em DEV e não entra no build.
- **2026-10-26** — Copilot Business com `token_based_billing`: todas as cotas vêm `unlimited: true` e o consumo aparece em `credits_used`. Exibimos "Ilimitado · N usados".
- **2026-10-26** — O refresh token do Claude tem validade própria (`refreshTokenExpiresAt`); depois dela é preciso `claude` → `/login` no terminal.

## Endpoints (não oficiais)
- Claude: `GET https://api.anthropic.com/api/oauth/usage` com `anthropic-beta: oauth-2025-04-20`. Campos `five_hour`, `seven_day`, `seven_day_opus`, `seven_day_sonnet` (`utilization` 0–100, `resets_at` RFC 3339) e `extra_usage`.
- Codex: `GET https://chatgpt.com/backend-api/wham/usage` com `ChatGPT-Account-Id`. Campos `plan_type`, `rate_limit.primary_window|secondary_window` (`used_percent`, `limit_window_seconds`, `reset_at`/`reset_after_seconds`).
- Codex offline: último evento `token_count` com `rate_limits` em `~/.codex/sessions/**/rollout-*.jsonl`.
- Copilot: `GET https://api.github.com/copilot_internal/user` com `Authorization: token …`. Campos `quota_snapshots.*` (planos pagos) ou `limited_user_quotas` + `monthly_quotas` (Free).

## Credenciais por plataforma
- Claude no Windows/Linux: `<config>/.credentials.json` → `claudeAiOauth`. No macOS: Keychain, serviço `Claude Code-credentials` (com diretório custom, sufixo `-<8 hex do sha256 do caminho>` — confirmar se mudar).
- E-mail do Claude: `~/.claude.json` → `oauthAccount.emailAddress` (ou `<config>/.claude.json` em diretório custom).
- Codex: `<CODEX_HOME>/auth.json` → `tokens.access_token`, `tokens.account_id`, `tokens.id_token` (JWT com `email` e `chatgpt_plan_type`).
- Copilot: `gh auth token --hostname github.com`; no macOS o PATH de apps GUI é reduzido, por isso tentamos também `/opt/homebrew/bin/gh` e `/usr/local/bin/gh`. Fallback: `hosts.yml` do gh.

## Armadilhas conhecidas
- No Windows, processos filhos (`gh`) abrem console se não usar `CREATE_NO_WINDOW` (ver `providers::command`).
- **2026-10-26** — Neste Mac Apple Silicon, o primeiro build Rust encerrou vários build scripts com `SIGKILL`; repetir com `CARGO_BUILD_JOBS=2 npm run tauri build` concluiu o build e gerou o `.app` e o `.dmg` arm64 em `src-tauri/target/release/bundle/`. O Rust foi instalado temporariamente em `/private/tmp/ai-usage-cargo` e `/private/tmp/ai-usage-rustup` (usar `CARGO_HOME`, `RUSTUP_HOME` e adicionar o primeiro `/bin` ao `PATH` ao repetir nesta sessão).
- No macOS, transparência exige `macOSPrivateApi: true` e a feature `macos-private-api`; o app usa `ActivationPolicy::Accessory` para não aparecer no Dock.
- `window.prompt` não é confiável no WebView do macOS; use edição inline.
- Em React 19 o namespace global `JSX` não existe; use `ReactElement`.
- `app.state::<T>()` entra em pânico se o estado ainda não foi registrado; eventos de janela podem chegar antes do `setup`, então use `try_state` neles.
- O bundle NSIS falhou uma vez com "Access is denied" ao extrair o NSIS para o cache do Tauri; na tentativa seguinte funcionou (falha transitória, provavelmente antivírus). Se repetir, rode o build de novo.
- Instalador de macOS só pode ser gerado em um Mac (ou no CI `.github/workflows/build.yml`); não há cross-compile a partir do Windows. Sem assinatura/notarização, o Gatekeeper bloqueia na primeira abertura.

## Como validar
- `cd src-tauri && cargo test` — testes de parsing dos três provedores.
- `cd src-tauri && cargo test probe_default_accounts -- --ignored --nocapture` — consulta as APIs reais com as contas locais e imprime status/janelas (sem tokens).
- `npm run tauri dev` — conferir números contra `/usage` (Claude Code), `/status` (Codex) e a página do Copilot no GitHub.
