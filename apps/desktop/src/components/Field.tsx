import { useContext, useLayoutEffect, useRef } from "react";
import { Navigation } from "./Control";
export function Field({ id, label, value, onChange, disabled, scope = "modal", type = "text", min, max, step, placeholder, unit, invalid, describedBy, immediate = false }: { id: string; label: string; value: string; onChange: (value: string) => void; disabled: boolean; scope?: string; type?: "text" | "number"; min?: number; max?: number; step?: number; placeholder?: string; unit?: string; invalid?: boolean; describedBy?: string; immediate?: boolean }) {
  const manager = useContext(Navigation);
  const ref = useRef<HTMLInputElement>(null);
  useLayoutEffect(() => manager.register(id, { element: ref.current!, scope, group: "content" }), [manager, id, scope]);
  const original = useRef(value);
  const current = useRef({ value, onChange, immediate });
  current.current = { value, onChange, immediate };
  useLayoutEffect(() => {
    const input = ref.current!;
    const begin = () => { original.current = current.current.value; };
    const cancel = () => { if (!current.current.immediate) current.current.onChange(original.current); };
    input.addEventListener("focus", begin);
    input.addEventListener("launcher:text-begin", begin);
    input.addEventListener("launcher:text-cancel", cancel);
    return () => { input.removeEventListener("focus", begin); input.removeEventListener("launcher:text-begin", begin); input.removeEventListener("launcher:text-cancel", cancel); };
  }, []);
  const input = <input type={type} min={min} max={max} step={step} placeholder={placeholder} id={id} ref={ref} value={value} disabled={disabled} onChange={e => onChange(e.target.value)} autoComplete="off" aria-invalid={invalid || undefined} aria-describedby={[unit ? `${id}-unit` : "", describedBy].filter(Boolean).join(" ") || undefined} />;
  return <div className={`config-field${unit ? " config-field-with-unit" : ""}`}><label htmlFor={id}>{label}</label>{unit ? <div className="field-unit-row">{input}<span id={`${id}-unit`}>{unit}</span></div> : input}</div>;
}
