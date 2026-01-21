/**
 * Tests for FloorDetails component
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '../test-utils';
import { FloorDetails } from '../components/FloorDetails/FloorDetails';
import { useGameStore } from '../store/gameStore';

// Mock the game store
vi.mock('../store/gameStore', () => ({
  useGameStore: vi.fn(),
}));

describe('FloorDetails', () => {
  const mockClearPath = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();

    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      seed: null,
      seedValue: null,
      ascension: 20,
      encounters: null,
      boss: null,
      currentAct: 1,
      selectedPath: [],
      eventPredictions: {},
      clearPath: mockClearPath,
    });
  });

  it('renders panel title', () => {
    render(<FloorDetails />);

    expect(screen.getByText('Seed Details')).toBeInTheDocument();
  });

  it('shows empty state when no seed is selected', () => {
    render(<FloorDetails />);

    expect(screen.getByText('No Seed Selected')).toBeInTheDocument();
    expect(screen.getByText('Enter a seed to view predictions')).toBeInTheDocument();
  });

  it('displays seed information when seed is loaded', () => {
    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      seed: 'TESTSEED',
      seedValue: 123456789,
      ascension: 20,
      encounters: null,
      boss: null,
      currentAct: 1,
      selectedPath: [],
      eventPredictions: {},
      clearPath: mockClearPath,
    });

    render(<FloorDetails />);

    expect(screen.getByText('TESTSEED')).toBeInTheDocument();
    expect(screen.getByText('123456789')).toBeInTheDocument();
    expect(screen.getByText('Ascension 20')).toBeInTheDocument();
  });

  it('shows path building prompt when path is empty', () => {
    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      seed: 'TESTSEED',
      seedValue: 123456789,
      ascension: 20,
      encounters: null,
      boss: null,
      currentAct: 1,
      selectedPath: [],
      eventPredictions: {},
      clearPath: mockClearPath,
    });

    render(<FloorDetails />);

    expect(screen.getByText('Click nodes to build a path')).toBeInTheDocument();
    expect(screen.getByText('Start from floor 0 and trace your route')).toBeInTheDocument();
  });

  it('displays path with clear button when nodes selected', () => {
    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      seed: 'TESTSEED',
      seedValue: 123456789,
      ascension: 20,
      encounters: { normal: ['Cultist', 'Jaw Worm'], elite: ['Gremlin Nob'] },
      boss: null,
      currentAct: 1,
      selectedPath: [
        { x: 0, y: 0, type: 'MONSTER' },
        { x: 0, y: 1, type: 'EVENT' },
      ],
      eventPredictions: {
        '0,1': { outcome: 'EVENT', roll: 0.5, event_name: 'Big Fish' },
      },
      clearPath: mockClearPath,
    });

    render(<FloorDetails />);

    expect(screen.getByText('Path')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Clear' })).toBeInTheDocument();
  });

  it('calls clearPath when clear button clicked', () => {
    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      seed: 'TESTSEED',
      seedValue: 123456789,
      ascension: 20,
      encounters: { normal: ['Cultist'], elite: [] },
      boss: null,
      currentAct: 1,
      selectedPath: [{ x: 0, y: 0, type: 'MONSTER' }],
      eventPredictions: {},
      clearPath: mockClearPath,
    });

    render(<FloorDetails />);

    const clearButton = screen.getByRole('button', { name: 'Clear' });
    fireEvent.click(clearButton);

    expect(mockClearPath).toHaveBeenCalled();
  });

  it('displays monster encounters from path', () => {
    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      seed: 'TESTSEED',
      seedValue: 123456789,
      ascension: 20,
      encounters: { normal: ['Cultist', 'Jaw Worm'], elite: ['Gremlin Nob'] },
      boss: null,
      currentAct: 1,
      selectedPath: [{ x: 0, y: 0, type: 'MONSTER' }],
      eventPredictions: {},
      clearPath: mockClearPath,
    });

    render(<FloorDetails />);

    // 'Cultist' appears in both path display and encounter list, use getAllByText
    const cultistElements = screen.getAllByText('Cultist');
    expect(cultistElements.length).toBeGreaterThan(0);
  });

  it('displays encounter list', () => {
    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      seed: 'TESTSEED',
      seedValue: 123456789,
      ascension: 20,
      encounters: { normal: ['Cultist', 'Jaw Worm', 'Two Louse'], elite: ['Gremlin Nob', 'Lagavulin'] },
      boss: null,
      currentAct: 1,
      selectedPath: [],
      eventPredictions: {},
      clearPath: mockClearPath,
    });

    render(<FloorDetails />);

    expect(screen.getByText('Encounters')).toBeInTheDocument();
    expect(screen.getByText('Elites')).toBeInTheDocument();
    expect(screen.getByText('Cultist')).toBeInTheDocument();
    expect(screen.getByText('Gremlin Nob')).toBeInTheDocument();
    expect(screen.getByText('Lagavulin')).toBeInTheDocument();
  });

  it('displays boss section', () => {
    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      seed: 'TESTSEED',
      seedValue: 123456789,
      ascension: 20,
      encounters: null,
      boss: {
        name: 'The Guardian',
        hp: 240,
        a9_hp: 250,
        move: 'Charging Up',
        details: 'Charges attack, then deals 32 damage',
      },
      currentAct: 1,
      selectedPath: [],
      eventPredictions: {},
      clearPath: mockClearPath,
    });

    render(<FloorDetails />);

    expect(screen.getByText('Act 1 Boss')).toBeInTheDocument();
    expect(screen.getByText('The Guardian')).toBeInTheDocument();
    expect(screen.getByText('HP: 250')).toBeInTheDocument();
    expect(screen.getByText('Charging Up')).toBeInTheDocument();
    expect(screen.getByText('Charges attack, then deals 32 damage')).toBeInTheDocument();
  });

  it('handles event predictions with different outcomes', () => {
    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      seed: 'TESTSEED',
      seedValue: 123456789,
      ascension: 20,
      encounters: { normal: ['Cultist'], elite: [] },
      boss: null,
      currentAct: 1,
      selectedPath: [
        { x: 0, y: 0, type: 'EVENT' },
        { x: 1, y: 1, type: 'EVENT' },
      ],
      eventPredictions: {
        '0,0': { outcome: 'SHOP', roll: 0.3 },
        '1,1': { outcome: 'TREASURE', roll: 0.7 },
      },
      clearPath: mockClearPath,
    });

    render(<FloorDetails />);

    expect(screen.getByText('Shop')).toBeInTheDocument();
    expect(screen.getByText('Treasure')).toBeInTheDocument();
  });

  it('shows unknown for event without prediction', () => {
    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      seed: 'TESTSEED',
      seedValue: 123456789,
      ascension: 20,
      encounters: { normal: [], elite: [] },
      boss: null,
      currentAct: 1,
      selectedPath: [{ x: 0, y: 0, type: 'EVENT' }],
      eventPredictions: {},
      clearPath: mockClearPath,
    });

    render(<FloorDetails />);

    expect(screen.getByText('? Unknown')).toBeInTheDocument();
  });

  it('displays rest site correctly', () => {
    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      seed: 'TESTSEED',
      seedValue: 123456789,
      ascension: 20,
      encounters: { normal: [], elite: [] },
      boss: null,
      currentAct: 1,
      selectedPath: [{ x: 0, y: 0, type: 'REST' }],
      eventPredictions: {},
      clearPath: mockClearPath,
    });

    render(<FloorDetails />);

    expect(screen.getByText('Rest Site')).toBeInTheDocument();
  });

  it('displays shop correctly', () => {
    (useGameStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      seed: 'TESTSEED',
      seedValue: 123456789,
      ascension: 20,
      encounters: { normal: [], elite: [] },
      boss: null,
      currentAct: 1,
      selectedPath: [{ x: 0, y: 0, type: 'SHOP' }],
      eventPredictions: {},
      clearPath: mockClearPath,
    });

    render(<FloorDetails />);

    // First "Shop" appears in path, second may appear elsewhere
    expect(screen.getAllByText('Shop').length).toBeGreaterThan(0);
  });
});
