import { t } from "../i18n";

// The only desktop IPC boundary; feature modules do not depend on instance storage.
type TauriWindow = Window & {
    __TAURI__?: {
        core: {
            invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
        };
    };
};
export const native = () => (window as TauriWindow).__TAURI__?.core;
export function invokeNative<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    const core = native();
    if (!core)
        return Promise.reject(new Error(t("This operation requires the desktop app")));
    return core.invoke<T>(command, args);
}
export const isDesktop = () => !!native();
