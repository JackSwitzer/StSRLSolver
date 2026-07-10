// Mirrors: packages/engine-rs/src/{combat_types,state,run,map}.rs

export type Stance = 'neutral' | 'wrath' | 'calm' | 'divinity';
export type RoomType = 'monster' | 'elite' | 'boss' | 'rest' | 'shop' | 'event' | 'treasure';
export type CardType = 'attack' | 'skill' | 'power' | 'status' | 'curse';
export type RunPhase = 'neow' | 'map' | 'combat' | 'card_reward' | 'campfire' | 'shop' | 'event' | 'game_over';

export interface CardRef {
  id: string;
  name: string;
  cost: number;
  type: CardType;
  upgraded: boolean;
}

export type Intent =
  | { kind: 'attack'; damage: number; hits: number }
  | { kind: 'block'; amount: number }
  | { kind: 'buff' }
  | { kind: 'debuff' }
  | { kind: 'attack_block'; damage: number; hits: number; block: number }
  | { kind: 'attack_buff'; damage: number; hits: number }
  | { kind: 'attack_debuff'; damage: number; hits: number }
  | { kind: 'defend_buff'; block: number }
  | { kind: 'spawn' }
  | { kind: 'escape' }
  | { kind: 'sleep' }
  | { kind: 'stun' }
  | { kind: 'unknown' };

export interface EnemyState {
  id: string;
  name: string;
  hp: number;
  maxHp: number;
  block: number;
  intent: Intent;
}

export interface MapNode {
  x: number;
  y: number;
  roomType: RoomType;
}

export const STANCE_COLORS: Record<Stance, string> = {
  neutral: '#8b95a5',
  calm: '#87ceeb',      // sky blue
  wrath: '#8b0000',     // blood red
  divinity: '#ffd700',  // gold
};

export const ROOM_COLORS: Record<RoomType, string> = {
  monster: '#cc5500',   // burnt orange
  elite: '#7b2d8e',     // royal purple
  boss: '#8b0000',      // blood red
  rest: '#20b2aa',      // turquoise
  shop: '#daa520',      // gold
  event: '#2ecc71',     // emerald green
  treasure: '#ffd700',  // gold
};

export const CARD_TYPE_COLORS: Record<CardType, string> = {
  attack: '#cc5500',    // burnt orange
  skill: '#87ceeb',     // sky blue
  power: '#ffd700',     // gold
  status: '#8b95a5',
  curse: '#7b2d8e',     // royal purple
};
