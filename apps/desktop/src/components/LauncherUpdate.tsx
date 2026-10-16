import { useLauncherUpdate } from "../state/launcherUpdate";
import { version } from "../../src-tauri/tauri.conf.json";
import { t, localizeMessage, useLocale } from "../i18n";
import { StandardButton } from "./StandardButton";

export function LauncherUpdate(){
    useLocale();
    const {status,error,pending,action}=useLauncherUpdate();
    const busy=!!pending||["checking","downloading","restarting"].includes(status.phase);
    const messages:Record<string,string>={
        checking:t("Checking launcher updates…"), current:t("The launcher is up to date."),
        unpublished:t("No public stable release is available yet."),
        available:t("Launcher update available: {version}",{version:status.version??""}),
        downloading:t("Downloading launcher update: {percent}%",{percent:status.total?Math.floor(status.completed/status.total*100):0}),
        restarting:t("Restarting the launcher…"),
    };
    return <div className="settings-group launcher-updater">
        <div className="java-control-row">
            <span>{t("Launcher version: {version}",{version})}</span>
            <StandardButton id="launcher-check-updates" tone="white" disabled={busy} onClick={()=>void action("check_launcher_update")}>{t("Check launcher updates")}</StandardButton>
            {status.version&&<StandardButton id="launcher-update-action" tone="green" disabled={busy} onClick={()=>void action("download_launcher_update")}>{t("Download launcher update")}</StandardButton>}
        </div>
        {messages[status.phase]&&<p role="status">{messages[status.phase]}</p>}
        {(error||status.error)&&<p className="config-error" role="alert">{localizeMessage(error||status.error)}</p>}
    </div>;
}
