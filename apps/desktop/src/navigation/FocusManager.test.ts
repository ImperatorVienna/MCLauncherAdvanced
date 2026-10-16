// @vitest-environment jsdom
import { beforeEach, expect, test, vi } from "vitest";
import { FocusManager } from "./FocusManager";
import { keyboardAction } from "../input/actions";
let manager: FocusManager;
beforeEach(() => {
  document.body.innerHTML = "";
  manager = new FocusManager();
});
function control(
  id: string,
  x: number,
  y: number,
  scope = "main",
  disabled = false,
) {
  const element = document.createElement("button");
  element.disabled = disabled;
  document.body.append(element);
  element.getBoundingClientRect = () => ({
    left: x,
    right: x + 50,
    top: y,
    bottom: y + 30,
    width: 50,
    height: 30,
    x,
    y,
    toJSON: () => ({}),
  });
  const unregister = manager.register(id, { element, scope, group: "test" });
  return { element, unregister };
}
test("controller navigation follows geometry and skips disabled controls", () => {
  const a = control("a", 0, 0);
  control("disabled", 60, 0, "main", true);
  const b = control("b", 120, 0);
  a.element.focus();
  manager.dispatch("NavigateRight");
  expect(document.activeElement).toBe(b.element);
  const click = vi.fn();
  b.element.onclick = click;
  manager.dispatch("Confirm");
  expect(click).toHaveBeenCalledTimes(1);
});
test("modal traps navigation and restores previous focus", () => {
  const a = control("a", 0, 0);
  a.element.focus();
  manager.enterModal();
  const modal = control("dialog", 100, 0, "modal");
  manager.focus("dialog");
  manager.dispatch("NavigateLeft");
  expect(document.activeElement).toBe(modal.element);
  manager.trapTab(
    new KeyboardEvent("keydown", { key: "Tab", cancelable: true }),
  );
  expect(document.activeElement).toBe(modal.element);
  manager.leaveModal();
  expect(document.activeElement).toBe(a.element);
});
test("deleted focus restores to available fallback and disabled button cannot activate", () => {
  const a = control("a", 0, 0);
  const b = control("b", 80, 0);
  a.element.focus();
  manager.enterModal();
  a.unregister();
  a.element.remove();
  manager.leaveModal();
  expect(document.activeElement).toBe(b.element);
  b.element.disabled = true;
  const click = vi.fn();
  b.element.onclick = click;
  manager.dispatch("Confirm");
  expect(click).not.toHaveBeenCalled();
});
test("modal blocks page switching and background activation", () => {
  const a = control("a", 0, 0);
  a.element.focus();
  manager.enterModal();
  control("dialog", 10, 10, "modal");
  const click = vi.fn();
  a.element.onclick = click;
  manager.onTab = vi.fn();
  manager.dispatch("Confirm");
  manager.dispatch("NextTab");
  expect(click).not.toHaveBeenCalled();
  expect(manager.onTab).not.toHaveBeenCalled();
});
test("mouse focus becomes controller focus; keyboard activation stays native", () => {
  control("a", 0, 0);
  const b = control("b", 80, 0);
  b.element.focus();
  const click = vi.fn();
  b.element.onclick = click;
  manager.dispatch("Confirm");
  expect(click).toHaveBeenCalledTimes(1);
  expect(
    keyboardAction(new KeyboardEvent("keydown", { key: "Enter" })),
  ).toBeUndefined();
  expect(
    keyboardAction(new KeyboardEvent("keydown", { key: "ArrowDown" })),
  ).toBe("NavigateDown");
});


test("entering sidebar targets the current page owner instead of closest item", () => {
  const nearest = control("other", 0, 100);
  const owner = control("owner", 0, 0);
  const content = control("content", 300, 100);
  manager.register("other", { element: nearest.element, scope: "main", group: "sidebar" });
  manager.register("owner", { element: owner.element, scope: "main", group: "sidebar" });
  manager.sidebarEntryId = "owner";
  content.element.focus();
  manager.dispatch("NavigateLeft");
  expect(document.activeElement).toBe(owner.element);
  manager.dispatch("NavigateDown");
  expect(document.activeElement).toBe(nearest.element);
  manager.sidebarEntryId = "other";
  content.element.focus();
  manager.dispatch("NavigateLeft");
  expect(document.activeElement).toBe(nearest.element);
});

test("left movement within content remains spatial before entering sidebar", () => {
  const side = control("owner", 0, 0);
  manager.register("owner", { element: side.element, scope: "main", group: "sidebar" });
  manager.sidebarEntryId = "owner";
  const left = control("left", 200, 0);
  const right = control("right", 400, 0);
  right.element.focus();
  manager.dispatch("NavigateLeft");
  expect(document.activeElement).toBe(left.element);
  manager.dispatch("NavigateLeft");
  expect(document.activeElement).toBe(side.element);
});


test("vertical navigation cannot cross the sidebar fence, including explicit neighbors", () => {
  const sidebar = control("sidebar", 0, 0);
  const content = control("content", 200, 100);
  manager.register("sidebar", { element: sidebar.element, scope: "main", group: "sidebar", neighbors: { NavigateDown: "content" } });
  manager.register("content", { element: content.element, scope: "main", group: "content", neighbors: { NavigateUp: "sidebar" } });
  const tab = control("tab", 200, 0);
  manager.register("tab", { element: tab.element, scope: "main", group: "tabs" });
  manager.activeTabId = "tab";
  manager.sidebarEntryId = "sidebar";
  sidebar.element.focus();
  manager.dispatch("NavigateDown");
  expect(document.activeElement).toBe(sidebar.element);
  manager.dispatch("NavigateRight");
  expect(document.activeElement).toBe(tab.element);
  content.element.focus();
  manager.dispatch("NavigateUp");
  expect(document.activeElement).toBe(tab.element);
  manager.dispatch("NavigateLeft");
  expect(document.activeElement).toBe(sidebar.element);
});


test("page updates preserve an existing sidebar or content focus", () => {
  const sidebar = control("sidebar", 0, 0);
  const content = control("content", 200, 0);
  sidebar.element.focus();
  manager.preserveFocus("content");
  expect(document.activeElement).toBe(sidebar.element);
  content.element.focus();
  manager.preserveFocus("sidebar");
  expect(document.activeElement).toBe(content.element);
  content.unregister(); content.element.remove();
  manager.preserveFocus("sidebar");
  expect(document.activeElement).toBe(sidebar.element);
});

test("modal cleanup without a modal does not steal focus", () => {
  control("first", 0, 0);
  const active = control("active", 100, 0); active.element.focus();
  manager.leaveModal();
  expect(document.activeElement).toBe(active.element);
});


test("controller resumes the last user focus after a blank-area click loses DOM focus", () => {
  control("account", 0, 0);
  const play = control("play", 200, 100);
  play.element.focus(); play.element.blur();
  manager.resumeControllerFocus();
  expect(document.activeElement).toBe(play.element);
  const confirm = vi.fn(); play.element.onclick = confirm;
  manager.dispatch("Confirm");
  expect(confirm).toHaveBeenCalledTimes(1);
});

test("controller preserves live focus and falls back to page owner for removed controls", () => {
  control("account", 0, 0);
  const owner = control("owner", 0, 100);
  manager.sidebarEntryId = "owner";
  const content = control("content", 200, 100);
  content.element.focus(); manager.resumeControllerFocus();
  expect(document.activeElement).toBe(content.element);
  content.unregister(); content.element.remove();
  manager.resumeControllerFocus();
  expect(document.activeElement).toBe(owner.element);
});

test("controller takeover in a modal never restores background focus", () => {
  const play = control("play", 200, 100); play.element.focus(); manager.enterModal();
  const dialog = control("dialog", 100, 100, "modal");
  dialog.element.focus(); dialog.element.blur();
  manager.resumeControllerFocus();
  expect(document.activeElement).toBe(dialog.element);
});


test("hidden and inert ancestors cannot receive navigation or confirmation", () => {
  const visible = control("visible", 0, 0);
  const hidden = control("hidden", 100, 0);
  const wrapper = document.createElement("div"); document.body.append(wrapper); wrapper.append(hidden.element);
  const click = vi.fn(); hidden.element.onclick = click;
  for (const mode of ["hidden", "inert", "display"] as const) {
    wrapper.hidden = mode === "hidden";
    wrapper.toggleAttribute("inert", mode === "inert");
    wrapper.style.display = mode === "display" ? "none" : "";
    visible.element.focus(); manager.dispatch("NavigateRight");
    expect(document.activeElement).toBe(visible.element);
    hidden.element.focus(); manager.dispatch("Confirm");
    expect(click).not.toHaveBeenCalled();
  }
});


function region(id: string, x: number, y: number, group: string, neighbors = {}) {
  const item = control(id, x, y);
  manager.register(id, { element: item.element, scope: "main", group, neighbors });
  return item.element;
}
test("tabs and content only cross vertically, including explicit neighbors", () => {
  const tab = region("tab", 100, 0, "tabs", { NavigateRight: "content" });
  const content = region("content", 200, 100, "content", { NavigateLeft: "tab" });
  manager.activeTabId = "tab";
  tab.focus(); manager.dispatch("NavigateRight");
  expect(document.activeElement).toBe(tab);
  manager.dispatch("NavigateDown"); expect(document.activeElement).toBe(content);
  manager.dispatch("NavigateLeft"); expect(document.activeElement).toBe(content);
  manager.dispatch("NavigateUp"); expect(document.activeElement).toBe(tab);
});
test("content returns to selected tab without activating it or choosing nearest tab", () => {
  const selected = region("selected", 100, 0, "tabs");
  region("nearest", 300, 0, "tabs");
  const content = region("content", 300, 100, "content");
  manager.activeTabId = "selected";
  const click = vi.fn(); selected.onclick = click;
  content.focus(); manager.dispatch("NavigateUp");
  expect(document.activeElement).toBe(selected); expect(click).not.toHaveBeenCalled();
  selected.disabled = true;
  content.focus(); manager.dispatch("NavigateUp");
  expect(document.activeElement).toBe(content);
});
test("sidebar enters leftmost tab and returns to page owner across both fences", () => {
  const owner = region("owner", 0, 100, "sidebar");
  const other = region("other", 0, 200, "sidebar");
  region("second", 300, 0, "tabs");
  const first = region("first", 100, 0, "tabs");
  const content = region("content", 100, 200, "content");
  manager.sidebarEntryId = "owner"; manager.activeTabId = "second";
  other.focus(); manager.dispatch("NavigateRight"); expect(document.activeElement).toBe(first);
  manager.dispatch("NavigateDown"); expect(document.activeElement).toBe(content);
  manager.dispatch("NavigateLeft"); expect(document.activeElement).toBe(owner);
  manager.dispatch("NavigateRight"); expect(document.activeElement).toBe(first);
});
test("dropdown interception takes precedence over all region routing", () => {
  const content = region("content", 100, 100, "content");
  region("tab", 100, 0, "tabs"); manager.activeTabId = "tab";
  content.focus(); manager.interceptAction = () => true;
  manager.dispatch("NavigateUp"); expect(document.activeElement).toBe(content);
});

test("modal inputs participate in navigation and Tab without activating background", () => {
  const origin = control("create", 0, 0); origin.element.focus(); manager.enterModal();
  const input = document.createElement("input"); document.body.append(input);
  input.getBoundingClientRect = () => ({ left: 0, right: 100, top: 0, bottom: 30 } as DOMRect);
  manager.register("name", { element: input, scope: "modal", group: "content" });
  const save = control("save", 0, 100, "modal");
  input.focus(); manager.dispatch("NavigateDown"); expect(document.activeElement).toBe(save.element);
  manager.trapTab(new KeyboardEvent("keydown", { key: "Tab", cancelable: true }));
  expect(document.activeElement).toBe(input);
  manager.leaveModal(); expect(document.activeElement).toBe(origin.element);
});
test("deleting the final configuration restores focus to the provided creation fallback", () => {
  control("account", 0, 0);
  const create = control("create", 100, 0);
  const remove = control("delete", 200, 0); remove.element.focus();
  manager.enterModal(); remove.element.disabled = true;
  manager.leaveModal("create"); expect(document.activeElement).toBe(create.element);
});


test("down into modal actions prefers the primary button and skips it when disabled", () => {
  const field = control("field", 100, 0, "modal");
  const primary = control("primary", 0, 80, "modal");
  const cancel = control("cancel", 100, 80, "modal");
  const row = document.createElement("div"); row.setAttribute("data-navigation-actions", "");
  document.body.append(row); row.append(primary.element, cancel.element);
  primary.element.dataset.tone = "green";
  manager.enterModal(); field.element.focus();
  manager.dispatch("NavigateDown"); expect(document.activeElement).toBe(primary.element);
  manager.dispatch("NavigateRight"); expect(document.activeElement).toBe(cancel.element);
  primary.element.disabled = true; field.element.focus();
  manager.dispatch("NavigateDown"); expect(document.activeElement).toBe(cancel.element);
});

test("log modal scrolling stays inside the dialog and respects dropdown interception",()=>{
  const page=document.createElement("div"),log=document.createElement("pre");
  page.scrollBy=vi.fn();log.scrollBy=vi.fn();manager.scroll=page;manager.modalScroll=log;
  manager.enterModal();manager.dispatch("ScrollDown");
  expect(log.scrollBy).toHaveBeenCalledTimes(1);expect(page.scrollBy).not.toHaveBeenCalled();
  manager.interceptAction=()=>true;manager.dispatch("ScrollUp");
  expect(log.scrollBy).toHaveBeenCalledTimes(1);
  manager.interceptAction=undefined;manager.leaveModal();manager.dispatch("ScrollDown");
  expect(page.scrollBy).toHaveBeenCalledTimes(1);
});

test("modal Tab follows DOM order when conditional controls register later", () => {
  const first = control("first", 0, 0, "modal");
  const last = control("last", 0, 100, "modal");
  const inserted = control("inserted", 0, 50, "modal");
  last.element.before(inserted.element);
  manager.enterModal(); first.element.focus();
  manager.trapTab(new KeyboardEvent("keydown", {key:"Tab",cancelable:true}));
  expect(document.activeElement).toBe(inserted.element);
  manager.trapTab(new KeyboardEvent("keydown", {key:"Tab",cancelable:true}));
  expect(document.activeElement).toBe(last.element);
  manager.trapTab(new KeyboardEvent("keydown", {key:"Tab",shiftKey:true,cancelable:true}));
  expect(document.activeElement).toBe(inserted.element);
});

test("controller scroll reaches ordinary dialogs without a dedicated content scroller", () => {
  const dialog=document.createElement("section"); dialog.scrollBy=vi.fn();
  manager.modalContainer=dialog; manager.enterModal(); manager.dispatch("ScrollDown");
  expect(dialog.scrollBy).toHaveBeenCalledWith({top:240,behavior:"smooth"});
});

test("vertical navigation visits offset small buttons before a farther aligned dropdown", () => {
  const top = control("top", 125, 0);
  const middle = control("middle", 400, 100);
  const bottom = control("bottom", 125, 180);
  top.element.focus(); manager.dispatch("NavigateDown");
  expect(document.activeElement).toBe(middle.element);
  manager.dispatch("NavigateDown"); expect(document.activeElement).toBe(bottom.element);
  manager.dispatch("NavigateUp"); expect(document.activeElement).toBe(middle.element);
  middle.element.disabled = true; bottom.element.focus();
  manager.dispatch("NavigateUp"); expect(document.activeElement).toBe(top.element);
});

test("vertical navigation ignores height differences within the current row and picks nearest horizontal target in next row", () => {
  const top = control("top", 100, 0);
  control("same-row", 170, 4);
  control("next-left", 0, 80);
  const next = control("next-right", 100, 84);
  control("far-aligned", 100, 150);
  top.element.focus(); manager.dispatch("NavigateDown");
  expect(document.activeElement).toBe(next.element);
});
