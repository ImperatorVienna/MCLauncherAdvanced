import { t } from "../i18n";
function FaceKey({ position }: {
    position: "south" | "east";
}) {
    return <svg className="face-key" viewBox="0 0 28 28" role="img" aria-label={position === "south" ? t("Bottom face button") : t("Right face button")}>
    <circle cx="14" cy="5" r="4"/><circle cx="5" cy="14" r="4"/>
    <circle cx="23" cy="14" r="4" className={position === "east" ? "lit" : ""}/>
    <circle cx="14" cy="23" r="4" className={position === "south" ? "lit" : ""}/>
  </svg>;
}
export function ControllerHints() {
    return <span className="controller-hints"><span>{t("D-pad / left stick: navigate")}</span><FaceKey position="south"/>{t("Confirm")}<FaceKey position="east"/>{t("Back")}<span>{t("Shoulder buttons: switch tabs")}</span></span>;
}
