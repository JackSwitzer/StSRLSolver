import { useState, useEffect, useCallback, useRef } from 'react';
import type { RunSummary } from '../components/RunHistory';
import type { AgentEpisodeMsg, CombatSummary } from '../types/training';

const LOGS_DIR = '/Users/jackswitzer/Desktop/SlayTheSpireRL/logs';
const API_BASE = '/api/data';

// --- Raw types ---

interface RunSummaryRaw {
  start: string;
  end: string;
  elapsed_hours: number;
  total_games: number;
  total_wins: number;
  final_win_rate: number;
  final_avg_floor: number;
  sweep_results?: Array<{
    config: Record<string, any>;
    games: number;
    avg_floor: number;
    win_rate: number;
    duration_min: number;
    train_steps: number;
  }>;
}

interface StatusRaw {
  total_games: number;
  total_wins: number;
  avg_floor_100: number;
  elapsed_hours: number;
  win_rate_100: number;
  sweep_config?: Record<string, any>;
  train_steps?: number;
  [key: string]: any;
}

// --- Data mapping ---

function mapCombat(c: any): CombatSummary {
  return {
    floor: c.floor ?? 0,
    enemy: c.enemy ?? c.room_type ?? 'Unknown',
    turns: c.turns ?? 0,
    hp_lost: c.hp_lost ?? 0,
    damage_dealt: c.damage_dealt ?? 0,
    used_potion: (c.potions_used ?? 0) > 0 || c.used_potion === true,
    stances: c.stance_changes != null
      ? (typeof c.stance_changes === 'number' ? { changes: c.stance_changes } : c.stance_changes)
      : c.stances,
    // Pass through turn-by-turn card data for FloorTimeline detail
    turns_detail: c.turns_detail,
  } as CombatSummary;
}

function mapEpisode(e: any, idx: number): AgentEpisodeMsg {
  return {
    type: 'agent_episode',
    agent_id: e.agent_id ?? 0,
    seed: e.seed ?? `unknown_${idx}`,
    won: e.won ?? false,
    floors_reached: e.floors_reached ?? e.floor ?? 0,
    hp_remaining: e.hp_remaining ?? e.hp ?? 0,
    total_steps: e.total_steps ?? e.num_transitions ?? (typeof e.decisions === 'number' ? e.decisions : 0),
    duration: e.duration ?? e.duration_s ?? 0,
    episode: e.episode ?? idx,
    mcts_calls: e.mcts_calls ?? 0,
    mcts_avg_ms: e.mcts_avg_ms ?? 0,
    trivial: e.trivial ?? false,
    combats: e.combats?.map(mapCombat),
    decisions: e.decisions_detail,
    hp_history: e.hp_history,
    deck_changes: e.deck_changes ?? e.deck_final,
    death_floor: e.death_floor ?? e.floor,
    death_enemy: (typeof e.death_enemy === 'string' && e.death_enemy.length > 0) ? e.death_enemy : undefined,
    deck_size: e.deck_size ?? (e.deck_final?.length ?? e.deck_changes?.length),
    relic_count: e.relic_count,
    floor_log: e.floor_log,
    neow_choice: e.neow_choice,
    events: e.events,
    relics_final: e.relics_final,
  };
}

// --- File I/O abstraction (fetch via Vite dev server proxy) ---

async function readTextFile(path: string): Promise<string | null> {
  const relPath = path.replace(LOGS_DIR, '');
  try {
    const res = await fetch(`${API_BASE}${relPath}`);
    if (!res.ok) return null;
    return await res.text();
  } catch {
    return null;
  }
}

async function readDir(path: string): Promise<string[]> {
  const relPath = path.replace(LOGS_DIR, '');
  try {
    const res = await fetch(`${API_BASE}${relPath}`);
    if (!res.ok) return [];
    return await res.json();
  } catch {
    return [];
  }
}

async function readJSON<T>(path: string): Promise<T | null> {
  const text = await readTextFile(path);
  if (!text) return null;
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}

async function readJSONL<T>(path: string, maxLines = 10000): Promise<T[]> {
  const text = await readTextFile(path);
  if (!text) return [];
  try {
    return text.trim().split('\n').slice(0, maxLines).map((line) => JSON.parse(line));
  } catch {
    return [];
  }
}

// --- Hook ---

export function useRunData(wsConnected?: boolean) {
  const [runs, setRuns] = useState<RunSummary[]>([]);
  const [currentEpisodes, setCurrentEpisodes] = useState<AgentEpisodeMsg[]>([]);
  const [floorCurveData, setFloorCurveData] = useState<number[]>([]);
  const [loading, setLoading] = useState(true);
  const [runConfigs, setRunConfigs] = useState<Map<string, Record<string, any>>>(new Map());
  const loadedOnce = useRef(false);
  const pollTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const disconnectedSinceRef = useRef<number | null>(null);

  const loadData = useCallback(async () => {
    // Don't reload if we already have data (prevents data loss on re-render)
    if (loadedOnce.current) return;
    setLoading(true);
    console.log('[useRunData] Loading data');

    // 1. Load run directory listing
    const runDirs = await readDir(`${LOGS_DIR}/runs`);
    const runList: RunSummary[] = [];
    const configs = new Map<string, Record<string, any>>();

    // Load summaries for named runs (run_* prefix)
    const namedRuns = runDirs.filter((d) => d.startsWith('run_'));
    const summaryPromises = namedRuns.map(async (dir) => {
      const summary = await readJSON<RunSummaryRaw>(`${LOGS_DIR}/runs/${dir}/summary.json`);
      if (summary) {
        const label = dir
          .replace(/^run_\d+_/, '')
          .replace(/^run_/, '')
          .replace(/_/g, ' ') || dir;
        const config = summary.sweep_results?.[0]?.config ?? {};
        configs.set(dir, config);
        runList.push({
          id: dir,
          label,
          totalGames: summary.total_games,
          avgFloor: summary.final_avg_floor,
          maxFloor: 16,
          winRate: summary.final_win_rate,
          durationHours: summary.elapsed_hours,
          config,
        });
      }
    });
    await Promise.all(summaryPromises);

    // Add current weekend-run
    const currentStatus = await readJSON<StatusRaw>(`${LOGS_DIR}/weekend-run/status.json`);
    if (currentStatus) {
      const config = currentStatus.sweep_config ?? {};
      configs.set('weekend-run', config);
      runList.push({
        id: 'weekend-run',
        label: 'v5 Weekend (500K)',
        totalGames: currentStatus.total_games,
        avgFloor: currentStatus.avg_floor_100,
        maxFloor: 16,
        winRate: currentStatus.win_rate_100,
        durationHours: currentStatus.elapsed_hours,
        config,
      });
    }

    setRuns(runList);
    setRunConfigs(configs);

    // 2. Load top episodes (pre-processed, ~200 best runs with full combat data)
    const topRaw = await readJSON<any>(`${LOGS_DIR}/weekend-run/top_episodes.json`);
    const topEpisodes = Array.isArray(topRaw) ? topRaw : topRaw?.episodes;
    console.log('[useRunData] top_episodes:', topEpisodes ? topEpisodes.length : 'null');
    if (topEpisodes && topEpisodes.length > 0) {
      setCurrentEpisodes(topEpisodes.map((e: any, i: number) => mapEpisode(e, i)));
    } else {
      // Fallback to recent_episodes.json
      const recentRaw = await readJSON<any[]>(`${LOGS_DIR}/weekend-run/recent_episodes.json`);
      if (recentRaw && recentRaw.length > 0) {
        setCurrentEpisodes(recentRaw.map((e, i) => mapEpisode(e, i)));
      }
    }

    // 3. Build floor curve from floor_curve.json (pre-processed) or episodes.jsonl
    const curveRaw = await readJSON<number[]>(`${LOGS_DIR}/weekend-run/floor_curve.json`);
    if (curveRaw && curveRaw.length > 0) {
      setFloorCurveData(curveRaw);
    } else {
      // Fallback: load from JSONL (expensive — only first 50K lines)
      const episodes = await readJSONL<{ floor?: number; won?: boolean }>(
        `${LOGS_DIR}/weekend-run/episodes.jsonl`,
        50000,
      );
      if (episodes.length > 0) {
        const windowSize = 100;
        const curve: number[] = [];
        let sum = 0;
        for (let i = 0; i < episodes.length; i++) {
          sum += episodes[i].floor ?? 0;
          if (i >= windowSize) sum -= episodes[i - windowSize].floor ?? 0;
          if (i >= windowSize - 1) curve.push(sum / windowSize);
        }
        if (curve.length > 500) {
          const step = Math.ceil(curve.length / 500);
          setFloorCurveData(curve.filter((_, i) => i % step === 0));
        } else {
          setFloorCurveData(curve);
        }
      }
    }

    loadedOnce.current = true;
    setLoading(false);
    console.log('[useRunData] Done loading');
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const forceReload = useCallback(async () => {
    loadedOnce.current = false;
    await loadData();
  }, [loadData]);

  const getRunConfig = useCallback(
    (runId: string) => runConfigs.get(runId) ?? {},
    [runConfigs],
  );

  // Polling fallback: when WS is disconnected for >10s, poll JSON files every 5s
  useEffect(() => {
    if (wsConnected === undefined) return; // Not tracking WS state

    if (!wsConnected) {
      if (disconnectedSinceRef.current === null) {
        disconnectedSinceRef.current = Date.now();
      }
      // Start polling after 10s of disconnection
      const elapsed = Date.now() - disconnectedSinceRef.current;
      if (elapsed >= 10_000 && !pollTimerRef.current) {
        console.log('[useRunData] WS disconnected >10s, starting poll fallback');
        pollTimerRef.current = setInterval(async () => {
          // Poll status.json for latest metrics
          const status = await readJSON<StatusRaw>(`${LOGS_DIR}/weekend-run/status.json`);
          if (status) {
            // Update the weekend-run entry in runs list
            setRuns((prev) => prev.map((r) =>
              r.id === 'weekend-run'
                ? { ...r, totalGames: status.total_games, avgFloor: status.avg_floor_100, winRate: status.win_rate_100, durationHours: status.elapsed_hours }
                : r
            ));
          }
        }, 5_000);
      }
    } else {
      // WS reconnected — stop polling
      disconnectedSinceRef.current = null;
      if (pollTimerRef.current) {
        clearInterval(pollTimerRef.current);
        pollTimerRef.current = null;
      }
    }

    return () => {
      if (pollTimerRef.current) {
        clearInterval(pollTimerRef.current);
        pollTimerRef.current = null;
      }
    };
  }, [wsConnected]);

  return { runs, currentEpisodes, floorCurveData, loading, loadData: forceReload, getRunConfig };
}
