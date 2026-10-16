// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { expect, test } from "vitest";
import { TransferProgress, formatBytes } from "./TransferProgress";
test("progress handles unknown, bounded byte progress and empty completion", () => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  const container = document.createElement("div"); const root = createRoot(container);
  act(() => root.render(<TransferProgress phase="\u6b63\u5728\u7edf\u8ba1" completed={0} total={0} />));
  expect(container.querySelector('[role="progressbar"]')?.hasAttribute("aria-valuenow")).toBe(false);
  act(() => root.render(<TransferProgress phase="\u6b63\u5728\u590d\u5236" completed={512} total={1024} />));
  expect(container.querySelector('[role="progressbar"]')?.getAttribute("aria-valuenow")).toBe("50");
  act(() => root.render(<TransferProgress phase="\u5b8c\u6210" completed={0} total={0} complete />));
  expect(container.querySelector('[role="progressbar"]')?.getAttribute("aria-valuenow")).toBe("100");
  expect(formatBytes(1024 ** 3)).toBe("1.0 GiB");
  act(() => root.unmount());
});
