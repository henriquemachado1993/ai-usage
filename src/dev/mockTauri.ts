import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";

const inHours = (h: number) => new Date(Date.now() + h * 3_600_000).toISOString();
const now = new Date().toISOString();

const snapshots = [
  {
    accountId: "codex-auto", provider: "codex", source: "auto", label: "henrique@agiletv.com", identity: "henrique@agiletv.com",
    plan: "Plus", status: "ok", message: null, fetchedAt: now,
    windows: [
      { name: "Sessão (5h)", usedPercent: 18, resetsAt: inHours(3), used: null, limit: null, unlimited: false },
      { name: "Semanal", usedPercent: 49, resetsAt: inHours(120), used: null, limit: null, unlimited: false },
    ],
  },
  {
    accountId: "claude-auto", provider: "claude", source: "auto", label: "Pessoal", identity: "rique@mail.com",
    plan: "Max", status: "ok", message: null, fetchedAt: now,
    windows: [
      { name: "Sessão (5h)", usedPercent: 72, resetsAt: inHours(2), used: null, limit: null, unlimited: false },
      { name: "Semanal", usedPercent: 31, resetsAt: inHours(90), used: null, limit: null, unlimited: false },
    ],
  },
  {
    accountId: "claude-work", provider: "claude", source: "manual", label: "Gabriel", identity: null,
    plan: "Team", status: "expired", message: "Token expirado — abra o Claude Code para renová-lo", fetchedAt: now, windows: [],
  },
  {
    accountId: "copilot-auto", provider: "copilot", source: "auto", label: "hsantos", identity: "hsantos",
    plan: "Business", status: "ok", message: null, fetchedAt: now,
    windows: [{ name: "Premium requests", usedPercent: 0, resetsAt: inHours(140), used: 1119, limit: null, unlimited: true }],
  },
  {
    accountId: "copilot-free", provider: "copilot", source: "manual", label: "rique-oss", identity: "rique-oss",
    plan: null, status: "ok", message: null, fetchedAt: now,
    windows: [{ name: "Premium requests", usedPercent: 91, resetsAt: inHours(200), used: 273, limit: 300, unlimited: false }],
  },
];

let settings = { refreshMinutes: 5, opacity: 0.94, alwaysOnTop: true, autostart: false, dismissed: [], collapsed: false, expandedWidth: 340, expandedHeight: 420 };
let dock = { edge: new URLSearchParams(location.search).get("edge") ?? "right", collapsed: location.search.includes("collapsed") };

mockWindows("main");
mockIPC(
  (cmd, args) => {
    const a = args as Record<string, unknown>;
    switch (cmd) {
      case "get_snapshots":
        return snapshots;
      case "get_settings":
        return settings;
      case "set_settings":
        settings = a.settings as typeof settings;
        return settings;
      case "get_dock":
        return dock;
      case "set_collapsed":
        dock = { ...dock, collapsed: a.collapsed as boolean };
        return dock;
      case "list_accounts":
        return snapshots.map((s) => ({ id: s.accountId, provider: s.provider, source: s.source, label: s.label, configDir: null, hasToken: false }));
      case "plugin:app|version":
        return "0.1.0";
      default:
        return null;
    }
  },
  { shouldMockEvents: true },
);
