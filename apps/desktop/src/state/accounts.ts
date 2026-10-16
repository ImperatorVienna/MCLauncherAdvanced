import { useCallback, useEffect, useRef, useState } from "react";
import { invokeNative, isDesktop } from "../platform/desktop";
import { localizeMessage, t } from "../i18n";
import { useMicrosoftLogin } from "./microsoftLogin";
export type MicrosoftAccount = { id: string; name: string };
export type Accounts = { revision: number; selected: string | null; microsoft?: MicrosoftAccount[] };
export const selectedMicrosoft = (data: Accounts) => data.microsoft?.find(a => `@${a.id}` === data.selected);
export const accountName = (data: Accounts) => selectedMicrosoft(data)?.name ?? null;
export const emptyAccounts = (): Accounts => ({ revision: 0, selected: null });
export function validPlayerName(name: string) { return /^[A-Za-z0-9_]{3,16}$/.test(name); }
const key = "launcher.accounts.v2";
export function validateAccounts(data: Accounts) {
    if (!data || !Number.isSafeInteger(data.revision) || data.revision < 0) throw Error(t("Invalid account data"));
    const microsoft=data.microsoft ?? [];
    if(!Array.isArray(microsoft) || microsoft.some(a=>!a || !/^[a-f0-9]{32}$/.test(a.id) || !validPlayerName(a.name)) || new Set(microsoft.map(a=>a.id)).size!==microsoft.length || microsoft.length>100) throw Error(t("Invalid account data"));
    if ((data.selected !== null && !microsoft.some(a=>`@${a.id}`===data.selected)) || (microsoft.length>0 && data.selected===null)) throw Error(t("Select an existing account"));
}
export function useAccounts() {
    const [data, setData] = useState(emptyAccounts);
    const [busy, setBusy] = useState(true);
    const [error, setError] = useState("");
    const [ready, setReady] = useState(false);
    const lock = useRef(false);
    useEffect(() => {
        let active = true;
        Promise.resolve().then(() => isDesktop() ? invokeNative<Accounts>("load_accounts") : JSON.parse(localStorage.getItem(key) ?? JSON.stringify(emptyAccounts()))).then(value => {
            validateAccounts(value); if (active) { setData(old=>value.revision>=old.revision ? value : old); setReady(true); }
        }).catch(e => { if (active) setError(localizeMessage(e)); }).finally(() => { if (active) setBusy(false); });
        return () => { active = false; };
    }, []);
    async function commit(next: Accounts) {
        if (!ready || lock.current) return false;
        lock.current = true; setBusy(true); setError("");
        try {
            validateAccounts(next);
            let saved: Accounts;
            if (isDesktop()) saved = await invokeNative<Accounts>("save_accounts", { accounts: next });
            else {
                const current = JSON.parse(localStorage.getItem(key) ?? JSON.stringify(emptyAccounts())) as Accounts;
                validateAccounts(current);
                if (current.revision !== next.revision) throw Error(t("Account data changed; reload and retry"));
                saved = { ...next, revision: next.revision + 1 }; localStorage.setItem(key, JSON.stringify(saved));
            }
            setData(saved); return true;
        } catch (e) { setError(localizeMessage(e)); return false; }
        finally { lock.current = false; setBusy(false); }
    }
    const refresh=useCallback(async () => {
        try {
            const value=isDesktop() ? await invokeNative<Accounts>("load_accounts") : JSON.parse(localStorage.getItem(key) ?? JSON.stringify(emptyAccounts()));
            validateAccounts(value); setData(old=>value.revision>=old.revision ? value : old); setReady(true); setError("");
        } catch(e) {setError(localizeMessage(e)); throw e;}
    },[]);
    const login=useMicrosoftLogin(refresh);
    return { data, busy: busy || !ready || login.busy, error, commit, refresh, login };
}
