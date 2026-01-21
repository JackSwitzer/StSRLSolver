/**
 * Custom hook for path management
 * Handles path building and event predictions
 */

import { useGameStore, PathStep } from '../store/gameStore';
import type { RoomType } from '../api/seedApi';

/**
 * Hook for managing path selection and predictions
 */
export function usePaths() {
  const {
    selectedPath,
    eventPredictions,
    pathEncounters,
    addToPath,
    removeFromPath,
    clearPath,
    canAddToPath,
    isNodeOnPath,
    getPathIndex,
  } = useGameStore();

  /**
   * Toggle a node on/off the path
   */
  const toggleNode = (x: number, y: number, type: RoomType) => {
    const pathIdx = getPathIndex(x, y);

    if (pathIdx >= 0) {
      // Remove from this point
      removeFromPath(pathIdx);
    } else if (canAddToPath({ x, y })) {
      // Add to path
      addToPath({ x, y, type });
    }
  };

  /**
   * Get prediction for an event room
   */
  const getEventPrediction = (x: number, y: number) => {
    const key = `${x},${y}`;
    return eventPredictions[key] ?? null;
  };

  return {
    // State
    selectedPath,
    eventPredictions,
    pathEncounters,
    pathLength: selectedPath.length,

    // Helpers
    isNodeOnPath,
    getPathIndex,
    canAddToPath: (node: { x: number; y: number }) => canAddToPath(node),
    getEventPrediction,

    // Actions
    addToPath,
    removeFromPath,
    clearPath,
    toggleNode,
  };
}

/**
 * Get encounter info for a path step
 */
export function getPathStepInfo(
  step: PathStep,
  encounters: { normal: string[]; elite: string[] } | null,
  eventPredictions: Record<string, { outcome: string; event_name?: string }>,
  normalIdx: number,
  eliteIdx: number
): {
  encounter: string;
  symbol: string;
  color: string;
  newNormalIdx: number;
  newEliteIdx: number;
} {
  const ROOM_COLORS: Record<RoomType, string> = {
    MONSTER: '#8b2635',
    ELITE: '#d4a857',
    REST: '#2d5a3d',
    SHOP: '#d4a857',
    EVENT: '#3d5a80',
    TREASURE: '#d4a857',
    BOSS: '#5a3d6e',
  };

  const ROOM_SYMBOLS: Record<RoomType, string> = {
    MONSTER: 'M',
    ELITE: 'E',
    REST: 'R',
    SHOP: '$',
    EVENT: '?',
    TREASURE: 'T',
    BOSS: 'B',
  };

  let encounter = '';
  let symbol = step.type ? ROOM_SYMBOLS[step.type] : '?';
  let color = step.type ? ROOM_COLORS[step.type] : '#3d5a80';
  let newNormalIdx = normalIdx;
  let newEliteIdx = eliteIdx;

  switch (step.type) {
    case 'MONSTER':
      encounter = encounters?.normal[normalIdx] || 'Unknown';
      newNormalIdx++;
      break;
    case 'ELITE':
      encounter = encounters?.elite[eliteIdx] || 'Unknown';
      newEliteIdx++;
      break;
    case 'REST':
      encounter = 'Rest Site';
      break;
    case 'SHOP':
      encounter = 'Shop';
      break;
    case 'TREASURE':
      encounter = 'Treasure';
      break;
    case 'EVENT': {
      const key = `${step.x},${step.y}`;
      const pred = eventPredictions[key];
      if (pred) {
        switch (pred.outcome) {
          case 'EVENT':
            encounter = pred.event_name || 'Event';
            break;
          case 'MONSTER':
            encounter = encounters?.normal[normalIdx] || 'Combat';
            newNormalIdx++;
            symbol = 'M';
            color = ROOM_COLORS.MONSTER;
            break;
          case 'ELITE':
            encounter = encounters?.elite[eliteIdx] || 'Elite';
            newEliteIdx++;
            symbol = 'E';
            color = ROOM_COLORS.ELITE;
            break;
          case 'SHOP':
            encounter = 'Shop';
            symbol = '$';
            color = ROOM_COLORS.SHOP;
            break;
          case 'TREASURE':
            encounter = 'Treasure';
            symbol = 'T';
            color = ROOM_COLORS.TREASURE;
            break;
        }
      } else {
        encounter = '? Unknown';
      }
      break;
    }
  }

  return { encounter, symbol, color, newNormalIdx, newEliteIdx };
}
