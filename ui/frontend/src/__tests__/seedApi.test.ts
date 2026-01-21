/**
 * Tests for seedApi module
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  fetchSeedData,
  fetchFloorDetails,
  fetchPathPredictions,
} from '../api/seedApi';
import { createMockSeedData } from '../test-utils';

describe('seedApi', () => {
  const originalFetch = global.fetch;

  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    global.fetch = originalFetch;
  });

  describe('fetchSeedData', () => {
    it('makes GET request to correct endpoint', async () => {
      const mockData = createMockSeedData();
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockData),
      });

      await fetchSeedData('TESTSEED');

      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/seed/TESTSEED')
      );
    });

    it('includes act and ascension in query params', async () => {
      const mockData = createMockSeedData();
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockData),
      });

      await fetchSeedData('TESTSEED', 2, 15);

      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('act=2')
      );
      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('ascension=15')
      );
    });

    it('uppercases seed in URL', async () => {
      const mockData = createMockSeedData();
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockData),
      });

      await fetchSeedData('lowercase');

      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/seed/LOWERCASE')
      );
    });

    it('returns parsed JSON response', async () => {
      const mockData = createMockSeedData();
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockData),
      });

      const result = await fetchSeedData('TESTSEED');

      expect(result).toEqual(mockData);
    });

    it('throws error on failed request', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: false,
        statusText: 'Not Found',
      });

      await expect(fetchSeedData('BADSEED')).rejects.toThrow(
        'Failed to fetch seed data: Not Found'
      );
    });

    it('uses default act and ascension values', async () => {
      const mockData = createMockSeedData();
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockData),
      });

      await fetchSeedData('TESTSEED');

      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('act=1')
      );
      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('ascension=20')
      );
    });
  });

  describe('fetchFloorDetails', () => {
    const mockFloorDetails = {
      floor: 3,
      room_type: 'MONSTER',
      enemy: 'Cultist',
      hp: 50,
      first_move: { move: 'Incantation', damage: 0, details: 'Ritual 3' },
      gold: 15,
      cards: [
        { name: 'Anger', rarity: 'COMMON', upgraded: false },
      ],
    };

    it('makes GET request to correct endpoint', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockFloorDetails),
      });

      await fetchFloorDetails('TESTSEED', 3, 'MONSTER');

      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/floor/TESTSEED/3')
      );
    });

    it('includes room type in query params', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockFloorDetails),
      });

      await fetchFloorDetails('TESTSEED', 3, 'ELITE');

      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('type=ELITE')
      );
    });

    it('includes encounter index in query params', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockFloorDetails),
      });

      await fetchFloorDetails('TESTSEED', 3, 'MONSTER', 2);

      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('idx=2')
      );
    });

    it('includes act and ascension in query params', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockFloorDetails),
      });

      await fetchFloorDetails('TESTSEED', 3, 'MONSTER', 0, 2, 15);

      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('act=2')
      );
      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('ascension=15')
      );
    });

    it('returns parsed JSON response', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockFloorDetails),
      });

      const result = await fetchFloorDetails('TESTSEED', 3, 'MONSTER');

      expect(result).toEqual(mockFloorDetails);
    });

    it('throws error on failed request', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: false,
        statusText: 'Server Error',
      });

      await expect(
        fetchFloorDetails('TESTSEED', 3, 'MONSTER')
      ).rejects.toThrow('Failed to fetch floor details: Server Error');
    });

    it('uppercases seed in URL', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockFloorDetails),
      });

      await fetchFloorDetails('lowercase', 3, 'MONSTER');

      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/floor/LOWERCASE/3')
      );
    });
  });

  describe('fetchPathPredictions', () => {
    const mockPredictions = {
      event_predictions: {
        '1,3': { outcome: 'EVENT', roll: 0.45, event_name: 'Big Fish' },
        '2,5': { outcome: 'MONSTER', roll: 0.82 },
      },
    };

    it('makes POST request to correct endpoint', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockPredictions),
      });

      const path = [
        { x: 0, y: 0, type: 'MONSTER' as const },
        { x: 1, y: 3, type: 'EVENT' as const },
      ];

      await fetchPathPredictions('TESTSEED', path);

      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/path/TESTSEED'),
        expect.objectContaining({
          method: 'POST',
        })
      );
    });

    it('includes act in query params', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockPredictions),
      });

      const path = [{ x: 0, y: 0, type: 'MONSTER' as const }];

      await fetchPathPredictions('TESTSEED', path, 2);

      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('act=2'),
        expect.anything()
      );
    });

    it('sends path in request body', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockPredictions),
      });

      const path = [
        { x: 0, y: 0, type: 'MONSTER' as const },
        { x: 1, y: 1, type: 'EVENT' as const },
      ];

      await fetchPathPredictions('TESTSEED', path);

      expect(global.fetch).toHaveBeenCalledWith(
        expect.anything(),
        expect.objectContaining({
          body: JSON.stringify({ path }),
        })
      );
    });

    it('sets correct content-type header', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockPredictions),
      });

      const path = [{ x: 0, y: 0, type: 'MONSTER' as const }];

      await fetchPathPredictions('TESTSEED', path);

      expect(global.fetch).toHaveBeenCalledWith(
        expect.anything(),
        expect.objectContaining({
          headers: {
            'Content-Type': 'application/json',
          },
        })
      );
    });

    it('returns parsed JSON response', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockPredictions),
      });

      const path = [{ x: 0, y: 0, type: 'MONSTER' as const }];

      const result = await fetchPathPredictions('TESTSEED', path);

      expect(result).toEqual(mockPredictions);
    });

    it('throws error on failed request', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: false,
        statusText: 'Bad Request',
      });

      const path = [{ x: 0, y: 0, type: 'MONSTER' as const }];

      await expect(
        fetchPathPredictions('TESTSEED', path)
      ).rejects.toThrow('Failed to fetch path predictions: Bad Request');
    });

    it('uppercases seed in URL', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockPredictions),
      });

      const path = [{ x: 0, y: 0, type: 'MONSTER' as const }];

      await fetchPathPredictions('lowercase', path);

      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/path/LOWERCASE'),
        expect.anything()
      );
    });

    it('handles empty path', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ event_predictions: {} }),
      });

      const result = await fetchPathPredictions('TESTSEED', []);

      expect(result.event_predictions).toEqual({});
    });
  });

  describe('error handling', () => {
    it('handles network errors', async () => {
      global.fetch = vi.fn().mockRejectedValue(new Error('Network error'));

      await expect(fetchSeedData('TESTSEED')).rejects.toThrow('Network error');
    });

    it('handles JSON parse errors', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.reject(new Error('Invalid JSON')),
      });

      await expect(fetchSeedData('TESTSEED')).rejects.toThrow('Invalid JSON');
    });
  });
});
