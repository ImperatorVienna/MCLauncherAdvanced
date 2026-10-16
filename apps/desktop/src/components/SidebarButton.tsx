import type { ReactNode } from "react";
import { Control } from "./Control";

type SidebarButtonProps = {
  id: string;
  icon: ReactNode;
  children: ReactNode;
  selected?: boolean;
  account?: boolean;
  onClick: () => void;
};

/** Shared sidebar appearance, selection semantics, and focus registration. */
export function SidebarButton({
  id, icon, children, selected = false, account = false, onClick,
}: SidebarButtonProps) {
  return (
    <Control
      id={id}
      group="sidebar"
      className={`side-item${account ? " account" : ""}${selected ? " selected" : ""}`}
      aria-current={selected ? "page" : undefined}
      onClick={onClick}
    >
      <span className="sidebar-icon" aria-hidden="true">{icon}</span>
      <span className="sidebar-label">{children}</span>
    </Control>
  );
}
