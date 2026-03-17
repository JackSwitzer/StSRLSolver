import { useMemo } from 'react';
import type { AgentEpisodeMsg, CombatSummary } from '../types/training';

interface PerformancePanelProps {
  episodes: AgentEpisodeMsg[];
}

const container: React.CSSProperties = {
  background: '#161b22',
  border: '1px solid #30363d',
  borderRadius: 6,
  padding: 14,
  fontFamily: 'inherit',
  fontSize: 12,
  color: '#c9d1d9',
};

const header: React.CSSProperties = {
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

interface RoomStats {
  type: string;
  count: number;
  avgTurns: number;
  avgHpLost: number;
  avgDamageDealt: number;
  potionRate: number;
}

function classifyRoom(enemy: string): string {
  const bossNames = [
    'SlimeBoss', 'Hexaghost', 'TheGuardian',
    'BronzeAutomaton', 'TheChamp', 'TheCollector',
    'AwakenedOne', 'DonuAndDeca', 'TimeEater',
    'CorruptHeart', 'ShieldAndSpear',
  ];
  const eliteNames = [
    'Lagavulin', 'GremlinNob', 'Sentries',
    'BookOfStabbing', 'GremlinLeader', 'Taskmaster',
    'Nemesis', 'Reptomancer', 'GiantHead',
  ];
  if (bossNames.some((b) => enemy.includes(b))) return 'Boss';
  if (eliteNames.some((e) => enemy.includes(e))) return 'Elite';
  return 'Monster';
}

function computeStats(episodes: AgentEpisodeMsg[]) {
  const allCombats: CombatSummary[] = [];
  let totalDuration = 0;
  let episodesWithDuration = 0;

  for (const ep of episodes) {
    if (ep.duration > 0) {
      totalDuration += ep.duration;
      episodesWithDuration++;
    }
    if (ep.combats) {
      for (const c of ep.combats) {
        allCombats.push(c);
      }
    }
  }

  const avgGameDuration = episodesWithDuration > 0 ? totalDuration / episodesWithDuration : 0;
  const avgFightsPerGame =
    episodes.length > 0
      ? episodes.reduce((sum, ep) => sum + (ep.combats?.length ?? 0), 0) / episodes.length
      : 0;

  // Group combats by room type
  const byType: Record<string, CombatSummary[]> = {};
  for (const c of allCombats) {
    const type = classifyRoom(c.enemy);
    if (!byType[type]) byType[type] = [];
    byType[type].push(c);
  }

  const roomStats: RoomStats[] = [];
  for (const [type, combats] of Object.entries(byType)) {
    const n = combats.length;
    roomStats.push({
      type,
      count: n,
      avgTurns: combats.reduce((s, c) => s + c.turns, 0) / n,
      avgHpLost: combats.reduce((s, c) => s + c.hp_lost, 0) / n,
      avgDamageDealt: combats.reduce((s, c) => s + c.damage_dealt, 0) / n,
      potionRate: combats.filter((c) => c.used_potion).length / n,
    });
  }

  // Sort: Boss, Elite, Monster
  const order = ['Boss', 'Elite', 'Monster'];
  roomStats.sort((a, b) => order.indexOf(a.type) - order.indexOf(b.type));

  return { avgGameDuration, avgFightsPerGame, roomStats, totalCombats: allCombats.length };
}

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

function formatSeconds(sec: number): string {
  if (sec < 60) return `${sec.toFixed(1)}s`;
  const m = Math.floor(sec / 60);
  const s = Math.floor(sec % 60);
  return `${m}m ${s.toString().padStart(2, '0')}s`;
}

function roomTypeColor(type: string): string {
  switch (type) {
    case 'Boss':
      return '#f85149';
    case 'Elite':
      return '#d2a038';
    case 'Monster':
      return '#8b949e';
    default:
      return '#8b949e';
  }
}

export const PerformancePanel = ({ episodes }: PerformancePanelProps) => {
  const data = useMemo(() => computeStats(episodes), [episodes]);

  if (episodes.length === 0) {
    return (
      <div style={container}>
        <div style={header}>Performance</div>
        <div style={emptyState}>No data</div>
      </div>
    );
  }

  return (
    <div style={container}>
      <div style={header}>Performance</div>

      {/* Top-level stats */}
      <div style={grid}>
        <StatCard
          label="Avg Game"
          value={formatSeconds(data.avgGameDuration)}
          sub={`${episodes.length} episodes`}
        />
        <StatCard
          label="Fights/Game"
          value={data.avgFightsPerGame.toFixed(1)}
          sub={`${data.totalCombats} total`}
        />
      </div>

      {/* Combat breakdown by room type */}
      {data.roomStats.length > 0 && (
        <div style={{ marginTop: 14 }}>
          <div style={{ ...statLabel, marginBottom: 8, fontSize: 10 }}>Combat by Type</div>
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: '70px 50px 55px 60px 50px',
              gap: '0 8px',
              fontSize: 11,
              fontVariantNumeric: 'tabular-nums',
            }}
          >
            {/* Column headers */}
            <span style={{ color: '#484f58', fontSize: 10 }}>TYPE</span>
            <span style={{ color: '#484f58', fontSize: 10, textAlign: 'right' }}>TURNS</span>
            <span style={{ color: '#484f58', fontSize: 10, textAlign: 'right' }}>HP LOST</span>
            <span style={{ color: '#484f58', fontSize: 10, textAlign: 'right' }}>DMG</span>
            <span style={{ color: '#484f58', fontSize: 10, textAlign: 'right' }}>POT%</span>

            {data.roomStats.map((rs) => (
              <div key={rs.type} style={{ display: 'contents' }}>
                <span style={{ color: roomTypeColor(rs.type), fontWeight: 600 }}>
                  {rs.type}
                  <span style={{ color: '#484f58', fontWeight: 400, marginLeft: 4 }}>
                    ({rs.count})
                  </span>
                </span>
                <span style={{ textAlign: 'right', color: '#c9d1d9' }}>
                  {rs.avgTurns.toFixed(1)}
                </span>
                <span style={{ textAlign: 'right', color: '#f85149' }}>
                  {rs.avgHpLost.toFixed(1)}
                </span>
                <span style={{ textAlign: 'right', color: '#c9d1d9' }}>
                  {rs.avgDamageDealt.toFixed(0)}
                </span>
                <span style={{ textAlign: 'right', color: '#8b949e' }}>
                  {(rs.potionRate * 100).toFixed(0)}%
                </span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
