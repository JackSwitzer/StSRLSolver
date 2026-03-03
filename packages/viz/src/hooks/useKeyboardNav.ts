import { useEffect, useRef } from 'react';
import type { ScreenMode } from '../types/training';

const MODE_BY_KEY: Record<string, ScreenMode> = {
  '1': 'grid', '2': 'combat', '3': 'map', '4': 'mcts', '5': 'stats',
  F1: 'grid', F2: 'combat', F3: 'map', F4: 'mcts', F5: 'stats',
};

interface KeyboardNavOptions {
  numAgents: number;
  columns: number;
  selectedIndex: number;
  screenMode: ScreenMode;
  onScreenChange: (mode: ScreenMode) => void;
  onAgentChange: (index: number) => void;
  onFocus: () => void;
  onUnfocus: () => void;
  onNextFocused?: () => void;
  onPrevFocused?: () => void;
}

export function useKeyboardNav({
  numAgents, columns, selectedIndex, screenMode,
  onScreenChange, onAgentChange, onFocus, onUnfocus,
  onNextFocused, onPrevFocused,
}: KeyboardNavOptions): void {
  const selRef = useRef(selectedIndex);
  selRef.current = selectedIndex;

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      const tag = (e.target as HTMLElement).tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA') return;

      const key = e.key;

      // Screen mode (always active)
      if (MODE_BY_KEY[key]) {
        e.preventDefault();
        onScreenChange(MODE_BY_KEY[key]);
        return;
      }

      // Escape (always active)
      if (key === 'Escape') {
        e.preventDefault();
        onUnfocus();
        return;
      }

      // Tab to cycle focused agents (in non-grid modes)
      if (key === 'Tab' && screenMode !== 'grid') {
        e.preventDefault();
        if (e.shiftKey) {
          onPrevFocused?.();
        } else {
          onNextFocused?.();
        }
        return;
      }

      // Grid-only: WASD / arrows for agent selection
      if (screenMode === 'grid') {
        const max = Math.max(0, numAgents - 1);
        const cur = selRef.current;

        if (key === 'w' || key === 'W' || key === 'ArrowUp') {
          e.preventDefault();
          onAgentChange(Math.max(0, cur - columns));
          return;
        }
        if (key === 's' || key === 'S' || key === 'ArrowDown') {
          e.preventDefault();
          onAgentChange(Math.min(max, cur + columns));
          return;
        }
        if (key === 'a' || key === 'A' || key === 'ArrowLeft') {
          e.preventDefault();
          onAgentChange(Math.max(0, cur - 1));
          return;
        }
        if (key === 'd' || key === 'D' || key === 'ArrowRight') {
          e.preventDefault();
          onAgentChange(Math.min(max, cur + 1));
          return;
        }
        if (key === 'Enter' || key === 'Return') {
          e.preventDefault();
          onFocus();
          return;
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [numAgents, columns, screenMode, onScreenChange, onAgentChange, onFocus, onUnfocus, onNextFocused, onPrevFocused]);
}
