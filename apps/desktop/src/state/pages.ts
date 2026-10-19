import { t } from "../i18n";
const labels = {
  play: "Play", instances: "Instances", "instance-mods": "Manage mods", "instance-resource-packs": "Manage resource packs", "instance-shaders": "Manage shaders", "patch-notes": "Patch notes", accounts: "Accounts", skins: "Skins", mods: "Browse mods", "resource-packs": "Browse resource packs", shaders: "Browse shaders", "launcher-updates": "What's new?", settings: "Settings", about: "About",
} as const;
export type PageId = keyof typeof labels;
export const pageLabel = (page: string) => t(labels[page as PageId] ?? "Page");

export const tabLabel = (page: string) => page === "launcher-updates" ? t("Launcher updates") : page === "accounts" ? t("Sign in") : page === "mods" ? t("Download mods") : page === "resource-packs" ? t("Download resource packs") : page === "shaders" ? t("Download shader packs") : pageLabel(page);
