import type { Instance } from "./library";
import { invokeNative } from "../platform/desktop";
export const contentSources = ["Modrinth"];
export type ContentKind = "mods" | "resource-packs" | "shaders";
export type Project = { id:string; title:string; description:string };
export type SearchResult = { projects:Project[]; more:boolean };
export type ContentUpdate = { name:string; enabled:boolean; hash:string; project:string; version:string };
export type CatalogState = { project:string; installed:boolean; update:ContentUpdate | null };
export type UpdateResult = { updates:ContentUpdate[]; unmatched:number };
export function searchFacets(item:Instance,kind:ContentKind) {
    const facets = [[`project_type:${kind==="mods" ? "mod" : kind==="resource-packs" ? "resourcepack" : "shader"}`],[`versions:${item.minecraftVersion}`]];
    if(kind==="mods" && item.loader) facets.push([`categories:${item.loader.kind}`]);
    return facets;
}
// Browser preview can read the public catalog but never installs local files.
// Desktop requests always re-resolve version/loader from the stored instance.
export async function searchContent(item:Instance,kind:ContentKind,query:string,offset:number,preview:boolean,signal:AbortSignal):Promise<SearchResult> {
    if(!preview)return invokeNative("search_instance_content",{instanceId:item.id,kind,query,offset});
    if(kind==="mods" && !item.loader)throw new Error("Select an instance with a mod loader to browse mods");
    const get=async (path:string)=>{
        const response=await fetch(`https://api.modrinth.com/v2${path}`,{signal});
        if(!response.ok)throw new Error(`Content provider returned HTTP ${response.status}`);
        return response.json();
    };
    const versions=await get("/tag/game_version") as {version:string}[];
    if(!versions.some(v=>v.version===item.minecraftVersion))throw new Error("This source does not support this game version; install files manually");
    const params=new URLSearchParams({query,facets:JSON.stringify(searchFacets(item,kind)),offset:String(offset),limit:"20"});
    const data=await get(`/search?${params}`) as {hits:{project_id:string;title:string;description:string}[];total_hits:number};
    return {projects:data.hits.map(p=>({id:p.project_id,title:p.title,description:p.description})),more:offset+data.hits.length<data.total_hits};
}
