import { useEffect, useState } from "react";
import { invokeNative, isDesktop } from "../platform/desktop";
const origin = "https://launchercontent.mojang.com";
export const maintainerHomepage = "https://github.com/ImperatorVienna";
export const homepage = "https://github.com/ImperatorVienna/MCLauncherAdvanced";
export type News = { id: string; title: string; date: string; image: string | null; url: string };
export type Patch = { title: string; version: string; body: string; date: string };
const cache = new Map<string, {until:number; promise:Promise<any>}>();
export function validContentPath(path:string) {
    return /^(news|javaPatchNotes)\.json$/.test(path) || /^javaPatchNotes\/[A-Za-z0-9_-]{1,160}\.json$/.test(path);
}
async function content(path:string):Promise<any> {
    if (!validContentPath(path)) throw Error("Invalid official content path");
    const old=cache.get(path); if(old && old.until>Date.now()) return old.promise;
    const promise=(async()=>{
        if(isDesktop()) return invokeNative("official_content",{path});
        const response=await fetch(`/official-content/${path}`, {signal:AbortSignal.timeout(20000)});
        if(!response.ok) throw Error("Official content is temporarily unavailable");
        const text=await response.text(); if(text.length>8*1024*1024) throw Error("Official content is too large");
        return JSON.parse(text);
    })();
    cache.set(path,{until:Date.now()+600000,promise});
    promise.catch(()=>{if(cache.get(path)?.promise===promise)cache.delete(path);});
    return promise;
}
export function informationLink(value:unknown):string|null {
    if(typeof value!=="string")return null;
    try {const u=new URL(value);return u.protocol==="https:" && !u.username && !u.password && !u.port &&
        (u.hostname==="www.minecraft.net" || (u.hostname==="github.com" && (u.pathname==="/ImperatorVienna" || u.pathname==="/ImperatorVienna/" || u.pathname==="/ImperatorVienna/MCLauncherAdvanced" || u.pathname.startsWith("/ImperatorVienna/MCLauncherAdvanced/")))) ? u.href : null;
    } catch {return null;}
}
export function imageLink(value:unknown):string|null {
    if(typeof value!=="string")return null;
    try {const u=new URL(value,origin);return u.origin===origin && u.pathname.startsWith("/v2/images/") && !u.username && !u.password ? u.href : null;}catch{return null;}
}
export async function openInformation(url:string) {
    if(!informationLink(url))throw Error("Unsupported information link");
    if(isDesktop())await invokeNative("open_information_link",{url});
    else window.open(url,"_blank","noopener,noreferrer");
}
export function parseNews(data:any):News[] {
    if(!Array.isArray(data?.entries))throw Error("Invalid official content response");
    const seen=new Set<string>();
    return data.entries.filter((e:any)=>Array.isArray(e.newsType) && e.newsType.includes("Java"))
        .flatMap((e:any)=>{
            const url=informationLink(e.readMoreLink);
            if(!url || typeof e.title!=="string" || !e.title.trim() || seen.has(url))return [];
            seen.add(url);return [{id:url,title:e.title.trim(),date:typeof e.date==="string" ? e.date : "",image:imageLink(e.playPageImage?.url),url}];
        }).sort((a:News,b:News)=>b.date.localeCompare(a.date)).slice(0,5);
}
export async function loadPatch(version:string):Promise<Patch|null> {
    const index=await content("javaPatchNotes.json");
    if(!Array.isArray(index?.entries))throw Error("Invalid official content response");
    const entry=index.entries.find((e:any)=>e.version===version);
    if(!entry)return null;
    const detail=typeof entry.body==="string" ? entry : await content(entry.contentPath);
    if(detail.version!==version || typeof detail.body!=="string" || typeof detail.title!=="string")throw Error("Invalid official content response");
    return {title:detail.title,version,body:detail.body,date:typeof detail.date==="string"?detail.date:""};
}
export const loadNews=async()=>parseNews(await content("news.json"));
// Key the displayed result as well as discarding late responses: no old instance flash.
export function useEditorial<T>(key:string,load:()=>Promise<T>) {
    const [attempt,setAttempt]=useState(0);
    const [state,setState]=useState<{key:string;data?:T;error:boolean;loading:boolean}>({key,error:false,loading:true});
    useEffect(()=>{let alive=true;setState({key,error:false,loading:true});
        load().then(data=>{if(alive)setState({key,data,error:false,loading:false});}).catch(()=>{if(alive)setState({key,error:true,loading:false});});
        return()=>{alive=false;};
    },[key,attempt]);
    return {...(state.key===key?state:{key,error:false,loading:true}),retry:()=>{setState({key,error:false,loading:true});setAttempt(n=>n+1);}};
}
