// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import { t, setLocale } from "../i18n";
import { InstanceContent } from "./InstanceContent";
import { Navigation } from "../components/Control";
import { FocusManager } from "../navigation/FocusManager";
import { invokeNative } from "../platform/desktop";
vi.mock("../platform/desktop",()=>({invokeNative:vi.fn()}));
const listing=(name:string,enabled=true)=>({directory:"/fixture/mods",entries:[{name,enabled,conflict:false}]});
test("late scans cannot cross instances; toggle preserves control identity and busy blocks writes",async()=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host), manager=new FocusManager();
    let resolveOld!:(value:unknown)=>void;
    const invoke=vi.mocked(invokeNative);
    invoke.mockImplementation((command,args)=>{
        if(args?.instanceId==="old")return new Promise(resolve=>{resolveOld=resolve;});
        return Promise.resolve(listing("new.jar",command!=="set_instance_content_enabled"));
    });
    const render=(id:string,busy=false)=>root.render(<Navigation.Provider value={manager}><InstanceContent title="instance-mods" instanceId={id} selection={{options:[id],value:id,onChange:()=>{}}} busy={busy} preview={false} hasLoader/></Navigation.Provider>);
    await act(async()=>render("old"));
    await act(async()=>render("new"));
    await act(async()=>resolveOld(listing("old.jar")));
    expect(host.textContent).not.toContain("old.jar");
    const checkbox=host.querySelector('[role="checkbox"]') as HTMLButtonElement;
    expect(checkbox.textContent).toBe("new.jar");
    await act(async()=>{checkbox.focus();checkbox.click();});
    expect(invoke).toHaveBeenCalledWith("set_instance_content_enabled",{instanceId:"new",kind:"mods",name:"new.jar",enabled:false});
    expect(host.querySelector('[role="checkbox"]')).toBe(checkbox);
    expect(checkbox.getAttribute("aria-checked")).toBe("false");
    await act(async()=>render("new",true));
    invoke.mockClear();
    await act(async()=>checkbox.click());
    expect(invoke).not.toHaveBeenCalled();
    act(()=>root.unmount());host.remove();
});

test.each(["instance-resource-packs","instance-shaders"])("%s lists files without enable controls",async title=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    vi.mocked(invokeNative).mockResolvedValue(listing("Example.zip"));
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host),manager=new FocusManager();
    await act(async()=>root.render(<Navigation.Provider value={manager}><InstanceContent title={title} instanceId="pack-test" selection={{options:["Test"],value:"Test",onChange:()=>{}}} busy={false} preview={false} hasLoader/></Navigation.Provider>));
    expect(host.textContent).toContain("Example.zip");
    expect(host.querySelector('[role="checkbox"]')).toBeNull();
    expect(host.textContent).not.toContain(t("Checked files are available to the game. Unchecked files are kept separately in this instance and can be restored."));
    expect(host.querySelector("#content-open")).not.toBeNull();
    expect(host.querySelector("#content-refresh")).not.toBeNull();
    act(()=>root.unmount());host.remove();
});

test.each(["instance-mods","instance-resource-packs","instance-shaders"])("%s deletes files and restores focus to refresh when empty",async title=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const invoke=vi.mocked(invokeNative);
    invoke.mockImplementation(command=>Promise.resolve(command==="delete_instance_content" ? {directory:"/fixture",entries:[]} : listing("Example.zip")));
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host),manager=new FocusManager();
    await act(async()=>root.render(<Navigation.Provider value={manager}><InstanceContent title={title} instanceId="delete-test" selection={{options:["Test"],value:"Test",onChange:()=>{}}} busy={false} preview={false} hasLoader/></Navigation.Provider>));
    expect(host.querySelector("#content-check-updates")!==null).toBe(true);
    expect((host.querySelector("#content-download") as HTMLButtonElement).disabled).toBe(true);
    const button=host.querySelector('[id^="content-delete-"]') as HTMLButtonElement;
    act(()=>manager.focus(button.id));
    await act(async()=>button.click());
    expect(invoke).toHaveBeenCalledWith("delete_instance_content",{instanceId:"delete-test",kind:title.replace("instance-",""),name:"Example.zip",enabled:true});
    expect(host.querySelector('[id^="content-delete-"]')).toBeNull();
    expect(manager.activeId()).toBe("content-refresh");
    act(()=>root.unmount());host.remove();
});

test.each(["mods","resource-packs","shaders"])("update action for %s appears before delete and keeps file identity",async(kind)=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const update={name:"renamed.jar",enabled:true,hash:"a".repeat(40),project:"project",version:"new"};
    const invoke=vi.mocked(invokeNative);
    invoke.mockImplementation(command=>Promise.resolve(command==="check_content_updates" ? {updates:[update],unmatched:0} : command==="start_content_installation" ? undefined : listing("renamed.jar")));
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host),manager=new FocusManager(),browse=vi.fn();
    await act(async()=>root.render(<Navigation.Provider value={manager}><InstanceContent title={`instance-${kind}`} instanceId="update-test" selection={{options:["Test"],value:"Test",onChange:()=>{}}} busy={false} preview={false} hasLoader={kind==="mods"} onBrowse={browse}/></Navigation.Provider>));
    act(()=>(host.querySelector("#content-download") as HTMLButtonElement).click());
    expect(browse).toHaveBeenCalledOnce();
    await act(async()=>(host.querySelector("#content-check-updates") as HTMLButtonElement).click());
    expect(invoke).toHaveBeenCalledWith("check_content_updates",{instanceId:"update-test",kind});
    const button=host.querySelector('[id^="content-update-"]') as HTMLButtonElement;
    expect(button.nextElementSibling?.id).toBe("content-delete-renamed.jar");
    await act(async()=>button.click());
    expect(invoke).toHaveBeenCalledWith("start_content_installation",{instanceId:"update-test",kind,project:"project",update});
    act(()=>root.unmount());host.remove();
});

test("completed update restores focus to a remaining file action after filename changes",async()=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    let completed=false;
    const invoke=vi.mocked(invokeNative);
    invoke.mockImplementation(command=>Promise.resolve(command==="check_content_updates" ? {updates:[{name:"old.jar",enabled:true,hash:"a".repeat(40),project:"p",version:"v2"}],unmatched:0} : listing(completed?"new.jar":"old.jar")));
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host),manager=new FocusManager();
    const render=(busy=false,refreshKey="initial")=>root.render(<Navigation.Provider value={manager}><InstanceContent title="instance-mods" instanceId="focus-test" selection={{options:["Test"],value:"Test",onChange:()=>{}}} busy={busy} refreshKey={refreshKey} preview={false} hasLoader/></Navigation.Provider>);
    await act(async()=>render());
    await act(async()=>(host.querySelector("#content-check-updates") as HTMLButtonElement).click());
    act(()=>manager.focus("content-update-old.jar"));
    await act(async()=>render(true,"running"));
    expect(manager.activeId()).toBe("content-update-old.jar");
    completed=true;
    await act(async()=>render(false,"complete"));
    expect(manager.activeId()).toBe("content-delete-new.jar");
    expect(host.querySelector('[id="content-update-old.jar"]')).toBeNull();
    act(()=>root.unmount());host.remove();
});

test.each(["mods","resource-packs","shaders"])("%s shows hash identity but deletes by local filename",async(kind)=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const invoke=vi.mocked(invokeNative);
    invoke.mockImplementation(command=>Promise.resolve(command==="identify_instance_content" ? [{name:"Original.zip",displayName:"Project_2.0"}] : listing("Original.zip")));
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host),manager=new FocusManager();
    await act(async()=>root.render(<Navigation.Provider value={manager}><InstanceContent title={`instance-${kind}`} instanceId="labels" selection={{options:["Test"],value:"Test",onChange:()=>{}}} busy={false} preview={false} hasLoader/></Navigation.Provider>));
    expect(host.textContent).toContain("Project_2.0");
    await act(async()=>(host.querySelector('[id^="content-delete-"]') as HTMLButtonElement).click());
    expect(invoke).toHaveBeenCalledWith("delete_instance_content",{instanceId:"labels",kind,name:"Original.zip",enabled:true});
    act(()=>root.unmount());host.remove();
});


test("update summary follows language changes without another request",async()=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    setLocale("en-US");
    const invoke=vi.mocked(invokeNative);
    invoke.mockImplementation(command=>Promise.resolve(command==="check_content_updates" ? {updates:[],unmatched:2} : listing("Example.zip")));
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host),manager=new FocusManager();
    const render=()=>root.render(<Navigation.Provider value={manager}><InstanceContent title="instance-resource-packs" instanceId="language" selection={{options:["Test"],value:"Test",onChange:()=>{}}} busy={false} preview={false} hasLoader={false}/></Navigation.Provider>);
    await act(async()=>render());
    await act(async()=>(host.querySelector("#content-check-updates") as HTMLButtonElement).click());
    const english=host.querySelector('[role="status"]')?.textContent;
    invoke.mockClear();
    await act(async()=>{setLocale("zh-CN");render();});
    expect(host.querySelector('[role="status"]')?.textContent).toBe(t("Found {updates} updates; {unmatched} entries could not be checked.",{updates:0,unmatched:2}));
    expect(host.querySelector('[role="status"]')?.textContent).not.toBe(english);
    expect(invoke).not.toHaveBeenCalled();
    act(()=>root.unmount());host.remove();setLocale("en-US");
});


test.each(["mods","resource-packs","shaders"])("%s retains other update buttons and their focus after a rescan",async kind=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const updates=["first.zip","second.zip"].map(name=>({name,enabled:true,hash:"a".repeat(40),project:name,version:"next"}));
    let completed=false;
    vi.mocked(invokeNative).mockImplementation(command=>Promise.resolve(
        command==="check_content_updates" ? {updates,unmatched:0} :
        command==="validate_content_updates" ? (completed ? updates.slice(1) : updates) :
        {directory:"/fixture",entries:updates.map(u=>({name:u.name,enabled:true,conflict:false}))}));
    const host=document.createElement("div");document.body.append(host);
    const root=createRoot(host),manager=new FocusManager();
    const render=(refreshKey:string)=>root.render(<Navigation.Provider value={manager}><InstanceContent title={`instance-${kind}`} instanceId="retain" selection={{options:["Test"],value:"Test",onChange:()=>{}}} busy={false} refreshKey={refreshKey} preview={false} hasLoader/></Navigation.Provider>);
    await act(async()=>render("initial"));
    await act(async()=>(host.querySelector("#content-check-updates") as HTMLButtonElement).click());
    act(()=>manager.focus("content-update-second.zip"));
    await act(async()=>render("failed-or-unchanged"));
    expect(host.querySelectorAll('[id^="content-update-"]')).toHaveLength(2);
    completed=true;
    await act(async()=>render("completed"));
    expect(host.querySelector('[id="content-update-first.zip"]')).toBeNull();
    expect(host.querySelector('[id="content-update-second.zip"]')).not.toBeNull();
    expect(manager.activeId()).toBe("content-update-second.zip");
    act(()=>root.unmount());host.remove();
});
