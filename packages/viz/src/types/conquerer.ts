// Types for the conquerer viewer

export type ViewMode = 'grid' | 'scroll' | 'single';

export interface PathResult {
  path_id: number;
  seed: string;
  won: boolean;
  floors_reached: number;
  hp_remaining: number;
  total_reward: number;
  strategy: string;
}

export interface DivergenceNode {
  floor: number;
  decision_type: 'path' | 'card' | 'rest' | 'event' | 'shop';
  branches: DivergenceBranch[];
}

export interface DivergenceBranch {
  label: string;
  path_ids: number[];
  children: DivergenceNode[];
}

export interface ConquererState {
  seed: string;
  paths: PathResult[];
  best_path_id: number;
  win_count: number;
  max_floor: number;
  active_paths: number;
  elapsed_seconds: number;
  divergence_tree?: DivergenceNode;
}

/** Creative agent names -- each numbered 1-16 */
export const AGENT_NAMES: Record<number, string> = {
  0: 'Oracle',
  1: 'Gambler',
  2: 'Wanderer',
  3: 'Wildcard',
  4: 'Sentinel',
  5: 'Guardian',
  6: 'Tactician',
  7: 'Drifter',
  8: 'Spectre',
  9: 'Pilgrim',
  10: 'Vanguard',
  11: 'Mystic',
  12: 'Reaper',
  13: 'Nomad',
  14: 'Arbiter',
  15: 'Seeker',
};

export function agentName(pathId: number): string {
  return AGENT_NAMES[pathId] ?? `Agent ${pathId}`;
}

/** Determine which act a floor belongs to */
export function floorToAct(floor: number): number {
  if (floor <= 17) return 1;
  if (floor <= 34) return 2;
  if (floor <= 51) return 3;
  return 4;
}
