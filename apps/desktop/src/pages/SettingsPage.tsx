import { LauncherUpdate } from "../components/LauncherUpdate";
import { Checkbox } from "../components/Checkbox";
import { SelectMenu } from "../components/SelectMenu";
import { StandardButton } from "../components/StandardButton";
import { t, useLocale, setLocale, type Locale } from "../i18n";
import type { useSteamSession } from "../input/steamSession";
import { contentSources } from "../state/contentSource";

export function SettingsPage({steam,preview,controllerStatus,onViewLogs,onBrowseControllerMod}: {
    steam: ReturnType<typeof useSteamSession>;
    preview: boolean;
    controllerStatus: string;
    onViewLogs: () => void;
    onBrowseControllerMod: () => void;
}) {
    const language = useLocale();
    return <section className="placeholder settings-page" aria-label={t("Settings")}>
        <LauncherUpdate/>
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
            <p>{controllerStatus}</p>
            <p className="settings-description">{t("Install a controller mod such as Controlify to use a controller in game. Availability depends on the selected game version and mod loader.")}</p>
            <StandardButton id="controller-browse-mod" tone="green" onClick={onBrowseControllerMod}>{t("Find Controlify")}</StandardButton>
            <p className="settings-description">{preview ? t("Controller input is available in the desktop launcher") : t("Native input · D-pad / left stick navigation")}</p>
        </div>
    </section>;
}
