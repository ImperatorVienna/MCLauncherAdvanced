// @vitest-environment jsdom
import { expect, test, vi } from "vitest";
import { NativeControllerRouter, type NativeBatch, type NativeFrame } from "./nativeController";
import { neutralFrame } from "./ControllerAdapter";
const device = { id: "1", name: "Test pad" };
const f = (buttons: NativeFrame["buttons"] = [], id = "1"): NativeFrame => ({ ...neutralFrame(), buttons, id, at: 0 });
const batch = (states: NativeFrame[], frames: NativeFrame[] = [], overrides: Partial<NativeBatch> = {}): NativeBatch => ({devices:[device],states,frames,error:null,overflow:false,suspended:false,...overrides});
test("native quick taps survive batching and use positional face keys", () => {
  const emit=vi.fn(),router=new NativeControllerRouter(emit);router.update(batch([f()]),0);
  router.update(batch([f()], [f(["South"]),f(),f(["East"]),f()]),1);
  expect(emit.mock.calls.map(c=>c[0])).toEqual(["Confirm","Back"]);
});
test("disconnect, overflow and focus takeover require neutral before activation", () => {
  const emit=vi.fn(),router=new NativeControllerRouter(emit);router.update(batch([f()]),0);
  router.update(batch([f(["South"])],[],{suspended:true}),1);
  router.update(batch([f(["South"])]),2);expect(emit).not.toHaveBeenCalled();
  router.update(batch([f()]),3);router.update(batch([f(["South"])]),4);expect(emit).toHaveBeenCalledTimes(1);
  router.update(batch([],[],{devices:[]}),5);router.update(batch([f(["South"])]),6);expect(emit).toHaveBeenCalledTimes(1);
  router.update(batch([f()], [f(["South"])],{overflow:true}),7);expect(emit).toHaveBeenCalledTimes(1);
  router.update(batch([f(["South"])]),8);expect(emit).toHaveBeenCalledTimes(2);
  router.reset();router.update(batch([f(["South"])]),9);expect(emit).toHaveBeenCalledTimes(2);
});
test("two devices cannot combine a held direction and confirmation", () => {
  const emit=vi.fn(),router=new NativeControllerRouter(emit);const devices=[device,{id:"2",name:"Other"}];
  router.update(batch([f(),f([],"2")],[],{devices}),0);
  router.update(batch([f(["Down"]),f(["South"],"2")],[],{devices}),1);
  expect(emit.mock.calls.map(c=>c[0])).toEqual(["NavigateDown"]);
  router.update(batch([f(),f(["South"],"2")],[],{devices}),500);
  expect(emit).toHaveBeenCalledTimes(1);
  router.update(batch([f(),f([],"2")],[],{devices}),501);
  router.update(batch([f(),f(["South"],"2")],[],{devices}),502);
  expect(emit).toHaveBeenLastCalledWith("Confirm",devices[1]);
});
