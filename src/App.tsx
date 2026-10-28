import { useEffect, useState } from "react";
import { IconChevronLeft, IconChevronRight, IconPower, IconRefresh, IconSettings } from "@tabler/icons-react";
import { getVersion } from "@tauri-apps/api/app";
import { AccountRow } from "./components/AccountRow";
import { AddAccount } from "./components/AddAccount";
import { DockStrip, STRIP_CHROME_HEIGHT, STRIP_ITEM_HEIGHT } from "./components/DockStrip";
import { Logo } from "./components/Logo";
import { ResizeHandles } from "./components/ResizeHandles";
import { Settings } from "./components/Settings";
import { api, onDockChanged, onOpenSettings, onRefreshing, onUsageUpdated } from "./lib/api";
import { formatAgo, overallLevel, snapshotLevel } from "./lib/format";
import type { DockState, Settings as SettingsModel, UsageSnapshot } from "./lib/types";

type View = "list" | "settings" | "add";

function useNow(intervalMs: number) {
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    const id = setInterval(() => setNow(Date.now()), intervalMs);
    return () => clearInterval(id);
  }, [intervalMs]);
  return now;
}

function statusText(snapshots: UsageSnapshot[], refreshing: boolean): string {
  if (refreshing) return "Atualizando…";
  if (snapshots.length === 0) return "Nenhuma conta";
  const danger = snapshots.filter((s) => snapshotLevel(s) === "danger").length;
  const warn = snapshots.filter((s) => snapshotLevel(s) === "warn").length;
  if (danger) return danger === 1 ? "1 conta perto do limite" : `${danger} contas perto do limite`;
  if (warn) return warn === 1 ? "1 conta precisa de atenção" : `${warn} contas precisam de atenção`;
  return "Tudo em dia";
}

const stripHeight = (count: number) => STRIP_CHROME_HEIGHT + count * STRIP_ITEM_HEIGHT;

export default function App() {
  const [snapshots, setSnapshots] = useState<UsageSnapshot[]>([]);
  const [settings, setSettings] = useState<SettingsModel | null>(null);
  const [dock, setDock] = useState<DockState>({ edge: "none", collapsed: false });
  const [refreshing, setRefreshing] = useState(false);
  const [view, setView] = useState<View>("list");
  const [version, setVersion] = useState("");
  const [selectedAccount, setSelectedAccount] = useState<string | null>(null);
  const [settingsError, setSettingsError] = useState<string | null>(null);
  const now = useNow(30_000);

  useEffect(() => {
    api.getSnapshots().then(setSnapshots);
    api.getSettings().then(setSettings);
    api.getDock().then(setDock);
    getVersion().then(setVersion);
    const unlisten = [
      onUsageUpdated(setSnapshots),
      onRefreshing(setRefreshing),
      onDockChanged(setDock),
      onOpenSettings(() => {
        api.setCollapsed(false, 0).then(setDock);
        setView("settings");
      }),
    ];
    return () => unlisten.forEach((p) => p.then((off) => off()));
  }, []);

  useEffect(() => {
    if (dock.collapsed) api.setCollapsed(true, stripHeight(snapshots.length));
  }, [dock.collapsed, snapshots.length]);

  const updateSettings = (next: SettingsModel) => {
    setSettingsError(null);
    api.setSettings(next).then(setSettings).catch((error) => setSettingsError(`Não foi possível salvar: ${String(error)}`));
  };

  const collapse = (collapsed: boolean) => {
    if (collapsed) {
      setView("list");
      setSelectedAccount(null);
    }
    api.setCollapsed(collapsed, stripHeight(snapshots.length)).then(setDock);
  };

  const scale = (settings?.uiScale ?? 100) / 100;
  const alpha = {
    "--alpha": settings?.opacity ?? 0.94,
    width: `${100 / scale}%`,
    height: `${100 / scale}%`,
    transform: `scale(${scale})`,
    transformOrigin: "top left",
  } as React.CSSProperties;

  if (dock.collapsed) {
    return (
      <div className="root" style={alpha}>
        <DockStrip snapshots={snapshots} edge={dock.edge} onExpand={(accountId) => { setSelectedAccount(accountId ?? null); collapse(false); }} />
      </div>
    );
  }

  const lastUpdate = snapshots
    .filter((s) => s.status !== "loading")
    .map((s) => s.fetchedAt)
    .sort()
    .at(-1);
  const CollapseIcon = dock.edge === "left" ? IconChevronLeft : IconChevronRight;

  return (
    <div className="root" style={alpha}>
      <div className={`widget edge-${dock.edge}`}>
        <header className="header" data-tauri-drag-region>
          <Logo />
          <span className="brand" data-tauri-drag-region>
            AI Usage
          </span>
          <button className="tool-btn" title="Configurações" onClick={() => setView(view === "list" ? "settings" : "list")}>
            <IconSettings size={17} />
          </button>
          <button className="tool-btn" title="Recolher para a borda" onClick={() => collapse(true)}>
            <CollapseIcon size={17} />
          </button>
        </header>

        <div className={`status-line text-${refreshing ? "muted" : overallLevel(snapshots)}`} data-tauri-drag-region>
          <span className={`dot level-${refreshing ? "muted" : overallLevel(snapshots)}`} />
          {statusText(snapshots, refreshing)}
        </div>

        <main className="content">
          {settingsError && <div role="alert" className="detail-message status-error">{settingsError}</div>}
          {view === "list" && (
            <>
              {snapshots.map((s) => (
                <AccountRow key={s.accountId} snapshot={s} now={now} selected={selectedAccount === s.accountId} onToggle={() => setSelectedAccount(selectedAccount === s.accountId ? null : s.accountId)} />
              ))}
              {snapshots.length === 0 && (
                <div className="empty muted">
                  Nenhuma conta detectada.
                  <button className="link" onClick={() => setView("add")}>
                    Adicionar conta
                  </button>
                </div>
              )}
            </>
          )}
          {view === "settings" && settings && (
            <Settings
              settings={settings}
              onChange={updateSettings}
              onAddAccount={() => setView("add")}
              onBack={() => setView("list")}
            />
          )}
          {view === "add" && <AddAccount onDone={() => setView("list")} onBack={() => setView("settings")} />}
        </main>

        <footer className="footer">
          <button className="footer-btn" title="Atualizar agora" onClick={() => api.refreshNow()}>
            <IconRefresh size={16} className={refreshing ? "spinning" : ""} />
            {lastUpdate ? `Atualizado ${formatAgo(lastUpdate, now)}` : "Atualizar"}
          </button>
          <span className="muted version">v{version}</span>
          <button className="footer-btn" title="Fechar o AI Usage" onClick={() => api.quitApp()}>
            <IconPower size={16} />
            Sair
          </button>
        </footer>

        <ResizeHandles />
      </div>
    </div>
  );
}
