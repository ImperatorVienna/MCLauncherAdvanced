// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import { App } from "./App";
import { t } from "./i18n";
const state=vi.hoisted(()=>({value:{status:{phase:"available",version:"1.0.2",completed:0,total:100,error:null,announcement:{version:"1.0.2",body:"Release notes\n<script>untrusted</script>"}},error:"",pending:"",locked:false,action:vi.fn()}}));
vi.mock("./state/launcherUpdate",()=>({LauncherUpdateProvider:({children}:{children:unknown})=>children,useLauncherUpdate:()=>state.value}));
test("update notice, about focus, task isolation and failure focus recovery",async()=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});localStorage.clear();
    const host=document.createElement("div");document.body.append(host);const root=createRoot(host);
    const click=async(id:string)=>act(async()=>{const button=host.querySelector(`#${id}`) as HTMLButtonElement;button.focus();button.click();});
    const key=async(key:string)=>act(async()=>{window.dispatchEvent(new KeyboardEvent("keydown",{key,bubbles:true,cancelable:true}));});
    try{
        await act(async()=>root.render(<App/>));
        expect(host.querySelector("#nav-launcher-updates .update-badge")).not.toBeNull();
        await click("nav-settings");
        expect(host.querySelector("#launcher-check-updates")).toBeNull();
        expect(host.querySelector("#launcher-language")).not.toBeNull();
        await click("tab-about");
        expect(host.querySelector("#launcher-check-updates")).not.toBeNull();
        expect(host.querySelector("#project-homepage")).not.toBeNull();
        expect(host.querySelector("#launcher-language")).toBeNull();
        // Each document opens independently, including third-party notices first.
        for(const id of ["third-party-notices","project-license","third-party-notices"]) {
            await click(id);
            expect(host.querySelector('[role="dialog"]')).not.toBeNull();
            expect(host.querySelector(".legal-content")?.textContent?.length).toBeGreaterThan(100);
            expect(document.activeElement?.id).toBe("modal-first");
            expect(host.querySelector("main")?.inert).toBe(true);
            if(id==="project-license") expect(host.querySelector(".legal-content")?.textContent).toContain("GNU GENERAL PUBLIC LICENSE");
            else expect(host.querySelector(".legal-content")?.textContent).toContain("OFL.txt");
            await key("Escape");
            expect(host.querySelector('[role="dialog"]')).toBeNull();
            expect(document.activeElement?.id).toBe(id);
        }

        expect(host.querySelector("#nav-settings")?.className).toContain("selected");
        await click("nav-launcher-updates");
        expect(host.querySelector(".release-notes")?.textContent).toBe(state.value.status.announcement.body);
        expect(host.querySelector(".release-notes script")).toBeNull();
        expect(host.querySelector("#announcement-settings")?.getAttribute("data-tone")).toBe("green");
        await click("announcement-settings");
        expect(document.activeElement?.id).toBe("launcher-update-action");
        state.value={...state.value,locked:true,status:{...state.value.status,phase:"downloading"}};
        await act(async()=>root.render(<App/>));
        expect(document.activeElement?.id).toBe("launcher-cancel-update");
        expect(host.querySelector(".sidebar")?.hasAttribute("inert")).toBe(true);
        expect(host.querySelector(".page-scroll")?.hasAttribute("inert")).toBe(true);
        await key("ArrowUp");await key("Tab");await key("Escape");
        expect(document.activeElement?.id).toBe("launcher-cancel-update");
        await click("nav-java"); // Programmatic clicks must also be unable to navigate.
        expect(host.querySelector("h1")?.textContent).toBe(t("Settings"));
        await key("Tab");
        state.value={...state.value,locked:false,status:{...state.value.status,phase:"error"}};
        await act(async()=>root.render(<App/>));
        expect(host.querySelector(".page-scroll")?.hasAttribute("inert")).toBe(false);
        expect(document.activeElement?.id).toBe("launcher-update-action");
    }finally{await act(async()=>root.unmount());host.remove();}
});
