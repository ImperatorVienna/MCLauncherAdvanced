// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import styles from "../styles/app.css?inline";
import { SelectMenu } from "./SelectMenu";
import { Navigation } from "./Control";
import { FocusManager } from "../navigation/FocusManager";

test("dropdown traps directional navigation until selection or Back", () => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  const container = document.createElement("div"); document.body.append(container);
  const root = createRoot(container); const manager = new FocusManager(); const change = vi.fn();
  act(() => root.render(<Navigation.Provider value={manager}><SelectMenu options={["One", "Two"]} value="One" onChange={change}/></Navigation.Provider>));
  act(() => (container.querySelector("#instance-select") as HTMLButtonElement).click());
  expect(manager.activeId()).toBe("instance-select-option-0");
  manager.onTab = vi.fn(); manager.onMenu = vi.fn();
  for (const action of ["NavigateLeft", "NavigateRight", "NextTab", "PreviousTab", "OpenMenu"] as const) {
    act(() => manager.dispatch(action));
    expect(manager.activeId()).toBe("instance-select-option-0");
  }
  expect(manager.onTab).not.toHaveBeenCalled(); expect(manager.onMenu).not.toHaveBeenCalled();
  act(() => manager.dispatch("NavigateDown"));
  expect(manager.activeId()).toBe("instance-select-option-1");
  act(() => manager.dispatch("Confirm"));
  expect(change).toHaveBeenCalledWith("Two"); expect(manager.activeId()).toBe("instance-select");
  expect(container.querySelector('[role="listbox"]')).toBeNull();
  act(() => manager.dispatch("Confirm"));
  act(() => manager.dispatch("Back"));
  expect(manager.activeId()).toBe("instance-select"); expect(change).toHaveBeenCalledTimes(1);
  act(() => root.unmount()); container.remove();
});


test("outside mouse input dismisses the controller-opened list without blocking clicks", () => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  const container = document.createElement("div"); document.body.append(container);
  const root = createRoot(container); const manager = new FocusManager();
  act(() => root.render(<Navigation.Provider value={manager}><SelectMenu options={["One", "Two"]} value="One" onChange={() => {}}/></Navigation.Provider>));
  act(() => manager.focus("instance-select"));
  act(() => manager.dispatch("Confirm"));
  const outside = new MouseEvent("pointerdown", { bubbles: true, cancelable: true });
  act(() => document.body.dispatchEvent(outside));
  expect(outside.defaultPrevented).toBe(false);
  expect(container.querySelector('[role="listbox"]')).toBeNull();
  expect(manager.interceptAction).toBeUndefined();
  act(() => root.unmount()); container.remove();
});

test("parent rerender cannot reset a navigated menu item", () => {
  Object.assign(globalThis,{IS_REACT_ACT_ENVIRONMENT:true});
  const host=document.createElement("div");document.body.append(host);const root=createRoot(host);const manager=new FocusManager();
  const render=()=>root.render(<Navigation.Provider value={manager}><SelectMenu options={["One","Two"]} value="One" onChange={()=>{}}/></Navigation.Provider>);
  act(render);act(()=>(host.querySelector("#instance-select") as HTMLButtonElement).click());
  act(()=>manager.dispatch("NavigateDown"));expect(manager.activeId()).toBe("instance-select-option-1");
  act(render);expect(manager.activeId()).toBe("instance-select-option-1");
  act(()=>root.unmount());host.remove();
});

test("official version picker stays in modal scope and never submits the config form", () => {
  Object.assign(globalThis, {IS_REACT_ACT_ENVIRONMENT:true});
  const host=document.createElement("div");document.body.append(host);const root=createRoot(host);const manager=new FocusManager();
  const change=vi.fn(),submit=vi.fn();manager.enterModal();
  act(()=>root.render(<Navigation.Provider value={manager}><form onSubmit={e=>{e.preventDefault();submit();}}><SelectMenu id="official-version" scope="modal" options={["1.17.1","1.21.1"]} value="1.17.1" onChange={change} showIcons={false}/></form></Navigation.Provider>));
  act(() => manager.focus("official-version"));act(()=>manager.dispatch("Confirm"));
  expect(manager.activeId()).toBe("official-version-option-0");
  act(()=>manager.dispatch("NavigateDown"));act(()=>manager.dispatch("Confirm"));
  expect(change).toHaveBeenCalledWith("1.21.1");expect(submit).not.toHaveBeenCalled();expect(manager.activeId()).toBe("official-version");
  act(()=>manager.dispatch("Confirm"));act(()=>manager.dispatch("Back"));expect(manager.activeId()).toBe("official-version");
  act(()=>root.unmount());host.remove();
});


test("menu appearance applies to the rendered component", () => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  const style = document.createElement("style"); style.textContent = styles; document.head.append(style);
  const host = document.createElement("div"); document.body.append(host);
  const root = createRoot(host); const manager = new FocusManager();
  act(() => root.render(<Navigation.Provider value={manager}><SelectMenu options={["One", "Two"]} value="One" onChange={() => {}}/></Navigation.Provider>));
  act(() => (host.querySelector("#instance-select") as HTMLButtonElement).click());
  const list = host.querySelector('[role="listbox"]')!;
  const selected = host.querySelector('[aria-selected="true"]')!;
  const other = host.querySelector('[aria-selected="false"]')!;
  expect(getComputedStyle(list).position).toBe("absolute");
  expect(getComputedStyle(list).backgroundColor).toBe("rgb(61, 57, 56)");
  expect(getComputedStyle(selected).boxShadow).toBe("inset 0 0 0 2px #fff");
  expect(getComputedStyle(other).backgroundColor).toBe("rgb(61, 57, 56)");
  expect(getComputedStyle(other).color).toBe("rgb(237, 229, 226)");
  act(() => root.unmount()); host.remove(); style.remove();
});

test("blank-pointer dismissal remembers the trigger for controller takeover", () => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  const host = document.createElement("div"); document.body.append(host);
  const root = createRoot(host); const manager = new FocusManager();
  act(() => root.render(<Navigation.Provider value={manager}><SelectMenu options={["One", "Two"]} value="One" onChange={() => {}}/></Navigation.Provider>));
  act(() => (host.querySelector("button") as HTMLButtonElement).click());
  act(() => document.body.dispatchEvent(new MouseEvent("pointerdown", { bubbles: true })));
  act(() => manager.resumeControllerFocus());
  expect(manager.activeId()).toBe("instance-select");
  act(() => root.unmount()); host.remove();
});

test("disabled or emptied menus release their input interception", () => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  const host = document.createElement("div"); document.body.append(host);
  const root = createRoot(host); const manager = new FocusManager(); manager.onBack = vi.fn();
  const render = (options: string[], disabled = false) => root.render(<Navigation.Provider value={manager}><SelectMenu options={options} disabled={disabled} value="One" onChange={() => {}}/></Navigation.Provider>);
  for (const disabled of [true, false]) {
    act(() => render(["One"]));
    act(() => (host.querySelector("button") as HTMLButtonElement).click());
    act(() => render(disabled ? ["One"] : [], disabled));
    expect(host.querySelector('[role="listbox"]')).toBeNull();
    expect(manager.interceptAction).toBeUndefined();
    expect((host.querySelector("button") as HTMLButtonElement).disabled).toBe(true);
    manager.dispatch("Back");
  }
  expect(manager.onBack).toHaveBeenCalledTimes(2);
  act(() => root.unmount()); host.remove();
});
