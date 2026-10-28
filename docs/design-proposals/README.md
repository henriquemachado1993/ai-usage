# Propostas de interface

Execute `npm run dev` na raiz e abra http://127.0.0.1:1420/propostas.html.

Três alternativas com compacto e expandido: Faixa lateral, Lista de bolso e Barra de canto. Clique nos ícones ou em Ver expandido. O seletor simula consumo, login expirado, offline e nenhuma conta. O tema claro se aplica aos widgets. Dados e configurações são demonstrativos; não há acesso a credenciais nem IPC Tauri.

Tamanho do widget: 70%, 85%, 100% ou 115%, disponível na comparação e nas preferências de cada proposta. Escala ícones, texto, espaçamento e superfície nos modos compacto e expandido; a página de comparação mantém sua escala. A preferência é salva apenas no localStorage do navegador, com retorno a 100% se o valor salvo for inválido.

O usuário confirmou compacto com ícone + percentual e detalhes ao abrir. Nenhuma alternativa está aprovada; não consolidar um novo sistema visual em DESIGN.md antes da escolha. A página usa os ícones de provedor do app e Tabler para controles, fonte de sistema e cores semânticas de consumo.

Validação: TypeScript; detector sem achados; inspeção visual em 1440 px e 390 px; cliques, detalhes, erros, tema, atualização e ausência de overflow horizontal verificados via Chrome DevTools Protocol. As imagens registram a exploração. O script `check.mjs` espera o Vite ativo e um Chrome headless de teste com depuração remota na porta 9333, e fecha essa instância ao terminar.

Não valida comportamento nativo de encaixe, arrasto ou sempre no topo. Esta exploração é independente do widget de produção.
