import { useState } from "react";
import { StandardButton } from "./StandardButton";
import { homepage, openInformation } from "../state/editorial";
import { t, localizeMessage } from "../i18n";
import type { LegalDocument } from "./LegalDialog";
export function AboutLauncher({onDocument}:{onDocument:(kind:LegalDocument)=>void}) {
    const [error,setError]=useState("");
    return <div className="settings-group launcher-about">
        <span>{t("About MC Launcher Advanced")}</span>
        <p>{t("An independent launcher for Minecraft: Java Edition.")}</p>
        <p className="settings-description">{t("Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft.")}</p>
        <p>{t("Project code: GNU GPL 3.0 license. Third-party components and assets retain their own licenses.")}</p>
        <div className="config-actions" data-navigation-actions>
            <StandardButton id="project-homepage" onClick={()=>{setError("");void openInformation(homepage).catch(e=>setError(localizeMessage(e)));}}>{t("Project homepage")}</StandardButton>
            <StandardButton id="project-license" aria-haspopup="dialog" onClick={()=>onDocument("license")}>{t("Open-source license")}</StandardButton>
            <StandardButton id="third-party-notices" aria-haspopup="dialog" onClick={()=>onDocument("notices")}>{t("Third-party notices")}</StandardButton>
        </div>
        {error&&<p className="config-error" role="alert">{error}</p>}
    </div>;
}
