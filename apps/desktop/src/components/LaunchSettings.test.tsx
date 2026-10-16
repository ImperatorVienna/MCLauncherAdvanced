// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import { LaunchSettings } from "./LaunchSettings";
import { defaultLaunchSettings } from "../state/launchSettings";
import { Navigation } from "./Control";
import { FocusManager } from "../navigation/FocusManager";

test("restores settings, rejects excess memory, and saves controller checkbox changes", async () => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  Object.assign(window, { __TAURI__: { core: { invoke: vi.fn(async () => 8 * 1024 ** 3) } } });
  const host = document.createElement("div"); document.body.append(host);
  const root = createRoot(host); const manager = new FocusManager(); const save = vi.fn(async () => true);
  try {
    await act(async () => root.render(<Navigation.Provider value={manager}><LaunchSettings busy={false} value={{ ...defaultLaunchSettings, maxMemoryMib: 4096, displayMode: "windowed", width: 1920, height: 1080 }} onSave={save}/></Navigation.Provider>));
    const input = host.querySelector("input")!;
    expect(input.value).toBe("4"); expect(input.max).toBe("8");
    expect(host.querySelector("#launch-resolution")!.textContent).toContain("1920 × 1080");
    const type = (value: string) => act(() => {
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, value);
      input.dispatchEvent(new Event("input", { bubbles: true }));
    });
    type("9");
    expect(host.querySelector('[role="alert"]')).not.toBeNull();
    expect(save).not.toHaveBeenCalled();
    type("6"); expect(host.querySelector('[role="alert"]')).toBeNull();
    act(() => manager.focus("launch-remember-maximized"));
    act(() => manager.dispatch("Confirm"));
    expect(host.querySelector('[role="checkbox"]')!.getAttribute("aria-checked")).toBe("true");
    await act(async () => {});
    expect(save).toHaveBeenCalledWith({ maxMemoryMib: 6144, displayMode: "windowed", width: 1920, height: 1080, rememberMaximized: true });
    expect(host.querySelector("#launch-settings-save")).toBeNull();
  } finally {
    act(() => root.unmount()); host.remove(); delete (window as Window & { __TAURI__?: unknown }).__TAURI__;
  }
});

test("autosave serializes rapid changes and does not retry a failed save forever", async () => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  Object.assign(window, { __TAURI__: { core: { invoke: vi.fn(async () => 8 * 1024 ** 3) } } });
  const host = document.createElement("div"); document.body.append(host);
  const root = createRoot(host); const manager = new FocusManager();
  let finish!: (ok: boolean) => void;
  const save = vi.fn(() => new Promise<boolean>(resolve => { finish = resolve; }));
  try {
    await act(async () => root.render(<Navigation.Provider value={manager}><LaunchSettings busy={false} value={{ ...defaultLaunchSettings, displayMode: "windowed" }} onSave={save}/></Navigation.Provider>));
    const checkbox = host.querySelector('[role="checkbox"]') as HTMLButtonElement;
    act(() => checkbox.click());
    expect(save).toHaveBeenCalledTimes(1);
    act(() => checkbox.click());
    expect(save).toHaveBeenCalledTimes(1);
    await act(async () => finish(true));
    expect(save).toHaveBeenCalledTimes(2);
    expect(save.mock.calls[1]).toEqual([expect.objectContaining({ rememberMaximized: false })]);
    await act(async () => finish(false));
    expect(save).toHaveBeenCalledTimes(2);
    expect(document.activeElement).not.toBeNull();
  } finally { act(() => root.unmount()); host.remove(); delete (window as Window & { __TAURI__?: unknown }).__TAURI__; }
});
