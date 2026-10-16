import type { Action } from "../input/actions";
type Entry = {
  element: HTMLElement;
  scope: string;
  group: string;
  neighbors?: Partial<Record<Action, string>>;
};
export class FocusManager {
  private entries = new Map<string, Entry>();
  private scope = "main";
  private returnId?: string;
  private lastFocusByScope = new Map<string, string>();
  interceptAction?: (action: Action) => boolean;
  textAction?: (action: Action) => boolean;
  cancelEditor?: () => boolean;
  onBack = () => {};
  onTab = (_delta: number) => {};
  onMenu = () => {};
  scroll: HTMLElement | null = null;
  modalScroll: HTMLElement | null = null;
  modalContainer: HTMLElement | null = null;
  sidebarEntryId?: string;
  activeTabId?: string;

  // Region fences apply to directional navigation, not pointer or native Tab.
  private canCross(from: Entry, to: Entry, horizontal: boolean) {
    if (this.scope !== "main") return true;
    if ((from.group === "sidebar") !== (to.group === "sidebar")) return horizontal;
    if ((from.group === "tabs") !== (to.group === "tabs")) return !horizontal;
    return true;
  }

  private focusNavigationTarget(id: string, from: Entry) {
    const target = this.entries.get(id);
    if (this.scope === "main" && from.group !== "sidebar" && target?.group === "sidebar") {
      const selected = this.available().find(
        ([key, entry]) => key === this.sidebarEntryId && entry.group === "sidebar",
      );
      // Do not enter an unrelated sidebar item if the page owner is unavailable.
      if (selected) this.focus(selected[0]);
      return;
    }
    if (this.scope === "main" && from.group === "sidebar" && target?.group !== "sidebar") {
      const first = this.available().filter(([, entry]) => entry.group === "tabs")
        .sort((a, b) => a[1].element.getBoundingClientRect().left - b[1].element.getBoundingClientRect().left)[0];
      if (first) this.focus(first[0]);
      return;
    }
    if (this.scope === "main" && from.group !== "tabs" && target?.group === "tabs") {
      const selected = this.available().find(([key, entry]) => key === this.activeTabId && entry.group === "tabs");
      if (selected) this.focus(selected[0]);
      return;
    }
    this.focus(id);
  }
  register(id: string, entry: Entry) {
    this.entries.set(id, entry);
    const remember = () => this.lastFocusByScope.set(entry.scope, id);
    entry.element.addEventListener("focus", remember);
    entry.element.addEventListener("pointerdown", remember);
    return () => {
      entry.element.removeEventListener("focus", remember);
      entry.element.removeEventListener("pointerdown", remember);
      if (this.entries.get(id) === entry) this.entries.delete(id);
    };
  }
  private hasDisplayNoneAncestor(element: HTMLElement) {
    for (let current: HTMLElement | null = element; current; current = current.parentElement) {
      if (getComputedStyle(current).display === "none") return true;
    }
    return false;
  }
  private visible() {
    return [...this.entries].filter(
      ([, e]) =>
        e.scope === this.scope && e.element.isConnected &&
        !e.element.closest('[hidden], [inert], [aria-hidden="true"]') &&
        getComputedStyle(e.element).visibility !== "hidden" &&
        !this.hasDisplayNoneAncestor(e.element),
    );
  }
  private available() {
    return this.visible().filter(([, entry]) => !entry.element.matches(':disabled, [aria-disabled="true"]'));
  }
  private current() {
    return this.visible().find(([, entry]) => entry.element === document.activeElement);
  }
  focus(id?: string) {
    const entry =
      this.available().find(([key]) => key === id) ?? this.available()[0];
    entry?.[1].element.focus();
    entry?.[1].element.scrollIntoView?.({ block: "nearest" });
  }
  rememberFocus(id: string) {
    const entry = this.entries.get(id);
    if (entry) this.lastFocusByScope.set(entry.scope, id);
  }
  resumeControllerFocus() {
    const items = this.available();
    if (this.current()) return;
    const remembered = this.lastFocusByScope.get(this.scope);
    const target = items.find(([id]) => id === remembered)
      ?? items.find(([id]) => this.scope === "main" && id === this.sidebarEntryId)
      ?? items[0];
    if (target) this.focus(target[0]);
  }
  preserveFocus(fallbackId: string) {
    if (!this.current()) {
      this.focus(fallbackId);
    }
  }
  activeId() {
    return [...this.entries].find(
      ([, e]) => e.element === document.activeElement,
    )?.[0];
  }
  enterModal() {
    this.returnId = this.activeId();
    this.scope = "modal";
  }
  leaveModal(fallbackId?: string) {
    if (this.scope !== "modal") return;
    this.scope = "main";
    const target = this.available().some(([id]) => id === this.returnId) ? this.returnId : fallbackId;
    this.focus(target);
    this.returnId = undefined;
  }
  trapTab(event: KeyboardEvent) {
    if (this.scope !== "modal" || event.key !== "Tab") return;
    event.preventDefault();
    // Conditional controls can register later while appearing earlier in the DOM.
    const items = this.available().sort((a, b) =>
      a[1].element.compareDocumentPosition(b[1].element) & Node.DOCUMENT_POSITION_FOLLOWING ? -1 : 1);
    const index = items.findIndex(
      ([, e]) => e.element === document.activeElement,
    );
    this.focus(
      items[
        (index < 0 ? (event.shiftKey ? items.length - 1 : 0) : (index + (event.shiftKey ? -1 : 1) + items.length) % items.length)
      ]?.[0],
    );
  }
  dispatch(action: Action) {
    if (this.interceptAction?.(action)) return;
    if (this.textAction?.(action)) return;
    const items = this.available();
    const current = this.current();
    if (action === "Back") { if (this.cancelEditor?.()) return; return this.onBack(); }
    if (action === "NextTab" || action === "PreviousTab") {
      if (this.scope === "main") this.onTab(action === "NextTab" ? 1 : -1);
      return;
    }
    if (action === "OpenMenu") {
      if (this.scope === "main") this.onMenu();
      return;
    }
    if (action === "ScrollDown" || action === "ScrollUp") {
      (this.scope === "main" ? this.scroll : this.modalScroll ?? this.modalContainer)?.scrollBy({
          top: action === "ScrollDown" ? 240 : -240,
          behavior: "smooth",
        });
      return;
    }
    if (!current) return this.focus();
    if (action === "Confirm") {
      if (items.some(([id]) => id === current[0])) current[1].element.click();
      return;
    }
    const horizontal = action === "NavigateLeft" || action === "NavigateRight";
    const canCrossTo = (entry: Entry) => this.canCross(current[1], entry, horizontal);
    const explicit = current[1].neighbors?.[action];
    if (horizontal && explicit && items.some(([id, entry]) => id === explicit && canCrossTo(entry)))
      return this.focusNavigationTarget(explicit, current[1]);
    const rect = current[1].element.getBoundingClientRect();
    const sign = action === "NavigateLeft" || action === "NavigateUp" ? -1 : 1;
    const candidates = items
      .filter(([id, entry]) => id !== current[0] && canCrossTo(entry))
      .map(([id, e]) => {
        const r = e.element.getBoundingClientRect();
        const dx = (r.left + r.right - rect.left - rect.right) / 2;
        const dy = (r.top + r.bottom - rect.top - rect.bottom) / 2;
        return {
          id,
          rect: r,
          edge: sign > 0 ? r.top - rect.bottom : rect.top - r.bottom,
          primary: (horizontal ? dx : dy) * sign,
          secondary: Math.abs(horizontal ? dy : dx),
        };
      })
      .filter((c) => c.primary > 1)
      .sort(
        (a, b) => a.primary + a.secondary * 3 - (b.primary + b.secondary * 3),
      );
    // Up/down visits the nearest visual row first. Horizontal distance must
    // never outweigh an intervening row, and different heights in the current
    // row must not turn a vertical move into a sideways move.
    let rowCandidates = candidates;
    if (!horizontal) {
      const vertical = candidates.filter(c => c.edge >= -1);
      const nearest = [...vertical].sort((a, b) => a.edge - b.edge)[0];
      rowCandidates = nearest ? vertical.filter(c =>
        Math.min(c.rect.bottom, nearest.rect.bottom) - Math.max(c.rect.top, nearest.rect.top) > 1)
        .sort((a, b) => a.secondary - b.secondary || a.primary - b.primary) : [];
    }
    const target = (!horizontal && explicit ? rowCandidates.find(c => c.id === explicit) : undefined) ?? rowCandidates[0];
    if (!target) return;
    // Enter a form action row through its enabled primary action, regardless of
    // the horizontal center of a wide field. Open menus intercept before here.
    if (action === "NavigateDown") {
      const row = this.entries.get(target.id)?.element.closest("[data-navigation-actions]");
      if (row && !row.contains(current[1].element)) {
        const primary = items.find(([id, entry]) => rowCandidates.some(c => c.id === id) && row.contains(entry.element) && entry.element.matches('[data-tone="green"]'));
        if (primary) return this.focusNavigationTarget(primary[0], current[1]);
      }
    }
    this.focusNavigationTarget(target.id, current[1]);
  }
}
