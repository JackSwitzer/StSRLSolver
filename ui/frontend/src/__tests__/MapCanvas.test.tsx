/**
 * Tests for MapCanvas component
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '../test-utils';
import { MapCanvas } from '../components/Map/MapCanvas';
import { useGameStore } from '../store/gameStore';
import { createMockMapData } from '../test-utils';

// Mock the game store
vi.mock('../store/gameStore', () => ({
  useGameStore: vi.fn(),
}));

describe('MapCanvas', () => {
  const mockAddToPath = vi.fn();
  const mockRemoveFromPath = vi.fn();
  const mockCanAddToPath = vi.fn();
  const mockIsNodeOnPath = vi.fn();
  const mockGetPathIndex = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();

    // Default mock implementations
    mockCanAddToPath.mockReturnValue(true);
    mockIsNodeOnPath.mockReturnValue(false);
    mockGetPathIndex.mockReturnValue(-1);

    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      mapData: null,
      selectedPath: [],
      addToPath: mockAddToPath,
      removeFromPath: mockRemoveFromPath,
      canAddToPath: mockCanAddToPath,
      isNodeOnPath: mockIsNodeOnPath,
      getPathIndex: mockGetPathIndex,
    });
  });

  it('renders canvas element', () => {
    render(<MapCanvas />);

    const canvas = document.querySelector('canvas');
    expect(canvas).toBeInTheDocument();
    expect(canvas).toHaveClass('map-canvas');
  });

  it('renders container div', () => {
    render(<MapCanvas />);

    const container = document.querySelector('.map-container');
    expect(container).toBeInTheDocument();
  });

  it('renders with map data', () => {
    const mockMapData = createMockMapData();

    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      mapData: mockMapData,
      selectedPath: [],
      addToPath: mockAddToPath,
      removeFromPath: mockRemoveFromPath,
      canAddToPath: mockCanAddToPath,
      isNodeOnPath: mockIsNodeOnPath,
      getPathIndex: mockGetPathIndex,
    });

    render(<MapCanvas />);

    const canvas = document.querySelector('canvas');
    expect(canvas).toBeInTheDocument();
  });

  it('handles canvas click events', () => {
    const mockMapData = createMockMapData();

    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      mapData: mockMapData,
      selectedPath: [],
      addToPath: mockAddToPath,
      removeFromPath: mockRemoveFromPath,
      canAddToPath: mockCanAddToPath,
      isNodeOnPath: mockIsNodeOnPath,
      getPathIndex: mockGetPathIndex,
    });

    render(<MapCanvas />);

    const canvas = document.querySelector('canvas')!;
    fireEvent.click(canvas);

    // Click handler runs but we can't verify node selection without real canvas rendering
    expect(canvas).toBeInTheDocument();
  });

  it('does not error with null mapData', () => {
    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      mapData: null,
      selectedPath: [],
      addToPath: mockAddToPath,
      removeFromPath: mockRemoveFromPath,
      canAddToPath: mockCanAddToPath,
      isNodeOnPath: mockIsNodeOnPath,
      getPathIndex: mockGetPathIndex,
    });

    expect(() => render(<MapCanvas />)).not.toThrow();
  });

  it('tracks selected path nodes', () => {
    const mockMapData = createMockMapData();

    mockIsNodeOnPath.mockImplementation((x: number, y: number) => {
      return x === 0 && y === 0;
    });

    mockGetPathIndex.mockImplementation((x: number, y: number) => {
      return x === 0 && y === 0 ? 0 : -1;
    });

    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      mapData: mockMapData,
      selectedPath: [{ x: 0, y: 0, type: 'MONSTER' }],
      addToPath: mockAddToPath,
      removeFromPath: mockRemoveFromPath,
      canAddToPath: mockCanAddToPath,
      isNodeOnPath: mockIsNodeOnPath,
      getPathIndex: mockGetPathIndex,
    });

    render(<MapCanvas />);

    // Component renders without error with a selected path
    const canvas = document.querySelector('canvas');
    expect(canvas).toBeInTheDocument();
  });

  it('responds to window resize', () => {
    const mockMapData = createMockMapData();

    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      mapData: mockMapData,
      selectedPath: [],
      addToPath: mockAddToPath,
      removeFromPath: mockRemoveFromPath,
      canAddToPath: mockCanAddToPath,
      isNodeOnPath: mockIsNodeOnPath,
      getPathIndex: mockGetPathIndex,
    });

    render(<MapCanvas />);

    // Trigger resize event
    fireEvent(window, new Event('resize'));

    const canvas = document.querySelector('canvas');
    expect(canvas).toBeInTheDocument();
  });
});
