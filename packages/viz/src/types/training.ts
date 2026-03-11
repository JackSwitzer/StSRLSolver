// TypeScript types for training viewer WS protocol

export type ScreenMode = 'grid' | 'combat' | 'map' | 'mcts' | 'stats';

export const SCREEN_MODES: ScreenMode[] = ['grid', 'combat', 'map', 'mcts', 'stats'];

export const AGENT_NAMES = [
  'Oracle', 'Gambler', 'Wanderer', 'Wildcard',
  'Sentinel', 'Guardian', 'Tactician', 'Drifter',
  'Spectre', 'Pilgrim', 'Vanguard', 'Mystic',
  'Reaper', 'Nomad', 'Arbiter', 'Seeker',
] as const;

export type AgentName = (typeof AGENT_NAMES)[number];

// --- Server -> Client messages ---

export type AgentStatus = 'idle' | 'playing' | 'starting' | 'restarting' | 'dead' | 'won';

export interface AgentInfo {
  id: number;
  name: string;
  phase: string;
  floor: number;
  hp: number;
  max_hp: number;
  episode: number;
  wins: number;
  seed: string;
  status: AgentStatus;
}

export interface GridUpdateMsg {
  type: 'grid_update';
  agents: AgentInfo[];
}

export interface TrainingStatsMsg {
  type: 'training_stats';
  total_episodes: number;
  win_count: number;
  win_rate: number;
  avg_floor: number;
  mcts_avg_ms: number;
  eps_per_min: number;
  uptime: number;
}

export interface AgentEpisodeMsg {
  type: 'agent_episode';
  agent_id: number;
  seed: string;
  won: boolean;
  floors_reached: number;
  hp_remaining: number;
  total_steps: number;
  duration: number;
  episode: number;
  mcts_calls: number;
  mcts_avg_ms: number;
  trivial: boolean;
}

export interface MCTSAction {
  id: string;
  visits: number;
  pct: number;
  q: number;
  selected: boolean;
}

export interface MCTSResultMsg {
  type: 'mcts_result';
  agent_id: number;
  sims: number;
  elapsed_ms: number;
  root_value: number;
  actions: MCTSAction[];
}

export interface AgentCombatMsg {
  type: 'agent_combat';
  agent_id: number;
  combat: any; // CombatState from game types
}

export interface SystemStatsMsg {
  type: 'system_stats';
  cpu_pct: number;
  ram_pct: number;
  ram_used_gb: number;
  ram_total_gb: number;
  workers: number;
}

export interface MetricsHistoryMsg {
  type: 'metrics_history';
  floor_history: number[];    // avg floor per episode (last N)
  loss_history: number[];     // training loss (last N)
  win_history: number[];      // 0/1 per episode (last N)
}

export type ServerMessage = GridUpdateMsg | TrainingStatsMsg | AgentEpisodeMsg | MCTSResultMsg | AgentCombatMsg | SystemStatsMsg | MetricsHistoryMsg;

// --- Client -> Server messages ---

export interface TrainingStartMsg {
  type: 'training_start';
  config: {
    num_agents: number;
    mcts_sims: number;
    ascension: number;
    seed: string;
  };
}

export interface TrainingFocusMsg {
  type: 'training_focus';
  agent_id: number;
}

export type ClientMessage = TrainingStartMsg | TrainingFocusMsg;

// --- Client state ---

export interface TrainingState {
  agents: AgentInfo[];
  stats: TrainingStatsMsg | null;
  episodes: AgentEpisodeMsg[];
  mctsResult: MCTSResultMsg | null;
  focusedAgentIds: number[];       // multi-select: list of focused agent IDs
  activeFocusIndex: number;        // which focused agent is currently viewed
  selectedAgentIndex: number;      // grid cursor position
  combatStates: Record<number, any>; // agent_id -> latest combat state
  systemStats: SystemStatsMsg | null;
  floorHistory: number[];
  lossHistory: number[];
  winHistory: number[];
  paused: boolean;
}
