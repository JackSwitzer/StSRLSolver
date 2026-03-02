// TypeScript types matching Python ObservationDict structure

export interface GameObservation {
  phase: string;
  run: RunState;
  map?: MapState;
  combat?: CombatState;
  event?: EventState;
  reward?: RewardState;
  shop?: ShopState;
  rest?: RestState;
}

export interface RunState {
  hp: number;
  max_hp: number;
  gold: number;
  floor: number;
  act: number;
  deck: CardInstance[];
  relics: RelicInstance[];
  potions: PotionSlot[];
  ascension: number;
}

export interface MapState {
  nodes: MapNode[][];
  edges: MapEdge[];
  current_node: { x: number; y: number } | null;
  available_next: { x: number; y: number }[];
  boss_name: string;
}

export interface MapNode {
  x: number;
  y: number;
  type: MapNodeType;
}

export type MapNodeType =
  | 'monster'
  | 'elite'
  | 'boss'
  | 'event'
  | 'shop'
  | 'rest'
  | 'treasure';

export interface MapEdge {
  from: { x: number; y: number };
  to: { x: number; y: number };
}

export interface CombatState {
  player: PlayerState;
  enemies: EnemyState[];
  hand: CardInstance[];
  draw_pile_count: number;
  discard_pile_count: number;
  exhaust_pile_count: number;
  energy: number;
  max_energy: number;
  turn: number;
  stance: string;
}

export interface PlayerState {
  hp: number;
  max_hp: number;
  block: number;
  powers: PowerInstance[];
}

export interface EnemyState {
  id: string;
  name: string;
  hp: number;
  max_hp: number;
  block: number;
  intent: EnemyIntent;
  powers: PowerInstance[];
  size: 'small' | 'medium' | 'large';
}

export interface EnemyIntent {
  type: 'attack' | 'defend' | 'buff' | 'debuff' | 'unknown';
  damage?: number;
  hits?: number;
}

export interface CardInstance {
  id: string;
  name: string;
  cost: number;
  type: 'attack' | 'skill' | 'power' | 'status' | 'curse';
  upgraded: boolean;
  playable?: boolean;
  description?: string;
}

export interface RelicInstance {
  id: string;
  name: string;
  counter?: number;
}

export interface PotionSlot {
  id: string | null;
  name: string | null;
}

export interface PowerInstance {
  id: string;
  name: string;
  amount: number;
}

export interface EventState {
  id: string;
  name: string;
  body: string;
  options: EventOption[];
}

export interface EventOption {
  index: number;
  label: string;
  disabled: boolean;
}

export interface RewardState {
  rewards: RewardItem[];
}

export interface RewardItem {
  type: 'card' | 'gold' | 'potion' | 'relic' | 'key';
  value: unknown;
}

export interface ShopState {
  cards: (CardInstance & { price: number })[];
  relics: (RelicInstance & { price: number })[];
  potions: (PotionSlot & { price: number })[];
  purge_cost: number;
}

export interface RestState {
  options: string[];
}
