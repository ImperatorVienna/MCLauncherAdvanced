// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, expect, test, vi } from "vitest";
import { LauncherUpdateProvider } from "../state/launcherUpdate";
import { LauncherUpdate } from "./LauncherUpdate";
import { Navigation } from "./Control";
import { FocusManager } from "../navigation/FocusManager";
import { setLocale, t } from "../i18n";
const api=vi.hoisted(()=>({invoke:vi.fn()}));
vi.mock("../platform/desktop",()=>({isDesktop:()=>true,invokeNative:(command:string)=>api.invoke(command)}));
afterEach(()=>{vi.useRealTimers();api.invoke.mockReset();});
test("download completion never offers a separate restart action",async()=>{
    vi.useFakeTimers();Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});setLocale("en-US");
    let phase="available";
    api.invoke.mockImplementation(async(command:string)=>{
        if(command==="download_launcher_update")phase="downloading";
        return {phase,version:"1.0.0",completed:0,total:100,error:null};
    });
    const host=document.createElement("div");document.body.append(host);const root=createRoot(host),manager=new FocusManager();
    await act(async()=>root.render(<Navigation.Provider value={manager}><LauncherUpdateProvider><LauncherUpdate/></LauncherUpdateProvider></Navigation.Provider>));
    act(()=>manager.focus("launcher-update-action"));
    await act(async()=>manager.dispatch("Confirm"));
    expect(host.querySelector("#launcher-update-action")?.getAttribute("aria-disabled")).toBe("true");
    expect(manager.activeId()).toBe("launcher-update-action");
    await act(async()=>manager.dispatch("Confirm"));
    expect(api.invoke.mock.calls.filter(([c])=>c==="download_launcher_update")).toHaveLength(1);
    phase="restarting";await act(async()=>{await vi.advanceTimersByTimeAsync(750);});
    expect(manager.activeId()).toBe("launcher-update-action");
    expect(host.querySelector("#launcher-update-action")?.textContent).toBe("Download launcher update");
    await act(async()=>manager.dispatch("Confirm"));
    expect(api.invoke).not.toHaveBeenCalledWith("apply_launcher_update");
    act(()=>root.unmount());host.remove();
});
test("an empty release channel is not reported as up to date and polling stops on unmount",async()=>{
    vi.useFakeTimers();Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});setLocale("en-US");
    api.invoke.mockResolvedValue({phase:"unpublished",version:null,completed:0,total:0,error:null});
    const host=document.createElement("div"),root=createRoot(host);document.body.append(host);
    await act(async()=>root.render(<Navigation.Provider value={new FocusManager()}><LauncherUpdateProvider><LauncherUpdate/></LauncherUpdateProvider></Navigation.Provider>));
    expect(host.textContent).toContain("No public stable release is available yet.");
    expect(host.textContent).not.toContain("The launcher is up to date.");
    expect(host.querySelector("#launcher-update-action")).toBeNull();
    act(()=>root.unmount());const count=api.invoke.mock.calls.length;
    await vi.advanceTimersByTimeAsync(3000);expect(api.invoke).toHaveBeenCalledTimes(count);host.remove();
});

test("update statuses and verification failures follow the selected interface language",async()=>{
    vi.useFakeTimers();Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});setLocale("en-US");
    api.invoke.mockResolvedValue({phase:"error",version:null,completed:0,total:0,error:"The release package has no SHA-256 digest"});
    const host=document.createElement("div"),root=createRoot(host);document.body.append(host);
    await act(async()=>root.render(<Navigation.Provider value={new FocusManager()}><LauncherUpdateProvider><LauncherUpdate/></LauncherUpdateProvider></Navigation.Provider>));
    expect(host.textContent).toContain("The release package has no SHA-256 digest");
    act(()=>setLocale("zh-CN"));
    expect(host.textContent).toContain(t("The release package has no SHA-256 digest"));
    expect(host.textContent).not.toContain("The release package has no SHA-256 digest");
    act(()=>setLocale("en-US"));
    expect(host.textContent).toContain("The release package has no SHA-256 digest");
    act(()=>root.unmount());host.remove();
});
