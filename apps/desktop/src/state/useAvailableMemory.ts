import { useEffect, useState } from "react";
import { invokeNative } from "../platform/desktop";

/** Current Linux MemAvailable, in the same binary GB units used by the memory input. */
export function useAvailableMemory() {
  const [availableGB, setAvailableGB] = useState<number>();
  useEffect(() => {
    let alive = true;
    const refresh = async () => {
      try {
        const bytes = await invokeNative<number>("available_memory_bytes");
        if (alive) setAvailableGB(Math.floor(bytes / (1024 ** 3) * 10) / 10);
      } catch { if (alive) setAvailableGB(undefined); }
    };
    void refresh();
    const timer = window.setInterval(refresh, 5000);
    window.addEventListener("focus", refresh);
    return () => { alive = false; clearInterval(timer); window.removeEventListener("focus", refresh); };
  }, []);
  return availableGB;
}
