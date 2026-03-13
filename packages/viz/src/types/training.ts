// TypeScript types for training viewer WS protocol

export type ScreenMode = 'grid' | 'combat' | 'map' | 'mcts' | 'stats' | 'feed' | 'stats_view' | 'training_view';

export const SCREEN_MODES: ScreenMode[] = ['grid', 'combat', 'map', 'mcts', 'stats', 'feed', 'stats_view', 'training_view'];

export const AGENT_NAMES = [
  'Oracle', 'Gambler', 'Wanderer', 'Wildcard',
  'Sentinel', 'Guardian', 'Tactician', 'Drifter',
  'Spectre', 'Pilgrim', 'Vanguard', 'Mystic',
  'Reaper', 'Nomad', 'Arbiter', 'Seeker',
] as const;

export type AgentName = (typeof AGENT_NAMES)[number];

// --- Server -> Client messages ---

export type AgentStatus = 'idle' | 'playing' | 'starting' | 'restarting' | 'dead' | 'won';

export interface CombatMiniSummary {
  enemy_name: string;
  enemy_hp: number;
  enemy_max_hp: number;
  stance: string;
  hand_size: number;
  energy: number;
  max_energy: number;
  turn: number;
}

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
  act?: number;
  enemy_name?: string;
  enemy_hp?: number;
  enemy_max_hp?: number;
  hand_size?: number;
  turn?: number;
  stance?: string;
  combat_summary?: CombatMiniSummary;
}

export interface GridUpdateMsg {
  type: 'grid_update';
  agents: AgentInfo[];
}

export interface TrainingStatsMsg {
  type: 'training_stats';
  run_id?: string;
  total_episodes: number;
  win_count: number;
  win_rate: number;
  avg_floor: number;
  max_floor?: number;
  train_steps?: number;
  mcts_avg_ms: number;
  eps_per_min: number;
  uptime: number;
}

export interface CombatSummary {
  floor: number;
  enemy: string;
  turns: number;
  hp_lost: number;
  damage_dealt: number;
  used_potion: boolean;
  stances?: Record<string, number>;
}

export interface DecisionSummary {
  floor: number;
  type: string;       // "path" | "rest" | "card_pick" | "shop" | "event" | "neow"
  choice: string;
  alternatives?: string[];
  score?: number;       // e.g. path score or MCTS confidence
  detail?: string;      // extra context: "HP: 45/72", "+250g, +curse", etc.
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
  // Rich data (from enhanced telemetry)
  combats?: CombatSummary[];
  decisions?: DecisionSummary[];
  hp_history?: number[];
  deck_changes?: string[];
  death_floor?: number;
  death_enemy?: string;
  deck_size?: number;
  relic_count?: number;
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
  policy_version?: number;
}

export interface AgentCombatMsg {
  type: 'agent_combat';
  agent_id: number;
  combat: any; // CombatState from game types
}

export interface PlannerResultMsg {
  type: 'planner_result';
  agent_id: number;
  lines_considered: number;
  strategy: string;
  turns_to_kill: number;
  expected_hp_loss: number;
  confidence: number;
  cards_played: string[];
  elapsed_ms?: number;
  policy_version?: number;
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

export type ServerMessage = GridUpdateMsg | TrainingStatsMsg | AgentEpisodeMsg | MCTSResultMsg | PlannerResultMsg | AgentCombatMsg | SystemStatsMsg | MetricsHistoryMsg;

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

export interface DeathStats {
  byFloor: Record<number, number>;     // floor -> death count
  byEnemy: Record<string, number>;     // enemy name -> death count
  floorEnemyPairs: Array<{ floor: number; enemy: string; count: number }>;
  totalDeaths: number;
}

export interface TrainingState {
  agents: AgentInfo[];
  stats: TrainingStatsMsg | null;
  episodes: AgentEpisodeMsg[];
  mctsResult: MCTSResultMsg | null;
  plannerResult: PlannerResultMsg | null;
  focusedAgentIds: number[];       // multi-select: list of focused agent IDs
  activeFocusIndex: number;        // which focused agent is currently viewed
  selectedAgentIndex: number;      // grid cursor position
  combatStates: Record<number, any>; // agent_id -> latest combat state
  mapStates: Record<number, any>;    // agent_id -> latest map state
  runStates: Record<number, { deck: any[]; relics: any[]; potions: any[]; gold: number }>;
  systemStats: SystemStatsMsg | null;
  floorHistory: number[];
  lossHistory: number[];
  winHistory: number[];
  paused: boolean;
  deathStats: DeathStats;
}
