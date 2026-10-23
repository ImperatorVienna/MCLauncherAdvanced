// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import { useSkins, emptySkins, type Skin } from "./skins";
const native = vi.hoisted(() => ({invoke:vi.fn()}));
vi.mock("../platform/desktop", () => ({isDesktop:()=>true,invokeNative:native.invoke}));
test("uploads target the selected account, prevent duplicates and discard stale responses", async () => {
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    let complete: (skin: Skin) => void = () => {};
    const skin: Skin = {id:"@a",name:"Alex",model:"slim",png:"YWJj"};
    native.invoke.mockImplementation((command:string) => {
        if(command === "load_skins") return Promise.resolve(emptySkins());
        if(command === "load_account_skin") return Promise.resolve(skin);
        if(command === "upload_account_skin") return new Promise<Skin>(resolve => {complete=resolve;});
        throw Error(command);
    });
    let state!: ReturnType<typeof useSkins>;
    function Probe({account}:{account:string}) { state=useSkins(account); return null; }
    const host=document.createElement("div"), root=createRoot(host);
    await act(async()=>root.render(<Probe account="@a"/>));
    let pending!: Promise<boolean>;
    act(()=>{pending=state.apply({...skin,model:"default",url:"unused"});});
    expect(state.busy).toBe(true);
    await act(async()=>{expect(await state.apply({...skin,url:"unused"})).toBe(false);});
    expect(native.invoke.mock.calls.filter(([command])=>command==="upload_account_skin")).toEqual([["upload_account_skin",{selected:"@a",model:"default",image:"YWJj"}]]);
    await act(async()=>root.render(<Probe account="@b"/>));
    await act(async()=>{complete({...skin,model:"default"}); expect(await pending).toBe(false);});
    expect(state.applied).toBe(false);
    expect(state.accountSkin?.model).toBe("slim");
    act(()=>{pending=state.apply({...skin,model:"default",url:"unused"});});
    await act(async()=>{complete({...skin,id:"@b",model:"default"});expect(await pending).toBe(true);});
    expect(state.accountSkin?.model).toBe("default");
    expect(state.applied).toBe(true);
    expect(state.data.skins).toEqual([]);
    native.invoke.mockImplementation(()=>Promise.reject(Error("Skin upload failed; retry")));
    await act(async()=>{expect(await state.apply({...skin,url:"unused"})).toBe(false);});
    expect(state.error).toBeTruthy(); expect(state.accountSkin?.model).toBe("default");
    act(()=>root.unmount());
});
