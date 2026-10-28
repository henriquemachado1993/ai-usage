---
name: AI Usage
description: Widget desktop escuro para consultar limites de IA em um relance.
colors:
  background: "rgba(10, 10, 11, var(--alpha))"
  foreground: "#f4f4f5"
  muted: "#8b8b93"
  border: "rgba(255, 255, 255, 0.08)"
  hover: "rgba(255, 255, 255, 0.045)"
  track: "#26262b"
  input: "rgba(255, 255, 255, 0.05)"
  accent: "#22c55e"
  ok: "#22c55e"
  warn: "#f5a524"
  danger: "#ef4444"
typography:
  body:
    fontFamily: 'Inter, -apple-system, "Segoe UI", system-ui, sans-serif'
    fontSize: "13px"
  title:
    fontSize: "16px"
    fontWeight: 700
    letterSpacing: "-0.01em"
  compact-value:
    fontSize: "11px"
    fontWeight: 700
rounded:
  field: "7px"
  action: "8px"
  tool: "9px"
  row: "10px"
  widget: "14px"
  strip: "16px"
  badge: "999px"
spacing:
  compact: "4px"
  small: "6px"
  regular: "8px"
  panel: "10px"
components:
  tool-button:
    textColor: "{colors.muted}"
    rounded: "{rounded.tool}"
    width: "32px"
    height: "32px"
  field:
    backgroundColor: "{colors.input}"
    rounded: "{rounded.field}"
    padding: "6px 8px"
  badge:
    backgroundColor: "{colors.track}"
    textColor: "{colors.muted}"
    rounded: "{rounded.badge}"
    padding: "1px 7px"
  account-row:
    rounded: "{rounded.row}"
    padding: "9px 8px 10px"
  compact-account:
    typography: "{typography.compact-value}"
    rounded: "{rounded.action}"
    width: "40px"
    height: "47px"
---

# Design System: AI Usage

## Overview

Sistema visual extraído de `src/App.css` e dos componentes existentes em 2026-10-28. Janela escura, densa e discreta, com contraste claro para valores de uso e verde como acento. A faixa compacta aprovada preserva esse sistema visual.

**Key Characteristics:**

- Superfícies escuras com opacidade ajustável.
- Percentuais legíveis e detalhes sob demanda.
- Cores semânticas para atenção e limite.
- Ícones de interface Tabler e textos em português.

## Colors

### Primary

O verde de acento identifica ações, foco e estado saudável. A faixa compacta mostra o percentual saudável na cor de texto principal, reservando âmbar e vermelho para atenção.

### Neutral

O fundo quase preto acompanha a opacidade escolhida pelo usuário. Texto claro contém a informação principal; cinza contém metadados. Bordas e hover brancos translúcidos distinguem áreas sem competir com os valores.

As cores de aviso e perigo comunicam consumo elevado e falhas. As marcas dos provedores mantêm suas cores existentes; não substituem os estados semânticos.

## Typography

Usar a pilha existente, com fontes de plataforma como fallback, sem adicionar uma família visual nova. O título é curto e mais pesado; o corpo mantém a densidade do widget. Percentuais usam algarismos tabulares. A faixa compacta usa o papel `compact-value`.

## Layout

Widget expandido em coluna: cabeçalho, status, conteúdo rolável e rodapé. A faixa recolhida mantém arrasto no topo, uma conta por linha e expansão no rodapé. Cada conta combina ícone do provedor com percentual; clicar abre o widget nos detalhes daquela conta.

As medidas documentadas são a referência a 100%. A faixa tem largura lógica de 44; sua altura é 65 + 47 por conta, limitada à área de trabalho. A UI oferece 70%, 85%, 100% e 115%, com 100% como padrão. A escala aplica-se à UI e à janela nativa, separadamente do DPI do sistema. O mínimo expandido de referência é 260 × 120; as dimensões expandidas persistidas ficam sem a escala da UI.

## Elevation & Depth

A implementação não usa sombras para criar elevação. O fundo, as bordas sutis e o hover tonal delimitam áreas. Detalhes entram com fade breve; barras animam a largura. Respeitar a preferência de movimento reduzido já existente.

## Shapes

Janela com cantos arredondados e controles menores também arredondados. A faixa tem cantos externos arredondados e cantos junto à borda retos. Barras de consumo são baixas e arredondadas; badges usam cápsula.

## Components

### Buttons

Ações de painel usam verde, texto escuro e peso seminegrito; ações secundárias usam o fundo de trilha com texto claro. Botões de ferramenta têm borda tênue e ganham acento no hover. Foco de teclado usa contorno verde de 2px, com deslocamento interno de 2px. Botões desabilitados reduzem a opacidade.

### Inputs / Fields

Campos e seletores usam fundo branco translúcido, borda tênue e raio `field`. O foco muda a borda para verde. Controles nativos de faixa e seleção também usam o acento.

### Cards / Containers

As contas expandidas são linhas sem cartão elevado. O hover tonal destaca a linha; os detalhes usam um separador tracejado e espaçamento compacto. O widget contém a rolagem para manter cabeçalho e rodapé disponíveis.

### Compact accounts

Ícone de provedor acima do valor, com espaçamento `compact`. Exibir percentual arredondado; carregamento usa reticências, erro ou login expirado usa exclamação, cota ilimitada usa infinito e ausência de dados usa traço. O nome da conta e a ação de abrir detalhes ficam disponíveis em texto acessível e tooltip. A seta inferior expande sem selecionar uma conta.

## Do's and Don'ts

- **Do** preservar texto em português, ícones Tabler e cores semânticas existentes.
- **Do** manter escala de UI e DPI como fatores distintos na geometria nativa.
- **Do** usar os detalhes expandidos para informações que não cabem na faixa.
- **Don't** tratar as propostas de lista e barra como alternativas aprovadas.
- **Don't** adicionar nova identidade visual ao implementar a faixa aprovada.
