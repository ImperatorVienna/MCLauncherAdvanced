import { useEffect, useState } from "react";
import { localizeMessage, t } from "../i18n";
import { isDesktop } from "../platform/desktop";
import { copyLogText, listLogs, openLogLocation, readLog, type LogContent, type LogEntry, type LogRequest } from "../state/logs";
import { DialogText } from "./DialogText";
import { SelectMenu } from "./SelectMenu";
import { StandardButton } from "./StandardButton";

export function LogDialog({request,onClose}: {request:LogRequest;onClose:()=>void}) {
    const [entries,setEntries]=useState<LogEntry[]>([]), [selected,setSelected]=useState("");
    const [refresh,setRefresh]=useState(0), [listing,setListing]=useState(true), [reading,setReading]=useState(false);
    const [data,setData]=useState<LogContent>(), [error,setError]=useState(""), [copied,setCopied]=useState(false);
    useEffect(()=>{
        let live=true;setListing(true);setError("");
        void listLogs(request).then(items=>{
            if(!live)return;
            setEntries(items);
            setSelected(old=>items.some(i=>i.name===old) ? old : items.find(i=>i.name===request.initialName)?.name ?? items[0]?.name ?? "");
        }).catch(e=>{if(live){setError(localizeMessage(e));setEntries([]);setSelected("");}})
            .finally(()=>{if(live)setListing(false);});
        return()=>{live=false};
    },[request,refresh]);
    useEffect(()=>{
        let live=true;setData(undefined);setCopied(false);
        if(!selected){setReading(false);return}
        setReading(true);setError("");
        void readLog(request,selected).then(value=>{if(live)setData(value);})
            .catch(e=>{if(live)setError(localizeMessage(e));})
            .finally(()=>{if(live)setReading(false);});
        return()=>{live=false};
    },[request,selected,refresh]);
    async function open() {try{await openLogLocation(request.instanceId,selected);}catch(e){setError(localizeMessage(e));}}
    async function copy() {try{await copyLogText(data?.text ?? "");setCopied(true);}catch(e){setError(localizeMessage(e));}}
    return <div className="log-viewer">
        <h2 id="dialog-title">{request.instanceId ? t("Instance logs: {name}",{name:request.label ?? request.instanceId}) : t("Launcher logs")}</h2>
        <div className="java-control-row">
            <SelectMenu id="modal-first" scope="modal" compact showIcons={false} label={t("Log file")} options={entries.map(e=>e.name)} value={selected} disabled={listing || !entries.length}
                labelForValue={name=>{const entry=entries.find(e=>e.name===name);return entry ? `${name} · ${new Date(entry.modified).toLocaleString("zh-CN")}` : t("No logs yet");}} onChange={setSelected}/>
            <StandardButton id="log-refresh" scope="modal" disabled={listing || reading} onClick={()=>setRefresh(n=>n+1)}>{t("Refresh logs")}</StandardButton>
            <StandardButton id="log-open" scope="modal" tone="yellow" disabled={!selected || !isDesktop()} onClick={()=>void open()}>{t("Open log file")}</StandardButton>
        </div>
        {!listing && !entries.length && <p>{t("No logs yet")}</p>}
        {(listing || reading) && <p role="status">{t("Loading logs…")}</p>}
        {data?.truncated && <p>{t("Only part of this large log is displayed. Open the original file to read it in full.")}</p>}
        <DialogText id="log-content" label={t("Log contents")} text={data?.text ?? ""}/>
        {error && <p className="config-error" role="alert">{error}</p>}
        <div className="config-actions">
            <StandardButton id="log-copy" scope="modal" disabled={!data?.text || reading || listing} onClick={()=>void copy()}>{t(copied ? "Copied" : "Copy log contents")}</StandardButton>
            <StandardButton id="log-close" scope="modal" onClick={onClose}>{t("Close")}</StandardButton>
        </div>
    </div>;
}
