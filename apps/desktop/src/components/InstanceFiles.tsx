import { useState } from "react";
import { localizeMessage, t } from "../i18n";
import { isDesktop } from "../platform/desktop";
import { openLogLocation } from "../state/logs";
import { StandardButton } from "./StandardButton";
export function InstanceFiles({instanceId,onViewLogs}: {instanceId:string;onViewLogs:()=>void}) {
    const [error,setError]=useState("");
    async function open() {setError("");try{await openLogLocation(instanceId);}catch(e){setError(localizeMessage(e));}}
    return <div className="instance-files">
        <div className="java-control-row">
            <StandardButton id="instance-open-directory" disabled={!isDesktop()} onClick={()=>void open()}>{t("Open instance directory")}</StandardButton>
            <StandardButton id="instance-view-logs" tone="yellow" onClick={onViewLogs}>{t("View logs")}</StandardButton>
        </div>
        {error && <p className="config-error" role="alert">{error}</p>}
    </div>;
}
