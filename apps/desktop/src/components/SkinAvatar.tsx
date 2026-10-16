import { t } from "../i18n";
/** Minecraft skin head preview, not a Microsoft/Xbox profile picture.
 * Supply a head image rendered by the future skin module (including hat layer).
 */
export function SkinAvatar({ headUrl }: {
    headUrl?: string;
}) {
    return headUrl ? (<img className="skin-avatar" src={headUrl} alt={t("Minecraft skin avatar")}/>) : (<span className="avatar skin-avatar-placeholder" title={t("Skin avatar is not available yet")}>?</span>);
}
