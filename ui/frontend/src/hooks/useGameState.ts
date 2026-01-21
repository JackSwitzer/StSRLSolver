/**
 * Custom hook for accessing game state
 * Provides a cleaner interface to the Zustand store
 */

import { useGameStore } from '../store/gameStore';

/**
 * Hook for accessing seed and map data
 */
export function useGameState() {
  const {
    seed,
    seedValue,
    ascension,
    currentAct,
    mapData,
    neowOptions,
    encounters,
    boss,
    isLoading,
    error,
    fetchMapData,
    setAscension,
    setCurrentAct,
  } = useGameStore();

  return {
    // Data
    seed,
    seedValue,
    ascension,
    currentAct,
    mapData,
    neowOptions,
    encounters,
    boss,

    // Status
    isLoading,
    error,
    hasData: !!mapData,

    // Actions
    fetchMapData,
    setAscension,
    setCurrentAct,
  };
}

/**
 * Hook for accessing map data specifically
 */
export function useMapData() {
  const { mapData, isLoading, error } = useGameStore();

  return {
    nodes: mapData?.nodes ?? [],
    edges: mapData?.edges ?? [],
    width: mapData?.width ?? 7,
    height: mapData?.height ?? 15,
    isLoading,
    error,
    hasData: !!mapData,
  };
}

/**
 * Hook for accessing encounter data
 */
export function useEncounters() {
  const { encounters, boss, currentAct } = useGameStore();

  return {
    normalEncounters: encounters?.normal ?? [],
    eliteEncounters: encounters?.elite ?? [],
    boss,
    currentAct,
    hasEncounters: !!encounters,
  };
}
