export type ProviderKind = "claude" | "codex" | "copilot";
export type AccountSource = "auto" | "manual";
export type Status = "loading" | "ok" | "stale" | "expired" | "error";

export interface UsageWindow {
  name: string;
  usedPercent: number;
  resetsAt: string | null;
  used: number | null;
  limit: number | null;
  unlimited: boolean;
}

export interface UsageSnapshot {
  accountId: string;
  provider: ProviderKind;
  source: AccountSource;
  label: string;
  identity: string | null;
  plan: string | null;
  windows: UsageWindow[];
  status: Status;
  message: string | null;
  fetchedAt: string;
}

export interface Account {
  id: string;
  provider: ProviderKind;
  source: AccountSource;
  label: string | null;
  configDir: string | null;
  hasToken: boolean;
}

export interface Settings {
  refreshMinutes: number;
  opacity: number;
  alwaysOnTop: boolean;
  autostart: boolean;
  dismissed: string[];
  collapsed: boolean;
  expandedWidth: number;
  expandedHeight: number;
  uiScale: number;
}

export interface NewAccount {
  provider: ProviderKind;
  label?: string;
  configDir?: string;
  token?: string;
}

export type Edge = "none" | "left" | "right";

export interface DockState {
  edge: Edge;
  collapsed: boolean;
}
