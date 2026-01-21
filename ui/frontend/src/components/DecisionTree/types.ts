/**
 * Decision Tree Types for Slay the Spire RL
 *
 * These types represent the decision tree structure used for
 * visualizing EV-based decision making in the game.
 */

export type DecisionType =
  | 'card_play'
  | 'path_choice'
  | 'card_reward'
  | 'shop'
  | 'event'
  | 'rest_choice'
  | 'potion_use'
  | 'boss_relic';

export interface DecisionNode {
  id: string;
  type: DecisionType;
  action: string;
  ev: number;
  winProbability: number;
  children: DecisionNode[];
  isExpanded: boolean;
  isPruned: boolean;
  // Optional metadata
  metadata?: {
    floor?: number;
    hp?: number;
    gold?: number;
    energy?: number;
    cardsPlayed?: number;
    stance?: 'Wrath' | 'Calm' | 'Neutral' | 'Divinity';
    [key: string]: unknown;
  };
}

export interface FloorState {
  floor: number;
  hp: number;
  maxHp: number;
  gold: number;
  deckSize: number;
  relicCount: number;
  relics: string[];
  potions: string[];
  roomType: string;
  encounter?: string;
  ev?: number;
}

export interface Path {
  id: string;
  name: string;
  floors: FloorState[];
  finalEV: number;
  createdAt: Date;
}

export interface TreeViewProps {
  root: DecisionNode;
  onNodeClick?: (node: DecisionNode) => void;
  onNodeExpand?: (node: DecisionNode) => void;
  onNodeCollapse?: (node: DecisionNode) => void;
  width?: number;
  height?: number;
  pruneThreshold?: number; // Probability threshold below which to prune (default 0.05)
}

export interface TreeNodeProps {
  node: DecisionNode;
  x: number;
  y: number;
  onNodeClick?: (node: DecisionNode) => void;
  isSelected?: boolean;
}

export interface TreeControlsProps {
  onExpandAll: () => void;
  onCollapseAll: () => void;
  onResetView: () => void;
  pruneThreshold: number;
  onPruneThresholdChange: (threshold: number) => void;
}

// Color coding for EV values
export function getEVColor(ev: number): string {
  if (ev > 0.1) return '#22c55e'; // green-500
  if (ev < -0.1) return '#ef4444'; // red-500
  return '#6b7280'; // gray-500 (neutral)
}

// Get background color with opacity for EV
export function getEVBackgroundColor(ev: number, opacity: number = 0.2): string {
  if (ev > 0.1) return `rgba(34, 197, 94, ${opacity})`;
  if (ev < -0.1) return `rgba(239, 68, 68, ${opacity})`;
  return `rgba(107, 114, 128, ${opacity})`;
}

// Format EV for display
export function formatEV(ev: number): string {
  const sign = ev >= 0 ? '+' : '';
  return `${sign}${ev.toFixed(2)}`;
}

// Format win probability
export function formatWinProbability(prob: number): string {
  return `${(prob * 100).toFixed(1)}%`;
}

// Decision type display names
export const DECISION_TYPE_LABELS: Record<DecisionType, string> = {
  card_play: 'Card Play',
  path_choice: 'Path Choice',
  card_reward: 'Card Reward',
  shop: 'Shop',
  event: 'Event',
  rest_choice: 'Rest Site',
  potion_use: 'Potion',
  boss_relic: 'Boss Relic',
};

// Decision type colors
export const DECISION_TYPE_COLORS: Record<DecisionType, string> = {
  card_play: '#3b82f6', // blue
  path_choice: '#8b5cf6', // purple
  card_reward: '#f59e0b', // amber
  shop: '#eab308', // yellow
  event: '#06b6d4', // cyan
  rest_choice: '#22c55e', // green
  potion_use: '#ec4899', // pink
  boss_relic: '#a855f7', // violet
};
