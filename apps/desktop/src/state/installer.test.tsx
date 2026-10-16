// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import { useInstaller, type Installed } from "./installer";

test("late installation inventory responses cannot overwrite the latest revision", async()=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const pending:Array<(value:Installed[])=>void>=[];
    const invoke=vi.fn((command:string)=>command==="installed_versions" ? new Promise(resolve=>pending.push(resolve)) : Promise.resolve(command==="install_platform" ? {supported:true} : {running:false,version:"",result:null,error:null}));
    Object.assign(window,{__TAURI__:{core:{invoke}}});
    let state!: ReturnType<typeof useInstaller>;
    function Harness({revision}:{revision:number}){state=useInstaller(false,revision);return null;}
    const host=document.createElement("div"),root=createRoot(host);
    try {
        await act(async()=>root.render(<Harness revision={0}/>));
        await act(async()=>root.render(<Harness revision={1}/>));
        const fresh=[{instanceId:"new",gameDirectory:"/test",minecraftVersion:"26.3",installed:true,java:null}];
        await act(async()=>pending.at(-1)!(fresh));
        for(const resolve of pending.slice(0,-1)) await act(async()=>resolve([]));
        expect(state.installed).toEqual(fresh);
    } finally {await act(async()=>root.unmount());delete (window as Window & {__TAURI__?:unknown}).__TAURI__;}
});
