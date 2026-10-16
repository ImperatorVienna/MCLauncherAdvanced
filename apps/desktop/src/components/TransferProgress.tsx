import { t } from "../i18n";
import "../styles/transfer-progress.css";
export function formatBytes(bytes: number) {
    if (bytes < 1024)
        return `${Math.max(0, bytes)} B`;
    const units = ["KiB", "MiB", "GiB", "TiB"];
    let value = bytes / 1024, unit = 0;
    while (value >= 1024 && unit < units.length - 1) {
        value /= 1024;
        unit++;
    }
    return `${value.toFixed(1)} ${units[unit]}`;
}
/** Shared transfer display: works with download or migration byte progress. */
export function TransferProgress({ phase, completed, total, complete = false }: {
    phase: string;
    completed: number;
    total: number;
    complete?: boolean;
}) {
    const determinate = complete || total > 0;
    const percent = complete ? 100 : total > 0 ? Math.max(0, Math.min(100, Math.floor(completed / total * 100))) : 0;
    return <div className="transfer-progress">
    <div className="transfer-caption"><span role="status">{phase}</span><strong>{determinate ? `${percent}%` : t("Preparing\u2026")}</strong></div>
    <div className={`transfer-track${determinate ? "" : " indeterminate"}`} role="progressbar" aria-label={phase} aria-valuemin={0} aria-valuemax={100} aria-valuenow={determinate ? percent : undefined} aria-valuetext={determinate ? `${percent}%` : t("Preparing")}>
      <div className="transfer-fill" style={determinate ? { width: `${percent}%` } : undefined}/>
    </div>
    <div className="transfer-bytes">{total > 0 ? `${formatBytes(Math.min(completed, total))} / ${formatBytes(total)}` : complete ? t("Complete") : t("Counting files")}</div>
  </div>;
}
