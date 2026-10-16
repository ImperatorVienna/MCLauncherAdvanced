import { useEffect, useState } from "react";
import { t } from "../i18n";
import type { Instance } from "../state/library";
import { loaderKinds, loaderName, useLoaderVersions, type LoaderKind } from "../state/loaders";
import { SelectMenu } from "./SelectMenu";
import { StandardButton } from "./StandardButton";

export function LoaderSettings({item,busy,installed,onChange,onBrowseModMenu}: {
    onBrowseModMenu?: () => void;
    item: Instance; busy: boolean; installed: boolean;
    onChange: (loader: NonNullable<Instance["loader"]>) => Promise<boolean>;
}) {
    const [choice,setChoice]=useState<LoaderKind | undefined>(item.loader?.kind);
    const kind=item.loader?.kind ?? choice;
    const [retry,setRetry]=useState(0), [version,setVersion]=useState("");
    const catalog=useLoaderVersions(kind,item.minecraftVersion,retry);
    useEffect(()=>{
        setVersion(catalog.versions.find(v=>v.stable)?.version ?? catalog.versions[0]?.version ?? "");
    },[kind,catalog.versions]);
    const valid=!!kind && (!(kind === "forge" || kind === "neoforge") || !!item.javaPath) && catalog.versions.some(v=>v.version===version) && !catalog.loading && !catalog.error;
    return <section className="loader-settings" aria-label={t("Mod loader")}>
        <p>{item.loader ? t("Installed mod loader: {name} {version}",{name:loaderName(item.loader.kind),version:item.loader.version}) : t("Installed mod loader: none")}</p>
        {!item.loader && <SelectMenu id="manage-loader-kind" scope="main" compact showIcons={false} label={t("Mod loader")} value={kind ?? "none"} options={["none", ...loaderKinds]} labelForValue={v=>v==="none" ? t("No mod loader") : loaderName(v as LoaderKind)} disabled={busy} onChange={v=>{setVersion("");setChoice(v==="none" ? undefined : v as LoaderKind);}}/>}
        {kind && <>
            <div className="java-control-row">
                <SelectMenu id="manage-loader-version" scope="main" compact showIcons={false} label={t("Mod loader version")} value={version} options={catalog.versions.map(v=>v.version)} labelForValue={v=>v ? `${loaderName(kind)} ${v}${v===item.loader?.version ? t(" (installed)") : ""}` : t("No version")} disabled={busy || catalog.loading || !catalog.versions.length} onChange={setVersion}/>
                <StandardButton id="manage-loader-apply" scope="main" tone="green" disabled={busy || !installed || !valid || version===item.loader?.version} onClick={()=>{if(kind && valid) void onChange({kind,version});}}>{t(item.loader ? "Change loader version" : "Install mod loader")}</StandardButton>
                {(item.loader?.kind === "fabric" || item.loader?.kind === "quilt") && onBrowseModMenu && <StandardButton id="manage-loader-mod-menu" scope="main" tone="green" onClick={onBrowseModMenu}>{t("Find Mod Menu")}</StandardButton>}
            </div>
            {catalog.loading ? <p role="status">{t("Fetching compatible mod loader versions…")}</p> : catalog.error ? <div className="java-control-row"><p role="alert">{catalog.error}</p><StandardButton id="manage-loader-retry" scope="main" disabled={busy} onClick={()=>setRetry(n=>n+1)}>{t("Retry")}</StandardButton></div> : !catalog.versions.length && <p>{t("No compatible mod loader version is available for installation.")}</p>}
            {!installed && <p>{t("Repair the instance before changing its mod loader")}</p>}
            <p>{t("Changing the loader version keeps mods and saves. Check mod compatibility before applying.")}</p>
        </>}
    </section>;
}
