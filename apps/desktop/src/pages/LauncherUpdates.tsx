import { useReleaseNotes } from "../state/releaseNotes";
import { t, localizeMessage } from "../i18n";
import { StandardButton } from "../components/StandardButton";
import { useLauncherUpdate } from "../state/launcherUpdate";
export function LauncherUpdates({onSettings}:{onSettings:()=>void}) {
    const {status,error}=useLauncherUpdate();
    const notes=useReleaseNotes(status.announcement?.version);
    return <section className="placeholder settings-page launcher-announcements" aria-label={t("Launcher updates")}>
        {status.version&&<div><StandardButton id="announcement-settings" tone="green" onClick={onSettings}>{t("Go to About to update")}</StandardButton></div>}
        {status.announcement ? <div className="settings-group">
            <span>{t("Launcher version: {version}",{version:status.announcement.version})}</span>
            <p className="release-notes">{notes.loading ? t("Loading release notes…") : notes.failed ? t("Could not load release notes for this language.") : notes.body || t("No release notes are available in this language.")}</p>
        </div> : !(error||status.error)&&<p role="status">{status.phase==="checking" ? t("Checking launcher updates…") : t("No public stable release is available yet.")}</p>}
        {(error||status.error)&&<p className="config-error" role="alert">{localizeMessage(error||status.error)}</p>}
    </section>;
}
