import { useMemo } from 'react';
import type { AgentEpisodeMsg } from '../types/training';

// ---- Types ----

interface EfficiencyPanelProps {
  episodes: AgentEpisodeMsg[];
  stats?: {
    eps_per_min?: number;
    [key: string]: any;
  } | null;
}

interface MetricEntry {
  label: string;
  value: string;
  sub?: string;
  color: string;
}

// ---- Component ----

export const EfficiencyPanel = ({ episodes, stats }: EfficiencyPanelProps) => {
  const metrics = useMemo((): MetricEntry[] => {
    const n = episodes.length;
    if (n === 0) {
      return [
        { label: 'Games/min', value: '--', color: '#3d444d' },
        { label: 'Useful%', value: '--', sub: 'F16+ signal', color: '#3d444d' },
        { label: 'Avg steps', value: '--', sub: 'per game', color: '#3d444d' },
        { label: 'Avg stances', value: '--', sub: 'per game', color: '#3d444d' },
        { label: 'Avg combats', value: '--', sub: 'per game', color: '#3d444d' },
      ];
    }

    // Games per minute
    let gamesPerMin: number | null = null;
    if (stats?.eps_per_min != null && stats.eps_per_min > 0) {
      gamesPerMin = stats.eps_per_min;
    } else if (n >= 2) {
      const totalDuration = episodes.reduce((sum, ep) => sum + (ep.duration || 0), 0);
      if (totalDuration > 0) gamesPerMin = (n / totalDuration) * 60;
    }

    // Useful% (reach floor 16+)
    const useful = episodes.filter((ep) => ep.floors_reached >= 16).length;
    const usefulPct = n > 0 ? (useful / n) * 100 : 0;

    // Avg transitions/game
    const totalSteps = episodes.reduce((sum, ep) => sum + (ep.total_steps || 0), 0);
    const avgSteps = n > 0 ? totalSteps / n : 0;

    // Avg stance changes/game (from combat summaries)
    let totalStanceChanges = 0;
    let gamesWithCombats = 0;
    for (const ep of episodes) {
      if (ep.combats && ep.combats.length > 0) {
        gamesWithCombats++;
        for (const combat of ep.combats) {
          if (combat.stances) {
            // Sum all stance entries (each value = times entered that stance)
            for (const count of Object.values(combat.stances)) {
              totalStanceChanges += count;
            }
          }
        }
      }
    }
    const avgStances = gamesWithCombats > 0 ? totalStanceChanges / gamesWithCombats : 0;

    // Avg combats/game
    const totalCombats = episodes.reduce(
      (sum, ep) => sum + (ep.combats?.length || 0),
      0,
    );
    const avgCombats = n > 0 ? totalCombats / n : 0;

    return [
      {
        label: 'Games/min',
        value: gamesPerMin != null ? gamesPerMin.toFixed(1) : '--',
        color: gamesPerMin != null && gamesPerMin >= 5 ? '#00ff41' : '#c9d1d9',
      },
      {
        label: 'Useful%',
        value: `${usefulPct.toFixed(1)}%`,
        sub: `${useful}/${n} reach F16`,
        color: usefulPct >= 20 ? '#00ff41' : usefulPct >= 5 ? '#ffb700' : '#ff4444',
      },
      {
        label: 'Avg steps',
        value: avgSteps > 0 ? avgSteps.toFixed(0) : '--',
        sub: 'per game',
        color: '#c9d1d9',
      },
      {
        label: 'Avg stances',
        value: avgStances > 0 ? avgStances.toFixed(1) : '--',
        sub: 'per game',
        color: avgStances >= 5 ? '#00ff41' : '#c9d1d9',
      },
      {
        label: 'Avg combats',
        value: avgCombats > 0 ? avgCombats.toFixed(1) : '--',
        sub: 'per game',
        color: '#c9d1d9',
      },
    ];
  }, [episodes, stats]);

  return (
    <div>
      {/* Header */}
      <div style={{
        fontSize: '9px',
        color: '#8b949e',
        textTransform: 'uppercase',
        letterSpacing: '0.5px',
        fontWeight: 600,
        marginBottom: '8px',
      }}>
        Efficiency
      </div>

      {/* 2-column stat grid */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: '1fr 1fr',
        gap: '8px 16px',
      }}>
        {metrics.map((m) => (
          <div key={m.label} style={{
            padding: '6px 8px',
            background: '#161b22',
            border: '1px solid #21262d',
          }}>
            <div style={{
              fontSize: '8px',
              color: '#8b949e',
              textTransform: 'uppercase',
              letterSpacing: '0.5px',
              marginBottom: '2px',
            }}>
              {m.label}
            </div>
            <div style={{
              fontSize: '14px',
              fontWeight: 700,
              color: m.color,
              fontFamily: "'JetBrains Mono', monospace",
              lineHeight: '18px',
            }}>
              {m.value}
            </div>
            {m.sub && (
              <div style={{
                fontSize: '8px',
                color: '#3d444d',
                marginTop: '1px',
              }}>
                {m.sub}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};
