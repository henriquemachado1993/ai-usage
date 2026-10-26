import { IconChevronLeft, IconChevronRight } from "@tabler/icons-react";
import { PROVIDER_NAMES, criticalWindow, overallLevel, snapshotLevel } from "../lib/format";
import type { Edge, UsageSnapshot } from "../lib/types";
import { Logo } from "./Logo";
import { ProviderIcon } from "./ProviderIcon";

interface Props {
  snapshots: UsageSnapshot[];
  edge: Edge;
  onExpand: () => void;
}

export const STRIP_ITEM_HEIGHT = 46;
export const STRIP_CHROME_HEIGHT = 124;

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
      <div className="strip-logo" data-tauri-drag-region>
        <Logo size={24} />
      </div>
      <span className={`dot level-${overallLevel(snapshots)}`} />
      {snapshots.map((s) => (
        <div
          key={s.accountId}
          className="strip-item"
          title={`${PROVIDER_NAMES[s.provider]} · ${s.label}${s.message ? `\n${s.message}` : ""}`}
          onClick={onExpand}
        >
          <ProviderIcon provider={s.provider} size={20} />
          <span className={`strip-value text-${snapshotLevel(s)}`}>{stripValue(s)}</span>
        </div>
      ))}
      <button className="strip-toggle" title="Expandir" onClick={onExpand}>
        <Chevron size={18} />
      </button>
    </div>
  );
}
