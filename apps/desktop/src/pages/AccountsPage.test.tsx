// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import { AccountsPage } from "./AccountsPage";
import { Navigation } from "../components/Control";
import { FocusManager } from "../navigation/FocusManager";
test("deletion requires confirmation and chooses remaining identity without touching instances", async () => {
    Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
    const host = document.createElement("div"); document.body.append(host);
    const root = createRoot(host); const manager = new FocusManager();
    const commit = vi.fn().mockResolvedValue(true);
    act(() => root.render(<Navigation.Provider value={manager}><AccountsPage accounts={{ data: { revision: 4, microsoft: [{id:"a".repeat(32),name:"Steve"},{id:"b".repeat(32),name:"Alex"}], selected: "@"+ "a".repeat(32) }, busy: false, error: "", commit, refresh: vi.fn(), login: {status:{running:false,phase:"",userCode:null,verificationUri:null,expiresAt:null,error:null,completed:false},busy:false,error:"",start:vi.fn(),cancel:vi.fn(),open:vi.fn()} }}/></Navigation.Provider>));
    act(() => (host.querySelector("#account-delete") as HTMLButtonElement).click());
    expect(commit).not.toHaveBeenCalled(); expect(manager.activeId()).toBe("account-cancel");
    await act(async () => (host.querySelector("#account-save") as HTMLButtonElement).click());
    expect(commit).toHaveBeenCalledWith({ revision: 4, selected: "@"+ "b".repeat(32), microsoft: [{id:"b".repeat(32),name:"Alex"}] });
    act(() => root.unmount()); host.remove();
});

test("verified account names are read-only and local creation is absent", async()=>{
    const id="a".repeat(32), manager=new FocusManager(), host=document.createElement("div");document.body.append(host);
    const root=createRoot(host),commit=vi.fn().mockResolvedValue(true);
    const login={status:{running:false,phase:"",userCode:null,verificationUri:null,expiresAt:null,error:null,completed:false},busy:false,error:"",start:vi.fn(),cancel:vi.fn(),open:vi.fn()};
    const data={revision:2,selected:`@${id}`,microsoft:[{id,name:"Alex"}]};
    await act(async()=>root.render(<Navigation.Provider value={manager}><AccountsPage accounts={{data,busy:false,error:"",commit,refresh:vi.fn(),login}}/></Navigation.Provider>));
    expect(host.querySelector("#account-select")?.textContent).toContain("Alex");
    expect(host.querySelector("#account-rename")).toBeNull();
    expect(host.querySelector("#account-create")).toBeNull();
    act(()=>(host.querySelector("#account-select") as HTMLButtonElement).click());
    expect(host.querySelector("#account-select-option-0")?.textContent).toContain("Alex");
    act(()=>manager.dispatch("Back"));
    act(()=>(host.querySelector("#account-delete") as HTMLButtonElement).click());
    await act(async()=>(host.querySelector("#account-save") as HTMLButtonElement).click());
    expect(commit).toHaveBeenCalledWith({...data,microsoft:[],selected:null});
    act(()=>root.unmount());host.remove();
});

test("device authorization focuses its controls and back cancels without navigating away", async()=>{
    const manager=new FocusManager(),host=document.createElement("div");document.body.append(host);const root=createRoot(host);
    const cancel=vi.fn().mockResolvedValue(undefined),open=vi.fn();
    const login={status:{running:true,phase:"waiting",userCode:"ABCD-EFGH",verificationUri:"https://www.microsoft.com/link",expiresAt:1,error:null,completed:false},busy:true,error:"",start:vi.fn(),cancel,open};
    await act(async()=>root.render(<Navigation.Provider value={manager}><AccountsPage accounts={{data:{revision:0,selected:null},busy:true,error:"",commit:vi.fn(),refresh:vi.fn(),login}}/></Navigation.Provider>));
    expect(manager.activeId()).toBe("account-login-open");
    expect(host.textContent).toContain("ABCD-EFGH");
    act(()=>(host.querySelector("#account-login-open") as HTMLButtonElement).click());expect(open).toHaveBeenCalledOnce();
    await act(async()=>manager.dispatch("Back"));expect(cancel).toHaveBeenCalledOnce();
    act(()=>root.unmount());host.remove();
});

