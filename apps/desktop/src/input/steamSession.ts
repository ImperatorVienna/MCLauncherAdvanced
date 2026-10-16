import { localizeMessage } from "../i18n";
import { useEffect, useRef, useState } from "react";
import { invokeNative } from "../platform/desktop";
import type { FocusManager } from "../navigation/FocusManager";
export type SteamStatus = { enabled: boolean; launched: boolean; active: boolean };
export function isExitShortcut(e: KeyboardEvent) {
  return (e.altKey && !e.ctrlKey && !e.metaKey && e.key === "F4") ||
    ((e.ctrlKey || e.metaKey) && !e.altKey && !e.shiftKey && e.key.toLowerCase() === "q");
}
export function useSteamSession(manager: FocusManager, preview: boolean) {
  const [status, setStatus] = useState<SteamStatus>({ enabled: false, launched: false, active: false });
  const [ready, setReady] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const editing = useRef<HTMLElement | null>(null);
  const queue = useRef<Promise<unknown>>(Promise.resolve());
  const locked = useRef(false);
  const request = (show: boolean) => {
    queue.current = queue.current.catch(() => {}).then(() => invokeNative("steam_keyboard", { show })).catch(e => { setError(localizeMessage(e)); });
  };
  useEffect(() => {
    if (preview) { setStatus({ enabled: localStorage.getItem("launcher.steam-adaptation") === "true", launched: false, active: false }); setReady(true); return; }
    let alive = true;
    invokeNative<SteamStatus>("steam_status").then(s => { if (alive) { setStatus(s); setReady(true); } }).catch(e => { if (alive) setError(localizeMessage(e)); });
    return () => { alive = false; };
  }, [preview]);
  useEffect(() => {
    const ended = new WeakSet<Element>();
    const end = (cancel = false) => { if (!editing.current) return false; ended.add(editing.current); if (cancel) editing.current.dispatchEvent(new Event("launcher:text-cancel")); editing.current = null; if (status.active) request(false); return true; };
    const click = (e: Event) => {
      const target = e.target;
      if (!(target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) || target.disabled || target.readOnly || (target instanceof HTMLInputElement && !["text", "search", "email", "url", "tel", "password", "number"].includes(target.type))) return;
      if (editing.current === target) return;
      ended.delete(target); editing.current = target; target.dispatchEvent(new Event("launcher:text-begin")); if (status.active) { setError(""); request(true); }
    };
    const key = (e: KeyboardEvent) => {
      if (e.isComposing) return;
      if (isExitShortcut(e)) {
        if (preview) return;
        e.preventDefault(); e.stopImmediatePropagation();
        if (!e.repeat) void invokeNative("request_close").catch(err => setError(localizeMessage(err)));
      } else if (e.key === "Escape" && !editing.current && (document.activeElement instanceof HTMLInputElement || document.activeElement instanceof HTMLTextAreaElement) && !ended.has(document.activeElement)) {
        e.preventDefault(); e.stopImmediatePropagation(); manager.dispatch("Back");
      } else if ((e.key === "Escape" || e.key === "Enter") && editing.current) {
        e.preventDefault(); e.stopImmediatePropagation(); end(e.key === "Escape");
      }
    };
    const focus = (e: FocusEvent) => {
      if (editing.current && e.target !== editing.current) end();
      const field = e.target;
      if (field instanceof Element) ended.delete(field);
    };
    const input = (e: Event) => { if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) { ended.delete(e.target); editing.current = e.target; } };
    manager.textAction = action => {
      if (!editing.current) {
        const field = document.activeElement;
        if (action === "Back" && (field instanceof HTMLInputElement || field instanceof HTMLTextAreaElement) && !ended.has(field)) {
          editing.current = field; end(true); return true;
        }
        if (action === "Confirm" && (field instanceof HTMLInputElement || field instanceof HTMLTextAreaElement) && !field.disabled && !field.readOnly && (field instanceof HTMLTextAreaElement || ["text", "search", "email", "url", "tel", "password", "number"].includes(field.type))) {
          ended.delete(field); editing.current = field;
          field.dispatchEvent(new Event("launcher:text-begin"));
          if (status.active) { setError(""); request(true); }
          return true;
        }
        return false;
      }
      // Any new navigation ends editing, while confirm/back only dismiss the keyboard.
      end(action === "Back"); return action === "Confirm" || action === "Back";
    };
    document.addEventListener("click", click);
    document.addEventListener("input", input);
    document.addEventListener("beforeinput", click);
    window.addEventListener("keydown", key, true);
    document.addEventListener("focusin", focus);
    return () => { end(); manager.textAction = undefined; document.removeEventListener("click", click); document.removeEventListener("input", input); document.removeEventListener("beforeinput", click); window.removeEventListener("keydown", key, true); document.removeEventListener("focusin", focus); };
  }, [status.active, preview, manager]);
  async function toggle() {
    if (!ready || locked.current) return;
    locked.current = true; setSaving(true); setError("");
    try {
      if (preview) { localStorage.setItem("launcher.steam-adaptation", String(!status.enabled)); setStatus({ ...status, enabled: !status.enabled }); }
      else setStatus(await invokeNative<SteamStatus>("set_steam_adaptation", { enabled: !status.enabled }));
    } catch (e) { setError(localizeMessage(e)); }
    finally { locked.current = false; setSaving(false); }
  }
  return { status, ready, saving, error, toggle };
}
