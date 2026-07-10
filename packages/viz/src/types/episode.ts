import type { Stance, RoomType, EnemyState, MapNode } from './engine';

export interface Episode {
  seed: string;
  won: boolean;
  floor: number;
  hp: number;
  maxHp: number;
  decisions: number;
  durationMs: number;
  totalReward: number;
  deckFinal: string[];
  relicsFinal: string[];
  deathEnemy: string | null;
  deathRoom: string | null;
  combats: Combat[];
  cardPicks: CardPick[];
  eventChoices: EventChoice[];
  pathChoices: PathChoice[];
  deckTimeline: DeckEvent[];
  timestamp: string;
  configName: string;
}

export interface Combat {
  floor: number;
  roomType: RoomType;
  encounterName: string;
  hpBefore: number;
  hpAfter: number;
  turns: Turn[];
  cardsPlayed: number;
  potionsUsed: number;
  stanceChanges: number;
  durationMs: number;
  solverMs: number;
}

export interface Turn {
  turn: number;
  cardsPlayed: string[];
  energyUsed: number;
  energyLeft: number;
  playerHp: number;
  playerBlock: number;
  stance: Stance;
  enemies: EnemyState[];
  handAtEnd: string[];
  unplayedPlayable: number;
  solverScores: [string, number][];
}

export interface CardPick {
  floor: number;
  offered: string[];
  chosen: string | null;
}

export interface EventChoice {
  floor: number;
  eventId: string;
  optionIndex: number;
  optionText: string;
}

export interface PathChoice {
  floor: number;
  options: MapNode[];
  chosen: number;
}

export interface DeckEvent {
  floor: number;
  action: 'add' | 'remove' | 'upgrade' | 'transform';
  card: string;
  source: 'combat_reward' | 'shop' | 'event' | 'boss_reward' | 'neow';
}
