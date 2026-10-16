import { expect, test, vi } from "vitest";
import { ControllerAdapter, neutralFrame } from "./ControllerAdapter";
function setup() {
  const emit = vi.fn(); const adapter = new ControllerAdapter(emit);
  adapter.update(neutralFrame(), 0);
  return { emit, adapter };
}
test("confirm fires once per press while directions repeat after delay", () => {
  const { emit, adapter } = setup();
  const frame = { ...neutralFrame(), buttons: ["South", "Down"] as const };
  for (const t of [1, 100, 350, 351, 400, 451]) adapter.update(frame, t);
  expect(emit.mock.calls.map(c => c[0])).toEqual(["Confirm", "NavigateDown", "NavigateDown", "NavigateDown"]);
});
test("stick deadzone, hysteresis and direction change", () => {
  const { emit, adapter } = setup();
  for (const [leftX, t] of [[.2,1],[.3,2],[.2,400],[.1,500],[-.5,600]]) adapter.update({...neutralFrame(),leftX},t);
  expect(emit.mock.calls.map(c=>c[0])).toEqual(["NavigateRight","NavigateRight","NavigateLeft"]);
});
test("reset blocks held activation until a neutral frame", () => {
  const { emit, adapter } = setup(); adapter.reset();
  adapter.update({...neutralFrame(),buttons:["South"]},1); expect(emit).not.toHaveBeenCalled();
  adapter.update(neutralFrame(),2); adapter.update({...neutralFrame(),buttons:["South"]},3);
  expect(emit).toHaveBeenCalledExactlyOnceWith("Confirm");
});
test("alternative controller mappings and stick / d-pad deduplication", () => {
  const { emit, adapter } = setup(); adapter.remap({East:"Confirm"});adapter.update(neutralFrame(),0);
  adapter.update({...neutralFrame(),buttons:["East","Right"],leftX:1},1);
  expect(emit.mock.calls.map(c=>c[0])).toEqual(["Confirm","NavigateRight"]);
});
test("shoulder, menu and right stick actions", () => {
  const { emit, adapter } = setup();
  adapter.update({...neutralFrame(),buttons:["LB","RB","Menu"],rightY:-1},1);
  expect(emit.mock.calls.map(c=>c[0])).toEqual(["OpenMenu","ScrollUp"]);
});

test("opposed controls cancel and d-pad has priority over the left stick", () => {
  const {emit,adapter}=setup();
  adapter.update({...neutralFrame(),buttons:["South","East","Up","Down"]},1);
  expect(emit).not.toHaveBeenCalled();
  adapter.update(neutralFrame(),2);
  adapter.update({...neutralFrame(),buttons:["Right"],leftX:-1},3);
  expect(emit).toHaveBeenCalledExactlyOnceWith("NavigateRight");
});
