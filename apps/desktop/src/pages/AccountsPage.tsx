import { useEffect, useContext, useLayoutEffect, useRef, useState } from "react";
import { useAccounts } from "../state/accounts";
import { SelectMenu } from "../components/SelectMenu";
import { StandardButton } from "../components/StandardButton";
import { Navigation } from "../components/Control";
import { isDesktop } from "../platform/desktop";
import { copyText } from "../platform/clipboard";
import { localizeMessage, t } from "../i18n";
export function AccountsPage({ accounts }: { accounts: ReturnType<typeof useAccounts> }) {
    const [deleting, setDeleting] = useState(false);
    const manager = useContext(Navigation);
    const [copied, setCopied] = useState(false), [copyError, setCopyError] = useState("");
    const copyGeneration = useRef(0);
    useEffect(() => { copyGeneration.current++; setCopied(false); setCopyError(""); return () => { copyGeneration.current++; }; }, [accounts.login.status.userCode, accounts.login.status.running]);
    async function copyCode() {
        const code = accounts.login.status.userCode, generation = copyGeneration.current;
        if (!code) return;
        try { await copyText(code); if (generation === copyGeneration.current) { setCopied(true); setCopyError(""); } }
        catch (e) { if (generation === copyGeneration.current) { setCopied(false); setCopyError(localizeMessage(e)); } }
    }
    const { data, busy, error, commit, login } = accounts;
    const options=(data.microsoft ?? []).map(a=>`@${a.id}`);
    const labelForValue=(value:string) => data.microsoft?.find(a=>`@${a.id}`===value)?.name ?? t("No accounts yet");
    const wasSigningIn=useRef(false);
    useLayoutEffect(()=>{
        if(login.status.running) manager.focus(login.status.phase==="waiting" ? "account-login-open" : "account-login-cancel");
        else if(login.status.completed) {manager.focus("account-select");}
        else if(wasSigningIn.current) manager.focus("account-login");
        wasSigningIn.current=login.status.running;
    },[login.status.running,login.status.phase,login.status.completed,manager]);
    useLayoutEffect(()=>{
        if(!login.status.running) return;
        const cancel=()=>{void login.cancel();return true;};
        manager.cancelEditor=cancel;
        return ()=>{if(manager.cancelEditor===cancel) manager.cancelEditor=undefined;};
    },[login.status.running,login.cancel,manager]);
    useLayoutEffect(() => { if (deleting) manager.focus("account-cancel"); }, [deleting, manager]);
    const cancelDeletion=() => {
        if(!busy) {setDeleting(false);requestAnimationFrame(()=>manager.focus("account-delete"));}
        return true;
    };
    useLayoutEffect(()=>{
        if(!deleting)return;
        manager.cancelEditor=cancelDeletion;
        return()=>{if(manager.cancelEditor===cancelDeletion)manager.cancelEditor=undefined;};
    },[deleting,busy,manager]);
    async function remove() {
        const microsoft=(data.microsoft ?? []).filter(a=>`@${a.id}`!==data.selected);
        if(await commit({...data,microsoft,selected:microsoft[0] ? `@${microsoft[0].id}` : null})) {
            setDeleting(false);
            requestAnimationFrame(()=>manager.focus(microsoft.length ? "account-select" : "account-login"));
        }
    }
    return <section className="accounts-page" aria-label={t("Accounts")}>
        <div className="instance-management-row">
            <SelectMenu id="account-select" compact showIcons={false} label={t("Select account")} options={options} value={data.selected ?? ""} labelForValue={labelForValue} disabled={busy || deleting || !options.length} onChange={selected=>{void commit({...data,selected});}}/>
            <StandardButton id="account-delete" tone="red" disabled={busy || deleting || !data.selected} onClick={()=>setDeleting(true)}>{t("Delete")}</StandardButton>
        </div>
        <div className="config-actions">
            <StandardButton id="account-login" tone="green" disabled={!isDesktop() || busy || deleting} onClick={()=>void login.start()}>{t("Sign in")}</StandardButton>
        </div>
        {login.status.running && <div className="account-editor" aria-live="polite">
            <p>{t(login.status.phase==="waiting" ? "Open the Microsoft page and enter this code to sign in." : login.status.phase==="verifying" ? "Verifying Xbox and Minecraft account…" : "Requesting a Microsoft sign-in code…")}</p>
            {login.status.userCode && <><p className="sign-in-code">{login.status.userCode}</p><p>{login.status.verificationUri}</p></>}
            <div className="config-actions" data-navigation-actions>
                {login.status.userCode && <StandardButton id="account-login-open" tone="green" onClick={()=>void login.open()}>{t("Open sign-in page")}</StandardButton>}
                {login.status.userCode && <StandardButton id="account-login-copy" onClick={() => void copyCode()}>{t(copied ? "Copied" : "Copy authorization code")}</StandardButton>}
                <StandardButton id="account-login-cancel" onClick={()=>void login.cancel().then(()=>requestAnimationFrame(()=>manager.focus("account-login")))}>{t("Cancel")}</StandardButton>
            </div>
        </div>}
        {copyError && <p className="config-error" role="alert">{copyError}</p>}
        {login.error && <p className="config-error" role="alert">{login.error}</p>}
        {deleting && <div className="account-editor">
            <p>{t("Delete this account? Game instances and saves will not be deleted.")}</p>
            <div className="config-actions" data-navigation-actions>
                <StandardButton id="account-save" tone="red" disabled={busy} onClick={()=>void remove()}>{t("Delete")}</StandardButton>
                <StandardButton id="account-cancel" disabled={busy} onClick={cancelDeletion}>{t("Cancel")}</StandardButton>
            </div>
        </div>}
        {error && <p className="config-error" role="alert">{error}</p>}
    </section>;
}
