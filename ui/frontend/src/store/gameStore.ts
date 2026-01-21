/**
 * Zustand store for game state management
 */

import { create } from 'zustand';
import {
  fetchSeedData,
  fetchPathPredictions,
  SeedData,
  MapData,
  NeowOption,
  Encounters,
  BossData,
  PathNode,
  EventPrediction,
  RoomType,
} from '../api/seedApi';

// Path step with encounter info
export interface PathStep {
  x: number;
  y: number;
  type: RoomType;
}

// Store state interface
export interface GameState {
  // Seed data
  seed: string | null;
  seedValue: number | null;
  ascension: number;
  currentAct: number;

  // Map data
  mapData: MapData | null;
  neowOptions: NeowOption[];
  encounters: Encounters | null;
  boss: BossData | null;

  // UI state
  selectedFloor: number | null;
  isLoading: boolean;
  error: string | null;

  // Path tracking
  selectedPath: PathStep[];
  eventPredictions: Record<string, EventPrediction>;

  // Computed encounter indices along path
  pathEncounters: {
    normal: number;
    elite: number;
  };

  // Actions
  setSeed: (seed: string) => void;
  setAscension: (ascension: number) => void;
  setCurrentAct: (act: number) => void;
  fetchMapData: (seed: string, act?: number, ascension?: number) => Promise<void>;
  selectFloor: (floor: number | null) => void;

  // Path actions
  addToPath: (node: PathStep) => void;
  removeFromPath: (index: number) => void;
  clearPath: () => void;
  canAddToPath: (node: { x: number; y: number }) => boolean;
  isNodeOnPath: (x: number, y: number) => boolean;
  getPathIndex: (x: number, y: number) => number;

  // Internal
  updateEventPredictions: () => Promise<void>;
  recalculatePathEncounters: () => void;
}

export const useGameStore = create<GameState>((set, get) => ({
  // Initial state
  seed: null,
  seedValue: null,
  ascension: 20,
  currentAct: 1,
  mapData: null,
  neowOptions: [],
  encounters: null,
  boss: null,
  selectedFloor: null,
  isLoading: false,
  error: null,
  selectedPath: [],
  eventPredictions: {},
  pathEncounters: { normal: 0, elite: 0 },

  // Actions
  setSeed: (seed: string) => {
    set({ seed: seed.toUpperCase() });
  },

  setAscension: (ascension: number) => {
    set({ ascension });
  },

  setCurrentAct: async (act: number) => {
    const state = get();
    set({
      currentAct: act,
      selectedPath: [],
      eventPredictions: {},
      pathEncounters: { normal: 0, elite: 0 },
    });

    if (state.seed) {
      await get().fetchMapData(state.seed, act, state.ascension);
    }
  },

  fetchMapData: async (seed: string, act?: number, ascension?: number) => {
    const state = get();
    const targetAct = act ?? state.currentAct;
    const targetAscension = ascension ?? state.ascension;

    set({ isLoading: true, error: null });

    try {
      const data: SeedData = await fetchSeedData(seed, targetAct, targetAscension);

      set({
        seed: data.seed,
        seedValue: data.seed_value,
        ascension: data.ascension,
        currentAct: data.act,
        mapData: data.map,
        neowOptions: data.neow_options,
        encounters: data.encounters,
        boss: data.boss,
        isLoading: false,
        selectedPath: [],
        eventPredictions: {},
        pathEncounters: { normal: 0, elite: 0 },
      });
    } catch (error) {
      set({
        isLoading: false,
        error: error instanceof Error ? error.message : 'Failed to fetch seed data',
      });
    }
  },

  selectFloor: (floor: number | null) => {
    set({ selectedFloor: floor });
  },

  // Path management
  addToPath: (node: PathStep) => {
    const state = get();
    if (!state.canAddToPath(node)) return;

    const newPath = [...state.selectedPath, node];
    set({ selectedPath: newPath });

    get().recalculatePathEncounters();
    get().updateEventPredictions();
  },

  removeFromPath: (index: number) => {
    const state = get();
    const newPath = state.selectedPath.slice(0, index);
    set({ selectedPath: newPath });

    get().recalculatePathEncounters();
    get().updateEventPredictions();
  },

  clearPath: () => {
    set({
      selectedPath: [],
      eventPredictions: {},
      pathEncounters: { normal: 0, elite: 0 },
    });
  },

  canAddToPath: (node: { x: number; y: number }) => {
    const state = get();
    if (!state.mapData) return false;

    // If path is empty, must start from floor 0 (y === 0)
    if (state.selectedPath.length === 0) {
      return node.y === 0;
    }

    // Otherwise, check if there's an edge from last node to this node
    const lastNode = state.selectedPath[state.selectedPath.length - 1];
    return state.mapData.edges.some(
      (e) =>
        e.src_x === lastNode.x &&
        e.src_y === lastNode.y &&
        e.dst_x === node.x &&
        e.dst_y === node.y
    );
  },

  isNodeOnPath: (x: number, y: number) => {
    return get().selectedPath.some((p) => p.x === x && p.y === y);
  },

  getPathIndex: (x: number, y: number) => {
    return get().selectedPath.findIndex((p) => p.x === x && p.y === y);
  },

  recalculatePathEncounters: () => {
    const state = get();
    let normal = 0;
    let elite = 0;

    for (const step of state.selectedPath) {
      if (step.type === 'MONSTER') normal++;
      if (step.type === 'ELITE') elite++;
    }

    set({ pathEncounters: { normal, elite } });
  },

  updateEventPredictions: async () => {
    const state = get();
    if (!state.seed || state.selectedPath.length === 0) {
      set({ eventPredictions: {} });
      return;
    }

    // Filter to only EVENT nodes
    const eventNodes = state.selectedPath.filter((p) => p.type === 'EVENT');
    if (eventNodes.length === 0) {
      set({ eventPredictions: {} });
      return;
    }

    try {
      const pathNodes: PathNode[] = state.selectedPath.map((p) => ({
        x: p.x,
        y: p.y,
        type: p.type,
      }));

      const result = await fetchPathPredictions(state.seed, pathNodes, state.currentAct);
      set({ eventPredictions: result.event_predictions });
    } catch (error) {
      console.error('Failed to fetch event predictions:', error);
    }
  },
}));
