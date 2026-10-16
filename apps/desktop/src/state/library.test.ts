// @vitest-environment jsdom
import { setLocale } from "../i18n";
import { beforeEach, expect, test, vi } from "vitest";
import { emptyLibrary, initialSelection, loadLibrary, saveLibrary, previewKey, type Library } from "./library";
beforeEach(() => { setLocale("zh-CN"); localStorage.clear(); vi.restoreAllMocks(); });
function example(): Library {
  return { ...emptyLibrary(), selectedId: "one", instances: [{ id: "one", name: "\u539f\u7248", minecraftVersion: "1.21.1", icon: "grass", directoryName: "vanilla" }] };
}
test("browser configurations persist edits and deletion without fabricating launches", async () => {
  expect(await loadLibrary()).toEqual(emptyLibrary());
  const created = await saveLibrary(example());
  expect(created.lastLaunchedId).toBeNull();
  const edited = await saveLibrary({ ...created, instances: [{ ...created.instances[0], name: "\u6539\u540d", icon: "sand" }] });
  expect((await loadLibrary()).instances[0]).toMatchObject({ name: "\u6539\u540d", icon: "sand" });
  const deleted = await saveLibrary({ ...edited, instances: [], selectedId: null });
  expect(deleted.instances).toEqual([]);
});
test("validation, stale revisions and forged launch records leave saved data intact", async () => {
  const saved = await saveLibrary(example());
  await expect(saveLibrary(example())).rejects.toThrow("\u66f4\u65b0");
  await expect(saveLibrary({ ...saved, lastLaunchedId: "one" })).rejects.toThrow("\u542f\u52a8\u8bb0\u5f55");
  await expect(saveLibrary({ ...saved, instances: [...saved.instances, { ...saved.instances[0], id: "two" }] })).rejects.toThrow("\u540d\u79f0");
  await expect(saveLibrary({ ...saved, selectedId: "missing" })).rejects.toThrow("\u5f15\u7528");
  expect(await loadLibrary()).toEqual(saved);
});
test("corruption and storage failures are surfaced, never silently replaced", async () => {
  localStorage.setItem(previewKey, "broken");
  await expect(loadLibrary()).rejects.toThrow();
  await expect(saveLibrary(example())).rejects.toThrow();
  expect(localStorage.getItem(previewKey)).toBe("broken");
  localStorage.clear();
  vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw Error("disk full"); });
  await expect(saveLibrary(example())).rejects.toThrow("disk full");
});
test("native mode uses Rust commands and never silently falls back to browser storage", async () => {
  const invoke = vi.fn().mockResolvedValueOnce(example()).mockRejectedValueOnce("native write failed");
  Object.defineProperty(window, "__TAURI__", { configurable: true, value: { core: { invoke } } });
  try {
    expect(await loadLibrary()).toEqual(example());
    await expect(saveLibrary(example())).rejects.toBe("native write failed");
    expect(invoke).toHaveBeenLastCalledWith("save_library", { library: example() });
    expect(localStorage.getItem(previewKey)).toBeNull();
  } finally { Reflect.deleteProperty(window, "__TAURI__"); }
});


test("startup prefers last successful launch without rewriting launch history on selection", async () => {
  const data = example();
  data.instances.push({ ...data.instances[0], id: "two", name: "\u53e6\u4e00\u914d\u7f6e" });
  data.lastLaunchedId = "one"; data.selectedId = "two";
  localStorage.setItem(previewKey, JSON.stringify(data));
  expect(initialSelection(await loadLibrary())).toBe("one");
  const selected = await saveLibrary({ ...data, selectedId: "two" });
  expect(selected.lastLaunchedId).toBe("one");
  const removed = await saveLibrary({ ...selected, instances: [data.instances[1]], lastLaunchedId: null });
  expect(initialSelection(removed)).toBe("two");
});

test("invalid icons are rejected without rewriting the stored file", async () => {
  const raw = JSON.stringify(example()).replace('"icon":"grass"', '"icon":"stone"');
  localStorage.setItem(previewKey, raw);
  await expect(loadLibrary()).rejects.toThrow();
  expect(localStorage.getItem(previewKey)).toBe(raw);
});

test("old schemas are rejected instead of implicitly migrated or overwritten", async () => {
  const raw = JSON.stringify({ ...example(), schemaVersion: 1 });
  localStorage.setItem(previewKey, raw);
  await expect(loadLibrary()).rejects.toThrow();
  await expect(saveLibrary(example())).rejects.toThrow();
  expect(localStorage.getItem(previewKey)).toBe(raw);
});

test("Java paths accept portable native Windows and Unix absolute paths", async () => {
  for (const path of ["/opt/java/bin/java", "C:\\Java\\bin\\java.exe", "C:/Java/bin/java.exe", "\\\\?\\C:\\Java\\bin\\java.exe", "\\\\server\\share\\java.exe"]) {
    localStorage.clear();
    const value=example();value.instances[0].javaPath=path;
    expect((await saveLibrary(value)).instances[0].javaPath).toBe(path);
  }
  for(const path of ["java.exe", "C:java.exe", "../java", "C:\\Java\0bin"]) {
    localStorage.clear();
    const value=example();value.instances[0].javaPath=path;
    await expect(saveLibrary(value)).rejects.toThrow();
  }
});
