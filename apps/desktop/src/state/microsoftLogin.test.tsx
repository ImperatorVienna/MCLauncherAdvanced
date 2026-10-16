// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, expect, test, vi } from "vitest";
import { useMicrosoftLogin, type LoginStatus } from "./microsoftLogin";
const native=vi.hoisted(()=>vi.fn());
vi.mock("../platform/desktop",()=>({isDesktop:()=>true,invokeNative:native}));
Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
const idle:LoginStatus={running:false,phase:"",userCode:null,verificationUri:null,expiresAt:null,error:null,completed:false};
afterEach(()=>{vi.useRealTimers();native.mockReset();});
test("late status cannot revive cancelled sign-in and completion reloads accounts once",async()=>{
    vi.useFakeTimers();let late!:(value:LoginStatus)=>void;
    let current=idle;
    native.mockImplementation(async(command:string)=>{
        if(command==="microsoft_login_status") return current;
        if(command==="start_microsoft_login") current={...idle,running:true,phase:"waiting",userCode:"TEST"};
        if(command==="cancel_microsoft_login") current=idle;
    });
    const complete=vi.fn().mockResolvedValue(undefined);let hook!:ReturnType<typeof useMicrosoftLogin>;
    function Harness(){hook=useMicrosoftLogin(complete);return null;}
    const host=document.createElement("div"),root=createRoot(host);
    await act(async()=>root.render(<Harness/>));
    await act(async()=>hook.start());expect(hook.status.running).toBe(true);
    native.mockImplementationOnce(()=>new Promise<LoginStatus>(resolve=>{late=resolve;}));
    await act(async()=>vi.advanceTimersByTime(1000));
    await act(async()=>hook.cancel());expect(hook.status.running).toBe(false);
    await act(async()=>late({...idle,running:true,phase:"waiting",userCode:"STALE"}));
    expect(hook.status.userCode).toBeNull();
    current={...idle,completed:true};
    await act(async()=>vi.advanceTimersByTime(1000));
    await act(async()=>vi.advanceTimersByTime(1000));
    expect(complete).toHaveBeenCalledOnce();expect(hook.status.completed).toBe(true);
    act(()=>root.unmount());
});
