import { useMemo } from 'react';
import type { AgentEpisodeMsg } from '../types/training';

// ---- Types ----

interface DeathAnalysisProps {
  episodes: AgentEpisodeMsg[];
  maxItems?: number;
}

interface KillerEntry {
  enemy: string;
  count: number;
  pct: number;
}

// ---- Constants ----

const BAR_HEIGHT = 20;
const BAR_GAP = 4;
const LABEL_WIDTH = 120;
const COUNT_WIDTH = 48;
const ACCENT = '#00ff41';

// ---- Component ----

export const DeathAnalysis = ({ episodes, maxItems = 8 }: DeathAnalysisProps) => {
  const killers = useMemo(() => {
    const counts: Record<string, number> = {};
    let total = 0;

    for (const ep of episodes) {
      if (!ep.won && ep.death_enemy) {
        const enemy = ep.death_enemy;
        counts[enemy] = (counts[enemy] || 0) + 1;
        total++;
      }
    }

    const entries: KillerEntry[] = Object.entries(counts)
      .map(([enemy, count]) => ({ enemy, count, pct: total > 0 ? count / total : 0 }))
      .sort((a, b) => b.count - a.count)
      .slice(0, maxItems);

    return { entries, total };
  }, [episodes, maxItems]);

  const { entries, total } = killers;
  const maxCount = entries.length > 0 ? entries[0].count : 0;

  if (entries.length === 0) {
    return (
      <div style={{
        padding: '16px',
        textAlign: 'center',
        color: '#3d444d',
        fontSize: '11px',
        fontFamily: "'JetBrains Mono', monospace",
      }}>
        No death data
      </div>
    );
  }

  const chartWidth = 300;
  const totalHeight = entries.length * (BAR_HEIGHT + BAR_GAP) - BAR_GAP;

  return (
    <div>
      {/* Header */}
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'baseline',
        marginBottom: '8px',
      }}>
        <span style={{
          fontSize: '9px',
          color: '#8b949e',
          textTransform: 'uppercase',
          letterSpacing: '0.5px',
          fontWeight: 600,
        }}>
          Top Killers
        </span>
        <span style={{
          fontSize: '9px',
          color: '#3d444d',
          fontFamily: "'JetBrains Mono', monospace",
        }}>
          {total} deaths
        </span>
      </div>

      {/* Bar chart */}
      <svg
        width={LABEL_WIDTH + chartWidth + COUNT_WIDTH}
        height={totalHeight}
        style={{ display: 'block' }}
      >
        {entries.map((entry, i) => {
          const y = i * (BAR_HEIGHT + BAR_GAP);
          const barWidth = maxCount > 0 ? (entry.count / maxCount) * chartWidth : 0;
          const isTop = i === 0;

          return (
            <g key={entry.enemy}>
              {/* Enemy name */}
              <text
                x={LABEL_WIDTH - 8}
                y={y + BAR_HEIGHT / 2 + 4}
                textAnchor="end"
                fill={isTop ? '#ff4444' : '#c9d1d9'}
                fontSize={10}
                fontFamily="'JetBrains Mono', monospace"
                fontWeight={isTop ? 700 : 400}
              >
                {entry.enemy.length > 16
                  ? entry.enemy.slice(0, 15) + '...'
                  : entry.enemy}
              </text>

              {/* Bar background */}
              <rect
                x={LABEL_WIDTH}
                y={y}
                width={chartWidth}
                height={BAR_HEIGHT}
                fill="#161b22"
                rx={2}
              />

              {/* Bar fill */}
              <rect
                x={LABEL_WIDTH}
                y={y}
                width={barWidth}
                height={BAR_HEIGHT}
                fill={isTop ? '#ff4444' : ACCENT}
                fillOpacity={isTop ? 0.6 : 0.4}
                rx={2}
              />

              {/* Count */}
              <text
                x={LABEL_WIDTH + chartWidth + 6}
                y={y + BAR_HEIGHT / 2 + 4}
                textAnchor="start"
                fill="#8b949e"
                fontSize={10}
                fontFamily="'JetBrains Mono', monospace"
              >
                {entry.count}
              </text>

              {/* Percentage inside bar (if bar is wide enough) */}
              {barWidth > 40 && (
                <text
                  x={LABEL_WIDTH + barWidth - 4}
                  y={y + BAR_HEIGHT / 2 + 4}
                  textAnchor="end"
                  fill="#0d1117"
                  fontSize={9}
                  fontFamily="'JetBrains Mono', monospace"
                  fontWeight={600}
                >
                  {(entry.pct * 100).toFixed(0)}%
                </text>
              )}
            </g>
          );
        })}
      </svg>
    </div>
  );
};
