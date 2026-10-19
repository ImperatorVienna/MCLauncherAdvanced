import license from "../../../../LICENSE?raw";
import { t } from "../i18n";
import { DialogText } from "./DialogText";
import { StandardButton } from "./StandardButton";
export type LegalDocument = "license" | "notices";
const notices = import.meta.glob(["../../src-tauri/licenses/*.txt", "../assets/fonts/OFL.txt"], { eager: true, query: "?raw", import: "default" }) as Record<string, string>;

export function LegalDialog({kind,onClose}:{kind:LegalDocument;onClose:()=>void}) {
    const title=t(kind==="license" ? "Open-source license" : "Third-party notices");
    return <div className="log-viewer">
        <h2 id="dialog-title">{title}</h2>
        {kind==="notices" && <p>{t("These notices cover the bundled components listed below; they do not grant rights to Minecraft artwork.")}</p>}
        <DialogText id="modal-first" documentText label={title} text={kind==="license" ? license : Object.entries(notices).sort(([a],[b])=>a.localeCompare(b)).map(([name,text])=>`${name.split("/").at(-1)}\n\n${text}`).join("\n\n────────────────────\n\n")}/>
        <div className="config-actions"><StandardButton id="legal-close" scope="modal" onClick={onClose}>{t("Close")}</StandardButton></div>
    </div>;
}
