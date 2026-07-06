import { useState, useEffect, useRef, useCallback } from 'react';
import { fetchJson } from '../api/client';

export function usePolling<T>(url: string, intervalMs: number) {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [lastUpdate, setLastUpdate] = useState<number>(0);
  const mountedRef = useRef(true);

  const refresh = useCallback(async () => {
    const result = await fetchJson<T>(url);
    if (!mountedRef.current) return;
    if (result !== null) {
      setData(result);
      setLastUpdate(Date.now());
    }
    setLoading(false);
  }, [url]);

  useEffect(() => {
    mountedRef.current = true;
    refresh();
    const id = setInterval(refresh, intervalMs);
    return () => {
      mountedRef.current = false;
      clearInterval(id);
    };
  }, [refresh, intervalMs]);

  const stale = lastUpdate > 0 && Date.now() - lastUpdate > intervalMs * 4;
  return { data, loading, stale, lastUpdate, refresh };
}
