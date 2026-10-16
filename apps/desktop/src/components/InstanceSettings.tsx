import { InstanceFiles } from "./InstanceFiles";
import { t } from "../i18n";
import type { Instance } from "../state/library";
import type { Installed } from "../state/installer";
import { JavaSettings, type JavaInventory } from "./JavaSettings";
import { LoaderSettings } from "./LoaderSettings";
import { LaunchSettings } from "./LaunchSettings";

/** One lifecycle boundary for all state belonging to the selected instance.
 * The parent keys this component by instance ID; children never share that key.
 * Application-wide inventories and transfer tasks remain owned by App.
 */
export function InstanceSettings({ item, installed, java, busy, saving, preview, supported, onPatch, onLoaderChange, onJavaInstall, onViewLogs, onBrowseModMenu }: {
    item: Instance;
    onBrowseModMenu?: () => void;
    onViewLogs: () => void;
    installed?: Installed;
    java: JavaInventory;
    busy: boolean;
    saving: boolean;
    preview: boolean;
    supported: boolean;
    onPatch: (patch: Pick<Instance, "javaPath" | "launchSettings">) => Promise<boolean>;
    onLoaderChange: (loader: NonNullable<Instance["loader"]>) => Promise<boolean>;
    onJavaInstall: (major: number) => void;
}) {
    return <div className="installation-details">
        {!installed?.installed && <p>{t("This instance is not fully installed.")}</p>}
        <p className="instance-directory">{t("This launcher requires instance isolation. Saves, mods, resource packs, and game settings stay in this instance directory:")}{installed?.gameDirectory ?? `.minecraft/versions/${item.directoryName}`}</p>
        <InstanceFiles instanceId={item.id} onViewLogs={onViewLogs}/>
        <LoaderSettings onBrowseModMenu={onBrowseModMenu} item={item} busy={busy || saving} installed={preview || !!installed?.installed} onChange={onLoaderChange}/>
        <JavaSettings requiredForInstallation={!installed?.installed && (item.loader?.kind === "forge" || item.loader?.kind === "neoforge")} inventory={java} version={item.minecraftVersion} value={item.javaPath} onChange={javaPath=>void onPatch({javaPath})} onInstall={onJavaInstall} busy={busy || saving} preview={preview}/>
        <LaunchSettings busy={busy} saving={saving} value={item.launchSettings} onSave={launchSettings=>onPatch({launchSettings})}/>
        {!preview && !supported && <p>{t("Installation requires a Windows or Linux x86_64 build.")}</p>}
    </div>;
}
