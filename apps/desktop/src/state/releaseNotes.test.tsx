// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect,test,vi } from "vitest";
import { setLocale } from "../i18n";
import { useReleaseNotes } from "./releaseNotes";
const api=vi.hoisted(()=>({invoke:vi.fn()}));
vi.mock("../platform/desktop",()=>({isDesktop:()=>true,invokeNative:api.invoke}));
test("requests only the selected language and ignores stale results without fallback",async()=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});setLocale("en-US");
    const pending:Array<{language:string;resolve:(body:string|null)=>void;reject:()=>void}>=[];
    api.invoke.mockImplementation((command:string,args:{language:string})=>{
        expect(command).toBe("launcher_release_notes");
        return new Promise<string|null>((resolve,reject)=>pending.push({language:args.language,resolve,reject:()=>reject(new Error("offline"))}));
    });
    let value:ReturnType<typeof useReleaseNotes>;
    function Probe(){value=useReleaseNotes("1.0.5");return <p>{value.body}</p>;}
    const host=document.createElement("div"),root=createRoot(host);
    try {
        await act(async()=>root.render(<Probe/>));
        expect(pending.map(p=>p.language)).toEqual(["en-US"]);
        await act(async()=>pending[0].resolve("English notes"));
        expect(host.textContent).toBe("English notes");
        await act(async()=>setLocale("zh-CN"));
        expect(host.textContent).toBe("");
        await act(async()=>setLocale("zh-TW"));
        await act(async()=>pending[1].resolve("过期简体公告"));
        expect(host.textContent).toBe("");
        await act(async()=>pending[2].resolve("繁體公告 <script>plain text</script>"));
        expect(host.textContent).toContain("繁體公告");
        expect(host.querySelector("script")).toBeNull();
        await act(async()=>setLocale("zh-CN"));
        await act(async()=>pending[3].resolve(null));
        expect(value!.loading).toBe(false);expect(value!.body).toBeNull();
        await act(async()=>setLocale("en-US"));
        await act(async()=>pending[4].reject());
        expect(value!.failed).toBe(true);expect(host.textContent).toBe("");
        expect(pending.map(p=>p.language)).toEqual(["en-US","zh-CN","zh-TW","zh-CN","en-US"]);
    }finally{act(()=>root.unmount());api.invoke.mockReset();}
});
