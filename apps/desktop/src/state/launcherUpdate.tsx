import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from "react";
import { invokeNative, isDesktop } from "../platform/desktop";
export type UpdateStatus = {phase:string;version:string|null;completed:number;total:number;error:string|null;announcement?:{version:string;body:string}|null};
const initial:UpdateStatus={phase:"idle",version:null,completed:0,total:0,error:null};
function useUpdateState(){
    const [status,setStatus]=useState(initial),[error,setError]=useState(""),[pending,setPending]=useState("");
    const alive=useRef(false),inFlight=useRef(false),generation=useRef(0);
    useEffect(()=>{
        alive.current=true;
        let stopped=false,timer:ReturnType<typeof setTimeout>;
        const poll=async()=>{
            const request=generation.current;
            try {const next=await invokeNative<UpdateStatus>("launcher_update_status");if(!stopped&&request===generation.current&&!inFlight.current)setStatus(next);}
            catch(e){if(!stopped)setError(String(e));}
            if(!stopped)timer=setTimeout(()=>void poll(),750);
        };
        if(isDesktop())void poll();
        return()=>{stopped=true;alive.current=false;clearTimeout(timer);};
    },[]);
    const action=async(command:string)=>{
        if(inFlight.current)return;
        if(!isDesktop()){setError("This operation requires the desktop app");return;}
        inFlight.current=true;generation.current++;setPending(command);setError("");
        let accepted=false;
        if(command==="download_launcher_update")setStatus(previous=>({...previous,phase:"downloading"}));
        try{await invokeNative(command);accepted=true;const next=await invokeNative<UpdateStatus>("launcher_update_status");if(alive.current)setStatus(next);}
        catch(e){if(alive.current){setError(e instanceof Error?e.message:String(e));if(!accepted&&command==="download_launcher_update")setStatus(status);}}
        finally{inFlight.current=false;if(alive.current)setPending("");}
    };
    return {status,error,pending,action,locked:pending==="download_launcher_update"||["downloading","restarting"].includes(status.phase)};
}
type UpdateState=ReturnType<typeof useUpdateState>;
const UpdateContext=createContext<UpdateState>({status:initial,error:"",pending:"",locked:false,action:async()=>{}});
export function LauncherUpdateProvider({children}:{children:ReactNode}){
    const value=useUpdateState();return <UpdateContext.Provider value={value}>{children}</UpdateContext.Provider>;
}
export const useLauncherUpdate=()=>useContext(UpdateContext);
