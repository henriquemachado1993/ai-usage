import React, { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { IconArrowUpRight, IconChevronLeft, IconChevronRight, IconLayoutSidebarRightCollapse, IconRefresh, IconSettings, IconX, IconAlertTriangle, IconGripVertical } from "@tabler/icons-react";
import { ProviderIcon } from "../components/ProviderIcon";
import type { ProviderKind } from "../lib/types";
import "./proposals.css";

type Scenario = "usage" | "expired" | "offline" | "empty";
type Account = { provider: ProviderKind; name: string; value: number; window: string; reset: string; secondary: number; identity: string };
const accounts: Account[] = [
  { provider: "claude", name: "Claude", value: 48, window: "Sessão · 5h", reset: "2h 14min", secondary: 26, identity: "Pessoal" },
  { provider: "codex", name: "Codex", value: 31, window: "Sessão · 5h", reset: "3h 42min", secondary: 18, identity: "Pessoal" },
  { provider: "copilot", name: "Copilot", value: 100, window: "Premium · mês", reset: "3 dias", secondary: 100, identity: "Trabalho" },
];
const concepts = [
  { id: "rail", name: "Faixa lateral", subtitle: "Sempre à vista, quase fora do caminho.", desc: "Uma evolução da sua referência: três leituras em uma faixa de 44 px. O painel abre para dentro da tela e mantém a ordem das contas.", size: "44 × 206 px", expanded: "320 px de largura", advantage: "Melhor para acompanhar enquanto programa.", risk: "O reset exige abrir o painel." },
  { id: "panel", name: "Lista de bolso", subtitle: "O contexto cabe em uma olhada.", desc: "Um painel flutuante de linhas curtas. Percentual e reset dividem a mesma linha, sem o peso de cartões individuais. Recolhe para uma pequena coluna.", size: "48 × 190 px", expanded: "304 px de largura", advantage: "Melhor equilíbrio entre leitura e espaço.", risk: "Linhas densas exigem leitura mais atenta." },
  { id: "bar", name: "Barra de canto", subtitle: "Três contas em uma única linha.", desc: "Uma barra horizontal para um canto livre do desktop. Cada provedor vira um botão; clicar abre a lista completa acima da barra.", size: "286 × 48 px", expanded: "320 px de largura", advantage: "Libera as laterais do editor.", risk: "Precisa de espaço horizontal livre." },
] as const;
const tone = (n: number) => n >= 85 ? "danger" : n >= 60 ? "warn" : "ok";
const scales = [70, 85, 100, 115];
const scaleKey = "ai-usage:proposals:scale";

function ScaleControl({ scale, onChange }: { scale: number; onChange: (scale: number) => void }) {
  return <label className="scale-control">Tamanho do widget<select value={scale} onChange={e => onChange(Number(e.target.value))}>{scales.map(value => <option key={value} value={value}>{value}%{value === 100 ? " · padrão" : ""}</option>)}</select></label>;
}

function Proposal({ concept, scenario, light, scale, onScaleChange }: { concept: typeof concepts[number]; scenario: Scenario; light: boolean; scale: number; onScaleChange: (scale: number) => void }) {
  const [expanded, setExpanded] = useState(false);
  const [selected, setSelected] = useState<string | null>(null);
  const [settings, setSettings] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [updated, setUpdated] = useState(false);
  const [pinned, setPinned] = useState(true);
  const list = scenario === "empty" ? [] : accounts;
  const value = (a: Account) => scenario === "expired" && a.provider === "claude" ? "!" : `${a.value}%`;
  const accountTone = (a: Account) => scenario === "expired" && a.provider === "claude" ? "warn" : tone(a.value);
  const open = (a?: Account) => { setExpanded(true); setSelected(a?.provider ?? null); setSettings(false); };
  const refresh = () => { setRefreshing(true); window.setTimeout(() => { setRefreshing(false); setUpdated(true); }, 1000); };
  const compactSize = concept.size.replace(/\d+/g, value => String(Math.round(Number(value) * scale / 100)));

  return <article className="proposal">
    <div className="proposal-title"><h2>{concept.name}</h2><span>{compactSize}</span></div>
    <p className="subtitle">{concept.subtitle}</p>
    <div className={`stage ${light ? "light" : ""} ${concept.id}`}>
      <span className="stage-label">{expanded ? "Expandido" : "Compacto"} · escala {scale}%</span>
      <div className={`positioner ${expanded ? "expanded" : ""}`} style={{ "--widget-scale": scale / 100 } as React.CSSProperties}>
        {expanded ? <section className="usage-panel" aria-label={`${concept.name} expandido`}>
          <header><strong>AI Usage</strong><div className="tools"><button title="Configurações da proposta" onClick={() => setSettings(!settings)}><IconSettings size={16}/></button><button title="Recolher" onClick={() => { setExpanded(false); setSettings(false); }}><IconLayoutSidebarRightCollapse size={17}/></button></div></header>
          {settings ? <div className="settings"><h3>Preferências da proposta</h3><ScaleControl scale={scale} onChange={onScaleChange}/><label><input type="checkbox" checked={pinned} onChange={e => setPinned(e.target.checked)}/> Sempre no topo</label><p>O tamanho fica salvo neste navegador e se aplica às três propostas. Nenhuma configuração do app será alterada.</p><button className="text-button" onClick={() => setSettings(false)}>Voltar ao consumo</button></div> : <>
            <div className={`summary ${scenario === "expired" || scenario === "offline" ? "warn" : scenario === "empty" ? "muted" : "danger"}`}><span className="status-dot"/>{scenario === "empty" ? "Adicione sua primeira conta" : scenario === "expired" ? "Claude precisa de login" : scenario === "offline" ? "Offline · último consumo salvo" : "Copilot atingiu o limite"}</div>
            <div className="account-list">{list.map(a => <div className="account" key={a.provider}>
              <button className="account-trigger" aria-expanded={selected === a.provider} onClick={() => setSelected(selected === a.provider ? null : a.provider)}>
                <ProviderIcon provider={a.provider} size={20}/><span className="account-name"><b>{a.name}</b><small>{a.identity}</small></span>
                <span className="account-stat"><b className={accountTone(a)}>{value(a)}</b><small>{scenario === "expired" && a.provider === "claude" ? "Login expirado" : `Reseta em ${a.reset}`}</small></span><IconChevronRight size={14} className={selected === a.provider ? "rotated" : ""}/>
              </button>
              <div className="meter" role="meter" aria-label={`Consumo ${a.name}`} aria-valuemin={0} aria-valuemax={100} aria-valuenow={scenario === "expired" && a.provider === "claude" ? undefined : a.value}><i className={scenario === "expired" && a.provider === "claude" ? "unavailable" : tone(a.value)} style={{width: `${a.value}%`}}/></div>
              {selected === a.provider && <div className="details">
                {scenario === "expired" && a.provider === "claude" ? <p>Login expirado. Abra o Claude Code e execute /login para reconectar.</p> : <><div><span>{a.window}</span><b>{a.value}% usados</b></div><div><span>{a.provider === "copilot" ? "Solicitações premium" : "Semanal · 7d"}</span><b>{a.provider === "copilot" ? "300 / 300" : `${a.secondary}% usados`}</b></div><p>{scenario === "offline" ? "Dados locais; reconecte à internet para atualizar." : `Próximo reset em ${a.reset}.`}</p></>}
                <a href={a.provider === "claude" ? "https://claude.ai/settings/usage" : a.provider === "codex" ? "https://chatgpt.com/codex/settings/usage" : "https://github.com/settings/copilot"} target="_blank" rel="noreferrer">Ver uso no provedor<IconArrowUpRight size={13}/></a>
              </div>}
            </div>)}{!list.length && <div className="empty-state"><p>Conecte as contas que você já usa no seu computador.</p><span>Claude · Codex · GitHub Copilot</span></div>}</div>
            <footer><button onClick={refresh} disabled={refreshing}><IconRefresh size={13} className={refreshing ? "spinning" : ""}/>{refreshing ? "Atualizando…" : updated ? "Atualizado agora" : "Atualizado há 1 min"}</button><span>v0.1</span></footer>
          </>}
        </section> : <div className="compact" aria-label={`${concept.name} compacto`}>
          {concept.id !== "bar" && <div className="compact-handle" aria-hidden="true"><IconGripVertical size={15}/></div>}
          {list.map(a => <button className="compact-account" key={a.provider} onClick={() => open(a)} aria-label={`${a.name}, ${scenario === "expired" && a.provider === "claude" ? "login expirado" : `${value(a)} usados`}. Abrir detalhes.`} title={`${a.name} · ${value(a)} · clique para detalhes`}><ProviderIcon provider={a.provider} size={20}/><b className={accountTone(a)}>{value(a)}</b></button>)}
          {!list.length && <button className="empty-compact" onClick={() => open()} title="Abrir contas"><IconAlertTriangle size={18}/></button>}
          <button className="expand-button" title="Expandir todas as contas" aria-label="Expandir todas as contas" onClick={() => open()}><IconChevronLeft size={16}/></button>
        </div>}
      </div>
      <button className="preview-toggle" onClick={() => { setExpanded(!expanded); setSettings(false); }}>{expanded ? "Ver compacto" : "Ver expandido"}<IconArrowUpRight size={14}/></button>
    </div>
    <p className="description">{concept.desc}</p>
    <dl><div><dt>Expandido</dt><dd>{concept.expanded.replace(/\d+/, value => String(Math.round(Number(value) * scale / 100)))}</dd></div><div><dt>Ideal para</dt><dd>{concept.advantage}</dd></div><div><dt>Troca</dt><dd>{concept.risk}</dd></div></dl>
  </article>;
}

function Proposals() {
  const [scenario, setScenario] = useState<Scenario>("usage");
  const [light, setLight] = useState(false);
  const [note, setNote] = useState(true);
  const [scale, setScale] = useState(() => {
    try {
      const saved = Number(localStorage.getItem(scaleKey));
      return scales.includes(saved) ? saved : 100;
    } catch { return 100; }
  });
  useEffect(() => {
    try { localStorage.setItem(scaleKey, String(scale)); } catch { /* A escala continua válida nesta sessão. */ }
  }, [scale]);
  return <main className="workbench">
    <header className="page-header"><a href="/propostas.html" className="wordmark">AI Usage<span>Estudo de interface</span></a><span className="prototype-label">Protótipos · dados demonstrativos</span></header>
    <section className="intro"><h1>Seu consumo, sem interromper.</h1><p>Três maneiras de acompanhar seus limites. Clique nos provedores para abrir os detalhes e compare o espaço que cada proposta ocupa no desktop.</p></section>
    <div className="comparison-tools"><label>Simular estado<select value={scenario} onChange={e => setScenario(e.target.value as Scenario)}><option value="usage">Uso · Copilot em 100%</option><option value="expired">Login expirado</option><option value="offline">Offline · dados salvos</option><option value="empty">Nenhuma conta</option></select></label><ScaleControl scale={scale} onChange={setScale}/><label className="theme-switch"><input type="checkbox" checked={light} onChange={e => setLight(e.target.checked)}/> Widgets em tema claro</label></div>
    <section className="proposal-grid" aria-label="Propostas de design">{concepts.map(concept => <Proposal key={concept.id} concept={concept} scenario={scenario} light={light} scale={scale} onScaleChange={setScale}/>)}</section>
    {note && <aside className="recommendation"><div><strong>Minha recomendação: Faixa lateral.</strong><p>É a mais próxima da sua referência e preserva o espaço de trabalho. Eu usaria a hierarquia da Lista de bolso quando ela estiver expandida. Percentuais sempre representam uso consumido; cor sinaliza proximidade do limite.</p></div><button title="Dispensar recomendação" onClick={() => setNote(false)}><IconX size={18}/></button></aside>}
    <footer className="page-footer">Exploração visual para Windows e macOS. As ações aqui simulam o comportamento; não consultam contas nem alteram o app.</footer>
  </main>;
}

createRoot(document.getElementById("root")!).render(<React.StrictMode><Proposals/></React.StrictMode>);
