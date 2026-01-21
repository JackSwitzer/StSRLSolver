/**
 * Map-related type definitions
 */

import type { RoomType } from '../../api/seedApi';

// Room colors matching the original CSS variables
export const ROOM_COLORS: Record<RoomType, string> = {
  MONSTER: '#8b2635',
  ELITE: '#d4a857',
  REST: '#2d5a3d',
  SHOP: '#d4a857',
  EVENT: '#3d5a80',
  TREASURE: '#d4a857',
  BOSS: '#5a3d6e',
};

// Room symbols for canvas rendering
export const ROOM_SYMBOLS: Record<RoomType, string> = {
  MONSTER: 'M',
  ELITE: 'E',
  REST: 'R',
  SHOP: '$',
  EVENT: '?',
  TREASURE: 'T',
  BOSS: 'B',
};

// Position for canvas rendering
export interface NodePosition {
  x: number;
  y: number;
}

// Extended node with rendering position
export interface RenderableNode {
  x: number;
  y: number;
  type: RoomType | null;
  symbol: string;
  hasEdges: boolean;
  position: NodePosition;
  radius: number;
}
