// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect,test,vi } from "vitest";
import { invokeNative } from "../platform/desktop";
import { t } from "../i18n";
vi.mock("../platform/desktop",()=>({invokeNative:vi.fn()}));
import { ContentBrowser } from "./ContentBrowser";
import { searchContent, type SearchResult } from "../state/contentSource";
import { Navigation } from "../components/Control";
import { FocusManager } from "../navigation/FocusManager";
import type { Instance } from "../state/library";
vi.mock("../state/contentSource",()=>({searchContent:vi.fn()}));
test("changing shared instance discards old search results and applies new context",async()=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host), manager=new FocusManager();
    let resolveOld!:(value:SearchResult)=>void;
    vi.mocked(searchContent).mockImplementation(item=>item.id==="old" ? new Promise(resolve=>{resolveOld=resolve;}) : Promise.resolve({projects:[{id:"new-project",title:"New result",description:"new"}],more:false}));
    const item:Instance={id:"old",name:"Old",minecraftVersion:"1.21.1",directoryName:"Old",icon:"grass",loader:{kind:"fabric",version:"1"}};
    const render=(item:Instance)=>root.render(<Navigation.Provider value={manager}><ContentBrowser kind="mods" item={item} busy={false} preview selection={{value:item.name,options:[item.name],onChange:()=>{}}} onStarted={()=>{}}/></Navigation.Provider>);
    await act(async()=>render(item));
    const other={...item,id:"new",name:"New",minecraftVersion:"1.21.2"};
    await act(async()=>render(other));
    await act(async()=>resolveOld({projects:[{id:"old-project",title:"Old result",description:"old"}],more:false}));
    expect(host.textContent).toContain("New result");
    expect(host.textContent).not.toContain("Old result");
    expect(vi.mocked(searchContent).mock.calls.at(-1)?.[0]).toEqual(other);
    act(()=>root.unmount());host.remove();
});

test("pagination keeps focus while disabled and async results do not steal it",async()=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host),manager=new FocusManager();
    let resolveNext!:(value:SearchResult)=>void;
    vi.mocked(searchContent).mockImplementation((_item,_kind,_query,offset)=>offset ? new Promise(resolve=>{resolveNext=resolve;}) : Promise.resolve({projects:[{id:"first",title:"First",description:""}],more:true}));
    const item:Instance={id:"one",name:"One",minecraftVersion:"1.21.1",directoryName:"One",icon:"grass",loader:{kind:"fabric",version:"1"}};
    await act(async()=>root.render(<Navigation.Provider value={manager}><ContentBrowser kind="mods" item={item} busy={false} preview selection={{value:item.name,options:[item.name],onChange:()=>{}}} onStarted={()=>{}}/></Navigation.Provider>));
    const next=host.querySelector("#content-next") as HTMLButtonElement;
    act(()=>manager.focus(next.id));
    await act(async()=>next.click());
    expect(document.activeElement).toBe(next);expect(next.getAttribute("aria-disabled")).toBe("true");
    await act(async()=>resolveNext({projects:[],more:false}));
    expect(document.activeElement).toBe(next);
    // It remains focused but noninteractive; moving away makes it unreachable.
    act(()=>manager.focus("content-query"));
    expect(next.disabled).toBe(true);
    expect(manager.activeId()).toBe("content-query");
    act(()=>root.unmount());host.remove();
});

test("controller shortcut pre-fills and submits the query with the selected instance",async()=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host),manager=new FocusManager();
    vi.mocked(searchContent).mockClear();
    vi.mocked(searchContent).mockResolvedValue({projects:[],more:false});
    const item:Instance={id:"controller",name:"Controller",minecraftVersion:"1.21.1",directoryName:"Controller",icon:"grass",loader:{kind:"neoforge",version:"1"}};
    await act(async()=>root.render(<Navigation.Provider value={manager}><ContentBrowser initialQuery="Controlify" kind="mods" item={item} busy={false} preview selection={{value:item.name,options:[item.name],onChange:()=>{}}} onStarted={()=>{}}/></Navigation.Provider>));
    expect((host.querySelector("#content-query") as HTMLInputElement).value).toBe("Controlify");
    expect(vi.mocked(searchContent)).toHaveBeenCalledWith(item,"mods","Controlify",0,true,expect.any(AbortSignal));
    act(()=>root.unmount());host.remove();
});

test("pagination focuses first next-page download and last previous-page download",async()=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host),manager=new FocusManager();
    vi.mocked(searchContent).mockImplementation(async(_item,_kind,_query,offset)=>({
        projects:[{id:offset ? "next-first" : "first",title:"First",description:""},{id:offset ? "next-last" : "last",title:"Last",description:""}],more:offset===0
    }));
    const item:Instance={id:"paging",name:"Paging",minecraftVersion:"1.21.1",directoryName:"Paging",icon:"grass",loader:{kind:"fabric",version:"1"}};
    await act(async()=>root.render(<Navigation.Provider value={manager}><ContentBrowser kind="mods" item={item} busy={false} preview selection={{value:item.name,options:[item.name],onChange:()=>{}}} onStarted={()=>{}}/></Navigation.Provider>));
    act(()=>manager.focus("content-next"));
    await act(async()=>manager.dispatch("Confirm"));
    expect(manager.activeId()).toBe("download-next-first");
    act(()=>manager.focus("content-previous"));
    await act(async()=>manager.dispatch("Confirm"));
    expect(manager.activeId()).toBe("download-last");
    const first=host.querySelector("#download-first") as HTMLButtonElement;
    const last=host.querySelector("#download-last") as HTMLButtonElement;
    first.getBoundingClientRect=()=>({left:900,right:1000,top:100,bottom:140,width:100,height:40,x:900,y:100,toJSON:()=>{}}) as DOMRect;
    last.getBoundingClientRect=()=>({left:900,right:1000,top:200,bottom:240,width:100,height:40,x:900,y:200,toJSON:()=>{}}) as DOMRect;
    act(()=>manager.focus(first.id));
    act(()=>manager.dispatch("NavigateDown"));
    expect(manager.activeId()).toBe(last.id);
    act(()=>manager.dispatch("NavigateUp"));
    expect(manager.activeId()).toBe(first.id);
    act(()=>root.unmount());host.remove();
});


test.each(["mods","resource-packs","shaders"] as const)("%s catalog shows installed and update actions, preserving disabled focus",async kind=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host),manager=new FocusManager();
    const update={name:"old.zip",enabled:false,hash:"hash",project:"old",version:"next"};
    let completed=false;
    vi.mocked(searchContent).mockResolvedValue({projects:["new","installed","old"].map(id=>({id,title:id,description:""})),more:false});
    vi.mocked(invokeNative).mockImplementation(command=>Promise.resolve(command==="catalog_content_states" ? [
        {project:"new",installed:false,update:null},{project:"installed",installed:true,update:null},
        {project:"old",installed:true,update:completed ? null : update}
    ] : undefined));
    const item:Instance={id:"status",name:"Status",minecraftVersion:"1.21.1",directoryName:"Status",icon:"grass",loader:{kind:"fabric",version:"1"}};
    const render=(busy=false)=>root.render(<Navigation.Provider value={manager}><ContentBrowser kind={kind} item={item} busy={busy} preview={false} selection={{value:item.name,options:[item.name],onChange:()=>{}}} onStarted={()=>{}}/></Navigation.Provider>);
    await act(async()=>render());
    const installed=host.querySelector("#download-installed") as HTMLButtonElement;
    expect(installed.textContent).toBe(t("Installed"));expect(installed.disabled).toBe(true);
    expect(installed.dataset.tone).toBe("green");
    const button=host.querySelector("#download-old") as HTMLButtonElement;
    expect(button.textContent).toBe(t("Update content"));
    act(()=>manager.focus(button.id));
    await act(async()=>button.click());
    expect(invokeNative).toHaveBeenCalledWith("start_content_installation",{instanceId:item.id,kind,project:"old",update});
    await act(async()=>render(true));completed=true;await act(async()=>render());
    expect(button.textContent).toBe(t("Installed"));expect(manager.activeId()).toBe(button.id);
    vi.mocked(invokeNative).mockClear();await act(async()=>button.click());expect(invokeNative).not.toHaveBeenCalled();
    act(()=>manager.focus("content-query"));expect(button.disabled).toBe(true);
    act(()=>root.unmount());host.remove();
});

test("desktop pagination waits for identification and skips installed rows",async()=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host),manager=new FocusManager();
    let resolveStates!:(value:unknown)=>void;
    vi.mocked(searchContent).mockImplementation(async(_i,_k,_q,offset)=>({projects:(offset ? ["installed","available"] : ["first"]).map(id=>({id,title:id,description:""})),more:!offset}));
    vi.mocked(invokeNative).mockImplementation((_command,args)=> (args?.projects as string[])[0]==="installed" ? new Promise(resolve=>{resolveStates=resolve;}) : Promise.resolve([{project:"first",installed:false,update:null}]));
    const item:Instance={id:"paging",name:"Paging",minecraftVersion:"1.21.1",directoryName:"Paging",icon:"grass",loader:{kind:"fabric",version:"1"}};
    await act(async()=>root.render(<Navigation.Provider value={manager}><ContentBrowser kind="mods" item={item} busy={false} preview={false} selection={{value:item.name,options:[item.name],onChange:()=>{}}} onStarted={()=>{}}/></Navigation.Provider>));
    act(()=>manager.focus("content-next"));await act(async()=>manager.dispatch("Confirm"));
    expect(manager.activeId()).toBe("content-next");
    expect((host.querySelector("#download-available") as HTMLButtonElement).disabled).toBe(true);
    await act(async()=>resolveStates([{project:"installed",installed:true,update:null},{project:"available",installed:false,update:null}]));
    expect(manager.activeId()).toBe("download-available");
    act(()=>root.unmount());host.remove();
});

test("first visit identifies installed projects without opening management and ignores an old instance response",async()=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host),manager=new FocusManager();
    vi.mocked(searchContent).mockResolvedValue({projects:[{id:"iris",title:"Iris Shaders",description:""},{id:"sodium",title:"Sodium",description:""}],more:false});
    let resolveOld!:(value:unknown)=>void;
    vi.mocked(invokeNative).mockImplementation((command,args)=>{
        expect(command).toBe("catalog_content_states");
        return args?.instanceId==="first" ? new Promise(resolve=>{resolveOld=resolve;}) : Promise.resolve([{project:"iris",installed:false,update:null},{project:"sodium",installed:false,update:null}]);
    });
    const item:Instance={id:"first",name:"First",minecraftVersion:"1.21.1",directoryName:"First",icon:"grass",loader:{kind:"fabric",version:"1"}};
    const render=(id:string)=>root.render(<Navigation.Provider value={manager}><ContentBrowser kind="mods" item={{...item,id}} busy={false} preview={false} selection={{value:item.name,options:[item.name],onChange:()=>{}}} onStarted={()=>{}}/></Navigation.Provider>);
    await act(async()=>render("first"));
    expect(host.querySelectorAll(".content-project")).toHaveLength(2);
    expect(host.textContent).toContain("Iris Shaders");expect(host.textContent).toContain("Sodium");
    await act(async()=>resolveOld([{project:"iris",installed:true,update:null},{project:"sodium",installed:true,update:null}]));
    expect(host.querySelectorAll(".content-project")).toHaveLength(2);
    expect(host.querySelector("#download-iris")?.textContent).toBe(t("Installed"));
    await act(async()=>render("second"));
    expect(host.querySelector("#download-iris")?.textContent).toBe(t("Download and install content"));
    await act(async()=>render("first"));
    const pending=resolveOld;
    await act(async()=>render("second"));
    await act(async()=>pending([{project:"iris",installed:true,update:null},{project:"sodium",installed:true,update:null}]));
    expect(host.querySelector("#download-iris")?.textContent).toBe(t("Download and install content"));
    act(()=>root.unmount());host.remove();
});
