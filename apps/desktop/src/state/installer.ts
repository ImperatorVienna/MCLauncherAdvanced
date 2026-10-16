import { localizeMessage } from "../i18n";
import { useEffect, useRef, useState } from "react";
import { invokeNative } from "../platform/desktop";
export type Java = { path: string; major: number; version: string; source: "system" | "managed" };
export type Installed = { instanceId: string; gameDirectory: string; minecraftVersion: string; installed: boolean; java: Java | null };
export type InstallTask = { content?: boolean; running: boolean; cancelling: boolean; version: string; progress: {phase: string; completed: number; total: number}; error: string | null; result: Java | null };
export type OfficialVersion = { id: string; type: "release" | "snapshot"; releaseTime: string };
export function useInstaller(preview: boolean, revision: number) {
  // Explicit development-only visual demos; never invoke downloads or alter instance data.
  const demo = import.meta.env.DEV && preview ? (new URLSearchParams(window.location.search).get("launchDemo") === "repair" ? "progress" : new URLSearchParams(window.location.search).get("transferDemo")) : null;
  const [task, setTask] = useState<InstallTask | null>(() => demo === "progress" || demo === "failure" ? {
    running: demo === "progress", cancelling: false, version: "26.3",
    progress: { phase: "Downloading and verifying game files", completed: 181 * 1024 ** 2, total: 581 * 1024 ** 2 },
    error: demo === "failure" ? "Download failed: connection timed out" : null, result: null,
  } : null);
  const [installed, setInstalled] = useState<Installed[]>([]);
  const [error, setError] = useState("");
  const [starting, setStarting] = useState(false);
  const [supported, setSupported] = useState(false);
  const lock = useRef(false);
  const refreshRef = useRef<(() => Promise<void>) | null>(null);
  useEffect(() => {
    if (preview) return;
    let alive = true, timer = 0, previous = "", inventoryRequest = 0;
    const refresh = async () => {
      const request = ++inventoryRequest;
      try { const value = await invokeNative<Installed[]>("installed_versions"); if (alive && request === inventoryRequest) setInstalled(value); }
      catch (e) { if (alive && request === inventoryRequest) setError(localizeMessage(e)); }
    };
    refreshRef.current = refresh;
    void invokeNative<{supported: boolean}>("install_platform").then(p => { if (alive) setSupported(p.supported); }).catch(e => { if (alive) setError(localizeMessage(e)); });
    const poll = async () => {
      try {
        const value = await invokeNative<InstallTask>("installation_task");
        if (!alive) return;
        setTask(old => JSON.stringify(old) === JSON.stringify(value) ? old : value);
        const terminal = JSON.stringify([value.running, value.version, value.result, value.error]);
        if (terminal !== previous && !value.running) await refresh();
        previous = terminal;
      } catch (e) { if (alive) setError(localizeMessage(e)); }
      if (alive) timer = window.setTimeout(poll, 400);
    };
    void poll();
    return () => { alive = false; clearTimeout(timer); refreshRef.current = null; };
  }, [preview]);
  useEffect(() => { void refreshRef.current?.(); }, [revision]);
  async function start(id: string, javaMajor?: number) {
    if (lock.current || task?.running) return;
    lock.current = true; setStarting(true); setError("");
    try {
      await invokeNative(javaMajor === undefined ? "start_installation" : "start_java_installation", javaMajor === undefined ? { instanceId: id } : { major: javaMajor });
      setTask(await invokeNative<InstallTask>("installation_task"));
    } catch (e) { setError(localizeMessage(e)); }
    finally { lock.current = false; setStarting(false); }
  }
  async function refreshTask() {
    if(preview)return;
    try {setTask(await invokeNative<InstallTask>("installation_task"));}
    catch(e){setError(localizeMessage(e));}
  }
  async function cancel() {
    if (demo) { setTask(null); return; }
    try { await invokeNative("cancel_installation"); }
    catch (e) { setError(localizeMessage(e)); }
  }
  return { refreshTask, task, installed, error, supported, busy: starting || !!task?.running, start, startJava: (major: number) => start("", major), cancel };
}
