import { useState } from "react";
import { PROVIDER_NAMES, avatarFor, criticalWindow, resetsIn, shortWindowName, snapshotLevel } from "../lib/format";
import type { UsageSnapshot } from "../lib/types";
import { DetailPanel } from "./DetailPanel";
import { ProviderIcon } from "./ProviderIcon";
import { UsageBar } from "./UsageBar";

interface Props {
  snapshot: UsageSnapshot;
  now: number;
  selected: boolean;
  onToggle: () => void;
}

const STATUS_TEXT: Partial<Record<UsageSnapshot["status"], string>> = {
  loading: "carregando…",
  expired: "login expirado",
  error: "erro",
};

export function AccountRow({ snapshot, now, selected, onToggle }: Props) {
  const [hovered, setHovered] = useState(false);
  const expanded = selected || hovered;
  const critical = criticalWindow(snapshot.windows);
  const unlimited = critical ? undefined : snapshot.windows.find((w) => w.unlimited);
  const loading = snapshot.status === "loading";
  const failed = snapshot.status === "expired" || snapshot.status === "error";
  const avatar = avatarFor(snapshot.label);
  const reset = critical && resetsIn(critical.resetsAt, now);

  let percent = "";
  let sub = STATUS_TEXT[snapshot.status] ?? "";
  if (!loading && !failed) {
    if (critical) {
      percent = `${Math.round(critical.usedPercent)}%`;
      sub = reset ? `reseta em ${reset}` : "";
    } else if (unlimited) {
      percent = "∞";
      sub = unlimited.used !== null ? `${Math.round(unlimited.used)} usados` : "ilimitado";
    } else {
      sub = "sem dados";
    }
    if (snapshot.status === "stale") sub = `${sub} · offline`;
  }

  return (
    <div
      className={`row ${expanded ? "row-expanded" : ""}`}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <button className="row-head" onClick={onToggle} aria-expanded={expanded} aria-label={`${PROVIDER_NAMES[snapshot.provider]} · ${snapshot.label}. ${expanded ? "Detalhes abertos" : "Abrir detalhes"}`}>
        <ProviderIcon provider={snapshot.provider} />
        <span className="avatar" style={{ background: avatar.color }} title={snapshot.label}>
          {avatar.letter}
        </span>
        <span className="row-name">{PROVIDER_NAMES[snapshot.provider]}</span>
        {(critical ?? unlimited) && <span className="row-window">{shortWindowName((critical ?? unlimited)!.name)}</span>}
        <span className={`row-sub status-${snapshot.status}`}>{sub}</span>
        <span className={`row-percent text-${snapshotLevel(snapshot)} status-${snapshot.status}`}>{failed ? "!" : percent}</span>
      </button>
      <UsageBar percent={loading || failed ? null : (critical?.usedPercent ?? 0)} />
      {expanded && !loading && <DetailPanel snapshot={snapshot} now={now} />}
    </div>
  );
}
