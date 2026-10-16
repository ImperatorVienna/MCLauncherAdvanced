import { useState } from "react";
import { t, localizeMessage } from "../i18n";
import type { GameStatus } from "../state/useGameLaunch";
import type { LogRequest } from "../state/logs";
import { copyLogText } from "../state/logs";
import { StandardButton } from "./StandardButton";
export function GameFailure({status,onViewLogs}: {status:GameStatus;onViewLogs:(request:LogRequest)=>void}) {
    const [copied,setCopied]=useState(false),[error,setError]=useState("");
    const message=localizeMessage(status.error);
    async function copy() {try{await copyLogText(message+(status.logPath ? "\n"+status.logPath : ""));setCopied(true);}catch(e){setError(localizeMessage(e));}}
    return <div className="config-error page-notice game-failure" role="alert">
        <p>{message}</p>
        {status.logPath && <p>{t("Game log: {path}",{path:status.logPath})}</p>}
        <div className="java-control-row">
            {status.instanceId && <StandardButton id="game-failure-logs" tone="yellow" onClick={()=>onViewLogs({instanceId:status.instanceId!,initialName:status.logPath ? "logs/"+status.logPath.split(/[\\/]/).pop() : undefined})}>{t("View this launch log")}</StandardButton>}
            <StandardButton id="game-failure-copy" onClick={()=>void copy()}>{t(copied ? "Copied" : "Copy error details")}</StandardButton>
        </div>
        {error && <p>{error}</p>}
    </div>;
}
