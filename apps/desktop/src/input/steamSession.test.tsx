// @vitest-environment jsdom
import React, { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import { FocusManager } from "../navigation/FocusManager";
import { isExitShortcut, useSteamSession } from "./steamSession";

test("exit shortcuts work independently of Steam and do not consume ordinary text keys", () => {
  expect(isExitShortcut(new KeyboardEvent("keydown", { key: "F4", altKey: true }))).toBe(true);
  expect(isExitShortcut(new KeyboardEvent("keydown", { key: "q", ctrlKey: true }))).toBe(true);
  expect(isExitShortcut(new KeyboardEvent("keydown", { key: "q" }))).toBe(false);
  expect(isExitShortcut(new KeyboardEvent("keydown", { key: "Escape" }))).toBe(false);
});

test.each([true, false])("editing ends without submitting or closing the form; navigation resumes (Steam=%s)", async (active) => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  const invoke = vi.fn(async (cmd: string) => cmd === "steam_status" ? {enabled:active,launched:active,active} : undefined);
  Object.assign(window, { __TAURI__: {core:{invoke}} });
  const manager = new FocusManager(); const back = vi.fn(); manager.onBack = back;
  const host = document.createElement("div"); document.body.append(host); const root=createRoot(host);
  const submit = vi.fn();
  function Harness() { useSteamSession(manager,false); return <form onSubmit={submit}><input /><button>\u4fdd\u5b58</button></form>; }
  try {
    await act(async()=>root.render(<Harness/>));
    const input=host.querySelector("input")!, button=host.querySelector("button")!;
    input.getBoundingClientRect=()=>({left:0,right:100,top:0,bottom:30} as DOMRect);
    button.getBoundingClientRect=()=>({left:0,right:100,top:60,bottom:90} as DOMRect);
    manager.register("input",{element:input,scope:"main",group:"content"}); manager.register("save",{element:button,scope:"main",group:"content"});
    input.focus(); await act(async()=>manager.dispatch("Confirm"));
    if (active) expect(invoke).toHaveBeenCalledWith("steam_keyboard",{show:true});
    else expect(invoke.mock.calls.some(c => c[0] === "steam_keyboard")).toBe(false);
    const typing = new KeyboardEvent("keydown", {key:"a", bubbles:true, cancelable:true});
    input.dispatchEvent(typing); expect(typing.defaultPrevented).toBe(false);
    await act(async()=>manager.dispatch("Back"));
    expect(back).not.toHaveBeenCalled(); expect(document.activeElement).toBe(input);
    manager.dispatch("NavigateDown"); expect(document.activeElement).toBe(button);
    input.focus(); await act(async()=>manager.dispatch("Confirm"));
    const enter=new KeyboardEvent("keydown",{key:"Enter",bubbles:true,cancelable:true});
    await act(async()=>input.dispatchEvent(enter));
    expect(enter.defaultPrevented).toBe(true); expect(submit).not.toHaveBeenCalled();
    await act(async()=>window.dispatchEvent(new KeyboardEvent("keydown",{key:"q",ctrlKey:true,cancelable:true})));
    expect(invoke).toHaveBeenCalledWith("request_close",undefined);
  } finally { await act(async()=>root.unmount()); host.remove(); delete (window as Window & {__TAURI__?:unknown}).__TAURI__; }
});

test.each([false, true])("mouse text editing cancels drafts but preserves immediate settings (immediate=%s)", async immediate => {
  const { Field } = await import("../components/Field");
  const { Navigation } = await import("../components/Control");
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  const manager = new FocusManager(); manager.onBack = vi.fn();
  const host = document.createElement("div"); document.body.append(host); const root = createRoot(host);
  let change!: (value: string) => void;
  function Harness() {
    useSteamSession(manager, true);
    const [value, setValue] = React.useState("Original"); change = setValue;
    return <Navigation.Provider value={manager}><Field id="draft" scope="main" label="Name" value={value} onChange={setValue} disabled={false} immediate={immediate}/></Navigation.Provider>;
  }
  try {
    await act(async () => root.render(<Harness/>));
    const input = host.querySelector("input")!;
    for (const method of ["keyboard", "controller"]) {
      act(() => change("Original"));
      act(() => { input.focus(); input.click(); });
      act(() => change("Changed"));
      await act(async () => {
        if (method === "keyboard") input.dispatchEvent(new KeyboardEvent("keydown", {key:"Escape",bubbles:true,cancelable:true}));
        else manager.dispatch("Back");
      });
      expect(input.value).toBe(immediate ? "Changed" : "Original");
      expect(document.activeElement).toBe(input);
      expect(manager.onBack).not.toHaveBeenCalled();
    }
  } finally { act(() => root.unmount()); host.remove(); }
});
