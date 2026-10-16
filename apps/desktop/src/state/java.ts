import { localizeMessage } from "../i18n";
import { useCallback, useEffect, useRef, useState } from "react";
import { invokeNative } from "../platform/desktop";
import type { Java } from "./installer";
export function useJava(preview: boolean, running: boolean) {
  const [runtimes, setRuntimes] = useState<Java[]>([]), [loading, setLoading] = useState(false), [error, setError] = useState("");
  const request = useRef(0);
  const refresh = useCallback(async () => {
    if (preview) return;
    const id = ++request.current;
    setLoading(true); setError("");
    try { const values = await invokeNative<Java[]>("list_java"); if (id === request.current) setRuntimes(values); } catch (e) { if (id === request.current) setError(localizeMessage(e)); }
    finally { if (id === request.current) setLoading(false); }
  }, [preview]);
  useEffect(() => { if (!running) void refresh(); return () => { request.current++; }; }, [refresh, running]);
  return { runtimes, loading, error, refresh };
}
