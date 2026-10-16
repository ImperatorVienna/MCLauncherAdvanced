import { useEffect, useRef, useState } from "react";
import { invokeNative } from "../platform/desktop";
import { localizeMessage } from "../i18n";
export type GameStatus = { running: boolean; preparing: boolean; instanceId: string | null; error: string | null; logPath: string | null };
export function useGameLaunch(preview: boolean) {
    const [status, setStatus] = useState<GameStatus>({ running: false, preparing: false, instanceId: null, error: null, logPath: null });
    const [commandError, setCommandError] = useState<string | null>(null);
    const [pending, setPending] = useState(false);
    const lock = useRef(false);
    useEffect(() => {
        if (preview) return;
        let active = true, timer: ReturnType<typeof setTimeout>;
        async function poll() {
            try { const next = await invokeNative<GameStatus>("game_status"); if (active && !lock.current) setStatus(next); }
            catch (error) { if (active) setStatus(s => ({ ...s, error: localizeMessage(error) })); }
            if (active) timer = setTimeout(poll, 750);
        }
        void poll(); return () => { active = false; clearTimeout(timer); };
    }, [preview]);
    async function launch(instanceId: string) {
        if (lock.current || status.running) return;
        lock.current = true; setPending(true); setCommandError(null);
        try {
            await invokeNative("launch_game", { instanceId });
            setStatus(await invokeNative<GameStatus>("game_status"));
        } catch (error) { setCommandError(localizeMessage(error)); }
        finally { lock.current = false; setPending(false); }
    }
    return { status: { ...status, error: commandError ?? status.error }, busy: pending || status.running, launch };
}
