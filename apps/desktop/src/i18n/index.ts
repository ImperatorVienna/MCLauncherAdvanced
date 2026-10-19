import { useSyncExternalStore } from "react";
import english from "./en-US.json";
import ui from "./zh-CN.json";
import backend from "./backend.zh-CN.json";
import backendEnglish from "./backend.en-US.json";

export type Locale = "en-US" | "zh-CN";
const preferenceKey = "launcher.language.v1";
export function readLocale(): Locale {
  try { return localStorage.getItem(preferenceKey) === "zh-CN" ? "zh-CN" : "en-US"; }
  catch { return "en-US"; }
}
export let locale: Locale = readLocale();
const listeners = new Set<() => void>();
export function setLocale(next: Locale) {
  if (next !== "en-US" && next !== "zh-CN") return;
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
type Values = Record<string, string | number>;
export function t(key: MessageKey, values: Values = {}): string {
  const message = locale === "zh-CN" ? messages[key] : (englishMessages as Partial<Record<MessageKey,string>>)[key] ?? key;
  return message.replace(/\{(\w+)\}/g, (token, name: string) => String(values[name] ?? token));
}

// Rust reports English diagnostics. Localize known templates at the presentation
// boundary, preserving paths, user names, and unknown OS/network diagnostics.
const templates = Object.entries(backend).map(([key, translated]) => {
  const names: string[] = [];
  const escaped = key.split(/(\{\w+\})/g).map(part => {
    if (/^\{\w+\}$/.test(part)) { names.push(part.slice(1, -1)); return "([\\s\\S]+?)"; }
    return part.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  }).join("");
  return { pattern: new RegExp(`^${escaped}$`), names, translated, english: englishMessages[key as keyof typeof englishMessages] ?? key };
});
export function localizeMessage(message: unknown): string {
  const text = message instanceof Error ? message.message : String(message ?? "");
  return translateDiagnostic(text, 0);
}
function translateDiagnostic(text: string, depth: number): string {
  if (depth > 8) return text;
  for (const { pattern, names, translated, english } of templates) {
    const match = pattern.exec(text);
    if (match) return (locale === "zh-CN" ? translated : english).replace(/\{(\w+)\}/g, (token, name: string) => {
      const index = names.indexOf(name);
      const value = match[index + 1];
      return index < 0 ? token : name === "e" ? translateDiagnostic(value, depth + 1) : value;
    });
  }
  return text;
}
