import { t } from "../i18n";
import type { OfficialVersion } from "./installer";
export const channels = ["release", "snapshot", "preview"] as const;
export function channelOf(v: OfficialVersion): string {
    return v.type === "release" ? "release" : /(?:-pre-?\d+|-rc-?\d+|pre-release|release candidate)/i.test(v.id) ? "preview" : "snapshot";
}
export function familyOf(v: OfficialVersion): string {
    const old = v.id.match(/^1\.(\d+)(?:[.\s-]|$)/);
    if (old)
        return `1.${old[1]}.x`;
    const modern = v.id.match(/^(\d{2,})\./);
    if (modern)
        return `${modern[1]}.x`;
    const weekly = v.id.match(/^(\d{2})w\d{2}/);
    return weekly ? `snapshots-20${weekly[1]}` : "historical-snapshots";
}
export const channelLabel = (channel: string) => channel === "release" ? t("Release") : channel === "snapshot" ? t("Snapshot") : t("Preview");
export const familyLabel = (family: string) => family === "historical-snapshots" ? t("Other historical snapshots") : family.startsWith("snapshots-") ? t("{value0} snapshots", { value0: family.slice(10) }) : family;
