import { useMemo } from 'react';
import type { Episode } from '../types/episode';

export interface EnemyStats {
  name: string;
  fights: number;
  avgHpLost: number;
  avgTurns: number;
  potionRate: number;
  avgSolverMs: number;
}

export interface CardPlayStats {
  card: string;
  playCount: number;
  episodeCount: number;
}

export interface DeathStats {
  enemy: string;
  count: number;
}

export function useEnemyStats(episodes: Episode[] | null): EnemyStats[] {
  return useMemo(() => {
    if (!episodes) return [];
    const map = new Map<string, { fights: number; totalHp: number; totalTurns: number; potions: number; solverMs: number }>();
    for (const ep of episodes) {
      for (const c of ep.combats) {
        const name = c.encounterName || 'Unknown';
        const entry = map.get(name) ?? { fights: 0, totalHp: 0, totalTurns: 0, potions: 0, solverMs: 0 };
        entry.fights++;
        entry.totalHp += c.hpBefore - c.hpAfter;
        entry.totalTurns += c.turns.length;
        entry.potions += c.potionsUsed;
        entry.solverMs += c.solverMs;
        map.set(name, entry);
      }
    }
    return Array.from(map.entries())
      .map(([name, s]) => ({
        name,
        fights: s.fights,
        avgHpLost: s.totalHp / s.fights,
        avgTurns: s.totalTurns / s.fights,
        potionRate: s.potions / s.fights,
        avgSolverMs: s.solverMs / s.fights,
      }))
      .sort((a, b) => b.avgHpLost - a.avgHpLost);
  }, [episodes]);
}

export function useDeathStats(episodes: Episode[] | null): DeathStats[] {
  return useMemo(() => {
    if (!episodes) return [];
    const map = new Map<string, number>();
    for (const ep of episodes) {
      if (!ep.won && ep.deathEnemy) {
        map.set(ep.deathEnemy, (map.get(ep.deathEnemy) ?? 0) + 1);
      }
    }
    return Array.from(map.entries())
      .map(([enemy, count]) => ({ enemy, count }))
      .sort((a, b) => b.count - a.count);
  }, [episodes]);
}

export function useCardPlayStats(episodes: Episode[] | null): CardPlayStats[] {
  return useMemo(() => {
    if (!episodes) return [];
    const map = new Map<string, { plays: number; episodes: Set<string> }>();
    for (const ep of episodes) {
      for (const c of ep.combats) {
        for (const t of c.turns) {
          for (const card of t.cardsPlayed) {
            const entry = map.get(card) ?? { plays: 0, episodes: new Set() };
            entry.plays++;
            entry.episodes.add(ep.seed);
            map.set(card, entry);
          }
        }
      }
    }
    return Array.from(map.entries())
      .map(([card, s]) => ({ card, playCount: s.plays, episodeCount: s.episodes.size }))
      .sort((a, b) => b.playCount - a.playCount);
  }, [episodes]);
}

export function usePathPreferences(episodes: Episode[] | null) {
  return useMemo(() => {
    if (!episodes) return [];
    const counts: Record<string, number> = {};
    let total = 0;
    for (const ep of episodes) {
      for (const p of ep.pathChoices) {
        if (p.options[p.chosen]) {
          const rt = p.options[p.chosen].roomType;
          counts[rt] = (counts[rt] ?? 0) + 1;
          total++;
        }
      }
    }
    return Object.entries(counts)
      .map(([roomType, count]) => ({ roomType, count, pct: total > 0 ? count / total : 0 }))
      .sort((a, b) => b.count - a.count);
  }, [episodes]);
}
