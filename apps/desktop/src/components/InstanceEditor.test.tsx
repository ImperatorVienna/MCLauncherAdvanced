// @vitest-environment jsdom
import { t } from "../i18n";
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import { InstanceEditor } from "./InstanceEditor";
import { Navigation } from "./Control";
import { FocusManager } from "../navigation/FocusManager";

test("cascaded versions default the name, preserve custom names and select icons without submitting", async () => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  Object.assign(window, { __TAURI__: { core: { invoke: vi.fn(async () => [
    { id: "26.3", type: "release" }, { id: "1.21.1", type: "release" }, { id: "1.21", type: "release" },
    { id: "26.3-pre1", type: "snapshot" }, { id: "26.3-snapshot-1", type: "snapshot" }
  ]) } } });
  const host = document.createElement("div"); document.body.append(host);
  const root = createRoot(host), manager = new FocusManager(), save = vi.fn(); manager.enterModal();
  const choose = (id: string, index: number) => { act(() => (host.querySelector(`#${id}`) as HTMLButtonElement).click()); act(() => (host.querySelector(`#${id}-option-${index}`) as HTMLButtonElement).click()); };
  const submit = () => { choose("modal-java-select",0); act(() => host.querySelector("form")!.dispatchEvent(new Event("submit", {bubbles:true,cancelable:true}))); };
  try {
    await act(async () => root.render(<Navigation.Provider value={manager}><InstanceEditor busy={false} error="" onSave={save} onCancel={()=>{}} /></Navigation.Provider>));
    expect(host.querySelectorAll("input")).toHaveLength(1);
    expect((host.querySelector("input") as HTMLInputElement).value).toBe("26.3");
    choose("version-family", 1);
    expect((host.querySelector("input") as HTMLInputElement).value).toBe("1.21.1");
    choose("official-version", 1); choose("config-icon", 2);
    expect(save).not.toHaveBeenCalled(); submit();
    expect(save).toHaveBeenLastCalledWith(expect.objectContaining({name:"1.21",minecraftVersion:"1.21",icon:"cobblestone"}));
    choose("modal-first", 2); submit();
    expect(save).toHaveBeenLastCalledWith(expect.objectContaining({name:"26.3-pre1",minecraftVersion:"26.3-pre1"}));
    act(() => { const input = host.querySelector("input")!; Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,"value")!.set!.call(input,"\u81ea\u5b9a\u4e49"); input.dispatchEvent(new Event("input",{bubbles:true})); });
    choose("modal-first", 0); submit();
    expect(save).toHaveBeenLastCalledWith(expect.objectContaining({name:"\u81ea\u5b9a\u4e49",minecraftVersion:"26.3"}));
  } finally { act(() => root.unmount()); host.remove(); delete (window as Window & {__TAURI__?:unknown}).__TAURI__; }
});

test("editing retains the installed version and works without a catalog request", async () => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  const invoke = vi.fn(async () => { throw Error("offline"); });
  Object.assign(window, { __TAURI__: { core: { invoke } } });
  const host = document.createElement("div"); document.body.append(host);
  const root = createRoot(host), manager = new FocusManager(), save = vi.fn(); manager.enterModal();
  const item = { id: "existing", name: "\u751f\u5b58", minecraftVersion: "26.3", directoryName: "\u751f\u5b58", icon: "grass" as const };
  try {
    await act(async () => root.render(<Navigation.Provider value={manager}><InstanceEditor item={item} busy={false} error="" onSave={save} onCancel={()=>{}} /></Navigation.Provider>));
    expect(invoke).not.toHaveBeenCalled();
    expect(host.querySelector("#official-version")).toBeNull();
    expect(host.querySelector("#config-name")).toBeNull();
    expect(host.querySelector("#config-save")?.textContent).toBe(t("Save changes"));
    act(() => (host.querySelector("#config-save") as HTMLButtonElement).click());
    expect(save).toHaveBeenCalledWith(item);
  } finally { act(() => root.unmount()); host.remove(); delete (window as Window & {__TAURI__?:unknown}).__TAURI__; }
});

test("loader changes clear stale versions, prefer stable and block unsupported installs", async () => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  let finishFabric: (v: unknown) => void = () => {};
  const invoke = vi.fn((command: string, args?: {kind:string}) => {
    if (command === "list_minecraft_versions") return Promise.resolve([{id:"1.21.1",type:"release"}]);
    if (args?.kind === "fabric") return new Promise(resolve => { finishFabric=resolve; });
    return Promise.resolve([{version:"0.30.0-beta.1",stable:false},{version:"0.29.2",stable:true}]);
  });
  Object.assign(window,{__TAURI__:{core:{invoke}}});
  const host=document.createElement("div"); document.body.append(host);
  const root=createRoot(host), manager=new FocusManager(), save=vi.fn(); manager.enterModal();
  const choose=async (id:string,index:number) => {
    await act(async () => (host.querySelector("#"+id) as HTMLButtonElement).click());
    await act(async () => (host.querySelector(`#${id}-option-${index}`) as HTMLButtonElement).click());
  };
  try {
    await act(async () => root.render(<Navigation.Provider value={manager}><InstanceEditor busy={false} error="" onSave={save} onCancel={()=>{}}/></Navigation.Provider>));
    await choose("mod-loader",1);
    expect((host.querySelector("#config-save") as HTMLButtonElement).disabled).toBe(true);
    await choose("mod-loader",2);
    expect(host.querySelector("#mod-loader-version")?.textContent).toContain("0.29.2");
    await act(async () => finishFabric([{version:"0.16.14",stable:true}]));
    expect(host.querySelector("#mod-loader-version")?.textContent).toContain("0.29.2");
    await choose("modal-java-select",0);
    await act(async () => (host.querySelector("#config-save") as HTMLButtonElement).click());
    expect(save).toHaveBeenLastCalledWith(expect.objectContaining({loader:{kind:"quilt",version:"0.29.2"},name:"1.21.1-Quilt-0.29.2"}));
    await choose("mod-loader",1);
    await act(async () => finishFabric([]));
    expect((host.querySelector("#config-save") as HTMLButtonElement).disabled).toBe(true);
    await choose("mod-loader",0);
    expect(host.querySelector("#mod-loader-version")).toBeNull();
    expect((host.querySelector("#config-save") as HTMLButtonElement).disabled).toBe(true);
    await choose("modal-java-select",0);
    expect((host.querySelector("#config-save") as HTMLButtonElement).disabled).toBe(false);
  } finally { await act(async () => root.unmount()); host.remove(); delete (window as Window & {__TAURI__?:unknown}).__TAURI__; }
});

test("installation requires explicit Java choice and keeps the draft when native validation rejects it", async()=>{
  Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
  const invoke=vi.fn(async(command:string)=>{
    if(command==="list_minecraft_versions")return [{id:"1.21.1",type:"release"}];
    if(command==="recommended_java")return 21;
    if(command==="validate_installation_java")throw Error("Selected Java is unavailable; rescan and select Java again");
    return [{version:"52.1.0",stable:true}];
  });
  Object.assign(window,{__TAURI__:{core:{invoke}}});
  const host=document.createElement("div");document.body.append(host);const root=createRoot(host),save=vi.fn();
  const inventory={runtimes:[{path:"/java/bin/java",major:21,version:"21",source:"system" as const}],loading:false,error:"",refresh:async()=>{}};
  const choose=async(id:string,index:number)=>{await act(async()=>(host.querySelector(`#${id}`) as HTMLButtonElement).click());await act(async()=>(host.querySelector(`#${id}-option-${index}`) as HTMLButtonElement).click());};
  try{
    await act(async()=>root.render(<Navigation.Provider value={new FocusManager()}><InstanceEditor java={inventory} busy={false} error="" onSave={save} onCancel={()=>{}}/></Navigation.Provider>));
    expect((host.querySelector("#config-save") as HTMLButtonElement).disabled).toBe(true);
    await choose("mod-loader",1);
    await choose("modal-java-select",0);
    expect((host.querySelector("#config-save") as HTMLButtonElement).disabled).toBe(false);
    await choose("mod-loader",3);
    expect((host.querySelector("#config-save") as HTMLButtonElement).disabled).toBe(true);
    await act(async()=>(host.querySelector("#modal-java-select") as HTMLButtonElement).click());
    expect(host.querySelectorAll('[id^="modal-java-select-option-"]')).toHaveLength(1);
    await act(async()=>(host.querySelector("#modal-java-select-option-0") as HTMLButtonElement).click());
    await act(async()=>(host.querySelector("#config-save") as HTMLButtonElement).click());
    expect(invoke).toHaveBeenCalledWith("validate_installation_java",{version:"1.21.1",path:"/java/bin/java"});
    expect(save).not.toHaveBeenCalled();
    expect(host.querySelector('[role="alert"]')).not.toBeNull();
  }finally{act(()=>root.unmount());host.remove();delete (window as Window & {__TAURI__?:unknown}).__TAURI__;}
});
