import { localizeMessage, t } from "../i18n";
import { useEffect, useRef, useState } from "react";
import { invokeNative } from "../platform/desktop";
import type { Java } from "../state/installer";
import { SelectMenu } from "./SelectMenu";
import { StandardButton } from "./StandardButton";
export type JavaInventory = {
    runtimes: Java[];
    loading: boolean;
    error: string;
    refresh: () => Promise<void>;
};
export function JavaSettings({ inventory, version, value, onChange, onInstall, busy, preview, requiredForInstallation = false, explicitSelection = false, selectionConfirmed = true, scope = "main" }: {
    inventory: JavaInventory;
    version: string;
    value?: string;
    onChange: (path?: string) => void;
    onInstall: (major: number) => void;
    busy: boolean;
    preview: boolean;
    requiredForInstallation?: boolean;
    scope?: string;
    explicitSelection?: boolean;
    selectionConfirmed?: boolean;
}) {
    const [recommended, setRecommended] = useState<number>(), [recommendError, setRecommendError] = useState("");
    const [download, setDownload] = useState("21");
    const downloadChosen = useRef(false);
    useEffect(() => {
        downloadChosen.current = false;
        let alive = true;
        setRecommended(undefined);
        setRecommendError("");
        if (version && !preview)
            void invokeNative<number>("recommended_java", { version }).then(v => { if (alive) {
                setRecommended(v);
                if (!downloadChosen.current) setDownload(String(v));
            } }).catch(e => { if (alive)
                setRecommendError(localizeMessage(e)); });
        return () => { alive = false; };
    }, [version, preview]);
    const paths = inventory.runtimes.map(j => j.path);
    if (value && !paths.includes(value))
        paths.unshift(value);
    return <section className="java-settings" aria-labelledby={`${scope}-java-heading`}>
    <span id={`${scope}-java-heading`}>{t("Java settings")}</span>
    {requiredForInstallation && !explicitSelection && !value && <p>{t("Select Java before installing this mod loader. Download Java here first if needed, then select it and resume installation.")}</p>}
    <p>{t("Select a Java version above to download (the recommended version is selected by default), or choose an existing Java below.")}</p>
    <div className="java-control-row">
    <SelectMenu id={`${scope}-java-download-version`} scope={scope} label={t("Java version to download")} compact showIcons={false} disabled={busy} options={Array.from(new Set([8, 11, 16, 17, 21, 25, ...(recommended ? [recommended] : [])])).sort((a, b) => b - a).map(String)} value={download} labelForValue={v => `Java ${v}`} onChange={major => { downloadChosen.current = true; setDownload(major); }}/>
    <StandardButton id={`${scope}-java-download`} scope={scope} type="button" tone="green" disabled={busy || preview} onClick={() => onInstall(Number(download))}>{t("Download Java {major}", {major: download})}</StandardButton>
    </div>
    <p>{t("After installing Java through the download feature or externally, use the button on the right to rescan if the new Java is missing from the list.")}</p>
    <div className="java-control-row">
    <SelectMenu id={`${scope}-java-select`} scope={scope} label={t("Java runtime")} compact showIcons={false} disabled={busy || inventory.loading} options={explicitSelection && requiredForInstallation ? paths : ["", ...paths]} value={explicitSelection && !selectionConfirmed ? "__unselected__" : value ?? ""} labelForValue={path => {
            if (path === "__unselected__") return t("Choose Java to continue");
            const j = inventory.runtimes.find(j => j.path === path);
            return !path ? t("No Java selected") : j ? `Java ${j.version} · ${j.source === "system" ? t("System") : t("Managed by launcher")} · ${path}` : t("Not detected \u00B7 {value0}", { value0: path });
        }} onChange={path => onChange(path || undefined)}/>
    <StandardButton id={`${scope}-java-refresh`} scope={scope} type="button" disabled={busy || inventory.loading || preview} onClick={() => void inventory.refresh()}>{inventory.loading ? t("Detecting\u2026") : t("Rescan Java")}</StandardButton>
    </div>
    {recommendError && <p role="alert">{t("Java recommendation is unavailable. You can still select Java manually.")}</p>}
    {inventory.error && <p role="alert">{inventory.error}</p>}
  </section>;
}
