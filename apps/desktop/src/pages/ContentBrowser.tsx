import { useContext, useEffect, useRef, useState, type ComponentProps } from "react";
import { Navigation } from "../components/Control";
import { SelectMenu } from "../components/SelectMenu";
import { StandardButton } from "../components/StandardButton";
import { Field } from "../components/Field";
import { localizeMessage, t } from "../i18n";
import { loaderName } from "../state/loaders";
import type { Instance } from "../state/library";
import { searchContent, type ContentKind, type SearchResult, type CatalogState } from "../state/contentSource";
import { invokeNative } from "../platform/desktop";
type Props={initialQuery?:string;kind:ContentKind;item?:Instance;selection:ComponentProps<typeof SelectMenu>;busy:boolean;preview:boolean;onStarted:()=>void};
export function ContentBrowser(props:Props) {
    return <section className="instance-content">
        <SelectMenu {...props.selection} id="browse-instance" disabled={props.busy || !props.item}/>
        {props.item ? <BrowserResults key={`${props.kind}:${props.item.id}:${props.item.minecraftVersion}:${props.item.loader?.kind}`} {...props} item={props.item}/> : <p>{t("No instances yet. Choose Download and install.")}</p>}
    </section>;
}
function BrowserResults({kind,item,busy,preview,onStarted,initialQuery=""}:Props & {item:Instance}) {
    const manager=useContext(Navigation);
    const pageFocus=useRef<{direction:"first"|"last";owner:string} | null>(null);
    function turnPage(direction:"first"|"last") {
        pageFocus.current={direction,owner:direction==="first" ? "content-next" : "content-previous"};
        setSubmitted(s=>({...s,offset:Math.max(0,s.offset+(direction==="first" ? 20 : -20))}));
    }
    const [query,setQuery]=useState(initialQuery);
    const [submitted,setSubmitted]=useState({query:initialQuery,offset:0});
    const [result,setResult]=useState<SearchResult>({projects:[],more:false});
    const [loading,setLoading]=useState(true);
    const [starting,setStarting]=useState(false);
    const [error,setError]=useState("");
    const alive=useRef(true), lock=useRef(false);
    useEffect(()=>{alive.current=true;return()=>{alive.current=false;};},[]);
    useEffect(()=>{
        const abort=new AbortController();
        setLoading(true);setError("");setResult({projects:[],more:false});
        void searchContent(item,kind,submitted.query,submitted.offset,preview,abort.signal).then(value=>{
            if(!abort.signal.aborted)setResult(value);
        }).catch(e=>{if(!abort.signal.aborted)setError(localizeMessage(e));}).finally(()=>{if(!abort.signal.aborted)setLoading(false);});
        return()=>abort.abort();
    },[item.id,item.minecraftVersion,item.loader?.kind,kind,submitted,preview]);
    const [catalog,setCatalog]=useState<{result:SearchResult;states:CatalogState[]} | null>(null);
    const [statusError,setStatusError]=useState("");
    const [refresh,setRefresh]=useState(0);
    const ready=preview || catalog?.result===result;
    useEffect(()=>{
        const refresh=()=>setRefresh(value=>value+1);
        window.addEventListener("focus",refresh);
        return()=>window.removeEventListener("focus",refresh);
    },[]);
    useEffect(()=>{
        if(preview || loading || busy)return;
        let active=true;
        setCatalog(null);setStatusError("");
        if(!result.projects.length)return;
        void invokeNative<CatalogState[]>("catalog_content_states",{instanceId:item.id,kind,projects:result.projects.map(p=>p.id)})
            .then(states=>{if(active)setCatalog({result,states});})
            .catch(error=>{if(active)setStatusError(localizeMessage(error));});
        return()=>{active=false;};
    },[preview,loading,busy,result,item.id,kind,refresh]);
    const stateFor=(project:string)=>catalog?.result===result ? catalog.states.find(s=>s.project===project) : undefined;
    useEffect(()=>{
        if(loading || (!ready && !statusError && result.projects.length>0) || !pageFocus.current)return;
        const pending=pageFocus.current;
        pageFocus.current=null;
        // A completed request must not steal focus after the user has moved away.
        if(manager.activeId()!==pending.owner)return;
        if(!error && !busy && !starting && result.projects.length) {
            const available=result.projects.filter(p=>{const state=stateFor(p.id);return preview || (ready && state && (!state.installed || state.update));});
            const project=pending.direction==="first" ? available[0] : available.at(-1);
            if(project)manager.focus(`download-${project.id}`);
            else manager.focus("content-query");
        }
    },[loading,result,error,busy,starting,manager,ready,catalog,statusError]);
    async function install(project:string) {
        if(lock.current||busy||!ready)return;
        const state=stateFor(project);
        if(!preview && (!state || (state.installed && !state.update)))return;
        if(preview){setError(t("Catalog preview is live; install files in the desktop launcher."));return;}
        lock.current=true;setStarting(true);setError("");
        try {await invokeNative("start_content_installation",{instanceId:item.id,kind,project,update:state?.update ?? null});onStarted();}
        catch(e){if(alive.current)setError(localizeMessage(e));}
        finally{lock.current=false;if(alive.current)setStarting(false);}
    }
    return <>
        <p>{t("Game version: {version} · Loader: {loader} · Source: Modrinth",{version:item.minecraftVersion,loader:item.loader ? loaderName(item.loader.kind) : t("Vanilla")})}</p>
        <div className="java-control-row content-search">
            <Field id="content-query" label={t("Search content")} value={query} onChange={setQuery} disabled={starting} scope="main"/>
            <StandardButton id="content-search" disabled={loading || starting} onClick={()=>setSubmitted({query,offset:0})}>{t("Search")}</StandardButton>
        </div>
        {loading && <p role="status">{t("Loading content list")}</p>}
        {(error || statusError) && <p className="config-error" role="alert">{error || statusError}</p>}
        {!loading && !error && result.projects.length===0 && <p>{t("No compatible content found")}</p>}
        <div className="content-catalog">
            {result.projects.map(project=><article key={project.id} className="content-project">
                <div><strong>{project.title}</strong><p>{project.description}</p></div>
                <StandardButton id={`download-${project.id}`} tone="green" disabled={busy||starting||!ready||(!preview && (!stateFor(project.id) || (stateFor(project.id)?.installed && !stateFor(project.id)?.update)))} onClick={()=>void install(project.id)}>{t(!ready ? (statusError ? "Installed content check failed" : "Checking installed content") : stateFor(project.id)?.update ? "Update content" : stateFor(project.id)?.installed ? "Installed" : "Download and install content")}</StandardButton>
            </article>)}
        </div>
        {(result.projects.length>0 || submitted.offset>0 || pageFocus.current!==null) && <div className="java-control-row">
            <StandardButton id="content-previous" disabled={loading||submitted.offset===0} onClick={()=>turnPage("last")}>{t("Previous page")}</StandardButton>
            <StandardButton id="content-next" disabled={loading||!result.more} onClick={()=>turnPage("first")}>{t("Next page")}</StandardButton>
        </div>}
        {preview && <p>{t("Catalog preview is live; install files in the desktop launcher.")}</p>}
    </>;
}
