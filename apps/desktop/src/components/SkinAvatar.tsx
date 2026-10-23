import { t } from "../i18n";
/** Placeholder while the account skin is unavailable; portraits render loaded skins. */
export function SkinAvatar() {
    return <span className="avatar skin-avatar-placeholder" title={t("Skin avatar is not available yet")}>?</span>;
}
