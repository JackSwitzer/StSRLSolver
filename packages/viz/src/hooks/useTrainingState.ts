import { useReducer, useEffect, useRef, useCallback } from 'react';
import type {
  TrainingState,
  AgentEpisodeMsg,
  MCTSResultMsg,
  AgentInfo,
} from '../types/training';

const MAX_EPISODES = 200;

type Action =
  | { type: 'connected' }
  | { type: 'disconnected' }
  | { type: 'grid_update'; agents: AgentInfo[] }
  | { type: 'training_stats'; stats: any }
  | { type: 'agent_episode'; episode: AgentEpisodeMsg }
  | { type: 'mcts_result'; result: MCTSResultMsg }
  | { type: 'agent_combat'; agent_id: number; combat: any }
  | { type: 'toggle_focus'; agentId: number }
  | { type: 'clear_focus' }
  | { type: 'select_agent'; index: number }
  | { type: 'next_focused' }
  | { type: 'prev_focused' };

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
    focusedAgentIds: [],
    activeFocusIndex: 0,
    selectedAgentIndex: 0,
    combatStates: {},
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
      return { ...state, training: { ...t, episodes } };
    }
    case 'mcts_result':
      return { ...state, training: { ...t, mctsResult: action.result } };
    case 'agent_combat': {
      const combatStates = { ...t.combatStates, [action.agent_id]: action.combat };
      return { ...state, training: { ...t, combatStates } };
    }
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
      return { ...state, training: { ...t, focusedAgentIds: [], activeFocusIndex: 0, combatStates: {} } };
    case 'select_agent':
      return { ...state, training: { ...t, selectedAgentIndex: action.index } };
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

      const ws = new WebSocket('ws://localhost:8080');
      wsRef.current = ws;

      ws.onopen = () => {
        dispatch({ type: 'connected' });
        ws.send(JSON.stringify({
          type: 'training_start',
          config: { num_agents: 4, mcts_sims: 64, ascension: 20, seed: 'Test123' },
        }));
      };

      ws.onclose = () => {
        dispatch({ type: 'disconnected' });
        wsRef.current = null;
        // Auto-reconnect after 2s unless unmounted
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
            case 'agent_combat':
              dispatch({ type: 'agent_combat', agent_id: msg.agent_id, combat: msg.combat });
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

  const toggleFocus = useCallback((agentId: number) => {
    dispatch({ type: 'toggle_focus', agentId });
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: 'training_focus', agent_id: agentId }));
    }
  }, []);

  const clearFocus = useCallback(() => {
    dispatch({ type: 'clear_focus' });
  }, []);

  const selectAgent = useCallback((index: number) => {
    dispatch({ type: 'select_agent', index });
  }, []);

  const nextFocused = useCallback(() => dispatch({ type: 'next_focused' }), []);
  const prevFocused = useCallback(() => dispatch({ type: 'prev_focused' }), []);

  const stopTraining = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: 'training_stop' }));
    }
  }, []);

  const resumeTraining = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'training_resume',
        config: { num_agents: 4, mcts_sims: 64, ascension: 20, seed: 'Test123' },
      }));
    }
  }, []);

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
  };
}
