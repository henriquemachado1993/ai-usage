# PRD — AI Usage

## Problema
Quem usa vários assistentes de IA (Claude Code, Codex, GitHub Copilot) precisa abrir cada ferramenta ou site para saber quanto do limite já consumiu e quando ele reseta. Isso interrompe o fluxo e faz o limite acabar de surpresa.

## Objetivo
Um widget desktop leve, sempre visível, que mostra em um relance o consumo de cada conta ativa e, sob demanda (hover), o detalhamento completo.

## Público
Desenvolvedores que usam um ou mais assistentes de IA com planos baseados em janelas de uso (sessão de 5h, semanal, mensal).

## Plataformas
Windows 10/11 e macOS 12+. Linux é desejável, mas não prioritário.

## Escopo da v1

### Provedores
| Provedor | Origem da credencial | Janelas exibidas |
|---|---|---|
| Claude (Claude Code) | `~/.claude/.credentials.json` ou Keychain do macOS | Sessão (5h), Semanal, Semanal Opus/Sonnet, Uso extra |
| Codex (ChatGPT) | `~/.codex/auth.json` (+ fallback nos logs de sessão) | Sessão (5h), Semanal |
| GitHub Copilot | `gh auth token` ou token colado | Premium requests (mensal); Chat/Completions no plano Free |

### Contas
- Auto-detecção da conta padrão de cada CLI já logado na máquina.
- Adição manual de contas extras:
  - Claude/Codex: apontando outro diretório de configuração (`CLAUDE_CONFIG_DIR` / `CODEX_HOME`).
  - Copilot: token do GitHub, salvo no cofre de credenciais do sistema.
- Renomear e remover contas; contas auto-detectadas removidas não voltam até "Redetectar".

### Widget
- Janela sem borda, transparente, sempre no topo (configurável), fora da barra de tarefas.
- Arrastável pelo cabeçalho; redimensionável pelas bordas; posição e tamanho persistidos.
- Gruda na borda/canto da tela ao ser solto perto dela e pode recolher para uma faixa vertical na borda (logo, status geral, ícone + % de cada conta e seta para expandir).
- Cada conta: ícone oficial do provedor, avatar com a inicial da conta, janela mais crítica (5h, 7d, mês), "reseta em X", % e barra de progresso (verde < 60%, âmbar < 85%, vermelho ≥ 85%).
- Linha de status geral ("Tudo em dia", "1 conta precisa de atenção"…) e rodapé com última atualização, versão e Sair.
- Hover expande a linha com todas as janelas, uso/limite, data e hora do reset, identidade, origem da conta, última atualização, mensagens de erro e link para a página oficial de uso.
- Estados: carregando, ok, offline (dados locais), login expirado, erro.

### Sistema
- Ícone na bandeja/menu bar: mostrar/ocultar, atualizar agora, configurações, sair. Tooltip com o resumo.
- Atualização automática (1–60 min, padrão 5) e manual.
- Opacidade ajustável, iniciar com o sistema, instância única.

## Fora do escopo (v1)
- Cursor, Gemini, Windsurf e outros provedores.
- Histórico/gráficos de consumo e notificações de limite.
- Login OAuth próprio dentro do app (depende das credenciais dos CLIs).

## Requisitos não funcionais
- Codex: nunca renovar o token (evita deslogar o CLI). Claude: renovar e gravar de volta no mesmo `.credentials.json`, mantendo o CLI logado.
- Tokens adicionados manualmente só no cofre do sistema (Credential Manager / Keychain), nunca em arquivo.
- Falha em um provedor não afeta os outros.
- Binário pequeno e baixo consumo de memória (Tauri).

## Riscos
- Os endpoints de uso são internos/não documentados e podem mudar sem aviso. Mitigação: cada provedor isolado em um módulo, com testes de parsing por fixture.
- No macOS, hover em janela sem foco pode não disparar em todas as versões do WebKit.

## Próximos passos (v2)
- Notificação ao atingir 80%/95% de uma janela.
- Novos provedores (Cursor, Gemini).
- Modo compacto (só barras) e histórico simples.
