/**
 * Tests for PathDiff component
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '../test-utils';
import { PathDiff } from '../components/PathComparison/PathDiff';
import { createMockPath } from '../test-utils';
import type { Path } from '../components/DecisionTree/types';

describe('PathDiff', () => {
  describe('empty state', () => {
    it('shows empty state when no paths provided', () => {
      render(<PathDiff paths={[]} />);

      expect(screen.getByText('Select paths to compare')).toBeInTheDocument();
    });
  });

  describe('with paths', () => {
    const path1: Path = createMockPath({
      id: 'path-1',
      name: 'Aggressive Path',
      finalEV: 0.72,
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
          hp: 60,
          maxHp: 80,
          gold: 130,
          deckSize: 11,
          relicCount: 1,
          relics: ['Burning Blood'],
          potions: [],
          roomType: 'ELITE',
        },
      ],
    });

    const path2: Path = createMockPath({
      id: 'path-2',
      name: 'Safe Path',
      finalEV: 0.65,
      floors: [
        {
          floor: 1,
          hp: 75,
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
          hp: 75,
          maxHp: 80,
          gold: 100,
          deckSize: 10,
          relicCount: 1,
          relics: ['Burning Blood'],
          potions: [],
          roomType: 'REST',
        },
      ],
    });

    it('renders path headers', () => {
      render(<PathDiff paths={[path1, path2]} />);

      expect(screen.getByText('Aggressive Path')).toBeInTheDocument();
      expect(screen.getByText('Safe Path')).toBeInTheDocument();
    });

    it('renders floor column header', () => {
      render(<PathDiff paths={[path1]} />);

      expect(screen.getByText('Floor')).toBeInTheDocument();
    });

    it('renders floor numbers', () => {
      render(<PathDiff paths={[path1, path2]} />);

      // Floor numbers appear in the floor column
      const floorCells = document.querySelectorAll('.path-diff-cell.floor-col');
      expect(floorCells.length).toBeGreaterThan(0);
      // Check that floor rows exist
      const rows = document.querySelectorAll('.path-diff-row');
      expect(rows.length).toBe(2); // 2 floors
    });

    it('renders room types', () => {
      render(<PathDiff paths={[path1, path2]} />);

      expect(screen.getAllByText('MONSTER').length).toBeGreaterThan(0);
      expect(screen.getByText('ELITE')).toBeInTheDocument();
      expect(screen.getByText('REST')).toBeInTheDocument();
    });

    it('renders HP values', () => {
      render(<PathDiff paths={[path1]} />);

      expect(screen.getByText('70/80')).toBeInTheDocument();
      expect(screen.getByText('60/80')).toBeInTheDocument();
    });

    it('renders gold values', () => {
      render(<PathDiff paths={[path1]} />);

      expect(screen.getByText('99')).toBeInTheDocument();
      expect(screen.getByText('130')).toBeInTheDocument();
    });

    it('renders deck size values', () => {
      render(<PathDiff paths={[path1]} />);

      // Deck sizes appear with labels
      const cells = document.querySelectorAll('.path-stat-value.deck');
      expect(cells.length).toBeGreaterThan(0);
    });

    it('renders relic count values', () => {
      render(<PathDiff paths={[path1]} />);

      const cells = document.querySelectorAll('.path-stat-value.relics');
      expect(cells.length).toBeGreaterThan(0);
    });

    it('renders final EV in summary row', () => {
      render(<PathDiff paths={[path1, path2]} />);

      expect(screen.getByText('Final EV')).toBeInTheDocument();
      expect(screen.getByText('+0.72')).toBeInTheDocument();
      expect(screen.getByText('+0.65')).toBeInTheDocument();
    });

    it('highlights differences when enabled', () => {
      render(<PathDiff paths={[path1, path2]} highlightDifferences={true} />);

      // HP differs between paths on floor 1
      const highlightedCells = document.querySelectorAll('.path-stat-value.highlight');
      expect(highlightedCells.length).toBeGreaterThan(0);
    });

    it('does not highlight differences when disabled', () => {
      const pathA = createMockPath({
        id: 'a',
        name: 'A',
        floors: [{ floor: 1, hp: 70, maxHp: 80, gold: 99, deckSize: 10, relicCount: 1, relics: [], potions: [], roomType: 'MONSTER' }],
      });
      const pathB = createMockPath({
        id: 'b',
        name: 'B',
        floors: [{ floor: 1, hp: 50, maxHp: 80, gold: 99, deckSize: 10, relicCount: 1, relics: [], potions: [], roomType: 'MONSTER' }],
      });

      render(<PathDiff paths={[pathA, pathB]} highlightDifferences={false} />);

      const highlightedCells = document.querySelectorAll('.path-stat-value.highlight');
      expect(highlightedCells.length).toBe(0);
    });

    it('handles paths with different floor counts', () => {
      const shortPath = createMockPath({
        id: 'short',
        name: 'Short',
        floors: [
          { floor: 1, hp: 70, maxHp: 80, gold: 99, deckSize: 10, relicCount: 1, relics: [], potions: [], roomType: 'MONSTER' },
        ],
      });

      const longPath = createMockPath({
        id: 'long',
        name: 'Long',
        floors: [
          { floor: 1, hp: 70, maxHp: 80, gold: 99, deckSize: 10, relicCount: 1, relics: [], potions: [], roomType: 'MONSTER' },
          { floor: 2, hp: 65, maxHp: 80, gold: 120, deckSize: 11, relicCount: 1, relics: [], potions: [], roomType: 'EVENT' },
          { floor: 3, hp: 60, maxHp: 80, gold: 150, deckSize: 12, relicCount: 2, relics: [], potions: [], roomType: 'ELITE' },
        ],
      });

      render(<PathDiff paths={[shortPath, longPath]} />);

      // Should show dashes for missing floors
      const emptyCells = document.querySelectorAll('.path-diff-cell.empty');
      expect(emptyCells.length).toBe(2); // Floor 2 and 3 for shortPath
    });

    it('applies custom className', () => {
      render(<PathDiff paths={[path1]} className="custom-diff" />);

      const container = document.querySelector('.path-diff');
      expect(container).toHaveClass('custom-diff');
    });

    it('renders single path correctly', () => {
      render(<PathDiff paths={[path1]} />);

      expect(screen.getByText('Aggressive Path')).toBeInTheDocument();
      expect(screen.getByText('+0.72')).toBeInTheDocument();
    });

    it('renders multiple paths (3+) correctly', () => {
      const path3: Path = createMockPath({
        id: 'path-3',
        name: 'Middle Path',
        finalEV: 0.68,
      });

      render(<PathDiff paths={[path1, path2, path3]} />);

      expect(screen.getByText('Aggressive Path')).toBeInTheDocument();
      expect(screen.getByText('Safe Path')).toBeInTheDocument();
      expect(screen.getByText('Middle Path')).toBeInTheDocument();
    });
  });

  describe('stat labels', () => {
    const path = createMockPath();

    it('renders HP label', () => {
      render(<PathDiff paths={[path]} />);

      expect(screen.getAllByText('HP').length).toBeGreaterThan(0);
    });

    it('renders Gold label', () => {
      render(<PathDiff paths={[path]} />);

      expect(screen.getAllByText('Gold').length).toBeGreaterThan(0);
    });

    it('renders Deck label', () => {
      render(<PathDiff paths={[path]} />);

      expect(screen.getAllByText('Deck').length).toBeGreaterThan(0);
    });

    it('renders Relics label', () => {
      render(<PathDiff paths={[path]} />);

      expect(screen.getAllByText('Relics').length).toBeGreaterThan(0);
    });
  });

  describe('room type styling', () => {
    it('applies correct class for monster rooms', () => {
      const path = createMockPath({
        floors: [
          { floor: 1, hp: 70, maxHp: 80, gold: 99, deckSize: 10, relicCount: 1, relics: [], potions: [], roomType: 'MONSTER' },
        ],
      });

      render(<PathDiff paths={[path]} />);

      const roomType = document.querySelector('.path-room-type.monster');
      expect(roomType).toBeInTheDocument();
    });

    it('applies correct class for elite rooms', () => {
      const path = createMockPath({
        floors: [
          { floor: 1, hp: 70, maxHp: 80, gold: 99, deckSize: 10, relicCount: 1, relics: [], potions: [], roomType: 'ELITE' },
        ],
      });

      render(<PathDiff paths={[path]} />);

      const roomType = document.querySelector('.path-room-type.elite');
      expect(roomType).toBeInTheDocument();
    });

    it('applies correct class for rest rooms', () => {
      const path = createMockPath({
        floors: [
          { floor: 1, hp: 70, maxHp: 80, gold: 99, deckSize: 10, relicCount: 1, relics: [], potions: [], roomType: 'REST' },
        ],
      });

      render(<PathDiff paths={[path]} />);

      const roomType = document.querySelector('.path-room-type.rest');
      expect(roomType).toBeInTheDocument();
    });
  });
});
