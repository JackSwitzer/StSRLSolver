/**
 * Tests for gameStore (Zustand store)
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useGameStore } from '../store/gameStore';
import { createMockSeedData, createMockMapData } from '../test-utils';

// Mock the API module
vi.mock('../api/seedApi', () => ({
  fetchSeedData: vi.fn(),
  fetchPathPredictions: vi.fn(),
}));

import { fetchSeedData, fetchPathPredictions } from '../api/seedApi';

describe('gameStore', () => {
  beforeEach(() => {
    // Reset store to initial state
    useGameStore.setState({
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
    });

    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('initial state', () => {
    it('has null seed initially', () => {
      const state = useGameStore.getState();
      expect(state.seed).toBeNull();
    });

    it('has ascension 20 by default', () => {
      const state = useGameStore.getState();
      expect(state.ascension).toBe(20);
    });

    it('has currentAct 1 by default', () => {
      const state = useGameStore.getState();
      expect(state.currentAct).toBe(1);
    });

    it('has empty selected path initially', () => {
      const state = useGameStore.getState();
      expect(state.selectedPath).toEqual([]);
    });

    it('has isLoading false initially', () => {
      const state = useGameStore.getState();
      expect(state.isLoading).toBe(false);
    });
  });

  describe('setSeed', () => {
    it('sets seed value', () => {
      useGameStore.getState().setSeed('testseed');
      expect(useGameStore.getState().seed).toBe('TESTSEED');
    });

    it('uppercases the seed', () => {
      useGameStore.getState().setSeed('lowercase');
      expect(useGameStore.getState().seed).toBe('LOWERCASE');
    });
  });

  describe('setAscension', () => {
    it('sets ascension value', () => {
      useGameStore.getState().setAscension(15);
      expect(useGameStore.getState().ascension).toBe(15);
    });

    it('allows ascension 0', () => {
      useGameStore.getState().setAscension(0);
      expect(useGameStore.getState().ascension).toBe(0);
    });
  });

  describe('setCurrentAct', () => {
    it('sets current act', async () => {
      await useGameStore.getState().setCurrentAct(2);
      expect(useGameStore.getState().currentAct).toBe(2);
    });

    it('clears selected path when changing act', async () => {
      useGameStore.setState({
        selectedPath: [{ x: 0, y: 0, type: 'MONSTER' }],
      });

      await useGameStore.getState().setCurrentAct(2);
      expect(useGameStore.getState().selectedPath).toEqual([]);
    });

    it('clears event predictions when changing act', async () => {
      useGameStore.setState({
        eventPredictions: { '0,0': { outcome: 'EVENT', roll: 0.5 } },
      });

      await useGameStore.getState().setCurrentAct(2);
      expect(useGameStore.getState().eventPredictions).toEqual({});
    });
  });

  describe('fetchMapData', () => {
    it('sets isLoading to true during fetch', async () => {
      const mockData = createMockSeedData();
      (fetchSeedData as ReturnType<typeof vi.fn>).mockResolvedValue(mockData);

      const fetchPromise = useGameStore.getState().fetchMapData('TESTSEED');
      expect(useGameStore.getState().isLoading).toBe(true);

      await fetchPromise;
    });

    it('sets isLoading to false after successful fetch', async () => {
      const mockData = createMockSeedData();
      (fetchSeedData as ReturnType<typeof vi.fn>).mockResolvedValue(mockData);

      await useGameStore.getState().fetchMapData('TESTSEED');
      expect(useGameStore.getState().isLoading).toBe(false);
    });

    it('updates seed data after successful fetch', async () => {
      const mockData = createMockSeedData();
      (fetchSeedData as ReturnType<typeof vi.fn>).mockResolvedValue(mockData);

      await useGameStore.getState().fetchMapData('TESTSEED');

      const state = useGameStore.getState();
      expect(state.seed).toBe('TESTSEED');
      expect(state.seedValue).toBe(123456789);
      expect(state.mapData).toEqual(mockData.map);
      expect(state.encounters).toEqual(mockData.encounters);
      expect(state.boss).toEqual(mockData.boss);
    });

    it('clears path after fetch', async () => {
      useGameStore.setState({
        selectedPath: [{ x: 0, y: 0, type: 'MONSTER' }],
      });

      const mockData = createMockSeedData();
      (fetchSeedData as ReturnType<typeof vi.fn>).mockResolvedValue(mockData);

      await useGameStore.getState().fetchMapData('TESTSEED');
      expect(useGameStore.getState().selectedPath).toEqual([]);
    });

    it('sets error on fetch failure', async () => {
      (fetchSeedData as ReturnType<typeof vi.fn>).mockRejectedValue(new Error('Network error'));

      await useGameStore.getState().fetchMapData('TESTSEED');

      const state = useGameStore.getState();
      expect(state.isLoading).toBe(false);
      expect(state.error).toBe('Network error');
    });

    it('uses provided act and ascension', async () => {
      const mockData = createMockSeedData();
      (fetchSeedData as ReturnType<typeof vi.fn>).mockResolvedValue(mockData);

      await useGameStore.getState().fetchMapData('TESTSEED', 2, 15);

      expect(fetchSeedData).toHaveBeenCalledWith('TESTSEED', 2, 15);
    });
  });

  describe('selectFloor', () => {
    it('sets selected floor', () => {
      useGameStore.getState().selectFloor(5);
      expect(useGameStore.getState().selectedFloor).toBe(5);
    });

    it('allows null to deselect', () => {
      useGameStore.getState().selectFloor(5);
      useGameStore.getState().selectFloor(null);
      expect(useGameStore.getState().selectedFloor).toBeNull();
    });
  });

  describe('path management', () => {
    beforeEach(() => {
      const mockMapData = createMockMapData();
      useGameStore.setState({
        mapData: mockMapData,
        seed: 'TESTSEED',
      });
    });

    describe('canAddToPath', () => {
      it('returns false when no map data', () => {
        useGameStore.setState({ mapData: null });
        const result = useGameStore.getState().canAddToPath({ x: 0, y: 0 });
        expect(result).toBe(false);
      });

      it('allows floor 0 nodes when path is empty', () => {
        const result = useGameStore.getState().canAddToPath({ x: 0, y: 0 });
        expect(result).toBe(true);
      });

      it('rejects non-floor-0 nodes when path is empty', () => {
        const result = useGameStore.getState().canAddToPath({ x: 0, y: 1 });
        expect(result).toBe(false);
      });

      it('allows connected nodes', () => {
        useGameStore.setState({
          selectedPath: [{ x: 0, y: 0, type: 'MONSTER' }],
        });

        const result = useGameStore.getState().canAddToPath({ x: 0, y: 1 });
        expect(result).toBe(true);
      });

      it('rejects unconnected nodes', () => {
        useGameStore.setState({
          selectedPath: [{ x: 0, y: 0, type: 'MONSTER' }],
        });

        const result = useGameStore.getState().canAddToPath({ x: 2, y: 5 });
        expect(result).toBe(false);
      });
    });

    describe('addToPath', () => {
      it('adds node to path', () => {
        useGameStore.getState().addToPath({ x: 0, y: 0, type: 'MONSTER' });
        expect(useGameStore.getState().selectedPath).toHaveLength(1);
        expect(useGameStore.getState().selectedPath[0]).toEqual({ x: 0, y: 0, type: 'MONSTER' });
      });

      it('does not add invalid nodes', () => {
        useGameStore.getState().addToPath({ x: 5, y: 5, type: 'MONSTER' });
        expect(useGameStore.getState().selectedPath).toHaveLength(0);
      });

      it('recalculates path encounters', () => {
        useGameStore.getState().addToPath({ x: 0, y: 0, type: 'MONSTER' });
        expect(useGameStore.getState().pathEncounters.normal).toBe(1);
      });
    });

    describe('removeFromPath', () => {
      it('removes nodes from index onwards', () => {
        useGameStore.setState({
          selectedPath: [
            { x: 0, y: 0, type: 'MONSTER' },
            { x: 0, y: 1, type: 'MONSTER' },
            { x: 0, y: 2, type: 'ELITE' },
          ],
        });

        useGameStore.getState().removeFromPath(1);
        expect(useGameStore.getState().selectedPath).toHaveLength(1);
      });

      it('clears path when index is 0', () => {
        useGameStore.setState({
          selectedPath: [
            { x: 0, y: 0, type: 'MONSTER' },
            { x: 0, y: 1, type: 'MONSTER' },
          ],
        });

        useGameStore.getState().removeFromPath(0);
        expect(useGameStore.getState().selectedPath).toHaveLength(0);
      });
    });

    describe('clearPath', () => {
      it('empties the path', () => {
        useGameStore.setState({
          selectedPath: [{ x: 0, y: 0, type: 'MONSTER' }],
        });

        useGameStore.getState().clearPath();
        expect(useGameStore.getState().selectedPath).toEqual([]);
      });

      it('clears event predictions', () => {
        useGameStore.setState({
          selectedPath: [{ x: 0, y: 0, type: 'EVENT' }],
          eventPredictions: { '0,0': { outcome: 'EVENT', roll: 0.5 } },
        });

        useGameStore.getState().clearPath();
        expect(useGameStore.getState().eventPredictions).toEqual({});
      });

      it('resets path encounters', () => {
        useGameStore.setState({
          selectedPath: [{ x: 0, y: 0, type: 'MONSTER' }],
          pathEncounters: { normal: 1, elite: 0 },
        });

        useGameStore.getState().clearPath();
        expect(useGameStore.getState().pathEncounters).toEqual({ normal: 0, elite: 0 });
      });
    });

    describe('isNodeOnPath', () => {
      it('returns true for nodes on path', () => {
        useGameStore.setState({
          selectedPath: [{ x: 0, y: 0, type: 'MONSTER' }],
        });

        expect(useGameStore.getState().isNodeOnPath(0, 0)).toBe(true);
      });

      it('returns false for nodes not on path', () => {
        useGameStore.setState({
          selectedPath: [{ x: 0, y: 0, type: 'MONSTER' }],
        });

        expect(useGameStore.getState().isNodeOnPath(1, 1)).toBe(false);
      });
    });

    describe('getPathIndex', () => {
      it('returns correct index for nodes on path', () => {
        useGameStore.setState({
          selectedPath: [
            { x: 0, y: 0, type: 'MONSTER' },
            { x: 0, y: 1, type: 'EVENT' },
          ],
        });

        expect(useGameStore.getState().getPathIndex(0, 0)).toBe(0);
        expect(useGameStore.getState().getPathIndex(0, 1)).toBe(1);
      });

      it('returns -1 for nodes not on path', () => {
        useGameStore.setState({
          selectedPath: [{ x: 0, y: 0, type: 'MONSTER' }],
        });

        expect(useGameStore.getState().getPathIndex(5, 5)).toBe(-1);
      });
    });
  });

  describe('recalculatePathEncounters', () => {
    it('counts monster encounters', () => {
      useGameStore.setState({
        selectedPath: [
          { x: 0, y: 0, type: 'MONSTER' },
          { x: 0, y: 1, type: 'MONSTER' },
          { x: 0, y: 2, type: 'MONSTER' },
        ],
      });

      useGameStore.getState().recalculatePathEncounters();
      expect(useGameStore.getState().pathEncounters.normal).toBe(3);
    });

    it('counts elite encounters', () => {
      useGameStore.setState({
        selectedPath: [
          { x: 0, y: 0, type: 'ELITE' },
          { x: 0, y: 1, type: 'ELITE' },
        ],
      });

      useGameStore.getState().recalculatePathEncounters();
      expect(useGameStore.getState().pathEncounters.elite).toBe(2);
    });

    it('handles mixed path', () => {
      useGameStore.setState({
        selectedPath: [
          { x: 0, y: 0, type: 'MONSTER' },
          { x: 0, y: 1, type: 'EVENT' },
          { x: 0, y: 2, type: 'ELITE' },
          { x: 0, y: 3, type: 'REST' },
          { x: 0, y: 4, type: 'MONSTER' },
        ],
      });

      useGameStore.getState().recalculatePathEncounters();
      expect(useGameStore.getState().pathEncounters).toEqual({ normal: 2, elite: 1 });
    });
  });

  describe('updateEventPredictions', () => {
    beforeEach(() => {
      useGameStore.setState({
        seed: 'TESTSEED',
        mapData: createMockMapData(),
      });
    });

    it('clears predictions when no seed', async () => {
      useGameStore.setState({ seed: null });
      await useGameStore.getState().updateEventPredictions();
      expect(useGameStore.getState().eventPredictions).toEqual({});
    });

    it('clears predictions when path is empty', async () => {
      useGameStore.setState({ selectedPath: [] });
      await useGameStore.getState().updateEventPredictions();
      expect(useGameStore.getState().eventPredictions).toEqual({});
    });

    it('clears predictions when no event nodes in path', async () => {
      useGameStore.setState({
        selectedPath: [{ x: 0, y: 0, type: 'MONSTER' }],
      });

      await useGameStore.getState().updateEventPredictions();
      expect(useGameStore.getState().eventPredictions).toEqual({});
    });

    it('fetches predictions for event nodes', async () => {
      (fetchPathPredictions as ReturnType<typeof vi.fn>).mockResolvedValue({
        event_predictions: {
          '0,0': { outcome: 'EVENT', roll: 0.5, event_name: 'Big Fish' },
        },
      });

      useGameStore.setState({
        selectedPath: [{ x: 0, y: 0, type: 'EVENT' }],
      });

      await useGameStore.getState().updateEventPredictions();

      expect(fetchPathPredictions).toHaveBeenCalled();
      expect(useGameStore.getState().eventPredictions['0,0']).toBeDefined();
    });

    it('handles fetch error gracefully', async () => {
      (fetchPathPredictions as ReturnType<typeof vi.fn>).mockRejectedValue(new Error('API error'));

      useGameStore.setState({
        selectedPath: [{ x: 0, y: 0, type: 'EVENT' }],
      });

      // Should not throw
      await expect(useGameStore.getState().updateEventPredictions()).resolves.not.toThrow();
    });
  });
});
