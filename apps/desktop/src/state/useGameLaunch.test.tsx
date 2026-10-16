// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test, vi } from "vitest";
import { useGameLaunch } from "./useGameLaunch";
import { invokeNative } from "../platform/desktop";
vi.mock("../platform/desktop", () => ({ invokeNative: vi.fn() }));
test("command failures persist across status polls and repeated launch is serialized", async () => {
    vi.useFakeTimers(); Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
    const host = document.createElement("div"); const root = createRoot(host);
    let game!: ReturnType<typeof useGameLaunch>;
    const idle = { running: false, preparing: false, instanceId: null, error: null, logPath: null };
    const invoke = vi.mocked(invokeNative);
    invoke.mockImplementation(async command => { if (command === "launch_game") throw Error("launch refused"); return idle; });
    function Test() { game = useGameLaunch(false); return null; }
    await act(async () => root.render(<Test/>));
    await act(async () => { await Promise.all([game.launch("one"),game.launch("one")]); });
    expect(invoke.mock.calls.filter(c => c[0] === "launch_game")).toHaveLength(1);
    expect(game.status.error).toBe("launch refused");
    await act(async () => { await vi.advanceTimersByTimeAsync(800); });
    expect(game.status.error).toBe("launch refused");
    act(() => root.unmount()); vi.useRealTimers();
});
