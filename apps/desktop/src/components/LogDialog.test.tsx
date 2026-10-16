// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import { LogDialog } from "./LogDialog";
import { Navigation } from "./Control";
import { FocusManager } from "../navigation/FocusManager";
import { listLogs, readLog, copyLogText } from "../state/logs";
vi.mock("../state/logs",()=>({listLogs:vi.fn(),readLog:vi.fn(),copyLogText:vi.fn(async()=>{}),openLogLocation:vi.fn(async()=>{})}));
test("log selection rejects stale reads and copy uses the displayed file",async()=>{
    Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
    const old={name:"logs/old.log",modified:1,size:10},latest={name:"logs/latest.log",modified:2,size:10};
    vi.mocked(listLogs).mockResolvedValue([latest,old]);
    const pending: Array<(data:{text:string;truncated:boolean})=>void>=[];
    vi.mocked(readLog).mockImplementation(()=>new Promise(resolve=>pending.push(resolve)));
    const host=document.createElement("div");document.body.append(host);const root=createRoot(host);
    const manager=new FocusManager();manager.enterModal();
    try {
        await act(async()=>root.render(<Navigation.Provider value={manager}><LogDialog request={{instanceId:"one",initialName:old.name}} onClose={()=>{}}/></Navigation.Provider>));
        expect(readLog).toHaveBeenLastCalledWith(expect.objectContaining({instanceId:"one"}),old.name);
        await act(async()=>{(host.querySelector("#modal-first") as HTMLButtonElement).click();});
        await act(async()=>{(host.querySelector("#modal-first-option-0") as HTMLButtonElement).click();});
        await act(async()=>pending[1]({text:"latest <script>plain text</script>",truncated:false}));
        await act(async()=>pending[0]({text:"stale",truncated:false}));
        expect(host.querySelector("#log-content")?.textContent).toBe("latest <script>plain text</script>");
        expect(host.querySelector("#log-content script")).toBeNull();
        await act(async()=>{(host.querySelector("#log-copy") as HTMLButtonElement).click();});
        expect(copyLogText).toHaveBeenCalledWith("latest <script>plain text</script>");
        expect(host.querySelector("#log-open")?.getAttribute("data-tone")).toBe("yellow");
        expect(manager.modalScroll).toBe(host.querySelector("#log-content"));
    } finally {act(()=>root.unmount());host.remove();expect(manager.modalScroll).toBeNull();}
});
