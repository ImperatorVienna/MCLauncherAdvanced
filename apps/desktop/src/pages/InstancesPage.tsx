import type { ComponentProps } from "react";
import { t, localizeMessage } from "../i18n";
import type { Instance, useLibrary } from "../state/library";
import type { useInstaller } from "../state/installer";
import { SelectMenu } from "../components/SelectMenu";
import { StandardButton } from "../components/StandardButton";
import { InstanceSettings } from "../components/InstanceSettings";
import type { JavaInventory } from "../components/JavaSettings";

export function InstancesPage({library,installer,java,gameBusy,selection,onOpen,onViewLogs,onBrowseModMenu}: {
    library: ReturnType<typeof useLibrary>;
    installer: ReturnType<typeof useInstaller>;
    java: JavaInventory;
    gameBusy: boolean;
    selection: ComponentProps<typeof SelectMenu>;
    onBrowseModMenu?: () => void;
    onViewLogs: (item: Instance) => void;
    onOpen: (kind: "create" | "edit" | "delete") => void;
}) {
    const selected=library.data.instances.find(item=>item.id===library.data.selectedId);
    const installed=installer.installed.find(item=>item.instanceId===selected?.id && item.minecraftVersion===selected?.minecraftVersion);
    const working=library.busy || installer.busy || gameBusy;
    const open=onOpen;
    return <section className="placeholder instances-page" aria-label={t("Instances")}>
        <StandardButton id="config-create" tone="green" disabled={library.loading || working} onClick={()=>open("create")}>{t("Download and install new instance")}</StandardButton>
        <p>{t("Download a new instance or manage an existing one. Select Java or download it manually.")}</p>

                    <div className="instance-management-row">
                    <SelectMenu {...selection} disabled={working || library.loading || !selected}/>
                    <div className="instance-management-actions">
                      <StandardButton id="config-edit" disabled={!selected || working} onClick={() => open("edit")}>{t("Edit")}</StandardButton>
                      <StandardButton id="config-install" tone="yellow" disabled={!selected || working || library.loading || !installer.supported} onClick={() => selected && void installer.start(selected.id)}>{installed?.installed ? t("Repair") : t("Resume installation")}</StandardButton>
                      <StandardButton id="config-delete" tone="red" disabled={!selected || working} onClick={() => open("delete")}>{t("Delete")}</StandardButton>
                    </div>
                    </div>
                    {selected && <InstanceSettings onBrowseModMenu={onBrowseModMenu} key={selected.id} item={selected} onViewLogs={()=>onViewLogs(selected)} installed={installed} java={java}
                      busy={installer.busy || gameBusy} saving={library.busy} preview={library.preview} supported={installer.supported}
                      onPatch={patch => library.commit({ ...library.data, instances: library.data.instances.map(item => item.id === selected.id ? { ...item, ...patch } : item) })}
                      onLoaderChange={loader => library.changeLoader(selected, loader)} onJavaInstall={major => void installer.startJava(major)}/>}
                    {installer.error && <p className="config-error" role="alert">{installer.error}</p>}
                    {installer.task?.error && !installer.task.content && <p className="config-error" role="alert">{installer.task.version}：{localizeMessage(installer.task.error)}{t(". Resume installation to reuse verified files.")}</p>}
                    {!selected && <p>{t("No instances yet. Choose Download and install.")}</p>}
                    {library.preview && <p>{t("Browser preview: instances are stored in this browser separately from the desktop app.")}</p>}
                  
    </section>;
}
