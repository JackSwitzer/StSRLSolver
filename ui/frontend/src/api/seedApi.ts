/**
 * API layer for communicating with the Flask backend
 */

// Types
export interface MapNode {
  x: number;
  y: number;
  type: RoomType | null;
  symbol: string;
  has_edges: boolean;
}

export interface MapEdge {
  src_x: number;
  src_y: number;
  dst_x: number;
  dst_y: number;
  is_boss: boolean;
}

export interface MapData {
  nodes: MapNode[];
  edges: MapEdge[];
  width: number;
  height: number;
}

export interface NeowOption {
  slot: number;
  type: 'blessing' | 'bonus' | 'trade' | 'boss_swap';
  option: string;
  name: string;
  drawback: string | null;
  drawback_id?: string;
}

export interface Encounters {
  normal: string[];
  elite: string[];
}

export interface BossData {
  name: string;
  hp: number;
  a9_hp: number;
  move: string;
  details: string;
}

export interface SeedData {
  seed: string;
  seed_value: number;
  ascension: number;
  act: number;
  neow_options: NeowOption[];
  map: MapData;
  encounters: Encounters;
  boss: BossData;
}

export interface CardReward {
  name: string;
  rarity: 'COMMON' | 'UNCOMMON' | 'RARE';
  upgraded: boolean;
}

export interface PotionReward {
  name: string;
  rarity: string;
}

export interface RelicReward {
  name: string;
  tier: string;
}

export interface FirstMove {
  move: string;
  intent?: string;
  damage?: number;
  details?: string;
}

export interface ShopInventory {
  cards: [string, number][];
  colorless?: [string, number][];
  relics: [string, number][];
  potions?: [string, number][];
  purge_cost?: number;
  error?: string;
}

export interface FloorDetails {
  floor: number;
  room_type: string;
  enemy?: string;
  hp?: number;
  first_move?: FirstMove;
  gold?: number;
  cards?: CardReward[];
  potion?: PotionReward;
  relic?: RelicReward;
  note?: string;
  shop?: ShopInventory;
}

export interface PathNode {
  x: number;
  y: number;
  type: RoomType;
}

export interface EventPrediction {
  outcome: 'EVENT' | 'MONSTER' | 'ELITE' | 'SHOP' | 'TREASURE';
  roll: number;
  event_name?: string;
}

export interface PathPredictions {
  event_predictions: Record<string, EventPrediction>;
}

export type RoomType = 'MONSTER' | 'ELITE' | 'REST' | 'SHOP' | 'EVENT' | 'TREASURE' | 'BOSS';

// API Functions

const API_BASE = '/api';

/**
 * Fetch complete seed data including map, encounters, Neow options
 */
export async function fetchSeedData(
  seed: string,
  act: number = 1,
  ascension: number = 20
): Promise<SeedData> {
  const params = new URLSearchParams({
    act: act.toString(),
    ascension: ascension.toString(),
  });

  const response = await fetch(`${API_BASE}/seed/${seed.toUpperCase()}?${params}`);

  if (!response.ok) {
    throw new Error(`Failed to fetch seed data: ${response.statusText}`);
  }

  return response.json();
}

/**
 * Fetch detailed information for a specific floor
 */
export async function fetchFloorDetails(
  seed: string,
  floor: number,
  roomType: RoomType,
  encounterIdx: number = 0,
  act: number = 1,
  ascension: number = 20
): Promise<FloorDetails> {
  const params = new URLSearchParams({
    act: act.toString(),
    type: roomType,
    idx: encounterIdx.toString(),
    ascension: ascension.toString(),
  });

  const response = await fetch(`${API_BASE}/floor/${seed.toUpperCase()}/${floor}?${params}`);

  if (!response.ok) {
    throw new Error(`Failed to fetch floor details: ${response.statusText}`);
  }

  return response.json();
}

/**
 * Predict event room outcomes for a given path
 */
export async function fetchPathPredictions(
  seed: string,
  path: PathNode[],
  act: number = 1
): Promise<PathPredictions> {
  const params = new URLSearchParams({
    act: act.toString(),
  });

  const response = await fetch(`${API_BASE}/path/${seed.toUpperCase()}?${params}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ path }),
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch path predictions: ${response.statusText}`);
  }

  return response.json();
}
