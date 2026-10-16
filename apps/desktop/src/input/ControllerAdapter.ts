import type { Action } from "./actions";

export type Button = "Up" | "Down" | "Left" | "Right" | "South" | "East" | "LB" | "RB" | "Menu";
export type Mapping = Record<Button, Action>;
export const xboxMapping: Mapping = {
  Up: "NavigateUp", Down: "NavigateDown", Left: "NavigateLeft", Right: "NavigateRight",
  South: "Confirm", East: "Back", LB: "PreviousTab", RB: "NextTab", Menu: "OpenMenu",
};
export type ControllerFrame = {
  buttons: readonly Button[];
  leftX: number;
  leftY: number;
  rightY: number;
};
export const neutralFrame = (): ControllerFrame => ({ buttons: [], leftX: 0, leftY: 0, rightY: 0 });
const repeatable = (a: Action) => a.startsWith("Navigate") || a.startsWith("Scroll");

/** One active source supplies snapshots. No DOM knowledge or device polling here. */
export class ControllerAdapter {
  private held = new Map<Action, number>();
  private armed = false;
  private left?: Action;
  private right?: Action;
  constructor(private emit: (action: Action) => void, private mapping: Mapping = xboxMapping) {}

  reset() {
    this.held.clear();
    this.left = this.right = undefined;
    this.armed = false;
  }
  remap(mapping: Partial<Mapping>) {
    this.mapping = { ...this.mapping, ...mapping };
    this.reset();
  }
  update(frame: ControllerFrame, now: number) {
    if (![frame.leftX, frame.leftY, frame.rightY, now].every(Number.isFinite)) {
      this.reset();
      return;
    }
    const magnitude = Math.max(Math.abs(frame.leftX), Math.abs(frame.leftY));
    if (!this.armed) {
      // Reconnect / focus takeover must return to neutral before any activation.
      this.armed = frame.buttons.length === 0 && magnitude < .18 && Math.abs(frame.rightY) < .18;
      return;
    }
    const leftThreshold = this.left ? .18 : .25;
    this.left = magnitude > leftThreshold
      ? Math.abs(frame.leftX) > Math.abs(frame.leftY)
        ? frame.leftX > 0 ? "NavigateRight" : "NavigateLeft"
        : frame.leftY > 0 ? "NavigateDown" : "NavigateUp"
      : undefined;
    this.right = Math.abs(frame.rightY) > (this.right ? .18 : .25)
      ? frame.rightY > 0 ? "ScrollDown" : "ScrollUp" : undefined;
    const actions = new Set(frame.buttons.map(b => this.mapping[b]));
    if (this.left && !frame.buttons.some(b => ["Up", "Down", "Left", "Right"].includes(b))) actions.add(this.left);
    for (const [a, b] of [["NavigateUp", "NavigateDown"], ["NavigateLeft", "NavigateRight"], ["Confirm", "Back"], ["PreviousTab", "NextTab"]] as const) {
      if (actions.has(a) && actions.has(b)) { actions.delete(a); actions.delete(b); }
    }
    if (this.right) actions.add(this.right);
    for (const action of this.held.keys()) if (!actions.has(action)) this.held.delete(action);
    for (const action of actions) {
      const due = this.held.get(action);
      if (due === undefined) {
        this.held.set(action, now + 350);
        this.emit(action);
      } else if (repeatable(action) && now >= due) {
        this.held.set(action, now + 100);
        this.emit(action);
      }
    }
  }
}
