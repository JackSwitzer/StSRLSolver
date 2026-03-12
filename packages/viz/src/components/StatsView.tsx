import { useMemo } from 'react';
import type { AgentEpisodeMsg } from '../types/training';

interface Props {
  episodes: AgentEpisodeMsg[];
}

function Sparkline({ values, color = '#60a5fa' }: { values: number[]; color?: string }) {
  if (values.length < 2) return null;
  const max = Math.max(...values, 1);
  const min = Math.min(...values, 0);
  const range = max - min || 1;
  const w = 200;
  const h = 40;
  const points = values.map((v, i) =>
    `${(i / (values.length - 1)) * w},${h - ((v - min) / range) * h}`
  ).join(' ');

  return (
    <svg width={w} height={h} className="inline-block">
      <polyline points={points} fill="none" stroke={color} strokeWidth="1.5" />
    </svg>
  );
}

export function StatsView({ episodes }: Props) {
  const recentEpisodes = useMemo(() => episodes.slice(-200), [episodes]);

  const bestRuns = useMemo(() =>
    [...recentEpisodes].sort((a, b) => b.floors_reached - a.floors_reached).slice(0, 20),
    [recentEpisodes]
  );

  const floorTrajectory = useMemo(() =>
    recentEpisodes.slice(-50).map(e => e.floors_reached),
    [recentEpisodes]
  );

  const hpTrajectory = useMemo(() =>
    recentEpisodes.slice(-50).map(e => e.hp_remaining),
    [recentEpisodes]
  );

  const avgFloor = useMemo(() => {
    if (recentEpisodes.length === 0) return 0;
    return recentEpisodes.reduce((s, e) => s + e.floors_reached, 0) / recentEpisodes.length;
  }, [recentEpisodes]);

  // Per-fight stats from combat summaries
  const fightStats = useMemo(() => {
    const byEnemy: Record<string, { totalHp: number; count: number; totalTurns: number }> = {};
    for (const ep of recentEpisodes) {
      for (const c of (ep.combats ?? [])) {
        const key = c.enemy || 'unknown';
        if (!byEnemy[key]) byEnemy[key] = { totalHp: 0, count: 0, totalTurns: 0 };
        byEnemy[key].totalHp += c.hp_lost;
        byEnemy[key].count += 1;
        byEnemy[key].totalTurns += c.turns;
      }
    }
    return Object.entries(byEnemy)
      .map(([enemy, { totalHp, count, totalTurns }]) => ({
        enemy,
        avgHpLost: Math.round(totalHp / count),
        avgTurns: (totalTurns / count).toFixed(1),
        count,
      }))
      .sort((a, b) => b.avgHpLost - a.avgHpLost)
      .slice(0, 15);
  }, [recentEpisodes]);

  return (
    <div className="h-full overflow-y-auto bg-gray-900 text-gray-100 p-4">
      <h2 className="text-lg font-bold mb-4">
        Stats ({recentEpisodes.length} episodes, avg floor {avgFloor.toFixed(1)})
      </h2>

      <div className="grid grid-cols-2 gap-6 mb-6">
        <div>
          <h3 className="text-sm text-gray-400 mb-1">Floor Reached (last 50)</h3>
          <Sparkline values={floorTrajectory} color="#34d399" />
        </div>
        <div>
          <h3 className="text-sm text-gray-400 mb-1">Final HP (last 50)</h3>
          <Sparkline values={hpTrajectory} color="#f87171" />
        </div>
      </div>

      {fightStats.length > 0 && (
        <div className="mb-6">
          <h3 className="text-sm text-gray-400 mb-2">Avg HP Lost by Enemy</h3>
          <div className="space-y-1">
            {fightStats.map(({ enemy, avgHpLost, avgTurns, count }) => (
              <div key={enemy} className="flex items-center gap-2 text-sm font-mono">
                <span className="w-32 text-gray-300 truncate">{enemy}</span>
                <div className="flex-1 bg-gray-800 rounded h-4 overflow-hidden">
                  <div
                    className="h-full bg-red-500/60 rounded"
                    style={{ width: `${Math.min(100, (avgHpLost / 30) * 100)}%` }}
                  />
                </div>
                <span className="w-14 text-right text-gray-400">{avgHpLost} HP</span>
                <span className="w-12 text-right text-gray-500">{avgTurns}t</span>
                <span className="w-10 text-right text-gray-600">({count})</span>
              </div>
            ))}
          </div>
        </div>
      )}

      <div>
        <h3 className="text-sm text-gray-400 mb-2">Best Runs (by floor)</h3>
        <table className="w-full text-sm font-mono">
          <thead>
            <tr className="text-gray-500 border-b border-gray-700">
              <th className="text-left py-1">Agent</th>
              <th className="text-left">Seed</th>
              <th className="text-right">Floor</th>
              <th className="text-right">HP</th>
              <th className="text-right">Won</th>
            </tr>
          </thead>
          <tbody>
            {bestRuns.map((r, i) => (
              <tr key={i} className="border-b border-gray-800">
                <td className="py-1 text-gray-400">#{r.agent_id}</td>
                <td className="text-gray-300">{r.seed}</td>
                <td className="text-right">{r.floors_reached}</td>
                <td className="text-right">{r.hp_remaining}</td>
                <td className="text-right">{r.won ? 'W' : '-'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
