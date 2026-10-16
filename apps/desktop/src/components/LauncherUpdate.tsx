import { useEffect, useRef, useState } from "react";
import { version } from "../../src-tauri/tauri.conf.json";
import { invokeNative, isDesktop } from "../platform/desktop";
import { t, localizeMessage, useLocale } from "../i18n";
import { StandardButton } from "./StandardButton";

type Status = {phase:string;version:string|null;completed:number;total:number;error:string|null};
const initial:Status={phase:"idle",version:null,completed:0,total:0,error:null};
export function LauncherUpdate(){
    useLocale();
    const [status,setStatus]=useState(initial),[error,setError]=useState(""),[pending,setPending]=useState(false);
    const alive=useRef(false);
    useEffect(()=>{
        alive.current=true;
        if(!isDesktop())return()=>{alive.current=false;};
        let stopped=false,timer:ReturnType<typeof setTimeout>;
        const poll=async()=>{
            try {const next=await invokeNative<Status>("launcher_update_status");if(!stopped)setStatus(next);}
            catch(e){if(!stopped)setError(e instanceof Error?e.message:String(e));}
            if(!stopped)timer=setTimeout(()=>void poll(),750);
        };
        void poll();
        return()=>{stopped=true;alive.current=false;clearTimeout(timer);};
    },[]);
    const action=async(command:string)=>{
        if(!isDesktop()){setError("This operation requires the desktop app");return;}
        setPending(true);setError("");
        try{await invokeNative(command);const next=await invokeNative<Status>("launcher_update_status");if(alive.current)setStatus(next);}
        catch(e){if(alive.current)setError(e instanceof Error?e.message:String(e));}
        finally{if(alive.current)setPending(false);}
    };
    const busy=pending||["checking","downloading","restarting"].includes(status.phase);
    const messages:Record<string,string>={
        checking:t("Checking launcher updates…"), current:t("The launcher is up to date."),
        unpublished:t("No public stable release is available yet."),
        available:t("Launcher update available: {version}",{version:status.version??""}),
        downloading:t("Downloading launcher update: {percent}%",{percent:status.total?Math.floor(status.completed/status.total*100):0}),
        ready:t("Update verified. Restart to install; the previous executable will be retained."),
        restarting:t("Restarting the launcher…"),
    };
    return <div className="settings-group launcher-updater">
        <div className="java-control-row">
            <span>{t("Launcher version: {version}",{version})}</span>
            <StandardButton id="launcher-check-updates" tone="white" disabled={busy} onClick={()=>void action("check_launcher_update")}>{t("Check launcher updates")}</StandardButton>
            {status.version&&<StandardButton id="launcher-update-action" tone="green" disabled={busy} onClick={()=>void action(status.phase==="ready"?"apply_launcher_update":"download_launcher_update")}>{status.phase==="ready"?t("Restart and update"):t("Download launcher update")}</StandardButton>}
            {status.phase==="downloading"&&<StandardButton id="launcher-cancel-update" disabled={pending} onClick={()=>void action("cancel_launcher_update")}>{t("Cancel")}</StandardButton>}
        </div>
        {messages[status.phase]&&<p role="status">{messages[status.phase]}</p>}
        {(error||status.error)&&<p className="config-error" role="alert">{localizeMessage(error||status.error)}</p>}
    </div>;
}
