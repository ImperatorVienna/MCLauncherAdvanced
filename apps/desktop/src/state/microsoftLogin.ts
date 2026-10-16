import { useCallback, useEffect, useRef, useState } from "react";
import { invokeNative, isDesktop } from "../platform/desktop";
import { localizeMessage } from "../i18n";
export type LoginStatus = { running: boolean; phase: string; userCode: string | null; verificationUri: string | null; expiresAt: number | null; error: string | null; completed: boolean };
const empty = (): LoginStatus => ({ running:false,phase:"",userCode:null,verificationUri:null,expiresAt:null,error:null,completed:false });
/** Polls only public status. Native code owns the authorization and all credentials. */
export function useMicrosoftLogin(onComplete: () => Promise<void>) {
    const [status,setStatus]=useState(empty), [pending,setPending]=useState(false), [error,setError]=useState("");
    const complete=useRef(onComplete); complete.current=onComplete;
    const alive=useRef(false), epoch=useRef(0), lock=useRef(false), delivered=useRef(false);
    const poll=useCallback(async (version:number) => {
        const next=await invokeNative<LoginStatus>("microsoft_login_status");
        if (!alive.current || version!==epoch.current) return;
        if(next.completed && !delivered.current) { await complete.current(); delivered.current=true; }
        if(alive.current && version===epoch.current) setStatus(next);
    },[]);
    useEffect(()=>{
        alive.current=true;
        if(!isDesktop()) return ()=>{alive.current=false;};
        let timer:ReturnType<typeof setTimeout>;
        const tick=async()=>{
            const version=epoch.current;
            try {if(!lock.current) await poll(version);} catch(e) {if(alive.current && version===epoch.current) setError(localizeMessage(e));}
            if(alive.current) timer=setTimeout(()=>void tick(),1000);
        };
        void tick();
        return ()=>{alive.current=false;clearTimeout(timer);epoch.current++;};
    },[poll]);
    async function command(name:string) {
        if(lock.current) return;
        lock.current=true;setPending(true);setError("");
        const version=++epoch.current;
        try {
            await invokeNative(name);
            if(name==="start_microsoft_login") delivered.current=false;
            await poll(version);
        } catch(e) {if(alive.current) setError(localizeMessage(e));}
        finally {lock.current=false;if(alive.current) setPending(false);}
    }
    return {status,busy:pending || status.running,error:error || (status.error ? localizeMessage(status.error):""),
        start:()=>command("start_microsoft_login"),cancel:()=>command("cancel_microsoft_login"),
        open:async()=>{try{await invokeNative("open_microsoft_login");}catch(e){setError(localizeMessage(e));}}};
}
