import { useMemo } from 'react';
import { Sparkline } from './Sparkline';
import type {
  AgentInfo,
  AgentEpisodeMsg,
  TrainingStatsMsg,
  SystemStatsMsg,
  DeathStats,
} from '../types/training';

// ---- Props ----

interface StatsOverviewProps {
  agents: AgentInfo[];
  episodes: AgentEpisodeMsg[];
  stats: TrainingStatsMsg | null;
  systemStats: SystemStatsMsg | null;
  deathStats: DeathStats;
  floorHistory: number[];
  winHistory: number[];
}

// ---- Helpers ----

function hpColor(ratio: number): string {
  if (ratio > 0.6) return '#00ff41';
  if (ratio > 0.3) return '#ffb700';
  return '#ff4444';
}

const STANCE_COLORS: Record<string, string> = {
  Neutral: '#8b949e',
  Calm: '#4488ff',
  Wrath: '#ff4444',
  Divinity: '#ffb700',
};

function floorColor(floor: number, maxFloor: number): string {
  if (maxFloor <= 0) return '#ff4444';
  const ratio = floor / maxFloor;
  if (ratio > 0.7) return '#00ff41';
  if (ratio > 0.4) return '#ffb700';
  return '#ff4444';
}

function fmtDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

// ---- Sub-components ----

const SectionHeader = ({ children, right }: { children: React.ReactNode; right?: React.ReactNode }) => (
  <div style={{
    fontSize: '9px',
    color: '#8b949e',
    textTransform: 'uppercase',
    letterSpacing: '0.8px',
    marginBottom: '6px',
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    fontWeight: 600,
  }}>
    <span>{children}</span>
    {right && <span>{right}</span>}
  </div>
);

const HBar = ({ label, value, maxValue, color, labelWidth = 50 }: {
  label: string;
  value: number;
  maxValue: number;
  color: string;
  labelWidth?: number;
}) => {
  const pct = maxValue > 0 ? (value / maxValue) * 100 : 0;
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: '10px', height: '14px' }}>
      <span style={{
        width: `${labelWidth}px`,
        textAlign: 'right',
        color: '#8b949e',
        flexShrink: 0,
        overflow: 'hidden',
        textOverflow: 'ellipsis',
        whiteSpace: 'nowrap',
      }}>
        {label}
      </span>
      <div style={{ flex: 1, background: '#21262d', height: '8px', position: 'relative', overflow: 'hidden' }}>
        <div style={{
          width: `${pct}%`,
          height: '100%',
          background: color,
          opacity: 0.8,
          transition: 'width 0.3s ease',
          minWidth: value > 0 ? '2px' : '0',
        }} />
      </div>
      <span style={{ width: '24px', textAlign: 'right', color: '#c9d1d9', flexShrink: 0 }}>
        {value}
      </span>
    </div>
  );
};

const ResourceBar = ({ label, value, max, unit, color }: {
  label: string;
  value: number;
  max?: number;
  unit?: string;
  color: string;
}) => {
  const pct = max && max > 0 ? (value / max) * 100 : value;
  return (
    <div style={{ marginBottom: '4px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '9px', marginBottom: '2px' }}>
        <span style={{ color: '#8b949e' }}>{label}</span>
        <span style={{ color: '#c9d1d9' }}>
          {max ? `${value.toFixed(1)}/${max.toFixed(1)} ${unit ?? ''}` : `${typeof value === 'number' && !Number.isInteger(value) ? value.toFixed(1) : value}${unit ? ` ${unit}` : '%'}`}
        </span>
      </div>
      <div style={{ height: '6px', background: '#21262d', overflow: 'hidden' }}>
        <div style={{
          width: `${Math.min(pct, 100)}%`,
          height: '100%',
          background: color,
          opacity: 0.8,
          transition: 'width 0.4s linear',
        }} />
      </div>
    </div>
  );
};

const EmptyState = ({ text }: { text: string }) => (
  <div style={{ color: '#3d444d', fontSize: '10px', textAlign: 'center', padding: '12px 4px' }}>
    {text}
  </div>
);

// ---- Main Component ----

export const StatsOverviewPanel = ({
  agents,
  episodes,
  stats,
  systemStats,
  deathStats,
  floorHistory,
  winHistory,
}: StatsOverviewProps) => {


  // ---- Computed: Agent Leaderboard ----
  const leaderboard = useMemo(() => {
    return [...agents]
      .map((a) => {
        const aa = a as any;
        return {
          ...a,
          hpRatio: a.max_hp > 0 ? a.hp / a.max_hp : 0,
          maxFloor: a.floor,
          deckSize: aa.deck_size ?? null,
          relicCount: aa.relic_count ?? null,
          potionCount: aa.potion_count ?? null,
          potionMax: aa.potion_max ?? null,
          gold: aa.gold ?? null,
          stance: aa.stance ?? 'Neutral',
        };
      })
      .sort((a, b) => {
        if (b.maxFloor !== a.maxFloor) return b.maxFloor - a.maxFloor;
        return b.hpRatio - a.hpRatio;
      });
  }, [agents]);

  // ---- Computed: Floor Distribution ----
  const floorDist = useMemo(() => {
    const counts: Record<number, number> = {};
    let maxCount = 0;
    let highestFloor = 0;
    for (const ep of episodes) {
      const f = ep.floors_reached;
      if (f <= 0) continue; // Skip construction failures
      counts[f] = (counts[f] || 0) + 1;
      if (counts[f] > maxCount) maxCount = counts[f];
      if (f > highestFloor) highestFloor = f;
    }
    const entries: { floor: number; count: number }[] = [];
    for (let f = 1; f <= Math.max(highestFloor, 20); f++) {
      if (counts[f]) {
        entries.push({ floor: f, count: counts[f] });
      }
    }
    return { entries, maxCount, highestFloor };
  }, [episodes]);

  // ---- Computed: Rolling Win Rate ----
  const rollingWinRate = useMemo(() => {
    if (winHistory.length === 0) return { data: [], current: 0 };
    const windowSize = 20;
    const data: number[] = [];
    for (let i = 0; i < winHistory.length; i++) {
      const start = Math.max(0, i - windowSize + 1);
      const window = winHistory.slice(start, i + 1);
      const rate = window.reduce((s, v) => s + v, 0) / window.length;
      data.push(rate);
    }
    const current = data.length > 0 ? data[data.length - 1] : 0;
    return { data, current };
  }, [winHistory]);

  // ---- Computed: Rolling Avg Floor (last 50) ----
  const rollingAvgFloor = useMemo(() => {
    if (floorHistory.length === 0) return { data: [], current: 0 };
    const windowSize = 50;
    const data: number[] = [];
    for (let i = 0; i < floorHistory.length; i++) {
      const start = Math.max(0, i - windowSize + 1);
      const window = floorHistory.slice(start, i + 1);
      const avg = window.reduce((s, v) => s + v, 0) / window.length;
      data.push(avg);
    }
    const current = data.length > 0 ? data[data.length - 1] : 0;
    return { data, current };
  }, [floorHistory]);

  // ---- Computed: Avg HP Lost Per Combat ----
  const avgHpLost = useMemo(() => {
    let totalLost = 0;
    let combatCount = 0;
    for (const ep of episodes) {
      if (ep.combats && ep.combats.length > 0) {
        for (const c of ep.combats) {
          totalLost += c.hp_lost;
          combatCount++;
        }
      }
    }
    if (combatCount === 0) return null;
    return totalLost / combatCount;
  }, [episodes]);

  // ---- Computed: HP Lost Trend (per-episode avg) ----
  const hpLostTrend = useMemo(() => {
    const values: number[] = [];
    for (const ep of episodes) {
      if (ep.combats && ep.combats.length > 0) {
        const avg = ep.combats.reduce((s, c) => s + c.hp_lost, 0) / ep.combats.length;
        values.push(avg);
      }
    }
    if (values.length < 2) return null;
    const recent = values.slice(-5).reduce((s, v) => s + v, 0) / Math.min(5, values.length);
    const older = values.slice(-10, -5);
    const olderAvg = older.length > 0 ? older.reduce((s, v) => s + v, 0) / older.length : recent;
    return recent < olderAvg ? 'down' : recent > olderAvg ? 'up' : 'flat';
  }, [episodes]);

  // ---- Computed: Death Analysis (top 5 enemies) ----
  const topKillers = useMemo(() => {
    const entries = Object.entries(deathStats.byEnemy)
      .map(([enemy, count]) => ({ enemy, count }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 5);
    const maxCount = entries.length > 0 ? entries[0].count : 0;
    return { entries, maxCount };
  }, [deathStats.byEnemy]);

  // ---- Computed: Death Floor Heatmap ----
  const deathFloors = useMemo(() => {
    const entries = Object.entries(deathStats.byFloor)
      .map(([f, count]) => ({ floor: Number(f), count }))
      .sort((a, b) => a.floor - b.floor);
    const maxCount = entries.reduce((m, e) => Math.max(m, e.count), 0);
    const deadliestFloor = entries.reduce<{ floor: number; count: number } | null>(
      (best, e) => (!best || e.count > best.count ? e : best),
      null,
    );
    return { entries, maxCount, deadliestFloor };
  }, [deathStats.byFloor]);

  // ---- Computed: Popular Card Picks ----
  const popularCards = useMemo(() => {
    const counts: Record<string, number> = {};
    let hasData = false;
    for (const ep of episodes) {
      if (ep.deck_changes && ep.deck_changes.length > 0) {
        hasData = true;
        for (const change of ep.deck_changes) {
          if (change.startsWith('+')) {
            const card = change.slice(1);
            counts[card] = (counts[card] || 0) + 1;
          }
        }
      }
    }
    if (!hasData) return null;
    const entries = Object.entries(counts)
      .map(([card, count]) => ({ card, count }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 8);
    const maxCount = entries.length > 0 ? entries[0].count : 0;
    return { entries, maxCount };
  }, [episodes]);

  // ---- Computed: Stance Distribution ----
  const stanceDist = useMemo(() => {
    const totals: Record<string, number> = {};
    let combatCount = 0;
    for (const ep of episodes) {
      if (ep.combats) {
        for (const c of ep.combats) {
          combatCount++;
          if (c.stances) {
            for (const [stance, count] of Object.entries(c.stances)) {
              totals[stance] = (totals[stance] || 0) + (count as number);
            }
          }
        }
      }
    }
    const entries = Object.entries(totals)
      .map(([stance, count]) => ({ stance, count }))
      .sort((a, b) => b.count - a.count);
    const total = entries.reduce((s, e) => s + e.count, 0);
    return { entries, total, combatCount };
  }, [episodes]);

  // ---- Computed: Summary Stats ----
  const summaryStats = useMemo(() => {
    const totalEp = stats?.total_episodes ?? episodes.length;
    const winCount = stats?.win_count ?? episodes.filter((e) => e.won).length;
    const avgFloor = stats?.avg_floor ?? (episodes.length > 0
      ? episodes.reduce((s, e) => s + e.floors_reached, 0) / episodes.length
      : 0);
    const maxFloor = episodes.length > 0
      ? Math.max(...episodes.map((e) => e.floors_reached))
      : agents.reduce((m, a) => Math.max(m, a.floor), 0);
    return { totalEp, winCount, avgFloor, maxFloor };
  }, [stats, episodes, agents]);

  return (
    <div style={{
      display: 'grid',
      gridTemplateColumns: '1fr 1fr 1fr',
      gap: '0',
      flex: 1,
      overflow: 'hidden',
      background: '#0d1117',
      fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
      fontSize: '10px',
      color: '#c9d1d9',
    }}>

      {/* ====== COLUMN 1: Agent Rankings + System ====== */}
      <div style={{
        borderRight: '1px solid #21262d',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}>

        {/* Agent Leaderboard */}
        <div style={{ flex: 1, padding: '8px', overflow: 'auto', borderBottom: '1px solid #21262d' }}>
          <SectionHeader
            right={<span style={{ color: '#00ff41' }}>{summaryStats.totalEp} ep</span>}
          >
            Agent Leaderboard
          </SectionHeader>

          {/* Summary row */}
          <div style={{
            display: 'flex',
            gap: '12px',
            marginBottom: '6px',
            fontSize: '9px',
            color: '#8b949e',
          }}>
            <span>W: <span style={{ color: '#00ff41' }}>{summaryStats.winCount}</span></span>
            <span>Avg: <span style={{ color: '#c9d1d9' }}>{summaryStats.avgFloor.toFixed(1)}</span></span>
            <span>Max: <span style={{ color: '#ffb700' }}>{summaryStats.maxFloor}</span></span>
            {stats?.eps_per_min != null && (
              <span>Rate: <span style={{ color: '#c9d1d9' }}>{stats.eps_per_min.toFixed(1)}/m</span></span>
            )}
          </div>

          {/* Table header */}
          <div style={{
            display: 'grid',
            gridTemplateColumns: '16px 1fr 24px 42px 24px 24px 24px 28px 36px',
            gap: '2px',
            fontSize: '8px',
            color: '#3d444d',
            textTransform: 'uppercase',
            letterSpacing: '0.3px',
            marginBottom: '2px',
            paddingBottom: '2px',
            borderBottom: '1px solid #161b22',
          }}>
            <span>#</span>
            <span>Name</span>
            <span style={{ textAlign: 'right' }}>Flr</span>
            <span style={{ textAlign: 'right' }}>HP</span>
            <span style={{ textAlign: 'right' }}>Dk</span>
            <span style={{ textAlign: 'right' }}>Rel</span>
            <span style={{ textAlign: 'right' }}>Pot</span>
            <span style={{ textAlign: 'right' }}>Gold</span>
            <span style={{ textAlign: 'center' }}>Stance</span>
          </div>

          {/* Agent rows */}
          {leaderboard.map((a, i) => {
            const rank = i + 1;
            const isFirst = rank === 1;
            const isDead = a.hp <= 0 || a.status === 'dead';
            return (
              <div
                key={a.id}
                style={{
                  display: 'grid',
                  gridTemplateColumns: '16px 1fr 24px 42px 24px 24px 24px 28px 36px',
                  gap: '2px',
                  fontSize: '10px',
                  height: '16px',
                  alignItems: 'center',
                  opacity: isDead ? 0.4 : 1,
                  background: isFirst ? 'rgba(0,255,65,0.05)' : 'transparent',
                }}
              >
                <span style={{ color: isFirst ? '#00ff41' : '#3d444d', fontWeight: isFirst ? 700 : 400 }}>
                  {rank}
                </span>
                <span style={{
                  color: isFirst ? '#00ff41' : '#c9d1d9',
                  fontWeight: isFirst ? 600 : 400,
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: 'nowrap',
                }}>
                  {a.name}
                </span>
                <span style={{ textAlign: 'right', color: floorColor(a.floor, 55) }}>
                  {Math.floor(a.floor)}
                </span>
                <span style={{ textAlign: 'right' }}>
                  <span style={{ color: hpColor(a.hpRatio) }}>{a.hp}</span>
                  <span style={{ color: '#3d444d' }}>/{a.max_hp}</span>
                </span>
                <span style={{ textAlign: 'right', color: '#4488ff' }}>
                  {a.deckSize ?? '-'}
                </span>
                <span style={{ textAlign: 'right', color: '#ffb700' }}>
                  {a.relicCount ?? '-'}
                </span>
                <span style={{ textAlign: 'right', color: '#ff44ff' }}>
                  {a.potionCount ?? '-'}
                </span>
                <span style={{ textAlign: 'right', color: '#ffb700' }}>
                  {a.gold ?? '-'}
                </span>
                <span style={{ textAlign: 'center', color: STANCE_COLORS[a.stance] ?? '#8b949e', fontSize: '8px' }}>
                  {a.stance === 'Neutral' ? '-' : a.stance?.slice(0, 3)}
                </span>
              </div>
            );
          })}

          {agents.length === 0 && <EmptyState text="No agents running" />}
        </div>

        {/* System Resources */}
        <div style={{ padding: '8px', flexShrink: 0 }}>
          <SectionHeader
            right={stats?.uptime != null ? (
              <span style={{ color: '#3d444d' }}>{fmtDuration(stats.uptime)}</span>
            ) : undefined}
          >
            System
          </SectionHeader>

          {systemStats ? (
            <>
              <ResourceBar label="CPU" value={systemStats.cpu_pct} color="#4488ff" />
              <ResourceBar
                label="RAM"
                value={systemStats.ram_used_gb}
                max={systemStats.ram_total_gb}
                unit="GB"
                color="#ffb700"
              />
              <ResourceBar
                label="GPU (MPS)"
                value={systemStats.gpu_mem_allocated_gb ?? 0}
                max={systemStats.gpu_mem_used_gb && systemStats.gpu_mem_used_gb > 0 ? systemStats.gpu_mem_used_gb : undefined}
                unit={systemStats.gpu_mem_allocated_gb ? 'GB' : undefined}
                color={systemStats.gpu_mem_allocated_gb > 0 ? '#00ff41' : '#8b949e'}
              />
              <div style={{
                display: 'flex',
                justifyContent: 'space-between',
                fontSize: '9px',
                color: '#8b949e',
                marginTop: '4px',
              }}>
                <span>Workers: <span style={{ color: '#c9d1d9' }}>{systemStats.workers}</span></span>
                <span>MCTS: <span style={{ color: '#c9d1d9' }}>
                  {stats?.mcts_avg_ms != null ? `${stats.mcts_avg_ms.toFixed(0)}ms` : '-'}
                </span></span>
              </div>
            </>
          ) : (
            <EmptyState text="Waiting for system stats..." />
          )}
        </div>
      </div>

      {/* ====== COLUMN 2: Charts ====== */}
      <div style={{
        borderRight: '1px solid #21262d',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}>

        {/* Floor Distribution */}
        <div style={{ flex: 1, padding: '8px', overflow: 'auto', borderBottom: '1px solid #21262d' }}>
          <SectionHeader
            right={episodes.length > 0 ? (
              <span style={{ color: '#3d444d' }}>{episodes.length} runs</span>
            ) : undefined}
          >
            Floor Distribution
          </SectionHeader>

          {/* Floor trend sparkline */}
          {floorHistory.length > 1 && (
            <div style={{ marginBottom: '6px' }}>
              <Sparkline
                data={floorHistory}
                width={280}
                height={28}
                color="#ffb700"
                fill={true}
                label="Avg Floor Trend"
              />
            </div>
          )}

          {floorDist.entries.length > 0 ? (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '1px' }}>
              {floorDist.entries.map(({ floor, count }) => (
                <HBar
                  key={floor}
                  label={`F${floor}`}
                  value={count}
                  maxValue={floorDist.maxCount}
                  color={floorColor(floor, floorDist.highestFloor)}
                  labelWidth={28}
                />
              ))}
            </div>
          ) : (
            <EmptyState text="No episode data yet" />
          )}
        </div>

        {/* Rolling Avg Floor (more useful than WR when WR is 0) */}
        <div style={{ padding: '8px', borderBottom: '1px solid #21262d', flexShrink: 0 }}>
          <SectionHeader
            right={
              <span style={{ color: '#ffb700', fontSize: '11px', fontWeight: 700 }}>
                {rollingAvgFloor.current.toFixed(1)}
              </span>
            }
          >
            Avg Floor (rolling 50)
          </SectionHeader>

          {rollingAvgFloor.data.length > 1 ? (
            <Sparkline
              data={rollingAvgFloor.data}
              width={280}
              height={36}
              color="#ffb700"
              fill={true}
              label="Floor"
            />
          ) : (
            <EmptyState text="Need 2+ episodes for trend" />
          )}

          {/* Show win rate below when it's non-zero */}
          {rollingWinRate.current > 0 && (
            <div style={{ marginTop: '4px', fontSize: '9px', color: '#8b949e' }}>
              Win Rate (rolling 20): <span style={{ color: '#00ff41' }}>{(rollingWinRate.current * 100).toFixed(1)}%</span>
            </div>
          )}
        </div>

        {/* Damage Per Fight */}
        <div style={{ padding: '8px', flexShrink: 0 }}>
          <SectionHeader>Avg HP Lost / Combat</SectionHeader>

          {avgHpLost !== null ? (
            <div style={{ display: 'flex', alignItems: 'baseline', gap: '6px' }}>
              <span style={{ fontSize: '18px', fontWeight: 700, color: '#ff4444' }}>
                {avgHpLost.toFixed(1)}
              </span>
              <span style={{ fontSize: '10px', color: '#8b949e' }}>HP</span>
              {hpLostTrend && (
                <span style={{
                  fontSize: '12px',
                  color: hpLostTrend === 'down' ? '#00ff41' : hpLostTrend === 'up' ? '#ff4444' : '#8b949e',
                }}>
                  {hpLostTrend === 'down' ? '\u2193' : hpLostTrend === 'up' ? '\u2191' : '\u2192'}
                </span>
              )}
            </div>
          ) : (
            <EmptyState text="Collecting combat data..." />
          )}
        </div>

        {/* Stance Distribution */}
        <div style={{ padding: '8px', flexShrink: 0 }}>
          <SectionHeader
            right={stanceDist.combatCount > 0 ? (
              <span style={{ color: '#3d444d' }}>{stanceDist.combatCount} combats</span>
            ) : undefined}
          >
            Stance Usage
          </SectionHeader>

          {stanceDist.entries.length > 0 ? (
            <>
              {/* Proportion bar */}
              <div style={{ display: 'flex', height: '10px', overflow: 'hidden', marginBottom: '4px', background: '#21262d' }}>
                {stanceDist.entries.map(({ stance, count }) => (
                  <div
                    key={stance}
                    style={{
                      width: `${stanceDist.total > 0 ? (count / stanceDist.total) * 100 : 0}%`,
                      height: '100%',
                      background: STANCE_COLORS[stance] ?? '#8b949e',
                      opacity: 0.8,
                    }}
                  />
                ))}
              </div>
              {/* Legend */}
              <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap', fontSize: '9px' }}>
                {stanceDist.entries.map(({ stance, count }) => (
                  <span key={stance} style={{ color: STANCE_COLORS[stance] ?? '#8b949e' }}>
                    {stance} {stanceDist.total > 0 ? `${((count / stanceDist.total) * 100).toFixed(0)}%` : '0%'}
                  </span>
                ))}
              </div>
            </>
          ) : (
            <EmptyState text="No stance data yet" />
          )}
        </div>
      </div>

      {/* ====== COLUMN 3: Analytics ====== */}
      <div style={{
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}>

        {/* Death Analysis: Top Killers */}
        <div style={{ padding: '8px', borderBottom: '1px solid #21262d', flexShrink: 0 }}>
          <SectionHeader
            right={deathStats.totalDeaths > 0 ? (
              <span style={{ color: '#ff4444' }}>{deathStats.totalDeaths} deaths</span>
            ) : undefined}
          >
            Top Killers
          </SectionHeader>

          {topKillers.entries.length > 0 ? (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '1px' }}>
              {topKillers.entries.map(({ enemy, count }) => (
                <HBar
                  key={enemy}
                  label={enemy.length > 12 ? enemy.slice(0, 11) + '\u2026' : enemy}
                  value={count}
                  maxValue={topKillers.maxCount}
                  color="#ff4444"
                  labelWidth={80}
                />
              ))}
            </div>
          ) : (
            <EmptyState text="No deaths recorded" />
          )}
        </div>

        {/* Death Floor Heatmap */}
        <div style={{ flex: 1, padding: '8px', overflow: 'auto', borderBottom: '1px solid #21262d' }}>
          <SectionHeader
            right={deathFloors.deadliestFloor ? (
              <span style={{ color: '#ff4444' }}>
                Peak: F{deathFloors.deadliestFloor.floor}
              </span>
            ) : undefined}
          >
            Death Floor Map
          </SectionHeader>

          {deathFloors.entries.length > 0 ? (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '1px' }}>
              {deathFloors.entries.map(({ floor, count }) => {
                const isDeadliest = deathFloors.deadliestFloor?.floor === floor;
                return (
                  <div key={floor} style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '4px',
                    fontSize: '10px',
                    height: '14px',
                  }}>
                    <span style={{
                      width: '28px',
                      textAlign: 'right',
                      color: isDeadliest ? '#ff4444' : '#8b949e',
                      fontWeight: isDeadliest ? 700 : 400,
                      flexShrink: 0,
                    }}>
                      F{floor}
                    </span>
                    <div style={{
                      flex: 1,
                      background: '#21262d',
                      height: '8px',
                      position: 'relative',
                      overflow: 'hidden',
                    }}>
                      <div style={{
                        width: `${deathFloors.maxCount > 0 ? (count / deathFloors.maxCount) * 100 : 0}%`,
                        height: '100%',
                        background: isDeadliest
                          ? 'linear-gradient(90deg, #cc2222, #ff4444)'
                          : 'rgba(255,68,68,0.5)',
                        transition: 'width 0.3s ease',
                        minWidth: count > 0 ? '2px' : '0',
                      }} />
                    </div>
                    <span style={{
                      width: '24px',
                      textAlign: 'right',
                      color: isDeadliest ? '#ff4444' : '#c9d1d9',
                      fontWeight: isDeadliest ? 700 : 400,
                      flexShrink: 0,
                    }}>
                      {count}
                    </span>
                  </div>
                );
              })}
            </div>
          ) : (
            <EmptyState text="No deaths recorded" />
          )}
        </div>

        {/* Popular Card Picks */}
        <div style={{ padding: '8px', flexShrink: 0 }}>
          <SectionHeader>Popular Card Picks</SectionHeader>

          {popularCards ? (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '1px' }}>
              {popularCards.entries.map(({ card, count }, i) => (
                <div key={card} style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '4px',
                  fontSize: '10px',
                  height: '14px',
                }}>
                  <span style={{
                    width: '12px',
                    textAlign: 'right',
                    color: i === 0 ? '#00ff41' : '#3d444d',
                    fontSize: '8px',
                    flexShrink: 0,
                  }}>
                    {i + 1}
                  </span>
                  <span style={{
                    width: '80px',
                    color: i === 0 ? '#00ff41' : '#c9d1d9',
                    fontWeight: i === 0 ? 600 : 400,
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: 'nowrap',
                    flexShrink: 0,
                  }}>
                    {card}
                  </span>
                  <div style={{
                    flex: 1,
                    background: '#21262d',
                    height: '8px',
                    overflow: 'hidden',
                  }}>
                    <div style={{
                      width: `${popularCards.maxCount > 0 ? (count / popularCards.maxCount) * 100 : 0}%`,
                      height: '100%',
                      background: i === 0 ? '#00ff41' : '#4488ff',
                      opacity: 0.7,
                      transition: 'width 0.3s ease',
                      minWidth: '2px',
                    }} />
                  </div>
                  <span style={{
                    width: '24px',
                    textAlign: 'right',
                    color: '#8b949e',
                    flexShrink: 0,
                  }}>
                    {count}
                  </span>
                </div>
              ))}
            </div>
          ) : (
            <EmptyState text="Collecting deck data..." />
          )}
        </div>
      </div>
    </div>
  );
};
