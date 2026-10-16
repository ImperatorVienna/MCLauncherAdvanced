import { t } from "../i18n";
export function LauncherUpdates() {
    return (<section className="placeholder" aria-label={t("Launcher updates")}>
      <p>{t("Launcher features, improvements, and fixes will appear here.")}</p>
      <div className="settings-row">{t("Updates are not available yet")}</div>
    </section>);
}
