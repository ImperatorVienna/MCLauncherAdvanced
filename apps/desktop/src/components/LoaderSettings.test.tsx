// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import { LoaderSettings } from "./LoaderSettings";
import { Navigation } from "./Control";
import { FocusManager } from "../navigation/FocusManager";

test.each(["fabric","quilt","forge","neoforge"] as const)("%s locks family, allows explicit downgrade and disables installed version", async(kind)=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const invoke=vi.fn(async()=>[{version:"0.19.0",stable:true},{version:"0.18.0",stable:true}]);
    Object.assign(window,{__TAURI__:{core:{invoke}}});
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host), change=vi.fn(async()=>true), browse=vi.fn(), manager=new FocusManager();
    const item={id:"test",name:"test",minecraftVersion:"1.21.1",directoryName:"test",icon:"grass" as const,javaPath:"/opt/java/bin/java",loader:{kind,version:"0.19.0"}};
    try {
        await act(async()=>root.render(<Navigation.Provider value={manager}><LoaderSettings item={item} busy={false} installed onChange={change} onBrowseModMenu={browse}/></Navigation.Provider>));
        const shortcut=host.querySelector("#manage-loader-mod-menu") as HTMLButtonElement | null;
        expect(!!shortcut).toBe(kind==="fabric" || kind==="quilt");
        if(shortcut) {
            act(()=>manager.focus(shortcut.id));
            expect(document.activeElement).toBe(shortcut);
            await act(async()=>shortcut.click());
            expect(browse).toHaveBeenCalledOnce();
            expect(change).not.toHaveBeenCalled();
        }
        expect(host.querySelector("#manage-loader-kind")).toBeNull();
        expect(invoke).toHaveBeenCalledWith("list_loader_versions",{kind,game:"1.21.1"});
        expect((host.querySelector("#manage-loader-apply") as HTMLButtonElement).disabled).toBe(true);
        act(()=>(host.querySelector("#manage-loader-version") as HTMLButtonElement).click());
        act(()=>(host.querySelector("#manage-loader-version-option-1") as HTMLButtonElement).click());
        expect(change).not.toHaveBeenCalled();
        await act(async()=>(host.querySelector("#manage-loader-apply") as HTMLButtonElement).click());
        expect(change).toHaveBeenCalledWith({kind,version:"0.18.0"});
    } finally {act(()=>root.unmount());host.remove();delete (window as Window & {__TAURI__?:unknown}).__TAURI__;}
});

test.each(["forge", "neoforge"] as const)("%s requires Java before a version change", async kind => {
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    Object.assign(window,{__TAURI__:{core:{invoke:vi.fn(async()=>[{version:"2",stable:true}])}}});
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host), change=vi.fn(async()=>true);
    const item={id:"test",name:"test",minecraftVersion:"1.21.1",directoryName:"test",icon:"grass" as const,loader:{kind,version:"1"}};
    try {
        await act(async()=>root.render(<Navigation.Provider value={new FocusManager()}><LoaderSettings item={item} busy={false} installed onChange={change}/></Navigation.Provider>));
        expect((host.querySelector("#manage-loader-apply") as HTMLButtonElement).disabled).toBe(true);
        act(()=>(host.querySelector("#manage-loader-apply") as HTMLButtonElement).click());
        expect(change).not.toHaveBeenCalled();
    } finally {act(()=>root.unmount());host.remove();delete (window as Window & {__TAURI__?:unknown}).__TAURI__;}
});
