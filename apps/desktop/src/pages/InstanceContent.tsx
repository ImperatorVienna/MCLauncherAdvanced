import { useCallback, useContext, useEffect, useLayoutEffect, useRef, useState, type ComponentProps } from "react";
import { localizeMessage, t } from "../i18n";
import { pageLabel } from "../state/pages";
import { Navigation } from "../components/Control";
import { SelectMenu } from "../components/SelectMenu";
import { Checkbox } from "../components/Checkbox";
import { StandardButton } from "../components/StandardButton";
import type { ContentUpdate, UpdateResult } from "../state/contentSource";
import { invokeNative } from "../platform/desktop";
export const managementPages = ["instance-mods", "instance-resource-packs", "instance-shaders"];
type Entry = { name: string; enabled: boolean; conflict: boolean };
type Listing = { directory: string; entries: Entry[] };
/** Remount on scope changes so old requests and focus registrations cannot leak. */
export function InstanceContent(props: {title: string; instanceId?: string; selection: ComponentProps<typeof SelectMenu>; busy: boolean; preview: boolean; hasLoader: boolean; onBrowse?: () => void; onStarted?: () => void; refreshKey?: string}) {
    return <section className="instance-content" aria-label={pageLabel(props.title)}>
        <SelectMenu {...props.selection} id="content-instance" disabled={props.busy || !props.instanceId}/>
        <ContentFiles key={`${props.instanceId}:${props.title}`} {...props}/>
    </section>;
}
function ContentFiles({title,instanceId,busy,preview,hasLoader,onBrowse,onStarted,refreshKey}: Parameters<typeof InstanceContent>[0]) {
    const manager=useContext(Navigation);
    const deletedFocus=useRef<string | null>(null);
    const kind=title.replace("instance-", "");
    const [listing,setListing]=useState<Listing>({directory:"",entries:[]});
    const [labels,setLabels]=useState<Record<string,string>>({});
    useEffect(()=>{
        setLabels({});
        if(preview || !instanceId || busy || !listing.entries.length)return;
        let active=true;
        void invokeNative<{name:string;displayName:string}[]>("identify_instance_content",{instanceId,kind})
            .then(values=>{if(active && Array.isArray(values))setLabels(Object.fromEntries(values.map(v=>[v.name,v.displayName])));})
            .catch(()=>{}); // Offline/unrecognized files retain their usable local names.
        return()=>{active=false;};
    },[listing,instanceId,kind,preview,busy]);
    const [updates,setUpdates]=useState<ContentUpdate[]>([]);
    const updatesRef=useRef(updates);updatesRef.current=updates;
    const [updateStatus,setUpdateStatus]=useState<{updates:number;unmatched:number} | null>(null);
    const [pending,setPending]=useState(false);
    const [error,setError]=useState("");
    const mounted=useRef(false);
    const inFlight=useRef(false);
    const run=useCallback(async (command:string,args:Record<string,unknown>={}) => {
        if (!instanceId || preview || inFlight.current) return;
        inFlight.current=true;setPending(true);setError("");
        try {
            const result=await invokeNative<Listing | undefined>(command,{instanceId,kind,...args});
            if (mounted.current && result) {
                const candidates=updatesRef.current.filter(u=>result.entries.some(e=>e.name===u.name && !e.conflict));
                const retained=candidates.length ? await invokeNative<ContentUpdate[]>("validate_content_updates",{instanceId,kind,updates:candidates}) : [];
                if(!mounted.current)return;
                if(command==="delete_instance_content" && manager.activeId()===`content-delete-${encodeURIComponent(String(args.name))}`) {
                    deletedFocus.current=String(args.name);
                }
                const focusedUpdate=updatesRef.current.find(u=>manager.activeId()===`content-update-${encodeURIComponent(u.name)}`);
                if(focusedUpdate && !retained.some(u=>u.name===focusedUpdate.name))deletedFocus.current=focusedUpdate.name;
                setListing(result);
                setUpdates(retained);setUpdateStatus(previous=>previous ? {...previous,updates:retained.length} : null);
            }
        } catch (error) {if(mounted.current)setError(localizeMessage(error));}
        finally {inFlight.current=false;if(mounted.current)setPending(false);}
    },[instanceId,kind,preview,manager]);
    async function checkUpdates() {
        if(inFlight.current || busy || preview || !instanceId)return;
        inFlight.current=true;setPending(true);setError("");setUpdateStatus(null);setUpdates([]);
        try {
            const result=await invokeNative<UpdateResult>("check_content_updates",{instanceId,kind});
            if(mounted.current) {
                setUpdates(result.updates);
                setUpdateStatus({updates:result.updates.length,unmatched:result.unmatched});
            }
        } catch(e){if(mounted.current)setError(localizeMessage(e));}
        finally{inFlight.current=false;if(mounted.current)setPending(false);}
    }
    async function updateFile(update:ContentUpdate,enabled:boolean) {
        if(inFlight.current||busy||preview)return;
        inFlight.current=true;setPending(true);setError("");
        try {
            await invokeNative("start_content_installation",{instanceId,kind,project:update.project,update:{...update,enabled}});
            onStarted?.();
        } catch(e){if(mounted.current)setError(localizeMessage(e));}
        finally{inFlight.current=false;if(mounted.current)setPending(false);}
    }
    useEffect(()=> {
        mounted.current=true;
        void run("scan_instance_content");
        return ()=>{mounted.current=false;};
    },[run]);
    useLayoutEffect(()=>{
        if(deletedFocus.current===null || pending)return;
        const name=deletedFocus.current;
        deletedFocus.current=null;
        const next=listing.entries.find(entry=>entry.name.localeCompare(name)>=0) ?? listing.entries.at(-1);
        manager.focus(next ? `content-delete-${encodeURIComponent(next.name)}` : "content-refresh");
    },[listing,pending,manager]);
    // Rescan after game/file tasks and when returning from the external file manager.
    useEffect(()=> {
        const refresh=()=>{void run("scan_instance_content");};
        if(!busy)refresh();
        window.addEventListener("focus",refresh);
        return ()=>window.removeEventListener("focus",refresh);
    },[run,busy,refreshKey]);
    const locked=busy || pending || preview || !instanceId;
    return <>
        {instanceId && <>
            <div className="java-control-row">
                <StandardButton id="content-open" disabled={locked} onClick={()=>void run("open_instance_content_directory")}>{t("Open content folder")}</StandardButton>
                <StandardButton id="content-refresh" disabled={pending || preview} onClick={()=>void run("scan_instance_content")}>{t("Rescan files")}</StandardButton>
                <StandardButton id="content-check-updates" disabled={locked || (kind==="mods" && !hasLoader)} onClick={()=>void checkUpdates()}>{t("Check content updates")}</StandardButton>
                <StandardButton id="content-download" tone="green" disabled={!onBrowse} onClick={onBrowse}>{t(kind==="mods" ? "Download mods" : kind==="resource-packs" ? "Download resource packs" : "Download shader packs")}</StandardButton>
            </div>
            {listing.directory && <p className="content-directory">{listing.directory}</p>}
            {kind==="mods" && <p>{t("Checked files are available to the game. Unchecked files are kept separately in this instance and can be restored.")}</p>}
            {kind==="resource-packs" && <p>{t("Select resource packs and their order inside the game.")}</p>}
            {kind==="shaders" && <p>{t("Select the active shader pack and its settings inside the game.")}{" "}{t("Shader packs require a shader mod such as Iris; a mod loader alone is not sufficient.")}</p>}
            {kind==="mods" && !hasLoader && <p>{t("This instance has no mod loader. Mod files can be managed, but vanilla Minecraft will not load them.")}</p>}
            {busy && <p>{t("Content editing is unavailable while the game or a file task is running.")}</p>}
            {updateStatus && <p role="status">{t("Found {updates} updates; {unmatched} entries could not be checked.",updateStatus)}</p>}
            {error && <p role="alert" className="error">{error}</p>}
            {!preview && !pending && !error && listing.entries.length===0 && <p>{t("No content files found. Place files in this folder, then rescan.")}</p>}
            <div className="content-file-list" aria-busy={pending}>
                {listing.entries.map(entry=><div className="content-file-row" key={entry.name}>
                    <div className="content-file-actions">
                    {kind==="mods" ? <Checkbox title={entry.name} id={`content-file-${encodeURIComponent(entry.name)}`} checked={entry.enabled} disabled={locked || entry.conflict}
                        onClick={()=>void run("set_instance_content_enabled",{name:entry.name,enabled:!entry.enabled})}>{labels[entry.name] ?? entry.name}</Checkbox> : <p title={entry.name}>{labels[entry.name] ?? entry.name}</p>}
                    <div className="content-file-buttons">
                    {updates.filter(update=>update.name===entry.name).map(update=><StandardButton key={update.name} id={`content-update-${encodeURIComponent(entry.name)}`} tone="green" disabled={locked || entry.conflict} onClick={()=>void updateFile(update,entry.enabled)}>{t("Update content")}</StandardButton>)}
                    <StandardButton id={`content-delete-${encodeURIComponent(entry.name)}`} tone="red" disabled={locked || entry.conflict} aria-label={t("Delete {name}",{name:entry.name})} onClick={()=>void run("delete_instance_content",{name:entry.name,enabled:entry.enabled})}>{t("Delete")}</StandardButton>
                    </div>
                    </div>
                    {kind==="mods" && entry.conflict && <p role="alert">{t("Enabled and disabled folders contain the same name. Resolve the duplicate before changing it.")}</p>}
                </div>)}
            </div>
        </>}
    </>;
}
