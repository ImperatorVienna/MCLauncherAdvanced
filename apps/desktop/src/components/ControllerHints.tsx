import { ControllerLegendSymbol } from "./ControllerSymbol";
import { ControllerIllustration } from "./ControllerIllustration";
import { t } from "../i18n";

import { controllerStyle } from "./controllerAppearance";
export function ControllerHints({device}:{device:string}) {
    const style=controllerStyle(device);
    return <div className="controller-guides"><div className="controller-guide">
            <span className="controller-guide-name">{device || t("Generic controller guide")}</span>
            <div className="controller-guide-body">
                <ControllerIllustration style={style}/>
                <dl className="controller-bindings">
                    <div><dt>{t("D-pad / left stick")}</dt><dd>{t("Navigate")}</dd></div>
                    <div><dt>{style.family==="PlayStation" ? <ControllerLegendSymbol kind="cross" label={t("Cross button")}/> : style.family ? style.south : t("Bottom face button")}</dt><dd>{t("Confirm")}</dd></div>
                    <div><dt>{style.family==="PlayStation" ? <ControllerLegendSymbol kind="circle" label={t("Circle button")}/> : style.family ? style.east : t("Right face button")}</dt><dd>{t("Back")}</dd></div>
                    <div><dt>{style.left} / {style.right}</dt><dd>{t("Switch tabs")}</dd></div>
                    <div><dt>{t("Right stick")}</dt><dd>{t("Scroll up / down")}</dd></div>
                    <div><dt><ControllerLegendSymbol kind={style.family==="Nintendo" ? "plus" : "menu"} label={t("Menu button")}/></dt><dd>{t("Open Settings")}</dd></div>
                </dl>
            </div>
        </div><p className="settings-description">{t("Button artwork follows the reported device name. Virtual controllers may appear as another model; confirmation uses the bottom face button and back uses the right face button.")}</p></div>;
}
