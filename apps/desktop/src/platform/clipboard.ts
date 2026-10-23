import { invokeNative, isDesktop } from "./desktop";
import { t } from "../i18n";
export async function copyText(text: string) {
    try {
        if (isDesktop() && await invokeNative<boolean>("copy_clipboard_text", {text})) return;
        if (!navigator.clipboard?.writeText) throw Error();
        await navigator.clipboard.writeText(text);
    } catch {
        throw Error(t("Clipboard is unavailable"));
    }
}
