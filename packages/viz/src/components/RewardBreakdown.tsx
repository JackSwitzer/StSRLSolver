import { useMemo } from 'react';
import type { AgentEpisodeMsg } from '../types/training';

// ---- Types ----

interface RewardBreakdownProps {
  episodes: AgentEpisodeMsg[];
}

interface FloorBucket {
  label: string;
  floor: number;
  count: number;
  pct: number;
}

interface BreakdownStats {
  floorDist: FloorBucket[];
  avgHpLost: number;
  potionRate: number;
  avgCombats: number;
  avgTurns: number;
  avgCardsPlayed: number;
  avgStanceChanges: number;
  totalGames: number;
  gamesWithCombats: number;
}

// ---- Styles ----

const container: React.CSSProperties = {
  background: '#161b22',
  border: '1px solid #30363d',
  borderRadius: 6,
  padding: 14,
  fontFamily: "'JetBrains Mono', monospace",
  fontSize: 12,
  color: '#c9d1d9',
};

const sectionHeader: React.CSSProperties = {
  fontSize: 11,
  color: '#8b949e',
  textTransform: 'uppercase',
  letterSpacing: '0.5px',
  marginBottom: 12,
};

const grid: React.CSSProperties = {
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fill, minmax(130px, 1fr))',
  gap: 10,
};

const statBox: React.CSSProperties = {
  background: '#0d1117',
  border: '1px solid #21262d',
  borderRadius: 4,
  padding: '8px 10px',
};

const statLabel: React.CSSProperties = {
  fontSize: 10,
  color: '#484f58',
  textTransform: 'uppercase',
  letterSpacing: '0.3px',
  marginBottom: 4,
};

const statValue: React.CSSProperties = {
  fontSize: 16,
  fontWeight: 700,
  fontVariantNumeric: 'tabular-nums',
  color: '#c9d1d9',
};

const subtext: React.CSSProperties = {
  fontSize: 10,
  color: '#484f58',
  marginTop: 2,
};

const emptyState: React.CSSProperties = {
  padding: '20px 0',
  color: '#484f58',
  textAlign: 'center',
  fontSize: 12,
};

// ---- Helpers ----

const FLOOR_THRESHOLDS = [
  { label: 'F5+', floor: 5 },
  { label: 'F10+', floor: 10 },
  { label: 'F16+', floor: 16 },
  { label: 'F17+', floor: 17 },
  { label: 'F34+', floor: 34 },
  { label: 'F51+', floor: 51 },
  { label: 'F55+', floor: 55 },
];

function computeBreakdown(episodes: AgentEpisodeMsg[]): BreakdownStats {
  const n = episodes.length;
  if (n === 0) {
    return {
      floorDist: FLOOR_THRESHOLDS.map((t) => ({ ...t, count: 0, pct: 0 })),
      avgHpLost: 0,
      potionRate: 0,
      avgCombats: 0,
      avgTurns: 0,
      avgCardsPlayed: 0,
      avgStanceChanges: 0,
      totalGames: 0,
      gamesWithCombats: 0,
    };
  }

  // Floor distribution
  const floorDist: FloorBucket[] = FLOOR_THRESHOLDS.map((t) => {
    const count = episodes.filter((ep) => ep.floors_reached >= t.floor).length;
    return { ...t, count, pct: count / n };
  });

  // Combat-level stats
  let totalHpLost = 0;
  let totalCombats = 0;
  let totalTurns = 0;
  let totalCardsPlayed = 0;
  let totalStanceChanges = 0;
  let gamesWithPotions = 0;
  let gamesWithCombats = 0;

  for (const ep of episodes) {
    if (!ep.combats || ep.combats.length === 0) continue;
    gamesWithCombats++;

    let gamePotionUsed = false;
    for (const c of ep.combats) {
      totalHpLost += c.hp_lost ?? 0;
      totalCombats++;
      totalTurns += c.turns ?? 0;
      // cards_played comes from the raw Python data, falls back to 0
      totalCardsPlayed += (c as any).cards_played ?? 0;

      if (c.used_potion || ((c as any).potions_used ?? 0) > 0) {
        gamePotionUsed = true;
      }

      // Stance changes: either explicit field or sum of stances record
      const sc = (c as any).stance_changes;
      if (typeof sc === 'number') {
        totalStanceChanges += sc;
      } else if (c.stances) {
        for (const count of Object.values(c.stances)) {
          totalStanceChanges += count;
        }
      }
    }
    if (gamePotionUsed) gamesWithPotions++;
  }

  return {
    floorDist,
    avgHpLost: gamesWithCombats > 0 ? totalHpLost / gamesWithCombats : 0,
    potionRate: n > 0 ? gamesWithPotions / n : 0,
    avgCombats: n > 0 ? totalCombats / n : 0,
    avgTurns: totalCombats > 0 ? totalTurns / totalCombats : 0,
    avgCardsPlayed: gamesWithCombats > 0 ? totalCardsPlayed / gamesWithCombats : 0,
    avgStanceChanges: gamesWithCombats > 0 ? totalStanceChanges / gamesWithCombats : 0,
    totalGames: n,
    gamesWithCombats,
  };
}

// ---- Sub-components ----

function StatCard({
  label,
  value,
  sub,
  accent,
}: {
  label: string;
  value: string;
  sub?: string;
  accent?: boolean;
}) {
  return (
    <div style={statBox}>
      <div style={statLabel}>{label}</div>
      <div style={{ ...statValue, color: accent ? '#00ff41' : '#c9d1d9' }}>{value}</div>
      {sub && <div style={subtext}>{sub}</div>}
    </div>
  );
}

function FloorDistChart({ buckets, totalGames }: { buckets: FloorBucket[]; totalGames: number }) {
  const maxCount = Math.max(...buckets.map((b) => b.count), 1);
  const BAR_HEIGHT = 22;
  const BAR_GAP = 4;
  const LABEL_W = 44;
  const COUNT_W = 56;
  const CHART_W = 220;
  const totalHeight = buckets.length * (BAR_HEIGHT + BAR_GAP) - BAR_GAP;

  return (
    <svg
      width={LABEL_W + CHART_W + COUNT_W}
      height={totalHeight}
      style={{ display: 'block' }}
    >
      {buckets.map((b, i) => {
        const y = i * (BAR_HEIGHT + BAR_GAP);
        const barW = maxCount > 0 ? (b.count / maxCount) * CHART_W : 0;
        const isHighlight = b.floor >= 34;

        return (
          <g key={b.label}>
            {/* Label */}
            <text
              x={LABEL_W - 6}
              y={y + BAR_HEIGHT / 2 + 4}
              textAnchor="end"
              fill={isHighlight ? '#00ff41' : '#8b949e'}
              fontSize={10}
              fontFamily="'JetBrains Mono', monospace"
              fontWeight={isHighlight ? 700 : 400}
            >
              {b.label}
            </text>

            {/* Bar bg */}
            <rect
              x={LABEL_W}
              y={y}
              width={CHART_W}
              height={BAR_HEIGHT}
              fill="#0d1117"
              rx={2}
            />

            {/* Bar fill */}
            <rect
              x={LABEL_W}
              y={y}
              width={barW}
              height={BAR_HEIGHT}
              fill={isHighlight ? '#00ff41' : '#238636'}
              fillOpacity={isHighlight ? 0.5 : 0.4}
              rx={2}
            />

            {/* Pct inside bar */}
            {barW > 36 && (
              <text
                x={LABEL_W + barW - 4}
                y={y + BAR_HEIGHT / 2 + 4}
                textAnchor="end"
                fill="#0d1117"
                fontSize={9}
                fontFamily="'JetBrains Mono', monospace"
                fontWeight={600}
              >
                {(b.pct * 100).toFixed(0)}%
              </text>
            )}

            {/* Count */}
            <text
              x={LABEL_W + CHART_W + 6}
              y={y + BAR_HEIGHT / 2 + 4}
              textAnchor="start"
              fill="#484f58"
              fontSize={10}
              fontFamily="'JetBrains Mono', monospace"
            >
              {b.count}/{totalGames}
            </text>
          </g>
        );
      })}
    </svg>
  );
}

// ---- Main Component ----

export const RewardBreakdown = ({ episodes }: RewardBreakdownProps) => {
  const data = useMemo(() => computeBreakdown(episodes), [episodes]);

  if (episodes.length === 0) {
    return (
      <div style={container}>
        <div style={sectionHeader}>Reward Breakdown</div>
        <div style={emptyState}>No episode data</div>
      </div>
    );
  }

  return (
    <div style={container}>
      <div style={sectionHeader}>Reward Breakdown</div>

      {/* Stat cards grid */}
      <div style={grid}>
        <StatCard
          label="Avg HP Lost"
          value={data.avgHpLost.toFixed(1)}
          sub="per game"
        />
        <StatCard
          label="Potion Use"
          value={`${(data.potionRate * 100).toFixed(0)}%`}
          sub={`${Math.round(data.potionRate * data.totalGames)}/${data.totalGames} games`}
          accent={data.potionRate >= 0.3}
        />
        <StatCard
          label="Combats/Game"
          value={data.avgCombats.toFixed(1)}
          sub={`${data.gamesWithCombats} w/ data`}
        />
        <StatCard
          label="Turns/Combat"
          value={data.avgTurns.toFixed(1)}
        />
        <StatCard
          label="Cards/Game"
          value={data.avgCardsPlayed.toFixed(1)}
          sub="played"
        />
        <StatCard
          label="Stances/Game"
          value={data.avgStanceChanges.toFixed(1)}
          accent={data.avgStanceChanges >= 5}
        />
      </div>

      {/* Floor milestone distribution */}
      <div style={{ marginTop: 14 }}>
        <div style={{ ...statLabel, marginBottom: 8, fontSize: 10 }}>
          Floor Milestones
        </div>
        <FloorDistChart buckets={data.floorDist} totalGames={data.totalGames} />
      </div>
    </div>
  );
};
