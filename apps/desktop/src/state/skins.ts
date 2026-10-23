import { useEffect, useRef, useState } from "react";
import { invokeNative, isDesktop } from "../platform/desktop";
import { localizeMessage, t } from "../i18n";
export type SkinModel = "default" | "slim";
export type Skin = { id: string; name: string; model: SkinModel; png: string };
export type DisplaySkin = Skin & { url: string; builtin?: boolean };
export type SkinLibrary = { revision: number; skins: Skin[]; assignments: Record<string, string>; builtinModels?: Record<string, SkinModel> };
const assets = import.meta.glob("../assets/skins/*.png", { eager: true, query: "?url", import: "default" }) as Record<string, string>;
export const builtinSkins: DisplaySkin[] = ["steve", "alex", "zuri", "sunny", "noor", "makena", "kai", "efe", "ari"].map(name => ({
    id: `builtin-${name}`, name: name[0].toUpperCase() + name.slice(1), model: name === "alex" ? "slim" : "default", png: "", url: assets[`../assets/skins/${name}.png`], builtin: true,
}));
export const emptySkins = (): SkinLibrary => ({ revision: 0, skins: [], assignments: {} });
export function validateSkins(data: SkinLibrary) {
    if (!data || !Number.isSafeInteger(data.revision) || data.revision < 0 || !Array.isArray(data.skins) || data.skins.length > 100 || !data.assignments || typeof data.assignments !== "object" || Array.isArray(data.assignments) || Object.keys(data.assignments).length > 1000) throw Error(t("Invalid skin library"));
    if (data.builtinModels !== undefined && (!data.builtinModels || typeof data.builtinModels !== "object" || Array.isArray(data.builtinModels) || Object.entries(data.builtinModels).some(([id,model]) => !builtinSkins.some(s => s.id === id) || !["default","slim"].includes(model)))) throw Error(t("Invalid skin library"));
    const ids = new Set(builtinSkins.map(s => s.id));
    for (const s of data.skins) {
        if (!s || typeof s.id !== "string" || !/^[a-f0-9-]{36}$/i.test(s.id) || ids.has(s.id) || typeof s.name !== "string" || !s.name.trim() || s.name !== s.name.trim() || [...s.name].length > 64 || /[\u0000-\u001f\u007f-\u009f]/.test(s.name) || !["default", "slim"].includes(s.model) || typeof s.png !== "string" || s.png.length > 174768 || !/^[A-Za-z0-9+/]+={0,2}$/.test(s.png)) throw Error(t("Invalid skin library"));
        ids.add(s.id);
    }
    for (const [name, id] of Object.entries(data.assignments)) if (!/^[a-z0-9_]{3,16}$/i.test(name) || !ids.has(id)) throw Error(t("Invalid skin library"));
}
export function removeSkin(data: SkinLibrary, id: string): SkinLibrary {
    return { ...data, skins: data.skins.filter(s => s.id !== id), assignments: Object.fromEntries(Object.entries(data.assignments).filter(([,skin]) => skin !== id)) };
}
export function setSkinModel(data: SkinLibrary, id: string, model: SkinModel): SkinLibrary {
    return builtinSkins.some(s => s.id === id)
        ? {...data,builtinModels:{...data.builtinModels,[id]:model}}
        : {...data,skins:data.skins.map(s => s.id === id ? {...s,model} : s)};
}
export function displaySkins(data: SkinLibrary): DisplaySkin[] {
    return [...data.skins.map(s => ({...s,url:`data:image/png;base64,${s.png}`})), ...builtinSkins.map(s => {
        const model = data.builtinModels?.[s.id] ?? s.model;
        return {...s,model,url:model === s.model ? s.url : assets[`../assets/skins/${s.id.slice(8)}-alternate.png`]};
    })];
}
export function useSkins(selectedAccount: string | null = null, accountRevision = 0) {
    const generation = useRef(0);
    const uploadLock = useRef(false);
    const [uploading, setUploading] = useState(false), [uploadError, setUploadError] = useState(""), [applied, setApplied] = useState(false);
    const [official, setOfficial] = useState<{ account: string; skin: DisplaySkin | null; error: string } | null>(null);
    useEffect(() => {
        let live = true;
        const request = ++generation.current;
        setUploadError(""); setApplied(false);
        setOfficial(null);
        if (selectedAccount && isDesktop()) {
            invokeNative<Skin | null>("load_account_skin", { selected: selectedAccount }).then(skin => {
                if (live && request === generation.current) setOfficial({ account: selectedAccount, skin: skin ? {...skin, url: `data:image/png;base64,${skin.png}`, builtin: true} : null, error: "" });
            }).catch(e => { if (live && request === generation.current) setOfficial({ account: selectedAccount, skin: null, error: localizeMessage(e) }); });
        }
        return () => { live = false; generation.current++; };
    }, [selectedAccount, accountRevision]);
    const accountSkin = official?.account === selectedAccount ? official.skin : null;
    const accountSkinError = official?.account === selectedAccount ? official.error : "";
    const [data, setData] = useState(emptySkins);
    const [ready, setReady] = useState(false), [busy, setBusy] = useState(false), [error, setError] = useState("");
    const lock = useRef(false); const native = isDesktop(); const key = "launcher.local-skins.v1";
    useEffect(() => {
        let live = true;
        Promise.resolve().then(() => native ? invokeNative<SkinLibrary>("load_skins") : JSON.parse(localStorage.getItem(key) ?? JSON.stringify(emptySkins()))).then(next => {
            validateSkins(next); if (live) { setData(next); setReady(true); }
        }).catch(e => { if (live) setError(localizeMessage(e)); });
        return () => { live = false; };
    }, [native]);
    async function commit(next: SkinLibrary) {
        if (!ready || lock.current) return false;
        lock.current = true; setBusy(true); setError("");
        try {
            validateSkins(next);
            let saved: SkinLibrary;
            if (native) saved = await invokeNative<SkinLibrary>("save_skins", { library: next });
            else {
                const old = JSON.parse(localStorage.getItem(key) ?? JSON.stringify(emptySkins())); validateSkins(old);
                if (old.revision !== next.revision) throw Error(t("Skin library changed; reload and retry"));
                saved = { ...next, revision: next.revision + 1 }; localStorage.setItem(key, JSON.stringify(saved));
            }
            setData(saved); return true;
        } catch (e) { setError(localizeMessage(e)); return false; }
        finally { lock.current = false; setBusy(false); }
    }
    async function apply(skin: DisplaySkin) {
        if (!selectedAccount || !native || uploadLock.current) return false;
        uploadLock.current = true; setUploading(true); setUploadError(""); setApplied(false);
        const request = ++generation.current;
        try {
            // Bundled skins use asset URLs; imported and official skins already contain PNG bytes.
            const image = skin.png || await importSkin(new File([await (await fetch(skin.url)).blob()], "skin.png", {type:"image/png"}));
            if (request !== generation.current) return false;
            const saved = await invokeNative<Skin>("upload_account_skin", { selected: selectedAccount, model: skin.model, image });
            if (request !== generation.current) return false;
            setOfficial({account: selectedAccount, skin: {...saved, url:`data:image/png;base64,${saved.png}`, builtin:true}, error:""});
            setApplied(true); return true;
        } catch(e) { if (request === generation.current) setUploadError(localizeMessage(e)); return false; }
        finally { uploadLock.current = false; setUploading(false); }
    }
    const all = [...(accountSkin ? [accountSkin] : []), ...displaySkins(data)];
    const current = () => accountSkin ?? all.find(s => s.id === builtinSkins[0].id)!;
    return { data, all, current, accountSkin, commit, apply, applied, canApply: native && !!selectedAccount, error: uploadError || error || accountSkinError, busy: busy || !ready || uploading };
}
export async function importSkin(file: File): Promise<string> {
    if (file.size > 128 * 1024) throw Error(t("Skin PNG exceeds 128 KiB"));
    const bytes = new Uint8Array(await file.arrayBuffer());
    if (bytes.length < 24 || ![137,80,78,71,13,10,26,10].every((b,i) => bytes[i] === b)) throw Error(t("Invalid skin PNG"));
    const view = new DataView(bytes.buffer);
    if (view.getUint32(16) !== 64 || ![32,64].includes(view.getUint32(20))) throw Error(t("Skin must be a static 64×64 or 64×32 PNG"));
    const source = `data:image/png;base64,${btoa(Array.from(bytes, b => String.fromCharCode(b)).join(""))}`;
    const img = new Image(); img.src = source;
    try { await img.decode(); } catch { throw Error(t("Invalid skin PNG")); }
    // Normalize the decoded image to static PNG; strips ancillary metadata and animation.
    const canvas = document.createElement("canvas"); canvas.width = img.width; canvas.height = img.height;
    const ctx = canvas.getContext("2d"); if (!ctx) throw Error(t("Skin image preview is unavailable"));
    ctx.drawImage(img,0,0); return canvas.toDataURL("image/png").split(",")[1];
}
