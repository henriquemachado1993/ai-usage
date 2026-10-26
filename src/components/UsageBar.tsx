import { level } from "../lib/format";

interface Props {
  percent: number | null;
  thin?: boolean;
}

export function UsageBar({ percent, thin }: Props) {
  const value = percent === null ? 0 : Math.min(100, Math.max(0, percent));
  return (
    <div className={`bar ${thin ? "bar-thin" : ""} ${percent === null ? "bar-loading" : ""}`}>
      <div className={`bar-fill level-${level(value)}`} style={{ width: `${value}%` }} />
    </div>
  );
}
