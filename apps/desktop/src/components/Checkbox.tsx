import type { ComponentProps } from "react";
import { Control } from "./Control";

/** Shared checkbox geometry and navigation; pages supply only state and action. */
export function Checkbox({ checked, children, ...props }: Omit<ComponentProps<typeof Control>, "role" | "aria-checked" | "className"> & { checked: boolean }) {
  return <Control {...props} type="button" role="checkbox" aria-checked={checked} className="launch-checkbox">
    <span className="launch-checkbox-box" aria-hidden="true"/>{children}
  </Control>;
}
