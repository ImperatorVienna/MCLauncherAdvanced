import { localizeMessage } from "../i18n";
import { ControllerAdapter, type ControllerFrame } from "./ControllerAdapter";
import type { Action } from "./actions";
import { invokeNative } from "../platform/desktop";
export type NativeDevice = { id: string; name: string };
export type NativeFrame = ControllerFrame & { id: string; at: number };
export type NativeBatch = { devices: NativeDevice[]; frames: NativeFrame[]; states: NativeFrame[]; overflow: boolean; suspended: boolean; error: string | null };
export type NativeStatus = { devices: NativeDevice[]; error: string | null };
const neutral = (f: ControllerFrame) => f.buttons.length === 0 && Math.max(Math.abs(f.leftX), Math.abs(f.leftY), Math.abs(f.rightY)) < .18;

/** Keeps devices independent; one held controller owns the action stream until neutral. */
export class NativeControllerRouter {
  private adapters = new Map<string, ControllerAdapter>();
  private owner?: string;
  constructor(private emit: (action: Action, device: NativeDevice) => void) {}
  reset() { this.adapters.forEach(a => a.reset()); this.owner = undefined; }
  update(batch: NativeBatch, now: number) {
    const devices = new Map(batch.devices.map(d => [d.id, d]));
    for (const id of this.adapters.keys()) if (!devices.has(id)) this.adapters.delete(id);
    if (this.owner && !devices.has(this.owner)) this.owner = undefined;
    if (batch.suspended || batch.error) { this.reset(); return; }
    if (batch.overflow) this.reset();
    for (const device of batch.devices) {
      if (!this.adapters.has(device.id)) this.adapters.set(device.id, new ControllerAdapter(action => {
        if (this.owner && this.owner !== device.id) return;
        this.owner = device.id; this.emit(action, device);
      }));
    }
    // On overflow only the final state is safe; replaying stale confirmations is not.
    for (const frame of [...(batch.overflow ? [] : batch.frames), ...batch.states]) {
      if (this.owner === frame.id && neutral(frame)) this.owner = undefined;
      if (this.owner && this.owner !== frame.id) this.adapters.get(frame.id)?.reset();
      this.adapters.get(frame.id)?.update(frame, now);
    }
  }
}

/** Serialized polling drains edge snapshots, preserving taps shorter than one IPC round trip. */
export function installNativeController(emit: (action: Action, device: NativeDevice) => void, status: (value: NativeStatus) => void) {
  const router = new NativeControllerRouter(emit);
  let stopped = false, composing = false, timer = 0, epoch = 0, lastStatus = "";
  const reset = () => { epoch++; router.reset(); };
  const takeover = (event: Event) => { if ((event as CustomEvent).detail !== "native") reset(); };
  const poll = async () => {
    const started = performance.now(), generation = epoch;
    try {
      const batch = await invokeNative<NativeBatch>("poll_gamepads");
      if (stopped) return;
      const next = JSON.stringify({ devices: batch.devices, error: batch.error ? localizeMessage(batch.error) : null });
      if (next !== lastStatus) { lastStatus = next; status({ devices: batch.devices, error: batch.error ? localizeMessage(batch.error) : null }); }
      if (generation !== epoch || !document.hasFocus() || document.hidden || composing || performance.now() - started > 250) router.reset();
      else router.update(batch, performance.now());
    } catch (e) {
      if (stopped) return;
      router.reset(); const message = localizeMessage(e);
      if (message !== lastStatus) { lastStatus = message; status({ devices: [], error: message }); }
    }
    if (!stopped) timer = window.setTimeout(poll, document.hidden ? 200 : 16);
  };
  const compositionStart = () => { composing = true; reset(); };
  const compositionEnd = () => { composing = false; reset(); };
  window.addEventListener("compositionstart", compositionStart);
  window.addEventListener("compositionend", compositionEnd);
  window.addEventListener("pointerdown", reset, true);
  window.addEventListener("keydown", reset, true);
  window.addEventListener("blur", reset);
  document.addEventListener("visibilitychange", reset);
  window.addEventListener("launcher:input-takeover", takeover);
  void poll();
  return () => {
    stopped = true; clearTimeout(timer); reset();
    window.removeEventListener("compositionstart", compositionStart); window.removeEventListener("compositionend", compositionEnd);
    window.removeEventListener("pointerdown", reset, true); window.removeEventListener("keydown", reset, true);
    window.removeEventListener("blur", reset); document.removeEventListener("visibilitychange", reset);
    window.removeEventListener("launcher:input-takeover", takeover);
  };
}
