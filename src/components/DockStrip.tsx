import { IconChevronLeft, IconChevronRight, IconGripVertical } from "@tabler/icons-react";
import { PROVIDER_NAMES, criticalWindow, snapshotLevel } from "../lib/format";
import type { Edge, UsageSnapshot } from "../lib/types";
import { ProviderIcon } from "./ProviderIcon";

interface Props {
  snapshots: UsageSnapshot[];
  edge: Edge;
  onExpand: (accountId?: string) => void;
}

export const STRIP_ITEM_HEIGHT = 47;
export const STRIP_CHROME_HEIGHT = 65;

function stripValue(s: UsageSnapshot): string {
  if (s.status === "expired" || s.status === "error") return "!";
  if (s.status === "loading") return "…";
  const critical = criticalWindow(s.windows);
  if (critical) return `${Math.round(critical.usedPercent)}%`;
  return s.windows.some((w) => w.unlimited) ? "∞" : "–";
}

export function DockStrip({ snapshots, edge, onExpand }: Props) {
  const Chevron = edge === "left" ? IconChevronRight : IconChevronLeft;
  return (
    <div className={`strip strip-${edge}`} data-tauri-drag-region>
      <div className="strip-handle" data-tauri-drag-region>
        <IconGripVertical size={15} />
      </div>
      {snapshots.map((s) => (
        <button
          key={s.accountId}
          className="strip-item"
          title={`${PROVIDER_NAMES[s.provider]} · ${s.label} · ${stripValue(s)}${s.message ? `\n${s.message}` : ""}`}
          aria-label={`${PROVIDER_NAMES[s.provider]} · ${s.label}. ${s.status === "expired" ? "Login expirado" : s.status === "error" ? "Erro" : stripValue(s)}. Abrir detalhes`}
          onClick={() => onExpand(s.accountId)}
        >
          <ProviderIcon provider={s.provider} size={20} />
          <span className={`strip-value text-${snapshotLevel(s)}`}>{stripValue(s)}</span>
        </button>
      ))}
      <button className="strip-toggle" title="Expandir" onClick={() => onExpand()}>
        <Chevron size={18} />
      </button>
    </div>
  );
}
