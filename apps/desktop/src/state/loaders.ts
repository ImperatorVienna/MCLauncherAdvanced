import { useEffect, useState } from "react";
import { invokeNative, isDesktop } from "../platform/desktop";
import { localizeMessage } from "../i18n";
export const loaderKinds = ["fabric", "quilt", "forge", "neoforge"] as const;
export type LoaderKind = typeof loaderKinds[number];
export type LoaderVersion = { version: string; stable: boolean };
export const loaderName = (kind: LoaderKind) => ({fabric:"Fabric", quilt:"Quilt", forge:"Forge", neoforge:"NeoForge"})[kind];
export async function fetchLoaderVersions(kind: LoaderKind, game: string, signal?: AbortSignal): Promise<LoaderVersion[]> {
    if (isDesktop()) return invokeNative("list_loader_versions",{kind,game});
    if (kind === "forge" || kind === "neoforge") {
        const base = kind === "forge" ? "https://maven.minecraftforge.net/net/minecraftforge/forge" : "https://maven.neoforged.net/releases/net/neoforged/neoforge";
        const response = await fetch(base + "/maven-metadata.xml", {signal});
        if (!response.ok) throw Error(`HTTP ${response.status}`);
        return parseInstallerCatalog(kind, game, await response.text());
    }
    const base = kind === "fabric" ? "https://meta.fabricmc.net/v2" : "https://meta.quiltmc.org/v3";
    const get = async (path: string) => {
        const response = await fetch(base+path,{signal});
        if (!response.ok) throw Error(`HTTP ${response.status}`);
        return response.json();
    };
    const games = await get("/versions/game");
    if (!Array.isArray(games)) throw Error("Invalid mod loader catalog");
    if (!games.some(v => v.version === game)) return [];
    const data = await get(`/versions/loader/${encodeURIComponent(game)}`);
    if (!Array.isArray(data)) throw Error("Invalid mod loader catalog");
    const versions = data.map(({loader}) => {
        if (typeof loader?.version !== "string" || !/^[a-zA-Z0-9.+-]{1,80}$/.test(loader.version)) throw Error("Invalid mod loader catalog");
        return {version:loader.version,stable: typeof loader.stable === "boolean" ? loader.stable : !loader.version.includes("-")};
    });
    return sortLoaderVersions(versions);
}
function sortLoaderVersions(versions: LoaderVersion[]) {
    return versions.sort((a,b) => {
        const [ac,ap] = a.version.split("-",2), [bc,bp] = b.version.split("-",2);
        return bc.localeCompare(ac,"en",{numeric:true}) || Number(!!ap)-Number(!!bp) || (bp ?? "").localeCompare(ap ?? "","en",{numeric:true});
    }).filter((v,i,all) => !i || v.version !== all[i-1].version);
}
export function parseInstallerCatalog(kind: "forge" | "neoforge", game: string, xml: string): LoaderVersion[] {
    const gameParts = game.split(".").map(Number);
    if (!/^\d+\.\d+(?:\.\d+)?$/.test(game) || !(gameParts[0] === 1 && gameParts[1] >= 13 || gameParts[0] >= 26)) return [];
    if (!xml.includes("<metadata") || !xml.includes("</metadata>")) throw Error("Invalid mod loader catalog");
    const versions: LoaderVersion[] = [];
    for (const match of xml.matchAll(/<version>([^<]+)<\/version>/g)) {
        const value = match[1].trim();
        let version: string | undefined;
        if (kind === "forge") {
            if (value.startsWith(game + "-")) version = value.slice(game.length + 1);
        } else if (!value.includes("+")) {
            const parts = value.split("-")[0].split(".").map(Number);
            const [major, minor, patch] = parts;
            const target = parts.every(Number.isInteger) && parts.length === 3 && major >= 20 && major < 26 ? `1.${major}${minor ? "." + minor : ""}`
                : parts.every(Number.isInteger) && parts.length === 4 && major >= 26 ? `${major}.${minor}${patch ? "." + patch : ""}` : "";
            if (target === game) version = value;
        }
        if (version !== undefined) {
            if (!/^[a-zA-Z0-9.+-]{1,80}$/.test(version)) throw Error("Invalid mod loader catalog");
            versions.push({version,stable:!version.includes("-")});
        }
    }
    return sortLoaderVersions(versions);
}

export function useLoaderVersions(kind: LoaderKind | undefined, game: string, retry: number) {
    const key = `${kind}/${game}/${retry}`;
    const [state,setState] = useState<{key:string; versions:LoaderVersion[]; error:string}>({key:"",versions:[],error:""});
    useEffect(() => {
        if (!kind || !game) return;
        let live=true; const controller = new AbortController();
        void fetchLoaderVersions(kind,game,controller.signal).then(versions => {
            if (live) setState({key,versions,error:""});
        }).catch(e => { if (live) setState({key,versions:[],error:localizeMessage(e)}); });
        return () => { live=false; controller.abort(); };
    },[kind,game,retry]);
    return state.key === key ? {...state,loading:false} : {versions:[],error:"",loading:!!kind && !!game};
}
