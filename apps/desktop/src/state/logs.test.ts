// @vitest-environment jsdom
import {afterEach,expect,test,vi} from "vitest";
import {copyLogText} from "./logs";
const host=vi.hoisted(()=>({desktop:true,invoke:vi.fn()}));
vi.mock("../platform/desktop",()=>({isDesktop:()=>host.desktop,invokeNative:host.invoke}));
const original=Object.getOwnPropertyDescriptor(navigator,"clipboard");
afterEach(()=>{host.invoke.mockReset();host.desktop=true;if(original)Object.defineProperty(navigator,"clipboard",original);else delete (navigator as any).clipboard;});
test("native copying does not request WebView clipboard permission",async()=>{
    Object.defineProperty(navigator,"clipboard",{configurable:true,get:()=>{throw Error("must not access browser clipboard");}});
    host.invoke.mockResolvedValue(true);await copyLogText("sample");
    expect(host.invoke).toHaveBeenCalledWith("copy_clipboard_text",{text:"sample"});
});
test("browser permission errors use the clipboard message",async()=>{
    host.desktop=false;
    Object.defineProperty(navigator,"clipboard",{configurable:true,value:{writeText:vi.fn().mockRejectedValue(Error("NotAllowedError"))}});
    await expect(copyLogText("sample")).rejects.toThrow("clipboard");
});
test("hosts without native support retain browser copying",async()=>{
    host.invoke.mockResolvedValue(false);const writeText=vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator,"clipboard",{configurable:true,value:{writeText}});
    await copyLogText("sample");expect(writeText).toHaveBeenCalledWith("sample");
});
