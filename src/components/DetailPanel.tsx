import { IconExternalLink } from "@tabler/icons-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { USAGE_PAGES, formatAgo, formatDateTime, resetsIn } from "../lib/format";
import type { UsageSnapshot } from "../lib/types";
import { UsageBar } from "./UsageBar";

interface Props {
  snapshot: UsageSnapshot;
  now: number;
}

export function DetailPanel({ snapshot, now }: Props) {
  return (
    <div className="detail">
      <div className="detail-account">
        <span>{snapshot.label}</span>
        {snapshot.plan && <span className="badge">{snapshot.plan}</span>}
      </div>

      {snapshot.windows.map((w) => (
        <div className="detail-window" key={w.name}>
          <div className="detail-line">
            <span>{w.name}</span>
            <span className="muted">
              {w.unlimited ? "Ilimitado" : `${Math.round(w.usedPercent)}%`}
              {w.used !== null && w.limit !== null && ` · ${Math.round(w.used)}/${Math.round(w.limit)}`}
              {w.unlimited && w.used !== null && ` · ${Math.round(w.used)} usados`}
            </span>
          </div>
          {!w.unlimited && <UsageBar percent={w.usedPercent} thin />}
          {w.resetsAt && (
            <div className="detail-sub muted">
              Reseta {formatDateTime(w.resetsAt)} (em {resetsIn(w.resetsAt, now)})
            </div>
          )}
        </div>
      ))}

      {snapshot.message && <div className={`detail-message status-${snapshot.status}`}>{snapshot.message}</div>}

      <div className="detail-footer">
        <span className="muted">
          {snapshot.source === "auto" ? "Auto" : "Manual"} · atualizado {formatAgo(snapshot.fetchedAt, now)}
        </span>
        <button className="link" onClick={() => openUrl(USAGE_PAGES[snapshot.provider])}>
          Página de uso <IconExternalLink size={12} />
        </button>
      </div>
    </div>
  );
}
