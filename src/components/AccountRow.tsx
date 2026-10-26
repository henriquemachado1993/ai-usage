import { useState } from "react";
import { PROVIDER_NAMES, avatarFor, criticalWindow, resetsIn, shortWindowName } from "../lib/format";
import type { UsageSnapshot } from "../lib/types";
import { DetailPanel } from "./DetailPanel";
import { ProviderIcon } from "./ProviderIcon";
import { UsageBar } from "./UsageBar";

interface Props {
  snapshot: UsageSnapshot;
  now: number;
}

const STATUS_TEXT: Partial<Record<UsageSnapshot["status"], string>> = {
  loading: "carregando…",
  expired: "login expirado",
  error: "erro",
};

export function AccountRow({ snapshot, now }: Props) {
  const [expanded, setExpanded] = useState(false);
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
      onMouseEnter={() => setExpanded(true)}
      onMouseLeave={() => setExpanded(false)}
    >
      <div className="row-head">
        <ProviderIcon provider={snapshot.provider} />
        <span className="avatar" style={{ background: avatar.color }} title={snapshot.label}>
          {avatar.letter}
        </span>
        <span className="row-name">{PROVIDER_NAMES[snapshot.provider]}</span>
        {(critical ?? unlimited) && <span className="row-window">{shortWindowName((critical ?? unlimited)!.name)}</span>}
        <span className={`row-sub status-${snapshot.status}`}>{sub}</span>
        <span className={`row-percent status-${snapshot.status}`}>{failed ? "!" : percent}</span>
      </div>
      <UsageBar percent={loading || failed ? null : (critical?.usedPercent ?? 0)} />
      {expanded && !loading && <DetailPanel snapshot={snapshot} now={now} />}
    </div>
  );
}
