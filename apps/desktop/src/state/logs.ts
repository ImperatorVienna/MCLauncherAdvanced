import { invokeNative, isDesktop } from "../platform/desktop";
import { t } from "../i18n";
export type LogRequest = { instanceId?: string; label?: string; initialName?: string };
export type LogEntry = { name: string; modified: number; size: number };
export type LogContent = { text: string; truncated: boolean };
export const listLogs = (request: LogRequest) => isDesktop()
    ? invokeNative<LogEntry[]>("list_logs", {instanceId:request.instanceId ?? null})
    : Promise.resolve([]);
export const readLog = (request: LogRequest, name: string) =>
    invokeNative<LogContent>("read_log", {instanceId:request.instanceId ?? null,name});
export const openLogLocation = (instanceId?: string, name?: string) =>
    invokeNative<void>("open_log_location", {instanceId:instanceId ?? null,name:name ?? null});
export async function copyLogText(text: string) {
    if (!navigator.clipboard?.writeText) throw Error(t("Clipboard is unavailable"));
    await navigator.clipboard.writeText(text);
}
