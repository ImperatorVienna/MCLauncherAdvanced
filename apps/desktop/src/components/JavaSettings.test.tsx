// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import { JavaSettings } from "./JavaSettings";
import { Navigation } from "./Control";
import { FocusManager } from "../navigation/FocusManager";
test("recommendation never replaces a chosen Java or silently downloads it", async ()=>{
  Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
  Object.assign(window,{__TAURI__:{core:{invoke:vi.fn(async()=>21)}}});
  const host=document.createElement("div"); document.body.append(host);
  const root=createRoot(host), manager=new FocusManager(), change=vi.fn(), install=vi.fn();
  const inventory={loading:false,error:"",refresh:vi.fn(),runtimes:[{path:"/java8/bin/java",major:8,version:"1.8.0",source:"system" as const},{path:"/java21/bin/java",major:21,version:"21.0.1",source:"managed" as const}]};
  try {
    await act(async()=>root.render(<Navigation.Provider value={manager}><JavaSettings inventory={inventory} version="1.21" value="/java8/bin/java" onChange={change} onInstall={install} busy={false} preview={false}/></Navigation.Provider>));
    expect(host.querySelector("#main-java-download-version")!.textContent).toContain("Java 21");
    expect(host.querySelector("#main-java-select")!.textContent).toContain("1.8.0");
    expect(change).not.toHaveBeenCalled(); expect(install).not.toHaveBeenCalled();
    act(()=>(host.querySelector("#main-java-download") as HTMLButtonElement).click());
    expect(install).toHaveBeenCalledWith(21);
    act(()=>(host.querySelector("#main-java-select") as HTMLButtonElement).click());
    act(()=>(host.querySelector("#main-java-select-option-2") as HTMLButtonElement).click());
    expect(change).toHaveBeenCalledWith("/java21/bin/java");
  } finally {act(()=>root.unmount());host.remove();delete (window as Window & {__TAURI__?:unknown}).__TAURI__;}
});

test("a delayed recommendation does not overwrite an explicit download selection", async()=>{
  Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
  let recommend!: (value:number)=>void;
  Object.assign(window,{__TAURI__:{core:{invoke:vi.fn(()=>new Promise<number>(resolve=>{recommend=resolve;}))}}});
  const host=document.createElement("div");document.body.append(host);const root=createRoot(host);
  try {
    await act(async()=>root.render(<Navigation.Provider value={new FocusManager()}><JavaSettings inventory={{runtimes:[],loading:false,error:"",refresh:async()=>{}}} version="26.3" onChange={()=>{}} onInstall={()=>{}} busy={false} preview={false}/></Navigation.Provider>));
    act(()=>(host.querySelector("#main-java-download-version") as HTMLButtonElement).click());
    act(()=>(host.querySelector("#main-java-download-version-option-2") as HTMLButtonElement).click());
    expect(host.querySelector("#main-java-download-version")?.textContent).toContain("Java 17");
    await act(async()=>recommend(25));
    expect(host.querySelector("#main-java-download-version")?.textContent).toContain("Java 17");
  } finally {act(()=>root.unmount());host.remove();delete (window as Window & {__TAURI__?:unknown}).__TAURI__;}
});

test("installer requirement reuses the recommended download picker without selecting an available runtime", async()=>{
  Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
  Object.assign(window,{__TAURI__:{core:{invoke:vi.fn(async()=>25)}}});
  const host=document.createElement("div");document.body.append(host);const root=createRoot(host);
  const change=vi.fn(),install=vi.fn();
  try {
    await act(async()=>root.render(<Navigation.Provider value={new FocusManager()}><JavaSettings requiredForInstallation inventory={{runtimes:[{path:"/java25/bin/java",major:25,version:"25",source:"system"}],loading:false,error:"",refresh:async()=>{}}} version="26.3" onChange={change} onInstall={install} busy={false} preview={false}/></Navigation.Provider>));
    expect(host.querySelectorAll("#main-java-select")).toHaveLength(1);
    expect(host.querySelector("#main-java-download-version")?.textContent).toContain("Java 25");
    expect(host.querySelector("#main-java-select")?.textContent).not.toContain("/java25");
    expect(change).not.toHaveBeenCalled();expect(install).not.toHaveBeenCalled();
    act(()=>(host.querySelector("#main-java-download") as HTMLButtonElement).click());
    expect(install).toHaveBeenCalledWith(25);expect(change).not.toHaveBeenCalled();
  } finally {act(()=>root.unmount());host.remove();delete (window as Window & {__TAURI__?:unknown}).__TAURI__;}
});
