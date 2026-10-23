import { useContext, useLayoutEffect, useRef, useState } from "react";
import { t, localizeMessage } from "../i18n";
import { useSkins, setSkinModel, removeSkin, importSkin, builtinSkins, type Skin, type DisplaySkin, type SkinModel } from "../state/skins";
import { Control, Navigation } from "../components/Control";
import { StandardButton } from "../components/StandardButton";
import { SelectMenu } from "../components/SelectMenu";
import { Field } from "../components/Field";
import { SkinPreview } from "../components/SkinPreview";
import { SkinPortrait } from "../components/SkinPortrait";
export function SkinsPage({ library }: { library: ReturnType<typeof useSkins> }) {
    const manager = useContext(Navigation);
    const current = library.current();
    const [selectedId,setSelectedId] = useState<string | null>(null);
    const [officialModel, setOfficialModel] = useState<SkinModel | null>(null);
    const all = library.all.map(s => s.id.startsWith("@") && officialModel ? {...s, model: officialModel} : s);
    const selected = all.find(s => s.id === (selectedId ?? current.id)) ?? current;
    const [draft,setDraft] = useState<Skin | null>(null), [deleting,setDeleting] = useState(false), [reading,setReading] = useState(false), [error,setError] = useState("");
    const file = useRef<HTMLInputElement>(null), importing = useRef(false), alive = useRef(true);
    const returnFocus = useRef("skin-add");
    const restoreFocus = (id: string) => requestAnimationFrame(() => { if (alive.current) manager.focus(id); });
    const busy = library.busy || reading;
    const editing = !!draft || deleting;
    useLayoutEffect(() => { alive.current = true; return () => { alive.current = false; }; }, []);
    const cancel = () => { if (!busy) { setDraft(null); setDeleting(false); setError(""); restoreFocus(returnFocus.current); } return true; };
    useLayoutEffect(() => {
        if (!editing) return;
        manager.cancelEditor = cancel;
        return () => { if (manager.cancelEditor === cancel) manager.cancelEditor = undefined; };
    }, [editing,busy,manager]);
    useLayoutEffect(() => { if (draft) manager.focus("skin-name"); else if (deleting) manager.focus("skin-cancel"); }, [!!draft,deleting,manager]);
    const shown: DisplaySkin = draft?.png ? { ...draft, url:`data:image/png;base64,${draft.png}` } : selected;
    const validName = !!draft && draft.name.trim().length > 0 && [...draft.name.trim()].length <= 64 && !/[\u0000-\u001f\u007f-\u009f]/.test(draft.name);
    async function readFile(input: HTMLInputElement) {
        const picked = input.files?.[0]; input.value = "";
        if (!picked || importing.current) return;
        importing.current = true; setReading(true); setError("");
        try {
            const png = await importSkin(picked);
            if (alive.current) setDraft(old => old ? { ...old,png } : {id:crypto.randomUUID(),name:picked.name.replace(/\.png$/i,"").slice(0,64),model:"default",png});
        } catch (e) { if (alive.current) setError(localizeMessage(e)); }
        finally { importing.current = false; if (alive.current) setReading(false); }
    }
    async function save() {
        if (!draft || !validName || !draft.png) return;
        const item = {...draft,name:draft.name.trim()};
        const exists = library.data.skins.some(s => s.id === item.id);
        if (await library.commit({...library.data,skins:exists ? library.data.skins.map(s => s.id === item.id ? item : s) : [...library.data.skins,item]}) && alive.current) {
            setSelectedId(item.id); setDraft(null); restoreFocus(`skin-card-${item.id}`);
        }
    }
    async function erase() {
        if (selected.builtin) return;
        if (await library.commit(removeSkin(library.data,selected.id)) && alive.current) { setDeleting(false); setSelectedId(builtinSkins[0].id); restoreFocus("skin-add"); }
    }
    return <section className="skins-page" aria-label={t("Skin library")}>
        <div className="skins-layout">
            <section className="skin-current" aria-label={t("Skin preview")}>
                <h2>{t("Skin preview")}</h2>
                <SkinPreview skin={shown}/>
                <p>{shown.name}</p>
                <p className="skin-notice">{t("Choose an arm model, then use the skin to update your Minecraft account.")}</p>
                {!editing && <div className="skin-actions">
                    <StandardButton id="skin-use" tone="green" disabled={busy || !library.canApply} onClick={() => { void library.apply(selected).then(ok => { if (ok && alive.current) setOfficialModel(null); }); }}>{t("Use skin")}</StandardButton>
                    <StandardButton id="skin-edit" disabled={busy || selected.builtin} onClick={() => { returnFocus.current="skin-edit"; setError(""); setDraft({id:selected.id,name:selected.name,model:selected.model,png:selected.png}); }}>{t("Edit")}</StandardButton>
                    <StandardButton id="skin-delete" tone="red" disabled={busy || selected.builtin} onClick={() => { returnFocus.current="skin-delete"; setDeleting(true); }}>{t("Delete")}</StandardButton>
                </div>}
            </section>
            <section className="skin-library" aria-label={t("Skin library")}>
                <h2>{t("Skin library")}</h2>
                <div className="skin-grid">
                    <div className="skin-library-item"><Control id="skin-add" className="skin-card skin-add" disabled={busy || editing || library.data.skins.length>=100} onClick={() => { returnFocus.current="skin-add"; setError(""); setDraft({id:crypto.randomUUID(),name:"",model:"default",png:""}); }}><span aria-hidden="true">＋</span>{t("Add skin")}</Control><span className="standard-button skin-button-space" aria-hidden="true">{t("Classic arms")}</span></div>
                    {all.map(skin => <div key={skin.id} className="skin-library-item"><Control id={`skin-card-${skin.id}`} className="skin-card" aria-pressed={selected.id===skin.id} disabled={busy || editing} onClick={() => setSelectedId(skin.id)}><span>{skin.name}</span><SkinPortrait skin={skin}/></Control><StandardButton id={`skin-model-${skin.id}`} disabled={busy || editing} aria-label={`${skin.name} · ${t("Skin model")}`} onClick={() => { const model = skin.model === "slim" ? "default" : "slim"; if (skin.id.startsWith("@")) setOfficialModel(model); else void library.commit(setSkinModel(library.data,skin.id,model)); }}>{t(skin.model === "slim" ? "Slim arms" : "Classic arms")}</StandardButton></div>)}
                </div>
            </section>
        </div>
        {editing && <div className="skin-editor account-editor">
            {draft && <>
                <Field id="skin-name" scope="main" label={t("Skin name")} value={draft.name} onChange={name => setDraft({...draft,name})} disabled={busy}/>
                <SelectMenu id="skin-model" compact showIcons={false} label={t("Skin model")} value={draft.model} options={["default","slim"]} labelForValue={value => t(value==="slim" ? "Slim arms" : "Classic arms")} disabled={busy} onChange={model => setDraft({...draft,model:model as SkinModel})}/>
                <StandardButton id="skin-file" disabled={busy} onClick={() => file.current?.click()}>{t(draft.png ? "Replace PNG" : "Choose PNG")}</StandardButton>
                <p>{t("Choose a 64×64 or 64×32 PNG, up to 128 KiB.")}</p>
            </>}
            {deleting && <p>{t("Delete skin {name}? Accounts using it will return to the default preview.",{name:selected.name})}</p>}
            <div className="config-actions" data-navigation-actions>
                <StandardButton id="skin-save" tone={deleting ? "red" : "green"} disabled={busy || (!deleting && (!validName || !draft?.png))} onClick={() => void (deleting ? erase() : save())}>{t(deleting ? "Delete" : "Save")}</StandardButton>
                <StandardButton id="skin-cancel" disabled={busy} onClick={cancel}>{t("Cancel")}</StandardButton>
            </div>
        </div>}
        <input ref={file} type="file" accept="image/png,.png" hidden onChange={event => void readFile(event.currentTarget)}/>
        {library.applied && <p role="status">{t("Skin updated")}</p>}
        {(error || library.error) && <p className="config-error" role="alert">{error || library.error}</p>}
    </section>;
}
