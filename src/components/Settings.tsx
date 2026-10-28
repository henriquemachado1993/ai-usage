import { useEffect, useState } from "react";
import { IconArrowLeft, IconPencil, IconX } from "@tabler/icons-react";
import { api } from "../lib/api";
import { PROVIDER_NAMES } from "../lib/format";
import type { Account, Settings as SettingsModel } from "../lib/types";
import { ProviderIcon } from "./ProviderIcon";

interface Props {
  settings: SettingsModel;
  onChange: (s: SettingsModel) => void;
  onAddAccount: () => void;
  onBack: () => void;
}

const INTERVALS = [1, 2, 5, 10, 15, 30, 60];

export function Settings({ settings, onChange, onAddAccount, onBack }: Props) {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [error, setError] = useState<string | null>(null);

  const loadAccounts = () => api.listAccounts().then(setAccounts);
  useEffect(() => {
    loadAccounts();
  }, []);

  const run = async (action: () => Promise<unknown>) => {
    setError(null);
    try {
      await action();
      await loadAccounts();
    } catch (e) {
      setError(String(e));
    }
  };

  const [editing, setEditing] = useState<{ id: string; label: string } | null>(null);

  const saveRename = () => {
    if (!editing) return;
    const { id, label } = editing;
    setEditing(null);
    run(() => api.renameAccount(id, label.trim() || null));
  };

  return (
    <div className="panel">
      <div className="panel-title">
        <button className="icon-btn" onClick={onBack} title="Voltar">
          <IconArrowLeft size={16} />
        </button>
        Configurações
      </div>

      <label className="field">
        <span>Atualizar a cada</span>
        <select
          value={settings.refreshMinutes}
          onChange={(e) => onChange({ ...settings, refreshMinutes: Number(e.target.value) })}
        >
          {INTERVALS.map((m) => (
            <option key={m} value={m}>{m} min</option>
          ))}
        </select>
      </label>

      <label className="field">
        <span>Tamanho do widget</span>
        <select value={settings.uiScale} onChange={(e) => onChange({ ...settings, uiScale: Number(e.target.value) })}>
          {[70, 85, 100, 115].map((scale) => <option key={scale} value={scale}>{scale}%{scale === 100 ? " · padrão" : ""}</option>)}
        </select>
      </label>

      <label className="field">
        <span>Opacidade</span>
        <input
          type="range"
          min={0.3}
          max={1}
          step={0.02}
          value={settings.opacity}
          onChange={(e) => onChange({ ...settings, opacity: Number(e.target.value) })}
        />
      </label>

      <label className="field field-check">
        <input
          type="checkbox"
          checked={settings.alwaysOnTop}
          onChange={(e) => onChange({ ...settings, alwaysOnTop: e.target.checked })}
        />
        <span>Sempre no topo</span>
      </label>

      <label className="field field-check">
        <input
          type="checkbox"
          checked={settings.autostart}
          onChange={(e) => onChange({ ...settings, autostart: e.target.checked })}
        />
        <span>Iniciar com o sistema</span>
      </label>

      <div className="panel-section">Contas</div>
      {accounts.length === 0 && <div className="muted small">Nenhuma conta encontrada.</div>}
      {accounts.map((a) => (
        <div className="account-item" key={a.id}>
          <ProviderIcon provider={a.provider} />
          {editing?.id === a.id ? (
            <input
              className="account-name"
              autoFocus
              value={editing.label}
              placeholder="Nome padrão"
              onChange={(e) => setEditing({ id: a.id, label: e.target.value })}
              onBlur={saveRename}
              onKeyDown={(e) => {
                if (e.key === "Enter") saveRename();
                if (e.key === "Escape") setEditing(null);
              }}
            />
          ) : (
            <span className="account-name" title={a.configDir ?? undefined}>
              {a.label ?? PROVIDER_NAMES[a.provider]}
              <span className="muted small"> · {a.source === "auto" ? "auto" : "manual"}</span>
            </span>
          )}
          <button className="icon-btn" title="Renomear" onClick={() => setEditing({ id: a.id, label: a.label ?? "" })}>
            <IconPencil size={15} />
          </button>
          <button className="icon-btn" title="Remover" onClick={() => run(() => api.removeAccount(a.id))}>
            <IconX size={15} />
          </button>
        </div>
      ))}

      {error && <div className="detail-message status-error">{error}</div>}

      <div className="panel-actions">
        <button onClick={onAddAccount}>Adicionar conta</button>
        <button className="secondary" onClick={() => run(api.redetectAccounts)}>Redetectar</button>
      </div>
    </div>
  );
}
