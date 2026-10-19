import { useState } from "react";
import { ControllerHints } from "../components/ControllerHints";
import { Checkbox } from "../components/Checkbox";
import { SelectMenu } from "../components/SelectMenu";
import { StandardButton } from "../components/StandardButton";
import { t, useLocale, setLocale, type Locale } from "../i18n";
import type { useSteamSession } from "../input/steamSession";
import { contentSources } from "../state/contentSource";

export function SettingsPage({steam,preview,controllerDevices,controllerError,controllerSelection,onControllerSelection,onViewLogs,onBrowseControllerMod}: {
    steam: ReturnType<typeof useSteamSession>;
    preview: boolean;
    controllerDevices: string[];
    controllerError?: string;
    controllerSelection?: string;
    onControllerSelection?: (name:string)=>void;
    onViewLogs: () => void;
    onBrowseControllerMod: () => void;
}) {
    const language = useLocale();
    const devices = preview ? ["Xbox Wireless Controller", "DualSense Wireless Controller", "Nintendo Switch Pro Controller", t("Generic controller guide")] : controllerDevices;
    const [localChoice,setLocalChoice]=useState("");
    const chosenDevice=controllerSelection ?? localChoice;
    const setChosenDevice=onControllerSelection ?? setLocalChoice;
    const selectedDevice=devices.includes(chosenDevice) ? chosenDevice : devices[0] ?? "";

    return <section className="placeholder settings-page" aria-label={t("Settings")}>
        <div><StandardButton id="launcher-view-logs" tone="yellow" onClick={onViewLogs}>{t("View launcher logs")}</StandardButton></div>
        <div className="settings-group">
            <span>{t("Language")}</span>
            <SelectMenu id="launcher-language" compact showIcons={false} label={t("Language")} options={["en-US","zh-CN"]} value={language} labelForValue={value=>value==="zh-CN" ? t("Simplified Chinese") : t("English")} onChange={value=>setLocale(value as Locale)}/>
        </div>
        <div className="settings-group">
            <span>{t("Content download source")}</span>
            <SelectMenu id="content-source" compact options={contentSources} value="Modrinth" onChange={()=>{}} showIcons={false} label={t("Content download source")}/>
        </div>
        <div className="settings-group">
            <Checkbox id="steam-adaptation" checked={steam.status.enabled} disabled={!steam.ready || steam.saving} onClick={steam.toggle} aria-describedby="steam-adaptation-description">
                {t("Adapt to Steam launch")}
            </Checkbox>
            <p id="steam-adaptation-description" className="settings-description">{t("Use fullscreen and the Steam keyboard when launched through Steam.")}</p>
        </div>
        <div className="settings-group">
            <span>{t("Controller input")}</span>
            {controllerError && <p className="config-error" role="alert">{controllerError}</p>}
            <SelectMenu id="controller-device" compact showIcons={false} label={t("Select controller")} options={[...new Set(devices)]} value={selectedDevice} disabled={!devices.length} onChange={setChosenDevice}/>
            {selectedDevice && <ControllerHints device={selectedDevice}/>}
            <p className="settings-description">{t("Install a controller mod such as Controlify to use a controller in game. Availability depends on the selected game version and mod loader.")}</p>
            <StandardButton id="controller-browse-mod" tone="green" onClick={onBrowseControllerMod}>{t("Find Controlify")}</StandardButton>
            {preview && <p className="settings-description">{t("Controller input is available in the desktop launcher")}</p>}
        </div>
    </section>;
}
