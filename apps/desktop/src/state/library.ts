import { loaderKinds, type LoaderKind } from "./loaders";
import { native, invokeNative } from "../platform/desktop";
import { validLaunchSettings, type LaunchSettings } from "./launchSettings";
import { localizeMessage, t } from "../i18n";
import { useEffect, useRef, useState } from "react";
import { iconCatalog, type IconId } from "./instanceIcons";
export type Instance = {
    id: string;
    name: string;
    minecraftVersion: string;
    icon: IconId;
    loader?: { kind: LoaderKind; version: string };
    javaPath?: string;
    directoryName: string;
    launchSettings?: LaunchSettings;
};
export type Library = {
    schemaVersion: number;
    revision: number;
    instances: Instance[];
    selectedId: string | null;
    lastLaunchedId: string | null;
};
export const emptyLibrary = (): Library => ({ schemaVersion: 2, revision: 0, instances: [], selectedId: null, lastLaunchedId: null });
export const initialSelection = (data: Library) => data.lastLaunchedId ?? data.selectedId;
export const labelOf = (item: Instance) => `${item.name} · ${item.minecraftVersion}`;
export function validateLibrary(data: Library) {
    if (!data || data.schemaVersion !== 2 || !Number.isSafeInteger(data.revision) || data.revision < 0 || !Array.isArray(data.instances) || data.instances.length > 1000)
        throw Error(t("Unsupported instance library format"));
    const ids = new Set<string>(), names = new Set<string>();
    for (const i of data.instances) {
        if (i?.loader && (!loaderKinds.includes(i.loader.kind) || typeof i.loader.version !== "string" || !/^[a-zA-Z0-9.+-]{1,80}$/.test(i.loader.version))) throw Error(t("Invalid mod loader selection"));
        if (!i || typeof i.id !== "string" || !/^[a-zA-Z0-9-]{1,80}$/.test(i.id) || ids.has(i.id))
            throw Error(t("Invalid or duplicate instance ID"));
        if (typeof i.name !== "string" || !i.name.trim() || i.name !== i.name.trim() || [...i.name].length > 64 || /[\u0000-\u001f\u007f-\u009f]/.test(i.name) || names.has(i.name.toLowerCase()))
            throw Error(t("Instance names must be nonempty, unique, and at most 64 characters"));
        if (typeof i.minecraftVersion !== "string" || !/^[a-zA-Z0-9 ._-]{1,40}$/.test(i.minecraftVersion))
            throw Error(t("Invalid Minecraft version format"));
        if (typeof i.directoryName !== "string" || !i.directoryName.trim() || i.directoryName === "." || i.directoryName === ".." || /[\\/:*?"<>|\u0000-\u001f]/.test(i.directoryName)) throw Error(t("Invalid instance directory name"));
        if (i.launchSettings !== undefined && !validLaunchSettings(i.launchSettings)) throw Error(t("Invalid launch settings"));
        if (i.javaPath !== undefined && (typeof i.javaPath !== "string" || !(/^(?:\/|[A-Za-z]:[\\/]|\\\\)/.test(i.javaPath)) || i.javaPath.length > 4096 || i.javaPath.includes("\0")))
            throw Error(t("Invalid Java path"));
        if (!iconCatalog.includes(i.icon))
            throw Error(t("Unknown instance icon"));
        ids.add(i.id);
        names.add(i.name.toLowerCase());
    }
    if ((data.selectedId !== null && !ids.has(data.selectedId)) || (data.lastLaunchedId !== null && !ids.has(data.lastLaunchedId)) || (ids.size > 0 && !data.selectedId))
        throw Error(t("Referenced instance does not exist"));
}
export const previewKey = "launcher.local-library.v1";
export async function loadLibrary(): Promise<Library> {
    const core = native();
    const data = core ? await core.invoke<Library>("load_library") : JSON.parse(localStorage.getItem(previewKey) ?? JSON.stringify(emptyLibrary()));
    validateLibrary(data);
    return data;
}
export async function saveLibrary(data: Library): Promise<Library> {
    validateLibrary(data);
    const core = native();
    if (core)
        return core.invoke<Library>("save_library", { library: data });
    const current = await loadLibrary();
    if (current.revision !== data.revision)
        throw Error(t("The instance library changed; reload and retry"));
    const last = data.instances.some(i => i.id === current.lastLaunchedId) ? current.lastLaunchedId : null;
    if (data.lastLaunchedId !== last)
        throw Error(t("Editing instances cannot change launch history"));
    const next = { ...data, revision: data.revision + 1 };
    localStorage.setItem(previewKey, JSON.stringify(next));
    return next;
}
export function useLibrary() {
    const [data, setData] = useState(emptyLibrary);
    const [loading, setLoading] = useState(true);
    const [busy, setBusy] = useState(false);
    const [error, setError] = useState("");
    const lock = useRef(false);
    useEffect(() => {
        let active = true;
        loadLibrary().then(value => {
            if (active)
                setData({ ...value, selectedId: initialSelection(value) });
        }).catch(e => { if (active)
            setError(localizeMessage(e)); }).finally(() => { if (active)
            setLoading(false); });
        return () => { active = false; };
    }, []);
    async function mutate(operation: () => Promise<Library>) {
        if (lock.current || loading) return false;
        lock.current = true;
        setBusy(true);
        setError("");
        try { setData(await operation()); return true; }
        catch (e) { setError(localizeMessage(e)); return false; }
        finally { lock.current = false; setBusy(false); }
    }
    const commit = (next: Library) => mutate(() => saveLibrary(next));
    function lifecycle(command: "create_instance" | "delete_instance", item: Instance) {
        return mutate(async () => {
            if (native()) return invokeNative<Library>(command, command === "create_instance" ? { item, revision: data.revision } : { instanceId: item.id, revision: data.revision });
            const creating = command === "create_instance";
            if (creating && data.instances.some(i => i.directoryName.toLowerCase() === item.name.toLowerCase()))
                throw Error(t("Instance name conflict; choose another name"));
            const items = creating ? [...data.instances, { ...item, directoryName: item.name }] : data.instances.filter(i => i.id !== item.id);
            return saveLibrary({ ...data, instances: items, selectedId: creating ? item.id : items[0]?.id ?? null, lastLaunchedId: !creating && data.lastLaunchedId === item.id ? null : data.lastLaunchedId });
        });
    }
    function changeLoader(item: Instance, loader: NonNullable<Instance["loader"]>) {
        return mutate(async () => {
            if (item.loader && item.loader.kind !== loader.kind) throw Error(t("Switching mod loader families is not supported"));
            return native()
                ? invokeNative<Library>("change_mod_loader",{instanceId:item.id,revision:data.revision,loader})
                : saveLibrary({...data,instances:data.instances.map(i=>i.id===item.id ? {...i,loader} : i)});
        });
    }
    return { changeLoader, create: (item: Instance) => lifecycle("create_instance", item), remove: (item: Instance) => lifecycle("delete_instance", item), data, loading, busy, error, clearError: () => setError(""), commit, preview: !native() };
}
