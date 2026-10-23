// @vitest-environment jsdom
import { beforeEach, expect, test } from "vitest";
import ui from "./zh-CN.json";
import backend from "./backend.zh-CN.json";
import english from "./en-US.json";
import backendEnglish from "./backend.en-US.json";
import { readLocale, setLocale, localizeMessage, t } from ".";
import { pageLabel } from "../state/pages";
import { channels, channelLabel, familyOf, familyLabel } from "../state/versionCatalog";

beforeEach(()=>setLocale("zh-CN"));

test("all translations preserve their English message placeholders", () => {
  const placeholders = (s: string) => [...s.matchAll(/\{(\w+)\}/g)].map(m => m[1]).sort();
  for (const [key, value] of Object.entries({...backend, ...ui})) {
    expect(value.trim(), key).not.toBe("");
    expect(placeholders(value), key).toEqual(placeholders(key));
    expect(key, key).not.toMatch(/\p{Script=Han}/u);
  }
});
test("English routes and version channels have independent Chinese display labels", () => {
  expect(pageLabel("instances")).toBe(t("Instances"));
  expect(channels).toEqual(["release", "snapshot", "preview"]);
  expect(channelLabel("release")).toBe(t("Release"));
  const family = familyOf({id: "23w31a", type: "snapshot", releaseTime: ""});
  expect(family).toBe("snapshots-2023");
  expect(familyLabel(family)).toBe(t("{value0} snapshots", {value0: "2023"}));
});
test("backend diagnostics and progress are localized without changing user values", () => {
  expect(localizeMessage("Installation complete")).toBe(t("Installation complete"));
  expect(localizeMessage("Fetching Java 25 download metadata")).toBe(t("Fetching Java {major} download metadata", {major: 25}));
  expect(localizeMessage('Instance name conflict: "Save $& {e}" already exists in versions; choose another name'))
    .toBe(t('Instance name conflict: "{0}" already exists in versions; choose another name', {0: "Save $& {e}"}));
  expect(localizeMessage("Failed to save the instance library: Invalid Java path"))
    .toBe(t("Failed to save the instance library: {e}", {e: t("Invalid Java path")}));
  expect(localizeMessage(new Error("OS error 123"))).toBe("OS error 123");
  expect(localizeMessage(null)).toBe("");
});


test("English catalog covers every UI message and preserves placeholders",()=>{
  const placeholders=(value:string)=>[...value.matchAll(/\{(\w+)\}/g)].map(m=>m[1]).sort();
  expect(Object.keys(english).sort()).toEqual(Object.keys(ui).sort());
  for(const [key,value] of Object.entries(english)) {
    expect(value.trim(),key).not.toBe("");
    expect(placeholders(value),key).toEqual(placeholders(key));
    if(key!=="Simplified Chinese")expect(value).not.toMatch(/\p{Script=Han}/u);
  }
});
test("fresh installation defaults to English and persists explicit language choice",()=>{
  localStorage.clear();
  expect(readLocale()).toBe("en-US");
  setLocale("zh-CN");
  expect(readLocale()).toBe("zh-CN");
  expect(document.documentElement.lang).toBe("zh-CN");
  setLocale("en-US");
  expect(readLocale()).toBe("en-US");
  expect(t("Accounts")).toBe("Account management");
  expect(t("Download Java {major}",{major:25})).toBe("Download Java 25");
  expect(localizeMessage("Fetching Java 25 download metadata")).toBe("Fetching Java 25 download metadata");
});

test("English diagnostics cover the Chinese catalog and preserve runtime values",()=>{
  expect(Object.keys(backendEnglish).sort()).toEqual(Object.keys(backend).sort());
  const placeholders=(value:string)=>[...value.matchAll(/\{(\w+)\}/g)].map(m=>m[1]).sort();
  for(const [key,value] of Object.entries(backendEnglish)) {
    expect(value.trim(),key).not.toBe("");
    expect(placeholders(value),key).toEqual(placeholders(key));
    expect(value).not.toMatch(/\p{Script=Han}/u);
  }
  setLocale("en-US");
  expect(localizeMessage("Game exited abnormally: exit status: 1")).toBe("Game exited abnormally: exit status: 1. Check the game log.");
  expect(localizeMessage("Selected Java 8 is too old; this game requires Java 21 or newer")).toContain("Select a suitable Java runtime and retry.");
  expect(localizeMessage("Failed to save the instance library: Maximum memory exceeds currently available memory")).toContain("lower it and retry");
  const path="/home/player/\u4e2d\u6587 $& {e}/game.log";
  expect(localizeMessage(`Official mod loader installer failed; see log: ${path}`)).toContain(path);
});

test("skin and clipboard diagnostics translate through the native error path", () => {
  for (const key of ["Invalid skin URL", "Skin image preview is unavailable", "Clipboard is unavailable", "Skin upload failed; retry", "Too many skin updates; try again later", "Skin update could not be confirmed; reload the account skin"] as const) {
    setLocale("zh-CN");
    expect(localizeMessage(new Error(key))).toBe(backend[key]);
    expect(localizeMessage(new Error(key))).not.toBe(key);
    setLocale("en-US");
    expect(localizeMessage(new Error(key))).toBe(backendEnglish[key]);
  }
  setLocale("zh-CN");
  expect(t("Skin updated")).toBe(ui["Skin updated"]);
});
