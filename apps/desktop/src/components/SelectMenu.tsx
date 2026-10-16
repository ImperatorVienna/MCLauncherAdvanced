import { t } from "../i18n";
import { useContext, useLayoutEffect, useRef, useState } from "react";
import { InstanceIcon } from "./InstanceIcon";
import type { IconId } from "../state/instanceIcons";
import { Control, Navigation } from "./Control";
export function SelectMenu({ options, value, onChange, iconForOption, disabled = false, id = "instance-select", scope = "main", statusForOption, showIcons = true, label = t("Select instance"), labelForValue, compact = false }: {
    label?: string;
    labelForValue?: (value: string) => string;
    compact?: boolean;
    id?: string;
    scope?: string;
    statusForOption?: (value: string) => string;
    showIcons?: boolean;
    disabled?: boolean;
    options: string[];
    value: string;
    onChange: (value: string) => void;
    iconForOption?: (value: string) => IconId;
}) {
    const manager = useContext(Navigation);
    const [expanded, setExpanded] = useState(false);
    const open = expanded && !disabled && options.length > 0;
    const root = useRef<HTMLDivElement>(null);
    const close = (restore = true) => {
        setExpanded(false);
        if (expanded) manager.rememberFocus(id);
        if (restore)
            manager.focus(id);
    };
    useLayoutEffect(() => {
        if (!open) {
            if (expanded) setExpanded(false);
            return;
        }
        // Parent source/status updates must not reset a user-navigated option.
        const focused = manager.activeId();
        if (!options.some((_, i) => focused === `${id}-option-${i}`))
            manager.focus(`${id}-option-${Math.max(0, options.indexOf(value))}`);
        const intercept: NonNullable<typeof manager.interceptAction> = action => {
            const index = options.findIndex((_, i) => manager.activeId() === `${id}-option-${i}`);
            if (action === "Back") {
                close();
                return true;
            }
            if (action === "NavigateDown" || action === "NavigateUp") {
                const next = (index + (action === "NavigateDown" ? 1 : -1) + options.length) % options.length;
                manager.focus(`${id}-option-${next}`);
                return true;
            }
            if (action === "NavigateLeft" || action === "NavigateRight" || action === "NextTab" || action === "PreviousTab" || action === "OpenMenu") {
                return true;
            }
            return action === "ScrollUp" || action === "ScrollDown";
        };
        manager.interceptAction = intercept;
        const outside = (event: PointerEvent) => {
            if (!root.current?.contains(event.target as Node)) {
                // Dismiss without swallowing the pointer: the clicked control still works.
                close(false);
            }
        };
        document.addEventListener("pointerdown", outside, true);
        return () => {
            if (manager.interceptAction === intercept) manager.interceptAction = undefined;
            document.removeEventListener("pointerdown", outside, true);
        };
    }, [open, expanded, manager, value, options, id, scope]);
    return (<div className="select-menu" ref={root} onBlur={event => {
            if (!event.currentTarget.contains(event.relatedTarget as Node | null))
                close(false);
        }}>
      <Control type="button" disabled={disabled || !options.length} id={id} scope={scope} className="select-trigger" aria-haspopup="listbox" aria-label={label} aria-expanded={open} aria-controls={open ? `${id}-list` : undefined} onClick={() => setExpanded(!expanded)}>
        {showIcons && <InstanceIcon icon={iconForOption?.(value)}/>}
        <span>{labelForValue ? labelForValue(value) : value.slice(0, value.lastIndexOf(" · ") < 0 ? undefined : value.lastIndexOf(" · "))}{!compact && <small>{value.includes(" · ") ? value.slice(value.lastIndexOf(" · ") + 3) : ""} · {statusForOption?.(value) ?? t("Not installed")}</small>}</span>
        <span aria-hidden="true">{open ? "⌃" : "⌄"}</span>
      </Control>
      {open && <div id={`${id}-list`} className="select-list" role="listbox" aria-label={label}>
        {options.map((option, i) => <Control type="button" key={option} id={`${id}-option-${i}`} scope={scope} className="select-option" role="option" aria-selected={value === option} tabIndex={-1} onClick={() => { onChange(option); close(); }}>
          {showIcons && <InstanceIcon icon={iconForOption?.(option)}/>}
          <span>{labelForValue ? labelForValue(option) : option.slice(0, option.lastIndexOf(" · ") < 0 ? undefined : option.lastIndexOf(" · "))}{!compact && <small>{option.includes(" · ") ? option.slice(option.lastIndexOf(" · ") + 3) : ""} · {statusForOption?.(option) ?? t("Not installed")}</small>}</span>
        </Control>)}
      </div>}
    </div>);
}
