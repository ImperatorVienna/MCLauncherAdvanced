import { JavaSettings, type JavaInventory } from "./JavaSettings";
import { localizeMessage, t } from "../i18n";
import { useContext, useEffect, useLayoutEffect, useState, useRef } from "react";
import { Field } from "./Field";
import { Navigation } from "./Control";
import { StandardButton } from "./StandardButton";
import { iconCatalog, iconNames } from "../state/instanceIcons";
import { invokeNative } from "../platform/desktop";
import type { Instance } from "../state/library";
import { SelectMenu } from "./SelectMenu";
import type { OfficialVersion } from "../state/installer";
import { channels, channelOf, familyOf, channelLabel, familyLabel } from "../state/versionCatalog";
import { useLoaderVersions, loaderKinds, loaderName, type LoaderKind } from "../state/loaders";
const emptyJavaInventory: JavaInventory = {runtimes:[],loading:false,error:"",refresh:async()=>{}};
const previewCatalog: OfficialVersion[] = [...["26.3", "26.2", "1.21.1", "1.21", "1.20.4"].map(id => ({ id, type: "release" as const, releaseTime: "" })), ...["26.4-snapshot-4", "26.3-pre-1", "1.21-pre1", "23w31a"].map(id => ({ id, type: "snapshot" as const, releaseTime: "" }))];
export function InstanceEditor({ item, busy, error, onSave, onCancel, preview = false, onCancelTask, taskRunning = false, java = emptyJavaInventory, onJavaInstall }: {
    java?: JavaInventory;
    onJavaInstall?: (major: number) => void;
    preview?: boolean;
    item?: Instance;
    busy: boolean;
    error: string;
    onSave: (item: Instance) => void;
    onCancel: () => void;
    onCancelTask?: () => void;
    taskRunning?: boolean;
}) {
    const manager = useContext(Navigation);
    const [draft, setDraft] = useState<Instance>(() => item ?? { id: crypto.randomUUID(), name: "", minecraftVersion: "", directoryName: "", icon: iconCatalog[Math.floor(Math.random() * iconCatalog.length)] });
    const [javaConfirmed, setJavaConfirmed] = useState(false);
    const [checkingJava, setCheckingJava] = useState(false);
    const [javaError, setJavaError] = useState("");
    const checking = useRef(false);
    const alive = useRef(true);
    useEffect(() => { alive.current = true; return () => { alive.current = false; }; }, []);
    const javaRequired = draft.loader?.kind === "forge" || draft.loader?.kind === "neoforge";
    useEffect(() => { if (item) return; setJavaConfirmed(false); setJavaError(""); setDraft(old => ({...old, javaPath: undefined})); }, [draft.minecraftVersion, draft.loader?.kind]);
    const customName = useRef(!!item);
    const [loaderRetry,setLoaderRetry] = useState(0);
    const loaderVersions = useLoaderVersions(item ? undefined : draft.loader?.kind,draft.minecraftVersion,loaderRetry);
    useEffect(() => {
        if (!draft.loader || item || loaderVersions.loading || loaderVersions.error) return;
        const version = loaderVersions.versions.find(v => v.stable)?.version ?? loaderVersions.versions[0]?.version ?? "";
        setDraft(old => old.loader && old.loader.version !== version && !loaderVersions.versions.some(v => v.version === old.loader!.version) ? {...old,loader:{...old.loader,version}} : old);
    },[loaderVersions.versions,loaderVersions.loading,loaderVersions.error, draft.loader?.kind, item]);
    useEffect(() => {
        if (customName.current || item) return;
        const name = draft.minecraftVersion + (draft.loader ? `-${loaderName(draft.loader.kind)}${draft.loader.version ? "-"+draft.loader.version : ""}` : "");
        setDraft(old => old.name === name ? old : {...old,name});
    },[draft.minecraftVersion,draft.loader?.kind,draft.loader?.version,item]);
    const [catalog, setCatalog] = useState<OfficialVersion[]>([]), [catalogError, setCatalogError] = useState("");
    useLayoutEffect(() => { if (!busy)
        manager.preserveFocus("modal-first");
    else if (taskRunning)
        manager.preserveFocus("modal-cancel-task"); }, [busy, taskRunning, manager, catalog.length]);
    const [channel, setChannel] = useState("release"), [family, setFamily] = useState(""), [refresh, setRefresh] = useState(0);
    const selectVersion = (minecraftVersion: string) => setDraft(old => ({ ...old, minecraftVersion, loader: old.loader ? {...old.loader,version:""} : undefined }));
    useEffect(() => {
        if (item)
            return;
        let alive = true;
        setCatalogError("");
        (preview ? Promise.resolve(previewCatalog) : invokeNative<OfficialVersion[]>("list_minecraft_versions")).then(v => {
            if (!alive)
                return;
            setCatalog(v);
            const selected = v.find(v => v.type === "release");
            if (selected) {
                setChannel(channelOf(selected));
                setFamily(familyOf(selected));
                selectVersion(selected.id);
            }
        }).catch(e => { if (alive)
            setCatalogError(localizeMessage(e)); });
        return () => { alive = false; };
    }, [preview, refresh]);
    const families = Array.from(new Set(catalog.filter(v => channelOf(v) === channel).map(familyOf)));
    const choices = catalog.filter(v => channelOf(v) === channel && familyOf(v) === family).map(v => v.id);
    const chooseGroup = (nextChannel: string, nextFamily?: string) => {
        const available = catalog.filter(v => channelOf(v) === nextChannel);
        const group = nextFamily ?? (available[0] ? familyOf(available[0]) : "");
        setChannel(nextChannel);
        setFamily(group);
        selectVersion(available.find(v => familyOf(v) === group)?.id ?? "");
    };
    const canSave = (!!item || (javaConfirmed && (!javaRequired || !!draft.javaPath))) && !checkingJava && !!draft.name.trim() && (!!item || choices.includes(draft.minecraftVersion)) && !busy && (!!item || !draft.loader || (!loaderVersions.loading && !loaderVersions.error && loaderVersions.versions.some(v => v.version === draft.loader?.version)));
    const submit = async () => {
        if (!canSave || checking.current) return;
        checking.current = true; setCheckingJava(true); setJavaError("");
        try {
            if (!item && draft.javaPath && !preview)
                await invokeNative("validate_installation_java", {version:draft.minecraftVersion, path:draft.javaPath});
            if (alive.current) onSave({ ...draft, name: draft.name.trim() });
        } catch (e) { setJavaError(localizeMessage(e)); }
        finally { checking.current = false; setCheckingJava(false); }
    };
    return <fieldset className="instance-editor-fields" disabled={checkingJava}><form className="instance-editor" onSubmit={e => { e.preventDefault(); submit(); }}>
    <h2 id="dialog-title">{item ? t("Edit instance") : t("Download and install")}</h2>
    {!item && <div className="official-versions">
      <span>{t("Release channel")}</span>
      <SelectMenu id="modal-first" scope="modal" label={t("Release channel")} compact showIcons={false} disabled={busy} labelForValue={channelLabel} options={[...channels]} value={channel} onChange={v => chooseGroup(v)}/>
      <span>{t("Version family")}</span>
      <SelectMenu id="version-family" scope="modal" label={t("Version family")} compact showIcons={false} disabled={busy || !families.length} labelForValue={familyLabel} options={families} value={family || t("No version family")} onChange={v => chooseGroup(channel, v)}/>
      <span>{t("Minecraft version")}</span>
      <SelectMenu id="official-version" scope="modal" label={t("Minecraft version")} compact showIcons={false} disabled={busy || !choices.length} options={choices} value={draft.minecraftVersion || t("No version")} onChange={selectVersion}/>
      <span>{t("Mod loader")}</span>
      <SelectMenu id="mod-loader" scope="modal" compact showIcons={false} label={t("Mod loader")} value={draft.loader?.kind ?? "none"} options={["none", ...loaderKinds]} labelForValue={v => v === "none" ? t("No mod loader") : loaderName(v as LoaderKind)} disabled={busy || !draft.minecraftVersion} onChange={kind => setDraft(old => ({...old,loader:kind === "none" ? undefined : {kind:kind as LoaderKind,version:""}}))}/>
      {draft.loader && <>
        <span>{t("Mod loader version")}</span>
        <SelectMenu id="mod-loader-version" scope="modal" compact showIcons={false} label={t("Mod loader version")} value={draft.loader.version} options={loaderVersions.versions.map(v => v.version)} labelForValue={v => v || t("No version")} disabled={busy || loaderVersions.loading || !loaderVersions.versions.length} onChange={version => setDraft(old => ({...old,loader:old.loader ? {...old.loader,version} : undefined}))}/>
        {loaderVersions.loading ? <p role="status">{t("Fetching compatible mod loader versions…")}</p> : loaderVersions.error ? <p role="alert">{loaderVersions.error} <StandardButton id="loader-retry" scope="modal" type="button" onClick={() => setLoaderRetry(n => n+1)}>{t("Retry")}</StandardButton></p> : !loaderVersions.versions.length && <p>{t("No compatible mod loader version is available for installation.")}</p>}
      </>}
      {catalogError && <p role="alert">{t("Failed to fetch the version catalog:")}{catalogError} <StandardButton id="catalog-retry" scope="modal" type="button" onClick={() => setRefresh(n => n + 1)}>{t("Retry")}</StandardButton></p>}
      {!catalog.length && !catalogError && <p role="status">{t("Fetching the official version catalog\u2026")}</p>}
    </div>}
    {item && <p>{t("Minecraft version:")}{item.minecraftVersion}</p>}
    {draft.minecraftVersion && <>
      <Field id={item ? "modal-first" : "config-name"} label={item ? t("Instance name:") : t("Instance name (also the directory name)")} value={draft.name} disabled={busy} onChange={name => { customName.current = true; setDraft({ ...draft, name }); }}/>

      <span className="config-icon-label">{t("Choose icon")}</span>
      <SelectMenu id={"config-icon"} scope="modal" label={t("Choose icon")} compact options={[...iconCatalog]} value={draft.icon} labelForValue={v => iconNames[v as keyof typeof iconNames]} iconForOption={v => v as Instance["icon"]} disabled={busy} onChange={icon => setDraft({ ...draft, icon: icon as Instance["icon"] })}/>
    </>}
    {!item && <JavaSettings key={`${draft.minecraftVersion}/${draft.loader?.kind}`} inventory={java} version={draft.minecraftVersion} value={draft.javaPath} onChange={javaPath => { setDraft(old => ({...old,javaPath})); setJavaConfirmed(true); setJavaError(""); }} onInstall={onJavaInstall ?? (()=>{})} busy={busy || checkingJava} preview={preview} scope="modal" requiredForInstallation={javaRequired} explicitSelection selectionConfirmed={javaConfirmed}/>}
    {javaError && <p role="alert" className="config-error">{javaError}</p>}
    {!item && <p>{preview ? t("The browser preview demonstrates the workflow. Use the desktop app to download files.") : t("Create a directory named after this instance under versions and start downloading. Interrupted installations can be resumed.")}</p>}
    {error && <p role="alert">{error}</p>}
    <div className="config-actions" data-navigation-actions>
      <StandardButton id="config-save" scope="modal" tone="green" type="submit" disabled={!canSave}>{item ? t("Save changes") : t("Download and install now")}</StandardButton>
      <StandardButton id="config-cancel" scope="modal" type="button" disabled={busy} onClick={onCancel}>{t("Cancel")}</StandardButton>
      {taskRunning && <StandardButton id="modal-cancel-task" scope="modal" type="button" onClick={onCancelTask}>{t("Cancel download")}</StandardButton>}
    </div>
  </form></fieldset>;
}
