import { useState, useEffect, useRef, useCallback } from 'react';
import type { GameObservation } from '../types/game';

export function useGameState(wsUrl?: string) {
  const [state, setState] = useState<GameObservation | null>(null);
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
        const data: GameObservation = JSON.parse(e.data);
        setState(data);
      } catch {
        console.warn('Failed to parse WebSocket message');
      }
    };

    return () => {
      ws.close();
      wsRef.current = null;
    };
  }, [wsUrl]);

  const send = useCallback(
    (data: unknown) => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify(data));
      }
    },
    [],
  );

  return { state, connected, send };
}
