import type { LegalDocument } from "../components/LegalDialog";
import { AboutLauncher } from "../components/AboutLauncher";
import { LauncherUpdate } from "../components/LauncherUpdate";
import { t } from "../i18n";

export function AboutPage({onDocument}:{onDocument:(kind:LegalDocument)=>void}) {
    return <section className="placeholder settings-page" aria-label={t("About")}>
        <LauncherUpdate/>
        <AboutLauncher onDocument={onDocument}/>
    </section>;
}
