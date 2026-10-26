import type { ProviderKind, UsageSnapshot, UsageWindow } from "./types";

export const PROVIDER_NAMES: Record<ProviderKind, string> = {
  claude: "Claude",
  codex: "Codex",
  copilot: "Copilot",
};

export const USAGE_PAGES: Record<ProviderKind, string> = {
  claude: "https://claude.ai/settings/usage",
  codex: "https://chatgpt.com/codex/settings/usage",
  copilot: "https://github.com/settings/copilot/features",
};

export function level(percent: number): "ok" | "warn" | "danger" {
  if (percent >= 85) return "danger";
  if (percent >= 60) return "warn";
  return "ok";
}

export function criticalWindow(windows: UsageWindow[]): UsageWindow | undefined {
  return windows
    .filter((w) => !w.unlimited)
    .reduce<UsageWindow | undefined>((max, w) => (!max || w.usedPercent > max.usedPercent ? w : max), undefined);
}

export function formatDuration(ms: number): string {
  if (ms <= 0) return "agora";
  const minutes = Math.round(ms / 60000);
  if (minutes < 60) return `${minutes}min`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return minutes % 60 ? `${hours}h ${minutes % 60}min` : `${hours}h`;
  const days = Math.floor(hours / 24);
  return hours % 24 ? `${days}d ${hours % 24}h` : `${days}d`;
}

export function resetsIn(resetsAt: string | null, now: number): string | null {
  return resetsAt ? formatDuration(new Date(resetsAt).getTime() - now) : null;
}

export function formatDateTime(iso: string): string {
  return new Date(iso).toLocaleString("pt-BR", {
    day: "2-digit",
    month: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function formatAgo(iso: string, now: number): string {
  const minutes = Math.floor((now - new Date(iso).getTime()) / 60000);
  if (minutes < 1) return "agora mesmo";
  if (minutes < 60) return `há ${minutes} min`;
  return `há ${Math.floor(minutes / 60)}h`;
}

const SHORT_NAMES: Record<string, string> = {
  "Sessão (5h)": "5h",
  Semanal: "7d",
  "Semanal Opus": "7d Opus",
  "Semanal Sonnet": "7d Sonnet",
  "Uso extra (mensal)": "extra",
  "Premium requests": "mês",
  Chat: "chat",
  Completions: "código",
  "30 dias": "30d",
};

export function shortWindowName(name: string): string {
  return SHORT_NAMES[name] ?? name;
}

const AVATAR_COLORS = ["#4f7cff", "#8b5cf6", "#ec4899", "#14b8a6", "#f59e0b", "#06b6d4", "#84cc16"];

export function avatarFor(text: string): { letter: string; color: string } {
  let hash = 0;
  for (const ch of text) hash = (hash * 31 + ch.charCodeAt(0)) >>> 0;
  return { letter: (text.trim()[0] ?? "?").toUpperCase(), color: AVATAR_COLORS[hash % AVATAR_COLORS.length] };
}

export function snapshotLevel(s: UsageSnapshot): "ok" | "warn" | "danger" | "muted" {
  if (s.status === "error") return "danger";
  if (s.status === "expired" || s.status === "stale") return "warn";
  if (s.status === "loading") return "muted";
  const critical = criticalWindow(s.windows);
  return critical ? level(critical.usedPercent) : "ok";
}

export function overallLevel(snapshots: UsageSnapshot[]): "ok" | "warn" | "danger" | "muted" {
  const levels = snapshots.map(snapshotLevel);
  if (levels.includes("danger")) return "danger";
  if (levels.includes("warn")) return "warn";
  if (levels.length === 0 || levels.every((l) => l === "muted")) return "muted";
  return "ok";
}
