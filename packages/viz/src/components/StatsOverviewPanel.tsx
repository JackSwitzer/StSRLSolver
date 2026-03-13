import { useMemo, useState } from 'react';
import { Sparkline } from './Sparkline';
import type { SparklineMarker } from './Sparkline';
import { WorkerGrid } from './WorkerGrid';
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
  trainStepMarkers: { index: number; step: number }[];
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

const HBar = ({ label, value, maxValue, color, labelWidth = 50, pctLabel }: {
  label: string;
  value: number;
  maxValue: number;
  color: string;
  labelWidth?: number;
  pctLabel?: string;
}) => {
  const pct = maxValue > 0 ? (value / maxValue) * 100 : 0;
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: '10px', height: '16px' }}>
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
      <div style={{ flex: 1, background: '#21262d', height: '10px', position: 'relative', overflow: 'hidden' }}>
        <div style={{
          width: `${pct}%`,
          height: '100%',
          background: color,
          opacity: 0.8,
          transition: 'width 0.3s ease',
          minWidth: value > 0 ? '2px' : '0',
        }} />
      </div>
      <span style={{ width: pctLabel ? '46px' : '28px', textAlign: 'right', color: '#c9d1d9', flexShrink: 0, fontSize: '9px' }}>
        {pctLabel ?? value}
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

/** Big stat card with prominent number */
const BigStat = ({ label, value, sub, color = '#c9d1d9', small }: {
  label: string;
  value: string;
  sub?: string;
  color?: string;
  small?: boolean;
}) => (
  <div style={{
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    padding: small ? '6px 8px' : '8px 10px',
    background: '#161b22',
    border: '1px solid #21262d',
    gap: '2px',
    minWidth: 0,
    flex: 1,
  }}>
    <span style={{
      fontSize: '8px',
      color: '#8b949e',
      textTransform: 'uppercase',
      letterSpacing: '0.5px',
      whiteSpace: 'nowrap',
    }}>{label}</span>
    <span style={{
      fontSize: small ? '16px' : '20px',
      fontWeight: 700,
      color,
      fontFamily: "'JetBrains Mono', monospace",
      lineHeight: 1.1,
    }}>{value}</span>
    {sub && <span style={{ fontSize: '8px', color: '#8b949e', whiteSpace: 'nowrap' }}>{sub}</span>}
  </div>
);

/** Floor sparkline with Y-axis labels, larger size */
const FloorChart = ({ data, markers, current, peak }: {
  data: number[];
  markers: SparklineMarker[];
  current: number;
  peak: number;
}) => {
  if (data.length < 2) return <EmptyState text="Need 2+ episodes for trend" />;
  const minVal = Math.min(...data);
  const maxVal = Math.max(...data);
  return (
    <div style={{ position: 'relative' }}>
      {/* Axis labels */}
      <div style={{
        position: 'absolute',
        left: 0,
        top: 0,
        bottom: 0,
        width: '28px',
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'space-between',
        padding: '14px 0 14px 0',
        pointerEvents: 'none',
        zIndex: 1,
      }}>
        <span style={{ fontSize: '8px', color: '#8b949e', fontFamily: 'monospace' }}>
          {maxVal.toFixed(0)}
        </span>
        <span style={{ fontSize: '8px', color: '#8b949e', fontFamily: 'monospace' }}>
          {((maxVal + minVal) / 2).toFixed(0)}
        </span>
        <span style={{ fontSize: '8px', color: '#8b949e', fontFamily: 'monospace' }}>
          {minVal.toFixed(0)}
        </span>
      </div>
      <div style={{ marginLeft: '30px' }}>
        <Sparkline
          data={data}
          width={320}
          height={130}
          color="#ffb700"
          fill={true}
          markers={markers}
        />
      </div>
      {/* Callout */}
      <div style={{
        position: 'absolute',
        top: '4px',
        right: '4px',
        background: 'rgba(13,17,23,0.9)',
        border: '1px solid #30363d',
        padding: '3px 8px',
        fontSize: '10px',
        fontFamily: "'JetBrains Mono', monospace",
        zIndex: 2,
      }}>
        <span style={{ color: '#ffb700', fontWeight: 700 }}>{current.toFixed(1)}</span>
        <span style={{ color: '#3d444d', margin: '0 4px' }}>|</span>
        <span style={{ color: '#8b949e', fontSize: '9px' }}>peak </span>
        <span style={{ color: '#c9d1d9' }}>{peak.toFixed(1)}</span>
      </div>
    </div>
  );
};

// ---- Main Component ----

export const StatsOverviewPanel = ({
  agents,
  episodes,
  stats,
  systemStats,
  deathStats,
  floorHistory,
  winHistory,
  trainStepMarkers,
}: StatsOverviewProps) => {

  // ---- Scope selector state ----
  const [scope, setScope] = useState<'overview' | 'act1' | 'act2' | 'act3' | 'rewards'>('overview');

  const floorRange = useMemo(() => {
    switch (scope) {
      case 'act1': return [1, 17] as const;
      case 'act2': return [18, 34] as const;
      case 'act3': return [35, 55] as const;
      default: return [0, 999] as const;
    }
  }, [scope]);

  const filteredEpisodes = useMemo(() => {
    if (scope === 'overview' || scope === 'rewards') return episodes;
    return episodes.filter(ep => ep.floors_reached >= floorRange[0] && ep.floors_reached <= floorRange[1]);
  }, [episodes, scope, floorRange]);

  // ---- Computed: Rolling Avg Floor (last 50) ----
  const rollingAvgFloor = useMemo(() => {
    if (floorHistory.length === 0) return { data: [], current: 0, peak: 0 };
    const windowSize = 50;
    const data: number[] = [];
    let peak = 0;
    for (let i = 0; i < floorHistory.length; i++) {
      const start = Math.max(0, i - windowSize + 1);
      const window = floorHistory.slice(start, i + 1);
      const avg = window.reduce((s, v) => s + v, 0) / window.length;
      data.push(avg);
      if (avg > peak) peak = avg;
    }
    const current = data.length > 0 ? data[data.length - 1] : 0;
    return { data, current, peak };
  }, [floorHistory]);

  // ---- Computed: F16+ Rate (rolling) ----
  const f16Rate = useMemo(() => {
    if (floorHistory.length === 0) return { data: [], current: 0 };
    const windowSize = 100;
    const data: number[] = [];
    for (let i = 0; i < floorHistory.length; i++) {
      const start = Math.max(0, i - windowSize + 1);
      const window = floorHistory.slice(start, i + 1);
      const rate = window.filter(f => f >= 16).length / window.length;
      data.push(rate * 100);
    }
    const current = data.length > 0 ? data[data.length - 1] : 0;
    return { data, current };
  }, [floorHistory]);

  // ---- Computed: Training Step Markers for Sparklines ----
  const sparklineMarkers: SparklineMarker[] = useMemo(() => {
    return trainStepMarkers.map((m) => ({
      index: m.index,
      label: `T${m.step}`,
    }));
  }, [trainStepMarkers]);

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

  // ---- Computed: Floor Bucket Distribution ----
  const floorBuckets = useMemo(() => {
    const buckets: { label: string; range: [number, number]; count: number; color: string }[] = [
      { label: '1-5', range: [1, 5], count: 0, color: '#ff4444' },
      { label: '6-10', range: [6, 10], count: 0, color: '#ff6b35' },
      { label: '11-15', range: [11, 15], count: 0, color: '#ffb700' },
      { label: '16', range: [16, 16], count: 0, color: '#c9d1d9' },
      { label: '17-25', range: [17, 25], count: 0, color: '#4488ff' },
      { label: '26-34', range: [26, 34], count: 0, color: '#a78bfa' },
      { label: '35+', range: [35, 999], count: 0, color: '#00ff41' },
    ];
    for (const ep of filteredEpisodes) {
      const f = ep.floors_reached;
      if (f <= 0) continue;
      for (const b of buckets) {
        if (f >= b.range[0] && f <= b.range[1]) {
          b.count++;
          break;
        }
      }
    }
    const maxCount = Math.max(...buckets.map(b => b.count), 1);
    // Filter out empty high buckets to save space
    let lastNonZero = buckets.length - 1;
    while (lastNonZero > 3 && buckets[lastNonZero].count === 0) lastNonZero--;
    return { buckets: buckets.slice(0, lastNonZero + 1), maxCount };
  }, [filteredEpisodes]);

  // ---- Computed: Per-floor distribution (for detailed view) ----
  const floorDist = useMemo(() => {
    const counts: Record<number, number> = {};
    let maxCount = 0;
    let highestFloor = 0;
    for (const ep of filteredEpisodes) {
      const f = ep.floors_reached;
      if (f <= 0) continue;
      counts[f] = (counts[f] || 0) + 1;
      if (counts[f] > maxCount) maxCount = counts[f];
      if (f > highestFloor) highestFloor = f;
    }
    const minFloor = scope === 'overview' || scope === 'rewards' ? 1 : floorRange[0];
    const maxFloorCap = scope === 'overview' || scope === 'rewards' ? Math.max(highestFloor, 20) : floorRange[1];
    const entries: { floor: number; count: number }[] = [];
    for (let f = minFloor; f <= Math.max(highestFloor, maxFloorCap); f++) {
      if (counts[f]) {
        entries.push({ floor: f, count: counts[f] });
      }
    }
    return { entries, maxCount, highestFloor };
  }, [filteredEpisodes, scope, floorRange]);

  // ---- Computed: Avg HP Lost Per Combat ----
  const avgHpLost = useMemo(() => {
    let totalLost = 0;
    let combatCount = 0;
    for (const ep of filteredEpisodes) {
      if (ep.combats && ep.combats.length > 0) {
        for (const c of ep.combats) {
          totalLost += c.hp_lost;
          combatCount++;
        }
      }
    }
    if (combatCount === 0) return null;
    return { avg: totalLost / combatCount, combatCount };
  }, [filteredEpisodes]);

  // ---- Computed: HP Lost Trend (per-episode avg) ----
  const hpLostTrend = useMemo(() => {
    const values: number[] = [];
    for (const ep of filteredEpisodes) {
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
  }, [filteredEpisodes]);

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

  // ---- Computed: Stance Distribution ----
  const stanceDist = useMemo(() => {
    const totals: Record<string, number> = {};
    let combatCount = 0;
    for (const ep of filteredEpisodes) {
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
  }, [filteredEpisodes]);

  // ---- Computed: Popular Card Picks ----
  const popularCards = useMemo(() => {
    const counts: Record<string, number> = {};
    let hasData = false;
    for (const ep of filteredEpisodes) {
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
  }, [filteredEpisodes]);

  // ---- Computed: Top Runners (best individual game runs) ----
  const topRunners = useMemo(() => {
    if (filteredEpisodes.length === 0) return [];
    return [...filteredEpisodes]
      .sort((a, b) => {
        if (a.won !== b.won) return a.won ? -1 : 1;
        return b.floors_reached - a.floors_reached;
      })
      .slice(0, 5)
      .map((ep, i) => ({
        rank: i + 1,
        floor: ep.floors_reached,
        won: ep.won,
        deathEnemy: ep.death_enemy ?? null,
        seed: ep.seed,
        agentId: ep.agent_id,
        hpRemaining: ep.hp_remaining ?? 0,
        combats: ep.combats?.length ?? 0,
      }));
  }, [filteredEpisodes]);

  // ---- Computed: Summary Stats ----
  const summaryStats = useMemo(() => {
    const isScoped = scope !== 'overview' && scope !== 'rewards';
    const totalEp = isScoped ? filteredEpisodes.length : (stats?.total_episodes ?? filteredEpisodes.length);
    const winCount = isScoped
      ? filteredEpisodes.filter((e) => e.won).length
      : (stats?.win_count ?? filteredEpisodes.filter((e) => e.won).length);
    const avgFloor = isScoped
      ? (filteredEpisodes.length > 0
        ? filteredEpisodes.reduce((s, e) => s + e.floors_reached, 0) / filteredEpisodes.length
        : 0)
      : (stats?.avg_floor ?? (filteredEpisodes.length > 0
        ? filteredEpisodes.reduce((s, e) => s + e.floors_reached, 0) / filteredEpisodes.length
        : 0));
    const maxFloor = filteredEpisodes.length > 0
      ? Math.max(...filteredEpisodes.map((e) => e.floors_reached))
      : agents.reduce((m, a) => Math.max(m, a.floor), 0);
    return { totalEp, winCount, avgFloor, maxFloor };
  }, [stats, filteredEpisodes, agents, scope]);

  // ---- Computed: Training info from systemStats ----
  const training = useMemo(() => {
    const ts = systemStats?.training_status;
    if (!ts || Object.keys(ts).length === 0) return null;
    return {
      trainSteps: ts.train_steps as number | undefined,
      totalLoss: ts.total_loss as number | undefined,
      policyLoss: ts.policy_loss as number | undefined,
      valueLoss: ts.value_loss as number | undefined,
      gpm: ts.games_per_min as number | undefined,
      totalGames: ts.total_games as number | undefined,
      avgFloor: ts.avg_floor_100 as number | undefined,
      elapsedH: ts.elapsed_hours as number | undefined,
      sweepPhase: ts.sweep_phase as string | undefined,
      bufferSize: ts.buffer_size as number | undefined,
      entropyCoeff: ts.entropy_coeff as number | undefined,
      configName: ts.config_name as string | undefined,
    };
  }, [systemStats]);

  // ---- Computed: Loss trend ----
  const lossTrend = useMemo(() => {
    if (!training?.totalLoss) return null;
    // Just report the current value with color
    const loss = training.totalLoss;
    return {
      value: loss,
      color: loss < 0.3 ? '#00ff41' : loss < 0.5 ? '#ffb700' : '#ff4444',
    };
  }, [training]);

  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      flex: 1,
      overflow: 'hidden',
      background: '#0d1117',
      fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
      fontSize: '10px',
      color: '#c9d1d9',
    }}>

      {/* ====== SESSION STATS BAR ====== */}
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: '16px',
        padding: '5px 10px',
        borderBottom: '1px solid #21262d',
        background: '#161b22',
        flexShrink: 0,
        fontSize: '10px',
        flexWrap: 'wrap',
      }}>
        {/* Status indicator */}
        <span style={{
          display: 'inline-flex',
          alignItems: 'center',
          gap: '4px',
          color: training ? '#00ff41' : '#8b949e',
          fontWeight: 600,
          fontSize: '9px',
          textTransform: 'uppercase',
          letterSpacing: '0.5px',
        }}>
          <span style={{
            width: '6px',
            height: '6px',
            borderRadius: '50%',
            background: training ? '#00ff41' : '#3d444d',
            display: 'inline-block',
            boxShadow: training ? '0 0 4px #00ff41' : 'none',
          }} />
          {training ? 'Training' : 'Idle'}
        </span>

        {/* Phase badge */}
        {training?.sweepPhase && (
          <span style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: '4px',
            padding: '1px 6px',
            fontSize: '8px',
            fontWeight: 700,
            textTransform: 'uppercase',
            letterSpacing: '0.5px',
            borderRadius: '3px',
            background: training.sweepPhase === 'collecting' ? 'rgba(0,255,65,0.12)' : 'rgba(255,183,0,0.12)',
            color: training.sweepPhase === 'collecting' ? '#00ff41' : '#ffb700',
            border: `1px solid ${training.sweepPhase === 'collecting' ? 'rgba(0,255,65,0.25)' : 'rgba(255,183,0,0.25)'}`,
          }}>
            <span style={{
              width: '5px',
              height: '5px',
              borderRadius: '50%',
              background: training.sweepPhase === 'collecting' ? '#00ff41' : '#ffb700',
              display: 'inline-block',
            }} />
            {training.sweepPhase}
          </span>
        )}

        {/* Inline session metrics */}
        {training?.totalGames != null && (
          <span style={{ color: '#8b949e' }}>
            <span style={{ color: '#c9d1d9', fontWeight: 600 }}>{training.totalGames.toLocaleString()}</span> games
          </span>
        )}
        {training?.gpm != null && (
          <span style={{ color: '#8b949e' }}>
            <span style={{ color: '#c9d1d9' }}>{training.gpm.toFixed(0)}</span> g/min
          </span>
        )}
        {training?.elapsedH != null && (
          <span style={{ color: '#8b949e' }}>
            <span style={{ color: '#c9d1d9' }}>{training.elapsedH.toFixed(1)}</span>h elapsed
          </span>
        )}
        {lossTrend && (
          <span style={{ color: '#8b949e' }}>
            Loss: <span style={{ color: lossTrend.color, fontWeight: 600 }}>{lossTrend.value.toFixed(4)}</span>
          </span>
        )}
        {training?.entropyCoeff != null && (
          <span style={{ color: '#8b949e' }}>
            Ent: <span style={{ color: '#c9d1d9' }}>{training.entropyCoeff.toFixed(3)}</span>
          </span>
        )}
        {training?.configName && (
          <span style={{ color: '#a78bfa', fontSize: '9px', marginLeft: 'auto' }}>
            {training.configName}
          </span>
        )}
      </div>

      {/* Scope selector */}
      <div style={{
        display: 'flex',
        gap: '2px',
        padding: '4px 8px',
        borderBottom: '1px solid #21262d',
        background: '#161b22',
        flexShrink: 0,
      }}>
        {(['overview', 'act1', 'act2', 'act3', 'rewards'] as const).map((s) => {
          const labels: Record<string, string> = {
            overview: 'Overview',
            act1: 'Act 1 (F1-17)',
            act2: 'Act 2 (F18-34)',
            act3: 'Act 3 (F35-55)',
            rewards: 'Rewards',
          };
          return (
            <button
              key={s}
              onClick={() => setScope(s)}
              style={{
                background: scope === s ? '#21262d' : 'transparent',
                border: scope === s ? '1px solid #30363d' : '1px solid transparent',
                color: scope === s ? '#c9d1d9' : '#8b949e',
                padding: '3px 10px',
                fontSize: '9px',
                fontFamily: "'JetBrains Mono', monospace",
                cursor: 'pointer',
                fontWeight: scope === s ? 600 : 400,
                textTransform: 'uppercase',
                letterSpacing: '0.5px',
              }}
            >
              {labels[s]}
            </button>
          );
        })}
      </div>

      {/* Content */}
      {scope === 'rewards' ? (
        <div style={{ flex: 1, overflow: 'auto', padding: '16px' }}>
          <SectionHeader>Reward Configuration</SectionHeader>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '16px', marginTop: '8px' }}>
            {/* Combat Rewards */}
            <div>
              <div style={{ fontSize: '10px', color: '#ffb700', marginBottom: '6px', fontWeight: 600 }}>Combat Events</div>
              {([
                ['Combat Win', '0.05'],
                ['Elite Win', '0.30'],
                ['Boss Win', '0.80'],
                ['Damage Penalty', '-0.01/HP'],
                ['Potion Waste', '-0.15/use'],
              ] as const).map(([label, val]) => (
                <div key={label} style={{ display: 'flex', justifyContent: 'space-between', fontSize: '10px', padding: '2px 0' }}>
                  <span style={{ color: '#8b949e' }}>{label}</span>
                  <span style={{ color: val.startsWith('-') ? '#ff4444' : '#00ff41' }}>{val}</span>
                </div>
              ))}
            </div>

            {/* Floor Milestones */}
            <div>
              <div style={{ fontSize: '10px', color: '#ffb700', marginBottom: '6px', fontWeight: 600 }}>Floor Milestones</div>
              {([
                ['F6 (Early)', '+0.10'],
                ['F10 (Mid A1)', '+0.15'],
                ['F15 (Pre-Boss)', '+0.20'],
                ['F16 (A1 Boss)', '+0.25'],
                ['F17 (Beat A1)', '+1.00'],
                ['F25 (Mid A2)', '+0.50'],
                ['F33 (A2 Boss)', '+1.00'],
                ['F34 (Beat A2)', '+2.00'],
                ['F50 (A3 Boss)', '+2.00'],
                ['F51 (Beat A3)', '+3.00'],
                ['F55 (Heart)', '+5.00'],
              ] as const).map(([label, val]) => (
                <div key={label} style={{ display: 'flex', justifyContent: 'space-between', fontSize: '10px', padding: '2px 0' }}>
                  <span style={{ color: '#8b949e' }}>{label}</span>
                  <span style={{ color: '#00ff41' }}>{val}</span>
                </div>
              ))}
            </div>

            {/* Stance + Card Picks */}
            <div>
              <div style={{ fontSize: '10px', color: '#ffb700', marginBottom: '6px', fontWeight: 600 }}>Stance Rewards</div>
              {([
                ['Calm', '+0.05'],
                ['Wrath', '+0.30'],
                ['Divinity', '+0.20'],
              ] as const).map(([label, val]) => (
                <div key={label} style={{ display: 'flex', justifyContent: 'space-between', fontSize: '10px', padding: '2px 0' }}>
                  <span style={{ color: '#8b949e' }}>{label}</span>
                  <span style={{ color: '#00ff41' }}>{val}</span>
                </div>
              ))}
              <div style={{ fontSize: '10px', color: '#ffb700', marginBottom: '6px', marginTop: '12px', fontWeight: 600 }}>Key Card Picks</div>
              {([
                ['Rushdown', '+0.30'],
                ['Tantrum', '+0.25'],
                ['MentalFortress', '+0.25'],
                ['TalkToTheHand', '+0.20'],
                ['InnerPeace', '+0.15'],
                ['Ragnarok', '+0.15'],
                ['Card Remove', '+0.40'],
              ] as const).map(([label, val]) => (
                <div key={label} style={{ display: 'flex', justifyContent: 'space-between', fontSize: '10px', padding: '2px 0' }}>
                  <span style={{ color: '#8b949e' }}>{label}</span>
                  <span style={{ color: '#00ff41' }}>{val}</span>
                </div>
              ))}
            </div>
          </div>

          {/* Terminal rewards */}
          <div style={{ marginTop: '16px', borderTop: '1px solid #21262d', paddingTop: '12px' }}>
            <div style={{ fontSize: '10px', color: '#ffb700', marginBottom: '6px', fontWeight: 600 }}>Terminal</div>
            <div style={{ display: 'flex', gap: '24px', fontSize: '10px' }}>
              <span style={{ color: '#8b949e' }}>Win: <span style={{ color: '#00ff41' }}>+2.0</span></span>
              <span style={{ color: '#8b949e' }}>Death: <span style={{ color: '#ff4444' }}>-1.0 * (1 - progress)</span></span>
              <span style={{ color: '#8b949e' }}>Act 1 bonus: <span style={{ color: '#ffb700' }}>1.5x card pick</span></span>
            </div>
          </div>
        </div>
      ) : (
      <>
      {/* ====== HERO STATS ROW ====== */}
      <div style={{
        display: 'flex',
        gap: '4px',
        padding: '6px 8px',
        borderBottom: '1px solid #21262d',
        background: '#0d1117',
        flexShrink: 0,
      }}>
        <BigStat
          label="Avg Floor"
          value={summaryStats.avgFloor > 0 ? summaryStats.avgFloor.toFixed(1) : '---'}
          color="#ffb700"
          sub={`peak ${summaryStats.maxFloor}`}
        />
        <BigStat
          label="F16+ Rate"
          value={f16Rate.current > 0 ? `${f16Rate.current.toFixed(1)}%` : '0%'}
          color={f16Rate.current > 10 ? '#00ff41' : f16Rate.current > 0 ? '#ffb700' : '#3d444d'}
          sub={`of ${floorHistory.length} games`}
        />
        <BigStat
          label="Train Steps"
          value={training?.trainSteps?.toLocaleString() ?? '---'}
          color="#a78bfa"
          sub={training?.bufferSize ? `buf ${training.bufferSize.toLocaleString()}` : undefined}
        />
        <BigStat
          label="Games"
          value={summaryStats.totalEp > 0 ? summaryStats.totalEp.toLocaleString() : '---'}
          color="#c9d1d9"
          sub={`${summaryStats.winCount} wins`}
        />
        <BigStat
          label="G/min"
          value={training?.gpm?.toFixed(0) ?? (stats?.eps_per_min?.toFixed(0) ?? '---')}
          color="#c9d1d9"
          sub={stats?.uptime != null ? fmtDuration(stats.uptime) : undefined}
        />
        {lossTrend && (
          <BigStat
            label="Loss"
            value={lossTrend.value.toFixed(4)}
            color={lossTrend.color}
            sub={training?.policyLoss != null ? `p:${training.policyLoss.toFixed(3)} v:${training.valueLoss?.toFixed(3) ?? '-'}` : undefined}
          />
        )}
      </div>

      {/* ====== TRAINING PROGRESS BAR ====== */}
      {training?.trainSteps != null && (
        <div style={{
          padding: '3px 10px',
          borderBottom: '1px solid #21262d',
          background: '#0d1117',
          flexShrink: 0,
        }}>
          <div style={{
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            fontSize: '9px',
          }}>
            <span style={{ color: '#8b949e', whiteSpace: 'nowrap' }}>Step {training.trainSteps.toLocaleString()}</span>
            <div style={{
              flex: 1,
              height: '6px',
              background: '#21262d',
              overflow: 'hidden',
              position: 'relative',
            }}>
              {/* Animated progress bar - pulses when training */}
              <div style={{
                width: `${Math.min(100, (training.trainSteps / Math.max(training.trainSteps, 1000)) * 100)}%`,
                height: '100%',
                background: 'linear-gradient(90deg, #a78bfa, #7c3aed)',
                transition: 'width 0.5s ease',
              }} />
              {training.sweepPhase === 'training' && (
                <div style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  right: 0,
                  bottom: 0,
                  background: 'linear-gradient(90deg, transparent 0%, rgba(167,139,250,0.3) 50%, transparent 100%)',
                  animation: 'shimmer 2s ease-in-out infinite',
                }} />
              )}
            </div>
            {training.sweepPhase && (
              <span style={{
                color: training.sweepPhase === 'collecting' ? '#00ff41' : '#ffb700',
                fontWeight: 600,
                whiteSpace: 'nowrap',
                fontSize: '8px',
                textTransform: 'uppercase',
              }}>
                {training.sweepPhase}
              </span>
            )}
          </div>
        </div>
      )}

      {/* ====== WORKER GRID ====== */}
      {agents.length > 0 && (
        <div style={{
          padding: '6px 8px',
          borderBottom: '1px solid #21262d',
          background: '#161b22',
          flexShrink: 0,
        }}>
          <WorkerGrid agents={agents} />
        </div>
      )}

      {/* ====== MAIN 3-COLUMN LAYOUT ====== */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: '1fr 1fr 1fr',
        gap: '0',
        flex: 1,
        overflow: 'hidden',
      }}>

      {/* ====== COLUMN 1: Floor Trend + Distribution ====== */}
      <div style={{
        borderRight: '1px solid #21262d',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}>
        {/* Avg Floor Trend - LARGE chart */}
        <div style={{ padding: '8px', borderBottom: '1px solid #21262d', flexShrink: 0 }}>
          <SectionHeader
            right={
              <span style={{ color: '#ffb700', fontSize: '11px', fontWeight: 700 }}>
                Avg: {rollingAvgFloor.current.toFixed(1)}
              </span>
            }
          >
            Avg Floor (rolling 50)
          </SectionHeader>
          <FloorChart
            data={rollingAvgFloor.data}
            markers={sparklineMarkers}
            current={rollingAvgFloor.current}
            peak={rollingAvgFloor.peak}
          />

          {/* Win rate inline if non-zero */}
          {rollingWinRate.current > 0 && (
            <div style={{ marginTop: '4px', fontSize: '9px', color: '#8b949e' }}>
              Win Rate (rolling 20): <span style={{ color: '#00ff41' }}>{(rollingWinRate.current * 100).toFixed(1)}%</span>
            </div>
          )}
        </div>

        {/* F16+ Rate sparkline */}
        <div style={{ padding: '8px', borderBottom: '1px solid #21262d', flexShrink: 0 }}>
          <SectionHeader
            right={
              <span style={{ color: f16Rate.current > 0 ? '#00ff41' : '#3d444d', fontSize: '11px', fontWeight: 700 }}>
                {f16Rate.current.toFixed(1)}%
              </span>
            }
          >
            F16+ Rate (rolling 100)
          </SectionHeader>
          {f16Rate.data.length > 1 ? (
            <Sparkline
              data={f16Rate.data}
              width={320}
              height={50}
              color="#00ff41"
              fill={true}
              markers={sparklineMarkers}
            />
          ) : (
            <EmptyState text="Need 2+ episodes" />
          )}
        </div>

        {/* Floor Distribution Buckets */}
        <div style={{ flex: 1, padding: '8px', overflow: 'auto' }}>
          <SectionHeader
            right={filteredEpisodes.length > 0 ? (
              <span style={{ color: '#3d444d' }}>{filteredEpisodes.length} runs</span>
            ) : undefined}
          >
            Floor Distribution
          </SectionHeader>

          {floorBuckets.buckets.some(b => b.count > 0) ? (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
              {floorBuckets.buckets.map(({ label, count, color }) => {
                const totalRuns = filteredEpisodes.length;
                const pctStr = totalRuns > 0 ? `${((count / totalRuns) * 100).toFixed(0)}%` : '0%';
                return (
                  <HBar
                    key={label}
                    label={`F${label}`}
                    value={count}
                    maxValue={floorBuckets.maxCount}
                    color={color}
                    labelWidth={40}
                    pctLabel={`${count} (${pctStr})`}
                  />
                );
              })}
            </div>
          ) : (
            <EmptyState text="No episode data yet" />
          )}

          {/* Compact per-floor bars for top floors */}
          {floorDist.entries.length > 0 && (
            <div style={{ marginTop: '8px' }}>
              <div style={{ fontSize: '8px', color: '#3d444d', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '4px' }}>
                Per-floor detail
              </div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '0px' }}>
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
            </div>
          )}
        </div>
      </div>

      {/* ====== COLUMN 2: Combat + Stance ====== */}
      <div style={{
        borderRight: '1px solid #21262d',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}>

        {/* Avg HP Lost / Combat */}
        <div style={{ padding: '8px', borderBottom: '1px solid #21262d', flexShrink: 0 }}>
          <SectionHeader
            right={avgHpLost ? (
              <span style={{ color: '#3d444d' }}>{avgHpLost.combatCount} combats</span>
            ) : undefined}
          >
            Avg HP Lost / Combat
          </SectionHeader>

          {avgHpLost !== null ? (
            <div style={{ display: 'flex', alignItems: 'baseline', gap: '6px' }}>
              <span style={{ fontSize: '22px', fontWeight: 700, color: '#ff4444' }}>
                {avgHpLost.avg.toFixed(1)}
              </span>
              <span style={{ fontSize: '10px', color: '#8b949e' }}>HP</span>
              {hpLostTrend && (
                <span style={{
                  fontSize: '14px',
                  color: hpLostTrend === 'down' ? '#00ff41' : hpLostTrend === 'up' ? '#ff4444' : '#8b949e',
                }}>
                  {hpLostTrend === 'down' ? '\u2193 improving' : hpLostTrend === 'up' ? '\u2191 worsening' : '\u2192 stable'}
                </span>
              )}
            </div>
          ) : (
            <EmptyState text="Collecting combat data..." />
          )}
        </div>

        {/* Stance Distribution */}
        <div style={{ padding: '8px', borderBottom: '1px solid #21262d', flexShrink: 0 }}>
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
              <div style={{ display: 'flex', height: '14px', overflow: 'hidden', marginBottom: '6px', background: '#21262d' }}>
                {stanceDist.entries.map(({ stance, count }) => (
                  <div
                    key={stance}
                    style={{
                      width: `${stanceDist.total > 0 ? (count / stanceDist.total) * 100 : 0}%`,
                      height: '100%',
                      background: STANCE_COLORS[stance] ?? '#8b949e',
                      opacity: 0.8,
                      position: 'relative',
                    }}
                    title={`${stance}: ${stanceDist.total > 0 ? ((count / stanceDist.total) * 100).toFixed(1) : 0}%`}
                  />
                ))}
              </div>
              {/* Legend with counts */}
              <div style={{ display: 'flex', gap: '12px', flexWrap: 'wrap', fontSize: '10px' }}>
                {stanceDist.entries.map(({ stance, count }) => (
                  <div key={stance} style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                    <div style={{
                      width: '8px',
                      height: '8px',
                      background: STANCE_COLORS[stance] ?? '#8b949e',
                      flexShrink: 0,
                    }} />
                    <span style={{ color: STANCE_COLORS[stance] ?? '#8b949e', fontWeight: 600 }}>
                      {stance}
                    </span>
                    <span style={{ color: '#8b949e' }}>
                      {stanceDist.total > 0 ? `${((count / stanceDist.total) * 100).toFixed(0)}%` : '0%'}
                    </span>
                    <span style={{ color: '#3d444d', fontSize: '9px' }}>
                      ({count})
                    </span>
                  </div>
                ))}
              </div>
              {/* Stance ratios insight */}
              {(() => {
                const wrathEntry = stanceDist.entries.find(e => e.stance === 'Wrath');
                const calmEntry = stanceDist.entries.find(e => e.stance === 'Calm');
                if (!wrathEntry || !calmEntry || calmEntry.count === 0) return null;
                const ratio = wrathEntry.count / calmEntry.count;
                return (
                  <div style={{ fontSize: '9px', color: '#8b949e', marginTop: '4px' }}>
                    Wrath/Calm ratio: <span style={{ color: ratio > 1.5 ? '#ff4444' : ratio > 0.8 ? '#ffb700' : '#4488ff', fontWeight: 600 }}>
                      {ratio.toFixed(2)}
                    </span>
                    {ratio > 2 && <span style={{ color: '#ff4444', marginLeft: '6px' }}>(aggressive)</span>}
                    {ratio < 0.5 && <span style={{ color: '#4488ff', marginLeft: '6px' }}>(defensive)</span>}
                  </div>
                );
              })()}
            </>
          ) : (
            <EmptyState text="No stance data yet" />
          )}
        </div>

        {/* Top Killers */}
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
              {topKillers.entries.map(({ enemy, count }) => {
                const totalDeaths = deathStats.totalDeaths || 1;
                return (
                  <HBar
                    key={enemy}
                    label={enemy.length > 12 ? enemy.slice(0, 11) + '\u2026' : enemy}
                    value={count}
                    maxValue={topKillers.maxCount}
                    color="#ff4444"
                    labelWidth={80}
                    pctLabel={`${count} (${((count / totalDeaths) * 100).toFixed(0)}%)`}
                  />
                );
              })}
            </div>
          ) : (
            <EmptyState text="No deaths recorded" />
          )}
        </div>

        {/* Death Floor Heatmap */}
        <div style={{ flex: 1, padding: '8px', overflow: 'auto' }}>
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
      </div>

      {/* ====== COLUMN 3: Best Runs + Cards + System ====== */}
      <div style={{
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}>

        {/* Best Runs Leaderboard */}
        <div style={{ padding: '8px', borderBottom: '1px solid #21262d', flexShrink: 0 }}>
          <SectionHeader
            right={topRunners.length > 0 ? (
              <span style={{ color: '#ffb700' }}>
                Best: F{topRunners[0]?.floor ?? 0}
              </span>
            ) : undefined}
          >
            Best Runs
          </SectionHeader>

          {topRunners.length > 0 ? (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
              {topRunners.map((r) => {
                const floorClr = r.won ? '#00ff41' : r.floor >= 40 ? '#ffb700' : r.floor >= 16 ? '#c9d1d9' : '#ff4444';
                return (
                  <div
                    key={`${r.seed}-${r.rank}`}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '6px',
                      padding: '2px 4px',
                      background: r.rank === 1 ? 'rgba(255,183,0,0.06)' : 'transparent',
                      borderLeft: `2px solid ${floorClr}`,
                    }}
                  >
                    <span style={{
                      fontSize: '14px',
                      fontWeight: 700,
                      color: floorClr,
                      width: '28px',
                      textAlign: 'right',
                      flexShrink: 0,
                      lineHeight: 1,
                    }}>
                      {r.floor}
                    </span>
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <div style={{
                        fontSize: '9px',
                        color: '#c9d1d9',
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                        whiteSpace: 'nowrap',
                      }}>
                        {r.won ? (
                          <span style={{ color: '#00ff41', fontWeight: 600 }}>WIN</span>
                        ) : (
                          <span style={{ color: '#8b949e' }}>{r.deathEnemy ?? 'Unknown'}</span>
                        )}
                      </div>
                      <div style={{ fontSize: '8px', color: '#3d444d', display: 'flex', gap: '6px' }}>
                        <span>HP <span style={{ color: r.hpRemaining > 0 ? hpColor(r.hpRemaining / 80) : '#3d444d' }}>{r.hpRemaining}</span></span>
                        <span>{r.combats} cmb</span>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          ) : (
            <EmptyState text="No completed runs yet" />
          )}
        </div>

        {/* Popular Card Picks */}
        <div style={{ padding: '8px', borderBottom: '1px solid #21262d', flexShrink: 0 }}>
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
                label={`GPU (Metal) ${systemStats.gpu_util_pct ? systemStats.gpu_util_pct.toFixed(0) + '%' : ''}`}
                value={systemStats.gpu_util_pct ?? 0}
                max={100}
                unit="%"
                color={systemStats.gpu_util_pct && systemStats.gpu_util_pct > 30 ? '#a78bfa' : '#8b949e'}
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
      </div>
      </>
      )}

      {/* Shimmer animation for training progress bar */}
      <style>{`
        @keyframes shimmer {
          0% { transform: translateX(-100%); }
          100% { transform: translateX(100%); }
        }
      `}</style>
    </div>
  );
};
