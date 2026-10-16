// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import { SettingsPage } from "./SettingsPage";
import { Navigation } from "../components/Control";
import { FocusManager } from "../navigation/FocusManager";
import { readLocale, setLocale } from "../i18n";
import { iconNames } from "../state/instanceIcons";
test("language selection updates in place and restores keyboard/controller focus",async()=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    setLocale("en-US");
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host),manager=new FocusManager();
    const steam={ready:true,saving:false,status:{enabled:false,active:false},toggle:vi.fn()} as unknown as Parameters<typeof SettingsPage>[0]["steam"];
    await act(async()=>root.render(<Navigation.Provider value={manager}><SettingsPage steam={steam} preview controllerStatus="Device" onViewLogs={()=>{}} onBrowseControllerMod={()=>{}}/></Navigation.Provider>));
    act(()=>manager.focus("launcher-language"));
    act(()=>manager.dispatch("Confirm"));
    act(()=>manager.dispatch("NavigateDown"));
    await act(async()=>manager.dispatch("Confirm"));
    expect(readLocale()).toBe("zh-CN");
    expect(manager.activeId()).toBe("launcher-language");
    expect(iconNames.grass).not.toBe("Grass block");
    act(()=>manager.dispatch("Confirm"));
    act(()=>manager.dispatch("NavigateUp"));
    await act(async()=>manager.dispatch("Confirm"));
    expect(readLocale()).toBe("en-US");
    expect(iconNames.grass).toBe("Grass block");
    expect(host.textContent).toContain("Content download source");
    expect(host.querySelector("#offline-accounts-enabled")).toBeNull();
    expect(manager.activeId()).toBe("launcher-language");
    act(()=>root.unmount());host.remove();
});

