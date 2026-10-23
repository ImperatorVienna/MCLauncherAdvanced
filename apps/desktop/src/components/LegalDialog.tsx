import license from "../../../../LICENSE?raw";
import assetNotices from "../../../../ASSET-NOTICES.md?raw";
import privacyEnglish from "../../../../PRIVACY.md?raw";
import privacyTraditional from "../../../../PRIVACY.zh-TW.md?raw";
import privacyChinese from "../../../../PRIVACY.zh-CN.md?raw";
import { t, useLocale } from "../i18n";
import { DialogText } from "./DialogText";
import { StandardButton } from "./StandardButton";
export type LegalDocument = "license" | "notices" | "privacy";
const notices = import.meta.glob(["../../src-tauri/licenses/*.txt", "../assets/fonts/OFL.txt"], { eager: true, query: "?raw", import: "default" }) as Record<string, string>;

const componentNames: Record<string,string> = {
    "OFL.txt": "Noto Sans — SIL Open Font License",
    "SDL-GameControllerDB.txt": "SDL GameControllerDB — License notice",
    "skinview3d.txt": "skinview3d — MIT License",
    "skinview-utils.txt": "skinview-utils — MIT License",
    "three.txt": "Three.js — MIT License",
};

// Trusted bundled prose only; no HTML injection or unregistered links.
function InlineText({text}:{text:string}) {
    return <>{text.split(/(`[^`]+`)/g).map((part,index)=>part.startsWith("`") && part.endsWith("`")
        ? <code key={index}>{part.slice(1,-1)}</code> : part)}</>;
}
function DocumentSections({text}:{text:string}) {
    const blocks=text.trim().split(/\n(?=#{1,2} )|\n\n+/);
    return <div className="document-sections">{blocks.flatMap((block,index)=> {
        const lines=block.split("\n");
        const heading=lines[0].match(/^(#{1,2}) (.+)$/);
        if(heading) {
            const title=heading[1].length===1 ? <h3 key={`${index}-title`}>{heading[2]}</h3> : <h4 key={`${index}-title`}>{heading[2]}</h4>;
            return [title,...lines.slice(1).filter(Boolean).map((line,i)=><p key={`${index}-${i}`}><InlineText text={line}/></p>)];
        }
        if(lines.every(line=>line.startsWith("- "))) return [<ul key={index}>{lines.map((line,i)=><li key={i}><InlineText text={line.slice(2)}/></li>)}</ul>];
        return [<p key={index}><InlineText text={lines.join(" ")}/></p>];
    })}</div>;
}

export function LegalDialog({kind,onClose}:{kind:LegalDocument;onClose:()=>void}) {
    const locale=useLocale();
    const title=t(kind==="privacy" ? "Privacy notice" : kind==="license" ? "Open-source license" : "Third-party notices");
    return <div className="log-viewer">
        <h2 id="dialog-title">{title}</h2>
        <DialogText id="modal-first" documentText label={title} text={kind==="license" ? license : undefined}>
            {kind==="notices" ? <article className="third-party-document" lang="en">
                <DocumentSections text={assetNotices}/>
                <section className="third-party-licenses">
                    <h3>Bundled component licenses</h3>
                    <p>The original copyright and license notices below are reproduced without modification. These licenses apply to the named components, not to Minecraft artwork or trademarks.</p>
                    {Object.entries(notices).sort(([a],[b])=>a.localeCompare(b)).map(([name,text])=>{
                        const file=name.split("/").at(-1)!;
                        return <section className="third-party-component" key={name}>
                            <h4>{componentNames[file] ?? file}</h4>
                            <p className="third-party-source">Notice file: {file}</p>
                            <pre lang="en">{text}</pre>
                        </section>;
                    })}
                </section>
            </article> : kind==="privacy" ? <article className="third-party-document" lang={locale}><DocumentSections text={locale==="zh-TW" ? privacyTraditional : locale==="zh-CN" ? privacyChinese : privacyEnglish}/></article> : undefined}
        </DialogText>
        <div className="config-actions"><StandardButton id="legal-close" scope="modal" onClick={onClose}>{t("Close")}</StandardButton></div>
    </div>;
}
