import { useMemo } from 'react';
import type { AgentEpisodeMsg } from '../types/training';

// ---- Types ----

interface CardAnalysisProps {
  episodes: AgentEpisodeMsg[];
}

interface CardEntry {
  name: string;
  count: number;
  pct: number;
  color: string;
}

interface AnalysisData {
  topPicked: CardEntry[];
  topPlayed: CardEntry[];
  avgDeckSize: number;
  starterRatio: number;
  gamesWithDecks: number;
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

const subsectionLabel: React.CSSProperties = {
  fontSize: 10,
  color: '#484f58',
  textTransform: 'uppercase',
  letterSpacing: '0.3px',
  marginBottom: 8,
};

// ---- Card classification ----

const STARTER_CARDS = new Set([
  'Strike_P', 'Strike_P+', 'Defend_P', 'Defend_P+',
  'Eruption', 'Eruption+', 'Vigilance', 'Vigilance+',
]);

const ATTACK_PATTERNS = [
  'strike', 'bash', 'eruption', 'ragnarok', 'tantrum', 'conclude',
  'searing', 'carnage', 'bludgeon', 'pommel', 'iron wave',
  'flying sleeves', 'flurry', 'wallop', 'windmill', 'wheel kick',
  'crush joints', 'cut through', 'follow up', 'reach heaven',
  'sands of time', 'signature move', 'talk to the hand',
  'weave', 'empty fist', 'bowling bash', 'brilliance',
  'carve reality', 'consecrate', 'just lucky', 'pressure points',
  'sash whip', 'fear no evil',
];

const POWER_PATTERNS = [
  'mental fortress', 'rushdown', 'like water', 'fasting',
  'battle hymn', 'deva form', 'devotion', 'establishment',
  'master reality', 'study', 'nirvana', 'omega',
  'demon form', 'barricade', 'inflame', 'metallicize',
];

function classifyCard(name: string): string {
  const lower = name.toLowerCase().replace(/\+$/, '');
  if (POWER_PATTERNS.some((p) => lower.includes(p))) return 'power';
  if (ATTACK_PATTERNS.some((p) => lower.includes(p))) return 'attack';
  return 'skill';
}

function cardTypeColor(type: string): string {
  switch (type) {
    case 'attack': return '#ff4444';
    case 'skill':  return '#4488ff';
    case 'power':  return '#e3b341';
    default:       return '#c9d1d9';
  }
}

// ---- Computation ----

function computeAnalysis(episodes: AgentEpisodeMsg[]): AnalysisData {
  // Deck composition analysis (from deck_changes / deck_size)
  const pickedCounts: Record<string, number> = {};
  let totalDeckSize = 0;
  let totalStarterCards = 0;
  let totalDeckCards = 0;
  let gamesWithDecks = 0;

  for (const ep of episodes) {
    // Count picked cards from deck_changes
    if (ep.deck_changes && ep.deck_changes.length > 0) {
      for (const card of ep.deck_changes) {
        pickedCounts[card] = (pickedCounts[card] || 0) + 1;
      }
    }

    // Deck size at end of game
    const deckSize = ep.deck_size ?? (ep.deck_changes ? ep.deck_changes.length + 4 : 0);
    if (deckSize > 0) {
      gamesWithDecks++;
      totalDeckSize += deckSize;

      // Estimate starter ratio: base deck is 4 starters + deck_changes
      // Cards in deck_changes that are starters were likely transforms/upgrades
      const deckCards = ep.deck_changes ?? [];
      const nonStarterAdds = deckCards.filter((c) => !STARTER_CARDS.has(c)).length;
      // Base starters = 4, but some might have been removed
      const estimatedStarters = Math.max(0, deckSize - nonStarterAdds - (deckCards.filter((c) => STARTER_CARDS.has(c)).length));
      const actualStarters = Math.min(4, estimatedStarters);
      totalStarterCards += actualStarters;
      totalDeckCards += deckSize;
    }
  }

  // Top picked cards
  const pickedTotal = Object.values(pickedCounts).reduce((s, c) => s + c, 0) || 1;
  const topPicked: CardEntry[] = Object.entries(pickedCounts)
    .map(([name, count]) => ({
      name,
      count,
      pct: count / pickedTotal,
      color: cardTypeColor(classifyCard(name)),
    }))
    .sort((a, b) => b.count - a.count)
    .slice(0, 15);

  // Played cards analysis (from turns_detail in combats)
  const playedCounts: Record<string, number> = {};
  let gamesWithCombats = 0;

  for (const ep of episodes) {
    if (!ep.combats || ep.combats.length === 0) continue;
    gamesWithCombats++;

    for (const combat of ep.combats) {
      const turnsDetail = (combat as any).turns_detail as Array<{ turn: number; cards: string[] }> | undefined;
      if (turnsDetail) {
        for (const turn of turnsDetail) {
          for (const card of turn.cards) {
            playedCounts[card] = (playedCounts[card] || 0) + 1;
          }
        }
      }
    }
  }

  const playedTotal = Object.values(playedCounts).reduce((s, c) => s + c, 0) || 1;
  const topPlayed: CardEntry[] = Object.entries(playedCounts)
    .map(([name, count]) => ({
      name,
      count,
      pct: count / playedTotal,
      color: cardTypeColor(classifyCard(name)),
    }))
    .sort((a, b) => b.count - a.count)
    .slice(0, 15);

  return {
    topPicked,
    topPlayed,
    avgDeckSize: gamesWithDecks > 0 ? totalDeckSize / gamesWithDecks : 0,
    starterRatio: totalDeckCards > 0 ? totalStarterCards / totalDeckCards : 0,
    gamesWithDecks,
    gamesWithCombats,
  };
}

// ---- Sub-components ----

function HorizontalBarChart({
  entries,
  maxItems = 15,
}: {
  entries: CardEntry[];
  maxItems?: number;
}) {
  const items = entries.slice(0, maxItems);
  if (items.length === 0) {
    return <div style={emptyState}>No data</div>;
  }

  const maxCount = items[0].count;
  const BAR_HEIGHT = 20;
  const BAR_GAP = 3;
  const LABEL_W = 130;
  const CHART_W = 160;
  const COUNT_W = 44;
  const totalHeight = items.length * (BAR_HEIGHT + BAR_GAP) - BAR_GAP;

  return (
    <svg
      width={LABEL_W + CHART_W + COUNT_W}
      height={totalHeight}
      style={{ display: 'block' }}
    >
      {items.map((entry, i) => {
        const y = i * (BAR_HEIGHT + BAR_GAP);
        const barW = maxCount > 0 ? (entry.count / maxCount) * CHART_W : 0;

        return (
          <g key={entry.name}>
            {/* Card name */}
            <text
              x={LABEL_W - 6}
              y={y + BAR_HEIGHT / 2 + 4}
              textAnchor="end"
              fill={entry.color}
              fontSize={10}
              fontFamily="'JetBrains Mono', monospace"
              fontWeight={i < 3 ? 700 : 400}
            >
              {entry.name.length > 18
                ? entry.name.slice(0, 17) + '\u2026'
                : entry.name}
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
              fill={entry.color}
              fillOpacity={0.35}
              rx={2}
            />

            {/* Pct inside bar */}
            {barW > 32 && (
              <text
                x={LABEL_W + barW - 4}
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

            {/* Count */}
            <text
              x={LABEL_W + CHART_W + 6}
              y={y + BAR_HEIGHT / 2 + 4}
              textAnchor="start"
              fill="#484f58"
              fontSize={10}
              fontFamily="'JetBrains Mono', monospace"
            >
              {entry.count}
            </text>
          </g>
        );
      })}
    </svg>
  );
}

// ---- Main Component ----

export const CardAnalysis = ({ episodes }: CardAnalysisProps) => {
  const data = useMemo(() => computeAnalysis(episodes), [episodes]);

  if (episodes.length === 0) {
    return (
      <div style={container}>
        <div style={sectionHeader}>Card Analysis</div>
        <div style={emptyState}>No episode data</div>
      </div>
    );
  }

  return (
    <div style={container}>
      <div style={sectionHeader}>Card Analysis</div>

      {/* Summary stats */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 10, marginBottom: 14 }}>
        <div style={statBox}>
          <div style={statLabel}>Avg Deck Size</div>
          <div style={statValue}>
            {data.avgDeckSize > 0 ? data.avgDeckSize.toFixed(1) : '--'}
          </div>
          <div style={subtext}>{data.gamesWithDecks} games</div>
        </div>
        <div style={statBox}>
          <div style={statLabel}>Starter Ratio</div>
          <div style={{
            ...statValue,
            color: data.starterRatio > 0.5 ? '#ff4444' : data.starterRatio > 0.3 ? '#e3b341' : '#00ff41',
          }}>
            {data.starterRatio > 0 ? `${(data.starterRatio * 100).toFixed(0)}%` : '--'}
          </div>
          <div style={subtext}>starter cards remaining</div>
        </div>
      </div>

      {/* Legend */}
      <div style={{
        display: 'flex',
        gap: 14,
        marginBottom: 12,
        fontSize: 10,
      }}>
        <span style={{ color: '#ff4444' }}>Attack</span>
        <span style={{ color: '#4488ff' }}>Skill</span>
        <span style={{ color: '#e3b341' }}>Power</span>
      </div>

      {/* Top picked cards */}
      <div style={{ marginBottom: 16 }}>
        <div style={subsectionLabel}>Top Picked Cards</div>
        <HorizontalBarChart entries={data.topPicked} />
      </div>

      {/* Top played cards */}
      <div>
        <div style={subsectionLabel}>Top Played Cards</div>
        {data.gamesWithCombats > 0 ? (
          <HorizontalBarChart entries={data.topPlayed} />
        ) : (
          <div style={emptyState}>No turn detail data</div>
        )}
      </div>
    </div>
  );
};
