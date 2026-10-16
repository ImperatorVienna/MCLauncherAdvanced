// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import { Control, Navigation } from "./Control";
import { FocusManager } from "../navigation/FocusManager";

test("disabled active controls retain focus, reject activation and cannot be revisited until enabled", () => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  const host = document.createElement("div"); document.body.append(host);
  const root = createRoot(host), manager = new FocusManager(), click = vi.fn();
  const render = (disabled: boolean) => act(() => root.render(
    <Navigation.Provider value={manager}>
      <Control id="apply" disabled={disabled} onClick={click}>Apply</Control>
      <Control id="next">Next</Control>
    </Navigation.Provider>,
  ));
  try {
    render(false);
    const apply = host.querySelector("#apply") as HTMLButtonElement;
    const next = host.querySelector("#next") as HTMLButtonElement;
    [apply, next].forEach((element, index) => {
      element.getBoundingClientRect = () => ({ left: index * 100, right: index * 100 + 50, top: 0, bottom: 30 } as DOMRect);
    });
    act(() => manager.focus("apply"));
    render(true);
    manager.preserveFocus("next"); manager.resumeControllerFocus();
    expect(document.activeElement).toBe(apply);
    expect(apply.disabled).toBe(false);
    expect(apply.getAttribute("aria-disabled")).toBe("true");
    expect(apply.tabIndex).toBe(-1);
    act(() => { manager.dispatch("Confirm"); apply.click(); });
    for (const key of ["Enter", " "]) {
      const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true });
      act(() => apply.dispatchEvent(event));
      expect(event.defaultPrevented).toBe(true);
    }
    expect(click).not.toHaveBeenCalled();
    act(() => manager.dispatch("NavigateRight"));
    expect(document.activeElement).toBe(next);
    expect(apply.disabled).toBe(true);
    act(() => { manager.dispatch("NavigateLeft"); });
    expect(document.activeElement).toBe(next);
    render(false);
    act(() => { manager.dispatch("NavigateLeft"); manager.dispatch("Confirm"); });
    expect(document.activeElement).toBe(apply);
    expect(click).toHaveBeenCalledTimes(1);
  } finally { act(() => root.unmount()); host.remove(); }
});
