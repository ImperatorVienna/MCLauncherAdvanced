// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import { SkinsPage } from "./SkinsPage";
import { Navigation } from "../components/Control";
import { FocusManager } from "../navigation/FocusManager";
import { emptySkins, type DisplaySkin, type useSkins } from "../state/skins";
vi.mock("../components/SkinPreview",()=>({SkinPreview:({skin}:{skin:DisplaySkin})=><div data-preview-model={skin.model}/> }));
vi.mock("../components/SkinPortrait",()=>({SkinPortrait:()=>null}));
test("changing official arms previews locally and only Use skin uploads", async()=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const skin:DisplaySkin={id:"@account",name:"Alex",model:"slim",png:"YWJj",url:"data:image/png;base64,YWJj",builtin:true};
    const apply=vi.fn().mockResolvedValue(true), commit=vi.fn();
    const library:ReturnType<typeof useSkins>={data:emptySkins(),all:[skin],current:()=>skin,accountSkin:skin,commit,apply,applied:false,canApply:true,error:"",busy:false};
    const host=document.createElement("div");document.body.append(host);const root=createRoot(host);
    const manager=new FocusManager();
    await act(async()=>root.render(<Navigation.Provider value={manager}><SkinsPage library={library}/></Navigation.Provider>));
    const model=host.querySelector('[id="skin-model-@account"]') as HTMLButtonElement;
    expect(model.disabled).toBe(false);
    act(()=>model.click());
    expect(host.querySelector('[data-preview-model="default"]')).not.toBeNull();
    expect(apply).not.toHaveBeenCalled(); expect(commit).not.toHaveBeenCalled();
    expect((host.querySelector('#skin-edit') as HTMLButtonElement).disabled).toBe(true);
    await act(async()=>(host.querySelector('#skin-use') as HTMLButtonElement).click());
    expect(apply).toHaveBeenCalledWith({...skin,model:"default"});
    act(()=>root.unmount());host.remove();
});
