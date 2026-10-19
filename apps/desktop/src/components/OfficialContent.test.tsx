// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import { ArticleBody } from "./OfficialContent";
import { Navigation } from "./Control";
import { FocusManager } from "../navigation/FocusManager";
import { informationLink, imageLink, parseNews, validContentPath, useEditorial } from "../state/editorial";

test("official feeds reject unsafe paths, links and images and select five Java stories",()=>{
    expect(validContentPath("javaPatchNotes/26-4-snapshot-4.json")).toBe(true);
    expect(validContentPath("javaPatchNotes/../news.json")).toBe(false);
    for(const url of ["javascript:alert(1)","https://www.minecraft.net.evil.test/","https://user@www.minecraft.net/","https://github.com/ImperatorVienna/MCLauncherAdvanced-evil"])
        expect(informationLink(url)).toBeNull();
    expect(imageLink("//evil.test/a.png")).toBeNull();
    expect(imageLink("/v2/images/a.png")).toBe("https://launchercontent.mojang.com/v2/images/a.png");
    const entry=(i:number)=>({title:`Story ${i}`,newsType:["Java"],date:`2026-10-${10+i}`,readMoreLink:`https://www.minecraft.net/article/${i}`,playPageImage:{url:"/v2/images/a.png"}});
    const data=parseNews({entries:[{...entry(8),newsType:["Bedrock"]},...Array.from({length:7},(_,i)=>entry(i)),entry(0),{...entry(9),readMoreLink:"javascript:evil()"}]});
    expect(data.map(n=>n.title)).toEqual(["Story 6","Story 5","Story 4","Story 3","Story 2"]);
});

test("remote article HTML is inert and allowed links use registered navigation controls",async()=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const host=document.createElement("div");document.body.append(host);const root=createRoot(host),manager=new FocusManager(),open=vi.fn();
    await act(async()=>root.render(<Navigation.Provider value={manager}><ArticleBody onOpen={open} html={'<script>window.pwned=1</script><img src=x onerror=evil()><p onclick="evil()">Text <a href="javascript:evil()">Bad</a><a href="https://www.minecraft.net/article/test">Read</a></p><iframe src="https://evil.test"></iframe>'}/></Navigation.Provider>));
    expect(host.querySelector("script,img,iframe,[onclick],a")).toBeNull();
    expect(host.textContent).toContain("Text BadRead");
    const button=host.querySelector("button")!;
    act(()=>manager.focus(button.id));expect(manager.activeId()).toBe(button.id);
    act(()=>button.click());expect(open).toHaveBeenCalledWith("https://www.minecraft.net/article/test");
    await act(async()=>root.unmount());host.remove();
});

test("switching versions cannot display a late result from the old configuration",async()=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const host=document.createElement("div");const root=createRoot(host);
    let oldDone!:(value:string)=>void,newDone!:(value:string)=>void;
    const old=new Promise<string>(r=>{oldDone=r;}),next=new Promise<string>(r=>{newDone=r;});
    function View({version}:{version:string}) {const state=useEditorial(version,()=>version==="old"?old:next);return <p>{state.data??"loading"}</p>;}
    await act(async()=>root.render(<View version="old"/>));
    await act(async()=>root.render(<View version="new"/>));
    await act(async()=>newDone("new notes"));expect(host.textContent).toBe("new notes");
    await act(async()=>oldDone("old notes"));expect(host.textContent).toBe("new notes");
    await act(async()=>root.unmount());
});
