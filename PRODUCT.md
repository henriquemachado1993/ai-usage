# AI Usage

<!-- impeccable:product-schema 1 -->

## Platform

web

Interface React em uma janela desktop Tauri 2, para Windows e macOS.

## Users

Desenvolvedores que consultam limites de Claude, Codex e GitHub Copilot enquanto trabalham. Fonte: docs/PRD.md.

## Product Purpose

Mostrar consumo e tempo para reset em um relance, com detalhes sob demanda, sem interromper o trabalho.

## Capabilities and Constraints

Contas múltiplas, atualização periódica, janela sempre no topo, encaixe nas bordas, faixa recolhida, detalhes por janela de uso e estados de erro independentes por provedor. Ícones de interface Tabler e textos em português. Sem histórico ou notificações na v1. Fontes: docs/PRD.md, docs/MEMORY.md e AGENTS.md.

## Evidence on Hand

Implementação em src/, PRD e memória em docs/. Referência visual enviada pelo usuário: faixa vertical escura com ícones e percentuais.

## Open Decisions

Em 2026-10-28, o usuário aprovou a faixa vertical compacta mais estreita: ícone e percentual por conta, com detalhes ao abrir. A faixa de 44 unidades lógicas na escala padrão e o seletor de tamanho de 70%, 85%, 100% e 115% foram implementados no widget. O tema escuro, o verde de acento, as cores semânticas e a tipografia existentes continuam como referência visual.

As alternativas de lista e barra permanecem propostas não selecionadas. Os protótipos interativos em `/propostas.html` continuam disponíveis para comparação; não representam aprovação dessas alternativas.
