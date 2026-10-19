import { createElement, useContext, useLayoutEffect, useMemo, useState, type ReactNode } from "react";
import { t, localizeMessage } from "../i18n";
import { Control, Navigation } from "./Control";
import { StandardButton } from "./StandardButton";
import { informationLink, loadNews, loadPatch, openInformation, useEditorial } from "../state/editorial";
import "../styles/editorial.css";
// Build React nodes from an inert document. Never insert remote HTML or attributes.
export function ArticleBody({html,onOpen}:{html:string;onOpen:(url:string)=>void}) {
    const nodes=useMemo(()=>{
        const doc=new DOMParser().parseFromString(html,"text/html");
        function render(node:Node,key:string):ReactNode {
            if(node.nodeType===3)return node.textContent;
            if(!(node instanceof Element))return null;
            const tag=node.tagName.toLowerCase();
            if(["script","style","iframe","object","embed","form","input","button","svg","math","img","video","audio"].includes(tag))return null;
            const children=Array.from(node.childNodes).map((n,i)=>render(n,`${key}-${i}`));
            if(tag==="a") {
                const url=informationLink(node.getAttribute("href"));
                return url ? <Control key={key} id={`article-link-${key}`} className="article-link" onClick={()=>onOpen(url)}>{children}</Control> : <span key={key}>{children}</span>;
            }
            const safe=["p","ul","ol","li","strong","em","b","i","code","pre","blockquote","br","hr","table","thead","tbody","tr","td","th","h2","h3","h4","h5","h6","div","section"];
            return createElement(tag==="h1"?"h2":safe.includes(tag)?tag:"span",{key},...(["br","hr"].includes(tag)?[]:children));
        }
        return Array.from(doc.body.childNodes).map((n,i)=>render(n,String(i)));
    },[html,onOpen]);
    return <div className="official-article">{nodes}</div>;
}
function EditorialRetry({id, loading, failed, retry, destination}:{id:string;loading:boolean;failed:boolean;retry:()=>void;destination:string}) {
    const manager=useContext(Navigation);
    const [pending,setPending]=useState(false);
    useLayoutEffect(()=>{
        if(pending && !loading && !failed) {
            if(manager.activeId()===id) manager.focus(destination);
            setPending(false);
        }
    },[pending,loading,failed,manager,id,destination]);
    return failed || pending ? <StandardButton id={id} disabled={loading} onClick={()=>{setPending(true);retry();}}>{t("Retry")}</StandardButton> : null;
}
export function OfficialNews() {
    const feed=useEditorial("news",loadNews);
    const [error,setError]=useState("");
    const open=(url:string)=>{setError("");void openInformation(url).catch(e=>setError(localizeMessage(e)));};
    return <section className="news" aria-label={t("Official Minecraft news")}>
        {feed.loading ? <p role="status">{t("Loading official news…")}</p> : feed.error ? <div className="editorial-status"><p role="status">{t("Official news could not be loaded.")}</p></div> : !feed.data?.length ? <p>{t("No official news is available.")}</p> :
        <div className="cards">{feed.data.map((item,i)=><Control key={item.id} id={`news-${i}`} className="news-card" onClick={()=>open(item.url)}>
            <span className="news-art">{item.image && <img src={item.image} alt="" loading="lazy" referrerPolicy="no-referrer" onError={e=>{e.currentTarget.hidden=true;}}/>}</span>
            <span className="card-copy"><small>{t("Minecraft.net")} · {item.date}</small><strong>{item.title}</strong></span>
        </Control>)}</div>}
        <EditorialRetry id="news-retry" loading={feed.loading} failed={feed.error} retry={feed.retry} destination={feed.data?.length?"news-0":"instance-select"}/>
        {error&&<p className="config-error" role="alert">{error}</p>}
    </section>;
}
export function GamePatchNotes({version}:{version?:string}) {
    const feed=useEditorial(`patch:${version??""}`,()=>version?loadPatch(version):Promise.resolve(null));
    const [error,setError]=useState("");
    const open=(url:string)=>{setError("");void openInformation(url).catch(e=>setError(localizeMessage(e)));};
    return <section className="placeholder settings-page" aria-label={t("Patch notes")}>
        <div className="settings-group"><span>{t("Game version: {version}",{version:version??t("No instance selected")})}</span>
        <div className="patch-source-row">
        <p className="settings-description">{t("Source: Mojang. Official articles are shown in their original language.")}</p>
        <StandardButton className="patch-source-button" id="patch-official-site" onClick={()=>open("https://www.minecraft.net/en-us/articles")}>{t("Visit Minecraft.net")}</StandardButton>
        </div></div>
        {!version ? <p>{t("Select an instance to view its game patch notes.")}</p> : feed.loading ? <p role="status">{t("Loading game patch notes…")}</p> : feed.error ? <div className="settings-group"><p role="status">{t("Game patch notes could not be loaded.")}</p></div> : feed.data ? <article className="patch-article"><h2>{feed.data.title}</h2>{feed.data.date&&<p>{feed.data.date.slice(0,10)}</p>}<ArticleBody html={feed.data.body} onOpen={open}/></article> : <p>{t("No official patch notes were found for this game version.")}</p>}
        <EditorialRetry id="patch-retry" loading={feed.loading} failed={feed.error} retry={feed.retry} destination="patch-official-site"/>
        {error&&<p className="config-error" role="alert">{error}</p>}
    </section>;
}
