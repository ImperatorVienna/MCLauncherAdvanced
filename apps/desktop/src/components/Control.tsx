import {
  createContext,
  useContext,
  useLayoutEffect,
  useRef,
  useState,
  type ButtonHTMLAttributes,
} from "react";
import { FocusManager } from "../navigation/FocusManager";
export const Navigation = createContext<FocusManager>(null!);
export function Control({
  id,
  scope = "main",
  group = "content",
  disabled, onClick, onPointerDown, onFocus, onBlur, onKeyDown, tabIndex,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  id: string;
  scope?: string;
  group?: string;
}) {
  const manager = useContext(Navigation);
  const ref = useRef<HTMLButtonElement>(null);
  const [focused, setFocused] = useState(false);
  useLayoutEffect(
    () => manager.register(id, { element: ref.current!, scope, group }),
    [manager, id, scope, group],
  );
  // Keep an already-focused button focusable during disable; after blur it
  // becomes natively disabled and cannot be reached again until re-enabled.
  return <button {...props} id={id} ref={ref}
    disabled={disabled && !focused} aria-disabled={disabled || undefined}
    tabIndex={disabled ? -1 : tabIndex}
    onFocus={event => { setFocused(true); onFocus?.(event); }}
    onBlur={event => { setFocused(false); onBlur?.(event); }}
    onPointerDown={event => { if (disabled) { event.preventDefault(); return; } onPointerDown?.(event); }}
    onClick={event => { if (disabled) { event.preventDefault(); event.stopPropagation(); return; } onClick?.(event); }}
    onKeyDown={event => {
      if (disabled && (event.key === "Enter" || event.key === " ")) { event.preventDefault(); return; }
      onKeyDown?.(event);
    }}/>;
}
