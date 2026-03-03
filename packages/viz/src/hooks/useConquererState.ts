import { useState, useEffect, useRef, useCallback } from 'react';
import type { ConquererState, PathResult } from '../types/conquerer';

/**
 * WebSocket hook for conquerer progress updates.
 *
 * Handles two message types from the server:
 * - conquerer_path_result: single path completed, incrementally builds state
 * - conquerer_complete: full snapshot replacing state
 *
 * Also supports sending conquerer_run commands.
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
        const msg = JSON.parse(e.data);

        if (msg.type === 'conquerer_path_result') {
          // Incremental: add this path to existing state
          setState((prev) => {
            const path: PathResult = msg.path;
            const paths = prev ? [...prev.paths, path] : [path];
            const winCount = paths.filter((p) => p.won).length;
            const maxFloor = Math.max(...paths.map((p) => p.floors_reached), 0);
            const bestPath = paths.reduce((best, p) => {
              if (p.won && !best.won) return p;
              if (p.won === best.won && p.floors_reached > best.floors_reached) return p;
              if (p.won === best.won && p.floors_reached === best.floors_reached && p.hp_remaining > best.hp_remaining) return p;
              return best;
            }, paths[0]);

            return {
              seed: prev?.seed || path.seed,
              paths,
              best_path_id: bestPath.path_id,
              win_count: winCount,
              max_floor: maxFloor,
              active_paths: msg.active_paths ?? 0,
              elapsed_seconds: prev?.elapsed_seconds || 0,
            };
          });
        } else if (msg.type === 'conquerer_complete') {
          // Full snapshot
          setState({
            seed: msg.seed,
            paths: msg.paths,
            best_path_id: msg.best_path_id,
            win_count: msg.win_count,
            max_floor: msg.max_floor,
            active_paths: 0,
            elapsed_seconds: msg.elapsed_seconds,
            divergence_tree: msg.divergence_tree,
          });
        } else {
          // Legacy: full snapshot message
          const data: ConquererState = msg;
          setState(data);
        }
      } catch {
        console.warn('Failed to parse conquerer WebSocket message');
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

  const runConquerer = useCallback(
    (seed: string, numPaths = 10, ascension = 20) => {
      // Reset state for new run
      setState(null);
      send({ type: 'conquerer_run', seed, num_paths: numPaths, ascension });
    },
    [send],
  );

  return { state, connected, send, runConquerer };
}
