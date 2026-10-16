import type { ComponentProps } from "react";
import { Control } from "./Control";
import "../styles/buttons.css";

/** Default for future ordinary buttons; specialized sidebar/Play/select styles remain separate. */
export function StandardButton({ tone = "white", className = "", ...props }:
  ComponentProps<typeof Control> & { tone?: "white" | "green" | "red" | "yellow" }) {
  return <Control {...props} data-tone={tone} className={`standard-button ${className}`} />;
}
