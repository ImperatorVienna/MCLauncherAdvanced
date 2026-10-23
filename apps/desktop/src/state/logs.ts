import { invokeNative, isDesktop } from "../platform/desktop";
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
export { copyText as copyLogText } from "../platform/clipboard";
