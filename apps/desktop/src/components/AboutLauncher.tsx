import { useState } from "react";
import { StandardButton } from "./StandardButton";
import { homepage, maintainerHomepage, openInformation } from "../state/editorial";
import { t, localizeMessage } from "../i18n";
import type { LegalDocument } from "./LegalDialog";
export function AboutLauncher({onDocument}:{onDocument:(kind:LegalDocument)=>void}) {
    const [error,setError]=useState("");
    return <div className="settings-group launcher-about">
        <span>{t("About MC Launcher Advanced")}</span>
        <p>{t("An independent launcher for Minecraft: Java Edition.")}</p>
        <p className="settings-description">{t("Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft.")}</p>
        <p>{t("Project code: GNU GPL 3.0 license. Third-party components and assets retain their own licenses.")}</p>
        <p className="settings-description">{t("The interface draws on the visual style of the official Minecraft launcher and is independently implemented, with improvements for keyboard, mouse and controller use.")}</p>
        <p className="settings-description">{t("No official launcher program code obtained through reverse engineering is used. Referenced artwork retains its original ownership; see Third-party notices for sources and permission status.")}</p>
        <p className="settings-description">{t("Developed, maintained and published by ImperatorVienna in an individual capacity.")}</p>
        <p className="settings-description">{t("Contact: imperator.vienna@outlook.com")}</p>
        <div className="config-actions" data-navigation-actions>
            <StandardButton id="maintainer-homepage" onClick={()=>{setError("");void openInformation(maintainerHomepage).catch(e=>setError(localizeMessage(e)));}}>{t("Maintainer homepage")}</StandardButton>
            <StandardButton id="project-homepage" onClick={()=>{setError("");void openInformation(homepage).catch(e=>setError(localizeMessage(e)));}}>{t("Project homepage")}</StandardButton>
            <StandardButton id="project-license" aria-haspopup="dialog" onClick={()=>onDocument("license")}>{t("Open-source license")}</StandardButton>
            <StandardButton id="third-party-notices" aria-haspopup="dialog" onClick={()=>onDocument("notices")}>{t("Third-party notices")}</StandardButton>
            <StandardButton id="privacy-notice" aria-haspopup="dialog" onClick={()=>onDocument("privacy")}>{t("Privacy notice")}</StandardButton>
        </div>
        {error&&<p className="config-error" role="alert">{error}</p>}
    </div>;
}
