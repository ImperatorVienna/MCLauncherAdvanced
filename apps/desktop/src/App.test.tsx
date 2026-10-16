// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test } from "vitest";
import { App } from "./App";
import { t } from "./i18n";

test("English route IDs retain Chinese labels and instance creation navigation", async () => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  localStorage.clear();
  const host = document.createElement("div"); document.body.append(host);
  const root = createRoot(host);
  const click = async (id: string) => { await act(async () => (host.querySelector(`#${id}`) as HTMLButtonElement).click()); };
  try {
    await act(async () => root.render(<App/>));
    await click("nav-mods");
    expect(host.querySelector("h1")?.textContent).toBe(t("Browse mods"));
    expect(host.querySelector("#tab-mods")?.getAttribute("aria-current")).toBe("page");
    expect(host.querySelector("#tab-mods")?.textContent).toBe(t("Download mods"));
    await click("nav-settings");
    expect(host.querySelector("h1")?.textContent).toBe(t("Settings"));
    await click("account");
    expect(host.querySelector("h1")?.textContent).toBe(t("Accounts"));
    await click("nav-java"); await click("tab-instances"); await click("config-create");
    expect(host.querySelector("#dialog-title")?.textContent).toBe(t("Download and install"));
    expect(host.querySelector("#config-save")?.textContent).toBe(t("Download and install now"));
    expect(host.querySelector("#config-save-install")).toBeNull();
    expect([...host.querySelectorAll("[id]")].some(n => /\p{Script=Han}/u.test(n.id))).toBe(false);
    await click("config-cancel");
    expect(host.querySelector('[role="dialog"]')).toBeNull();
  } finally { await act(async () => root.unmount()); host.remove(); }
});

test("switching instances replaces the complete settings subtree without duplicate controls", async () => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  const { previewKey } = await import("./state/library");
  const { vi } = await import("vitest");
  localStorage.clear();
  const items = [
    {id:"fabric",name:"Fabric test",loader:{kind:"fabric",version:"0.19.4"}},
    {id:"quilt",name:"Quilt test",loader:{kind:"quilt",version:"0.30.1"}},
    {id:"vanilla",name:"Vanilla test"},
  ].map(item=>({...item,minecraftVersion:"26.3",directoryName:item.name,icon:"grass"}));
  localStorage.setItem(previewKey,JSON.stringify({schemaVersion:2,revision:0,instances:items,selectedId:"fabric",lastLaunchedId:null}));
  vi.stubGlobal("fetch",vi.fn(async (url:string)=>({ok:true,json:async()=>url.endsWith("/versions/game") ? [{version:"26.3"}] : [{loader:{version:url.includes("quilt") ? "0.30.1" : "0.19.4",stable:true}}]})));
  const host=document.createElement("div");document.body.append(host);const root=createRoot(host);
  const click=async(id:string)=>{await act(async()=>(host.querySelector(`#${id}`) as HTMLButtonElement).click());};
  try {
    await act(async()=>root.render(<App/>)); await click("tab-instances");
    await click("main-java-download-version"); await click("main-java-download-version-option-0");
    expect(host.querySelector("#main-java-download-version")?.textContent).toContain("Java 25");
    for (const index of [1,0,2,1,2,0]) {
      await click("instance-select");await click(`instance-select-option-${index}`);
      expect(host.querySelectorAll(".loader-settings")).toHaveLength(1);
      expect(host.querySelectorAll(".java-settings")).toHaveLength(1);
      expect(host.querySelectorAll(".launch-settings")).toHaveLength(1);
      const ids=[...host.querySelectorAll("[id]")].map(el=>el.id);
      expect(new Set(ids).size).toBe(ids.length);
      expect(host.querySelector(".loader-settings")?.textContent).toContain(index===0 ? "Fabric 0.19.4" : index===1 ? "Quilt 0.30.1" : t("Installed mod loader: none"));
      expect(document.activeElement?.id).toBe("instance-select");
      expect(host.querySelector("#main-java-download-version")?.textContent).toContain("Java 21");
    }
  } finally {await act(async()=>root.unmount());host.remove();vi.unstubAllGlobals();localStorage.clear();}
});

test("launcher logs belong to Settings and Escape restores the yellow entry focus",async()=>{
  Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
  localStorage.clear();const host=document.createElement("div");document.body.append(host);const root=createRoot(host);
  try {
    await act(async()=>root.render(<App/>));
    act(()=>(host.querySelector("#nav-settings") as HTMLButtonElement).click());
    const button=host.querySelector("#launcher-view-logs") as HTMLButtonElement;
    expect(button.getAttribute("data-tone")).toBe("yellow");
    await act(async()=>{button.focus();button.click();});
    expect(host.querySelector('[role="dialog"] #log-content')).not.toBeNull();
    expect(host.querySelector("#dialog-title")?.textContent).toBe(t("Launcher logs"));
    act(()=>window.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true})));
    expect(host.querySelector('[role="dialog"]')).toBeNull();
    expect(document.activeElement).toBe(button);
  } finally {act(()=>root.unmount());host.remove();}
});
