import { useSyncExternalStore } from "react";
import english from "./en-US.json";
import ui from "./zh-CN.json";
import backend from "./backend.zh-CN.json";
import backendEnglish from "./backend.en-US.json";

import traditional from "./zh-TW.json";
import backendTraditional from "./backend.zh-TW.json";

export const locales = ["en-US", "zh-CN", "zh-TW"] as const;
export type Locale = typeof locales[number];
const isLocale = (value: unknown): value is Locale => locales.includes(value as Locale);
const preferenceKey = "launcher.language.v1";
export function readLocale(): Locale {
  try { const saved = localStorage.getItem(preferenceKey); return isLocale(saved) ? saved : "en-US"; }
  catch { return "en-US"; }
}
export let locale: Locale = readLocale();
const listeners = new Set<() => void>();
export function setLocale(next: Locale) {
  if (!isLocale(next)) return;
  try { localStorage.setItem(preferenceKey, next); } catch { /* Session selection still works. */ }
  locale = next;
  if (typeof document !== "undefined") document.documentElement.lang = next;
  listeners.forEach(listener => listener());
}
const subscribe = (listener: () => void) => { listeners.add(listener); return () => { listeners.delete(listener); }; };
export const useLocale = () => useSyncExternalStore(subscribe, () => locale);
if (typeof document !== "undefined") document.documentElement.lang = locale;
const messages = { ...backend, ...ui };
const englishMessages = { ...backendEnglish, ...english };
export type MessageKey = keyof typeof messages;
const catalogs: Record<Locale, Record<MessageKey, string>> = {
  "en-US": englishMessages, "zh-CN": messages, "zh-TW": {...backendTraditional, ...traditional},
};
export const localeLabel = (value: Locale) => t(({"en-US":"English", "zh-CN":"Simplified Chinese", "zh-TW":"Traditional Chinese"} as const)[value]);
type Values = Record<string, string | number>;
export function t(key: MessageKey, values: Values = {}): string {
  const message = catalogs[locale][key] ?? englishMessages[key] ?? key;
  return message.replace(/\{(\w+)\}/g, (token, name: string) => String(values[name] ?? token));
}

// Rust reports English diagnostics. Localize known templates at the presentation
// boundary, preserving paths, user names, and unknown OS/network diagnostics.
const templates = Object.keys(backend).map(key => {
  const names: string[] = [];
  const escaped = key.split(/(\{\w+\})/g).map(part => {
    if (/^\{\w+\}$/.test(part)) { names.push(part.slice(1, -1)); return "([\\s\\S]+?)"; }
    return part.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  }).join("");
  return { pattern: new RegExp(`^${escaped}$`), names, key: key as MessageKey };
});
export function localizeMessage(message: unknown): string {
  const text = message instanceof Error ? message.message : String(message ?? "");
  return translateDiagnostic(text, 0);
}
function translateDiagnostic(text: string, depth: number): string {
  if (depth > 8) return text;
  for (const { pattern, names, key } of templates) {
    const match = pattern.exec(text);
    if (match) return catalogs[locale][key].replace(/\{(\w+)\}/g, (token, name: string) => {
      const index = names.indexOf(name);
      const value = match[index + 1];
      return index < 0 ? token : name === "e" ? translateDiagnostic(value, depth + 1) : value;
    });
  }
  return text;
}
