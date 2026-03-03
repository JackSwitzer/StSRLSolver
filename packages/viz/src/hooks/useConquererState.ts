import { useState, useEffect, useRef } from 'react';
import type { ConquererState } from '../types/conquerer';

/**
 * WebSocket hook for receiving conquerer progress updates.
 *
 * The server is expected to send JSON messages conforming to the
 * ConquererState interface. Each message replaces the previous state
 * (the server sends full snapshots, not deltas).
 */
export function useConquererState(wsUrl?: string) {
  const [state, setState] = useState<ConquererState | null>(null);
  const [connected, setConnected] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    if (!wsUrl) return;

    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onopen = () => setConnected(true);
    ws.onclose = () => {
      setConnected(false);
      wsRef.current = null;
    };
    ws.onerror = () => {
      setConnected(false);
    };
    ws.onmessage = (e) => {
      try {
        const data: ConquererState = JSON.parse(e.data);
        setState(data);
      } catch {
        console.warn('Failed to parse conquerer WebSocket message');
      }
    };

    return () => {
      ws.close();
      wsRef.current = null;
    };
  }, [wsUrl]);

  return { state, connected };
}
