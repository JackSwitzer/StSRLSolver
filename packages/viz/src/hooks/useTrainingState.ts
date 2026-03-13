import { useReducer, useEffect, useRef, useCallback } from 'react';
import type {
  TrainingState,
  AgentEpisodeMsg,
  MCTSResultMsg,
  PlannerResultMsg,
  AgentInfo,
  SystemStatsMsg,
} from '../types/training';

const MAX_EPISODES = 200;
const MAX_HISTORY = 200;

type Action =
  | { type: 'connected' }
  | { type: 'disconnected' }
  | { type: 'grid_update'; agents: AgentInfo[] }
  | { type: 'training_stats'; stats: any }
  | { type: 'agent_episode'; episode: AgentEpisodeMsg }
  | { type: 'mcts_result'; result: MCTSResultMsg }
  | { type: 'planner_result'; result: PlannerResultMsg }
  | { type: 'agent_combat'; agent_id: number; combat: any }
  | { type: 'agent_map'; agent_id: number; map: any }
  | { type: 'agent_run_state'; agent_id: number; deck: any[]; relics: any[]; potions: any[]; gold: number }
  | { type: 'system_stats'; stats: SystemStatsMsg }
  | { type: 'metrics_history'; floor_history: number[]; loss_history: number[]; win_history: number[] }
  | { type: 'toggle_focus'; agentId: number }
  | { type: 'clear_focus' }
  | { type: 'select_agent'; index: number }
  | { type: 'next_focused' }
  | { type: 'prev_focused' }
  | { type: 'set_paused'; paused: boolean }
  | { type: 'command_ack'; action: string; paused?: boolean };

interface FullState {
  training: TrainingState;
  connected: boolean;
}

const initialState: FullState = {
  training: {
    agents: [],
    stats: null,
    episodes: [],
    mctsResult: null,
    plannerResult: null,
    focusedAgentIds: [],
    activeFocusIndex: 0,
    selectedAgentIndex: 0,
    combatStates: {},
    mapStates: {},
    runStates: {},
    systemStats: null,
    floorHistory: [],
    lossHistory: [],
    winHistory: [],
    paused: false,
    deathStats: { byFloor: {}, byEnemy: {}, floorEnemyPairs: [], totalDeaths: 0 },
  },
  connected: false,
};

function reducer(state: FullState, action: Action): FullState {
  const t = state.training;
  switch (action.type) {
    case 'connected':
      return { ...state, connected: true };
    case 'disconnected':
      return { ...state, connected: false };
    case 'grid_update':
      return { ...state, training: { ...t, agents: action.agents } };
    case 'training_stats':
      return { ...state, training: { ...t, stats: action.stats } };
    case 'agent_episode': {
      const episodes = [action.episode, ...t.episodes].slice(0, MAX_EPISODES);
      // Accumulate local history from episodes
      const floorHistory = [...t.floorHistory, action.episode.floors_reached].slice(-MAX_HISTORY);
      const winHistory = [...t.winHistory, action.episode.won ? 1 : 0].slice(-MAX_HISTORY);
      // Track death stats (skip floor 0 — those are construction failures, not real deaths)
      let deathStats = t.deathStats;
      const rawDeathFloor = action.episode.death_floor ?? action.episode.floors_reached;
      if (!action.episode.won && rawDeathFloor > 0) {
        const df = rawDeathFloor;
        const de = action.episode.death_enemy ?? 'Unknown';
        const byFloor = { ...deathStats.byFloor };
        byFloor[df] = (byFloor[df] ?? 0) + 1;
        const byEnemy = { ...deathStats.byEnemy };
        byEnemy[de] = (byEnemy[de] ?? 0) + 1;
        // Rebuild top pairs from byFloor+byEnemy (lightweight)
        const pairs = Object.entries(byFloor).map(([f, c]) => ({ floor: Number(f), enemy: de, count: c }));
        deathStats = { byFloor, byEnemy, floorEnemyPairs: pairs, totalDeaths: deathStats.totalDeaths + 1 };
      }
      return { ...state, training: { ...t, episodes, floorHistory, winHistory, deathStats } };
    }
    case 'mcts_result':
      return { ...state, training: { ...t, mctsResult: action.result } };
    case 'planner_result':
      return { ...state, training: { ...t, plannerResult: action.result } };
    case 'agent_combat': {
      const combatStates = { ...t.combatStates, [action.agent_id]: action.combat };
      return { ...state, training: { ...t, combatStates } };
    }
    case 'agent_map': {
      const mapStates = { ...t.mapStates, [action.agent_id]: action.map };
      return { ...state, training: { ...t, mapStates } };
    }
    case 'agent_run_state': {
      const runStates = { ...t.runStates, [action.agent_id]: {
        deck: action.deck, relics: action.relics, potions: action.potions, gold: action.gold,
      }};
      return { ...state, training: { ...t, runStates } };
    }
    case 'system_stats':
      return { ...state, training: { ...t, systemStats: action.stats } };
    case 'metrics_history':
      return {
        ...state,
        training: {
          ...t,
          floorHistory: action.floor_history.slice(-MAX_HISTORY),
          lossHistory: action.loss_history.slice(-MAX_HISTORY),
          winHistory: action.win_history.slice(-MAX_HISTORY),
        },
      };
    case 'toggle_focus': {
      const ids = [...t.focusedAgentIds];
      const idx = ids.indexOf(action.agentId);
      if (idx >= 0) {
        ids.splice(idx, 1);
      } else {
        ids.push(action.agentId);
      }
      const activeIdx = Math.min(t.activeFocusIndex, Math.max(0, ids.length - 1));
      return { ...state, training: { ...t, focusedAgentIds: ids, activeFocusIndex: activeIdx } };
    }
    case 'clear_focus':
      return { ...state, training: { ...t, focusedAgentIds: [], activeFocusIndex: 0, combatStates: {}, mapStates: {}, runStates: {} } };
    case 'select_agent': {
      // Clear stale combat/map/run data from previously selected agent
      const prevId = t.agents[t.selectedAgentIndex]?.id;
      const nextId = t.agents[action.index]?.id;
      if (prevId !== undefined && prevId !== nextId) {
        const combatStates = { ...t.combatStates };
        const mapStates = { ...t.mapStates };
        const runStates = { ...t.runStates };
        delete combatStates[prevId];
        delete mapStates[prevId];
        delete runStates[prevId];
        return { ...state, training: { ...t, selectedAgentIndex: action.index, combatStates, mapStates, runStates } };
      }
      return { ...state, training: { ...t, selectedAgentIndex: action.index } };
    }
    case 'next_focused': {
      if (t.focusedAgentIds.length === 0) return state;
      const next = (t.activeFocusIndex + 1) % t.focusedAgentIds.length;
      return { ...state, training: { ...t, activeFocusIndex: next } };
    }
    case 'prev_focused': {
      if (t.focusedAgentIds.length === 0) return state;
      const prev = (t.activeFocusIndex - 1 + t.focusedAgentIds.length) % t.focusedAgentIds.length;
      return { ...state, training: { ...t, activeFocusIndex: prev } };
    }
    case 'set_paused':
      return { ...state, training: { ...t, paused: action.paused } };
    case 'command_ack': {
      // Sync paused state from server acknowledgement
      if (action.paused !== undefined) {
        return { ...state, training: { ...t, paused: action.paused } };
      }
      return state;
    }
    default:
      return state;
  }
}

export function useTrainingState() {
  const [fullState, dispatch] = useReducer(reducer, initialState);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    let unmounted = false;

    function connect() {
      if (unmounted) return;

      const wsUrl = import.meta.env.VITE_WS_URL || 'ws://localhost:8080';
      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        dispatch({ type: 'connected' });
        ws.send(JSON.stringify({
          type: 'training_start',
          config: { num_agents: 8, mcts_sims: 32, ascension: 0, seed: 'Test123' },
        }));
      };

      ws.onclose = () => {
        dispatch({ type: 'disconnected' });
        wsRef.current = null;
        if (!unmounted) {
          reconnectTimerRef.current = setTimeout(connect, 2000);
        }
      };

      ws.onerror = () => {
        dispatch({ type: 'disconnected' });
      };

      ws.onmessage = (e) => {
        try {
          const msg = JSON.parse(e.data);
          switch (msg.type) {
            case 'grid_update':
              dispatch({ type: 'grid_update', agents: msg.agents });
              break;
            case 'training_stats':
              dispatch({ type: 'training_stats', stats: msg });
              break;
            case 'agent_episode':
              dispatch({ type: 'agent_episode', episode: msg });
              break;
            case 'mcts_result':
              dispatch({ type: 'mcts_result', result: msg });
              break;
            case 'planner_result':
              dispatch({ type: 'planner_result', result: msg });
              break;
            case 'agent_combat':
              dispatch({ type: 'agent_combat', agent_id: msg.agent_id, combat: msg.combat });
              break;
            case 'agent_map':
              dispatch({ type: 'agent_map', agent_id: msg.agent_id, map: msg.map });
              break;
            case 'agent_run_state':
              dispatch({ type: 'agent_run_state', agent_id: msg.agent_id, deck: msg.deck, relics: msg.relics, potions: msg.potions, gold: msg.gold });
              break;
            case 'system_stats':
              dispatch({ type: 'system_stats', stats: msg });
              break;
            case 'metrics_history':
              dispatch({ type: 'metrics_history', floor_history: msg.floor_history, loss_history: msg.loss_history, win_history: msg.win_history });
              break;
            case 'command_ack':
              dispatch({ type: 'command_ack', action: msg.action, paused: msg.paused });
              break;
          }
        } catch { /* ignore parse errors */ }
      };
    }

    connect();

    return () => {
      unmounted = true;
      if (reconnectTimerRef.current) {
        clearTimeout(reconnectTimerRef.current);
        reconnectTimerRef.current = null;
      }
      wsRef.current?.close();
      wsRef.current = null;
    };
  }, []);

  const sendMsg = useCallback((msg: object) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(msg));
    }
  }, []);

  const toggleFocus = useCallback((agentId: number) => {
    dispatch({ type: 'toggle_focus', agentId });
    sendMsg({ type: 'training_focus', agent_id: agentId });
  }, [sendMsg]);

  const clearFocus = useCallback(() => {
    dispatch({ type: 'clear_focus' });
  }, []);

  const selectAgent = useCallback((index: number) => {
    dispatch({ type: 'select_agent', index });
  }, []);

  const nextFocused = useCallback(() => dispatch({ type: 'next_focused' }), []);
  const prevFocused = useCallback(() => dispatch({ type: 'prev_focused' }), []);

  const stopTraining = useCallback(() => {
    sendMsg({ type: 'command', action: 'pause' });
    dispatch({ type: 'set_paused', paused: true });
  }, [sendMsg]);

  const resumeTraining = useCallback(() => {
    sendMsg({ type: 'command', action: 'resume' });
    dispatch({ type: 'set_paused', paused: false });
  }, [sendMsg]);

  const sendCommand = useCallback((action: string, params?: Record<string, unknown>) => {
    sendMsg({ type: 'command', action, params });
  }, [sendMsg]);

  const sendControl = useCallback((params: Record<string, unknown>) => {
    sendMsg({ type: 'command', action: 'set_config', params });
  }, [sendMsg]);

  return {
    state: fullState.training,
    connected: fullState.connected,
    toggleFocus,
    clearFocus,
    selectAgent,
    nextFocused,
    prevFocused,
    stopTraining,
    resumeTraining,
    sendCommand,
    sendControl,
    sendMsg,
  };
}
