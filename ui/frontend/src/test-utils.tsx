/**
 * Test utilities for React Testing Library with Vitest
 */

import '@testing-library/jest-dom/vitest';
import { cleanup, render } from '@testing-library/react';
import { afterEach, vi, beforeAll } from 'vitest';
import type { ReactElement } from 'react';

// Cleanup after each test
afterEach(() => {
  cleanup();
});

// Setup mocks before all tests (jsdom environment should be ready)
beforeAll(() => {
  // Mock ResizeObserver for components that use it
  global.ResizeObserver = vi.fn().mockImplementation(() => ({
    observe: vi.fn(),
    unobserve: vi.fn(),
    disconnect: vi.fn(),
  }));

  // Mock window.matchMedia
  if (typeof window !== 'undefined') {
    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      value: vi.fn().mockImplementation((query: string) => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
    });
  }

  // Mock canvas context for MapCanvas tests
  if (typeof HTMLCanvasElement !== 'undefined') {
    HTMLCanvasElement.prototype.getContext = vi.fn().mockImplementation((contextId: string) => {
      if (contextId === '2d') {
        return {
          fillStyle: '',
          strokeStyle: '',
          lineWidth: 1,
          lineCap: 'butt',
          font: '',
          textAlign: 'left',
          textBaseline: 'top',
          clearRect: vi.fn(),
          fillRect: vi.fn(),
          strokeRect: vi.fn(),
          fillText: vi.fn(),
          strokeText: vi.fn(),
          beginPath: vi.fn(),
          closePath: vi.fn(),
          moveTo: vi.fn(),
          lineTo: vi.fn(),
          arc: vi.fn(),
          fill: vi.fn(),
          stroke: vi.fn(),
          scale: vi.fn(),
          createRadialGradient: vi.fn().mockReturnValue({
            addColorStop: vi.fn(),
          }),
          createLinearGradient: vi.fn().mockReturnValue({
            addColorStop: vi.fn(),
          }),
          save: vi.fn(),
          restore: vi.fn(),
          translate: vi.fn(),
          rotate: vi.fn(),
          measureText: vi.fn().mockReturnValue({ width: 0 }),
        };
      }
      return null;
    });
  }
});

// Custom render function that can be extended with providers
function customRender(ui: ReactElement, options = {}) {
  return render(ui, {
    ...options,
  });
}

// Re-export everything from testing-library
export * from '@testing-library/react';
export { customRender as render };

// Helper to create mock fetch responses
export function mockFetchResponse<T>(data: T, ok = true) {
  return vi.fn().mockResolvedValue({
    ok,
    json: () => Promise.resolve(data),
    statusText: ok ? 'OK' : 'Error',
  });
}

// Helper to wait for state updates
export function waitForStateUpdate() {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

// Mock data generators
export function createMockMapData() {
  return {
    nodes: [
      { x: 0, y: 0, type: 'MONSTER' as const, symbol: 'M', has_edges: true },
      { x: 1, y: 0, type: 'EVENT' as const, symbol: '?', has_edges: true },
      { x: 2, y: 0, type: 'MONSTER' as const, symbol: 'M', has_edges: true },
      { x: 0, y: 1, type: 'MONSTER' as const, symbol: 'M', has_edges: true },
      { x: 1, y: 1, type: 'REST' as const, symbol: 'R', has_edges: true },
      { x: 0, y: 2, type: 'ELITE' as const, symbol: 'E', has_edges: true },
    ],
    edges: [
      { src_x: 0, src_y: 0, dst_x: 0, dst_y: 1, is_boss: false },
      { src_x: 0, src_y: 0, dst_x: 1, dst_y: 1, is_boss: false },
      { src_x: 1, src_y: 0, dst_x: 1, dst_y: 1, is_boss: false },
      { src_x: 0, src_y: 1, dst_x: 0, dst_y: 2, is_boss: false },
    ],
    width: 7,
    height: 15,
  };
}

export function createMockSeedData() {
  return {
    seed: 'TESTSEED',
    seed_value: 123456789,
    ascension: 20,
    act: 1,
    neow_options: [
      { slot: 0, type: 'blessing' as const, option: 'UPGRADE_CARD', name: 'Upgrade a Card', drawback: null },
      { slot: 1, type: 'bonus' as const, option: 'HUNDRED_GOLD', name: 'Obtain 100 Gold', drawback: null },
    ],
    map: createMockMapData(),
    encounters: {
      normal: ['Cultist', 'Jaw Worm', 'Two Louse', 'Small Slimes'],
      elite: ['Gremlin Nob', 'Lagavulin', 'Sentries'],
    },
    boss: {
      name: 'The Guardian',
      hp: 240,
      a9_hp: 250,
      move: 'Charging Up',
      details: 'Charges attack, then deals 32 damage',
    },
  };
}

export function createMockDecisionNode(overrides = {}) {
  return {
    id: 'test-node-1',
    type: 'card_play' as const,
    action: 'Play Eruption',
    ev: 0.15,
    winProbability: 0.65,
    children: [],
    isExpanded: false,
    isPruned: false,
    ...overrides,
  };
}

export function createMockPath(overrides = {}) {
  return {
    id: 'path-1',
    name: 'Path A',
    floors: [
      {
        floor: 1,
        hp: 70,
        maxHp: 80,
        gold: 99,
        deckSize: 10,
        relicCount: 1,
        relics: ['Burning Blood'],
        potions: [],
        roomType: 'MONSTER',
        encounter: 'Cultist',
      },
      {
        floor: 2,
        hp: 65,
        maxHp: 80,
        gold: 120,
        deckSize: 11,
        relicCount: 1,
        relics: ['Burning Blood'],
        potions: ['Fire Potion'],
        roomType: 'EVENT',
      },
    ],
    finalEV: 0.72,
    createdAt: new Date('2024-01-01'),
    ...overrides,
  };
}
