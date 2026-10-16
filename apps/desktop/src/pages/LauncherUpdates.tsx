import { t, localizeMessage } from "../i18n";
import { StandardButton } from "../components/StandardButton";
import { useLauncherUpdate } from "../state/launcherUpdate";
export function LauncherUpdates({onSettings}:{onSettings:()=>void}) {
    const {status,error}=useLauncherUpdate();
    return <section className="placeholder settings-page launcher-announcements" aria-label={t("Launcher updates")}>
        {status.version&&<div><StandardButton id="announcement-settings" tone="green" onClick={onSettings}>{t("Go to Settings to update")}</StandardButton></div>}
        {status.announcement ? <div className="settings-group">
            <span>{t("Launcher version: {version}",{version:status.announcement.version})}</span>
            <p className="release-notes">{status.announcement.body || t("No release notes were provided.")}</p>
        </div> : !(error||status.error)&&<p role="status">{status.phase==="checking" ? t("Checking launcher updates…") : t("No public stable release is available yet.")}</p>}
        {(error||status.error)&&<p className="config-error" role="alert">{localizeMessage(error||status.error)}</p>}
    </section>;
}
