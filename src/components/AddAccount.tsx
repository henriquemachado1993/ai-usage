import { useState } from "react";
import { IconArrowLeft } from "@tabler/icons-react";
import { api } from "../lib/api";
import type { ProviderKind } from "../lib/types";

interface Props {
  onDone: () => void;
  onBack: () => void;
}

const DIR_HINTS: Record<Exclude<ProviderKind, "copilot">, string> = {
  claude: "Pasta usada com CLAUDE_CONFIG_DIR, ex.: ~/.claude-trabalho",
  codex: "Pasta usada com CODEX_HOME, ex.: ~/.codex-trabalho",
};

export function AddAccount({ onDone, onBack }: Props) {
  const [provider, setProvider] = useState<ProviderKind>("claude");
  const [label, setLabel] = useState("");
  const [configDir, setConfigDir] = useState("");
  const [token, setToken] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    setError(null);
    try {
      await api.addAccount({
        provider,
        label,
        configDir: provider === "copilot" ? undefined : configDir,
        token: provider === "copilot" ? token : undefined,
      });
      onDone();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  return (
    <form className="panel" onSubmit={submit}>
      <div className="panel-title">
        <button type="button" className="icon-btn" onClick={onBack} title="Voltar">
          <IconArrowLeft size={16} />
        </button>
        Adicionar conta
      </div>

      <label className="field">
        <span>Provedor</span>
        <select value={provider} onChange={(e) => setProvider(e.target.value as ProviderKind)}>
          <option value="claude">Claude</option>
          <option value="codex">Codex</option>
          <option value="copilot">GitHub Copilot</option>
        </select>
      </label>

      <label className="field field-col">
        <span>Nome (opcional)</span>
        <input value={label} onChange={(e) => setLabel(e.target.value)} placeholder="Ex.: Trabalho" />
      </label>

      {provider === "copilot" ? (
        <label className="field field-col">
          <span>Token do GitHub</span>
          <input type="password" value={token} onChange={(e) => setToken(e.target.value)} placeholder="gho_… ou ghp_…" />
          <span className="muted small">Salvo no cofre de credenciais do sistema. Use a saída de `gh auth token` da conta desejada.</span>
        </label>
      ) : (
        <label className="field field-col">
          <span>Diretório de configuração</span>
          <input value={configDir} onChange={(e) => setConfigDir(e.target.value)} placeholder={DIR_HINTS[provider]} />
          <span className="muted small">{DIR_HINTS[provider]}</span>
        </label>
      )}

      {error && <div className="detail-message status-error">{error}</div>}

      <div className="panel-actions">
        <button type="submit" disabled={saving}>{saving ? "Salvando…" : "Adicionar"}</button>
      </div>
    </form>
  );
}
