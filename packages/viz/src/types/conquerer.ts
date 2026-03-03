// Types for the conquerer viewer

export type ViewMode = 'single' | 'top3' | 'grid' | 'scroll';

export interface PathResult {
  path_id: number;
  seed: string;
  won: boolean;
  floors_reached: number;
  hp_remaining: number;
  total_reward: number;
  strategy: string; // "greedy", "random_0.5", "heuristic_1", "mcts_64"
}

export interface DivergenceNode {
  floor: number;
  decision_type: 'path' | 'card' | 'rest' | 'event' | 'shop';
  /** Which path IDs went which way at this branch. */
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
  active_paths: number; // How many still running
  elapsed_seconds: number;
  divergence_tree?: DivergenceNode;
}
