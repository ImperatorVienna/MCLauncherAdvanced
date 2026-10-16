export type Action =
  | "NavigateUp"
  | "NavigateDown"
  | "NavigateLeft"
  | "NavigateRight"
  | "Confirm"
  | "Back"
  | "NextTab"
  | "PreviousTab"
  | "ScrollUp"
  | "ScrollDown"
  | "OpenMenu";
export const keyboardAction = (event: KeyboardEvent): Action | undefined => {
  if (event.altKey || event.metaKey || event.ctrlKey) return;
  return (
    {
      ArrowUp: "NavigateUp",
      ArrowDown: "NavigateDown",
      ArrowLeft: "NavigateLeft",
      ArrowRight: "NavigateRight",
      Escape: "Back",
      PageDown: "ScrollDown",
      PageUp: "ScrollUp",
      F10: "OpenMenu",
    } as Record<string, Action>
  )[event.key];
};
