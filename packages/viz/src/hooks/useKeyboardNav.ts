import { useEffect, useRef } from 'react';
import type { ScreenMode } from '../types/training';

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
  onPlayPause?: () => void;
}

export function useKeyboardNav({
  numAgents, columns, selectedIndex, screenMode,
  onScreenChange, onAgentChange, onFocus, onUnfocus,
  onNextFocused, onPrevFocused, onPlayPause,
}: KeyboardNavOptions): void {
  const selRef = useRef(selectedIndex);
  selRef.current = selectedIndex;

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      const tag = (e.target as HTMLElement).tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA') return;

      const key = e.key;

      // Escape: close overlay / unfocus
      if (key === 'Escape') {
        e.preventDefault();
        onUnfocus();
        return;
      }

      // Space: play/pause toggle
      if (key === ' ') {
        e.preventDefault();
        onPlayPause?.();
        return;
      }

      // D = Dashboard (grid overview)
      if (key === 'd' || key === 'D') {
        // Only switch view when NOT in grid mode (where D means arrow-right)
        if (screenMode !== 'grid') {
          e.preventDefault();
          onScreenChange('grid');
          return;
        }
      }

      // F = Combat Feed
      if (key === 'f' || key === 'F') {
        e.preventDefault();
        onScreenChange('feed');
        return;
      }

      // S = Stats view (only when NOT in grid mode where S means arrow-down)
      if ((key === 's' || key === 'S') && screenMode !== 'grid') {
        e.preventDefault();
        onScreenChange('stats_view');
        return;
      }

      // T = Training metrics view
      if (key === 't' || key === 'T') {
        e.preventDefault();
        onScreenChange('training_view');
        return;
      }

      // 1-8: Select agent directly
      const num = parseInt(key, 10);
      if (num >= 1 && num <= 8 && num <= numAgents) {
        e.preventDefault();
        onAgentChange(num - 1);
        return;
      }

      // [ / ]: Prev/Next agent
      if (key === '[') {
        e.preventDefault();
        if (onPrevFocused) onPrevFocused();
        else onAgentChange(Math.max(0, selRef.current - 1));
        return;
      }
      if (key === ']') {
        e.preventDefault();
        if (onNextFocused) onNextFocused();
        else onAgentChange(Math.min(numAgents - 1, selRef.current + 1));
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

      // Legacy F-key bindings
      const fKeyMap: Record<string, ScreenMode> = {
        F1: 'grid', F2: 'combat', F3: 'map', F4: 'mcts', F5: 'stats',
      };
      if (fKeyMap[key]) {
        e.preventDefault();
        onScreenChange(fKeyMap[key]);
        return;
      }
    }

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [numAgents, columns, screenMode, onScreenChange, onAgentChange, onFocus, onUnfocus, onNextFocused, onPrevFocused, onPlayPause]);
}
