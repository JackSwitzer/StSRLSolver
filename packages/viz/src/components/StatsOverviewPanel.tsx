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
    for (const ep of filteredEpisodes) {
      const f = ep.floors_reached;
      if (f <= 0) continue; // Skip construction failures
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

  // ---- Computed: Training Step Markers for Sparklines ----
  const sparklineMarkers: SparklineMarker[] = useMemo(() => {
    return trainStepMarkers.map((m) => ({
      index: m.index,
      label: `T${m.step}`,
    }));
  }, [trainStepMarkers]);

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
    return totalLost / combatCount;
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

  // ---- Computed: Top Runners (best individual game runs) ----
  const topRunners = useMemo(() => {
    if (filteredEpisodes.length === 0) return [];
    return [...filteredEpisodes]
      .sort((a, b) => {
        // Wins first, then by floors_reached descending
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
    // When scoped to an act, always compute from filtered data (server stats are global)
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

      {/* Training status banner */}
      {systemStats?.training_status && Object.keys(systemStats.training_status).length > 0 && (() => {
        const ts = systemStats.training_status!;
        const trainSteps = ts.train_steps as number | undefined;
        const totalLoss = ts.total_loss as number | undefined;
        const gpm = ts.games_per_min as number | undefined;
        const totalGames = ts.total_games as number | undefined;
        const avgFloor = ts.avg_floor_100 as number | undefined;
        const elapsedH = ts.elapsed_hours as number | undefined;
        const sweepPhase = ts.sweep_phase as string | undefined;
        const bufferSize = ts.buffer_size as number | undefined;
        const isActive = trainSteps != null && trainSteps > 0;
        const isTraining = sweepPhase === 'training';
        const isCollecting = sweepPhase === 'collecting';
        const lossColor = totalLoss != null
          ? totalLoss < 0.3 ? '#00ff41' : totalLoss < 0.5 ? '#ffb700' : '#ff4444'
          : '#ff8c00';
        return (
          <div style={{
            display: 'flex',
            alignItems: 'center',
            gap: '16px',
            padding: '4px 8px',
            borderBottom: '1px solid #21262d',
            background: isActive ? 'rgba(0,255,65,0.04)' : '#161b22',
            flexShrink: 0,
            fontSize: '10px',
            fontFamily: "'JetBrains Mono', monospace",
            flexWrap: 'wrap',
          }}>
            <span style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: '4px',
              color: isActive ? '#00ff41' : '#8b949e',
              fontWeight: 600,
              fontSize: '9px',
              textTransform: 'uppercase',
              letterSpacing: '0.5px',
            }}>
              <span style={{
                width: '6px',
                height: '6px',
                borderRadius: '50%',
                background: isActive ? '#00ff41' : '#3d444d',
                display: 'inline-block',
                boxShadow: isActive ? '0 0 4px #00ff41' : 'none',
              }} />
              {isActive ? 'Training' : 'Idle'}
            </span>
            {/* Phase badge */}
            {sweepPhase && (
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
                background: isCollecting ? 'rgba(0,255,65,0.12)' : 'rgba(255,183,0,0.12)',
                color: isCollecting ? '#00ff41' : '#ffb700',
                border: `1px solid ${isCollecting ? 'rgba(0,255,65,0.25)' : 'rgba(255,183,0,0.25)'}`,
                animation: isTraining ? 'phase-pulse 2s ease-in-out infinite' : 'none',
              }}>
                <span style={{
                  width: '5px',
                  height: '5px',
                  borderRadius: '50%',
                  background: isCollecting ? '#00ff41' : '#ffb700',
                  display: 'inline-block',
                }} />
                {sweepPhase}
              </span>
            )}
            {/* Training details: buffer + loss during training phase */}
            {isTraining && bufferSize != null && (
              <span style={{ color: '#8b949e', fontSize: '9px' }}>
                Training on <span style={{ color: '#c9d1d9', fontWeight: 600 }}>{bufferSize.toLocaleString()}</span> transitions
              </span>
            )}
            {trainSteps != null && (
              <span style={{ color: '#8b949e' }}>
                Steps: <span style={{ color: '#c9d1d9' }}>{trainSteps.toLocaleString()}</span>
              </span>
            )}
            {totalLoss != null && (
              <span style={{ color: '#8b949e' }}>
                Loss: <span style={{ color: lossColor, fontWeight: 600 }}>{totalLoss.toFixed(4)}</span>
              </span>
            )}
            {gpm != null && (
              <span style={{ color: '#8b949e' }}>
                <span style={{ color: '#c9d1d9' }}>{gpm.toFixed(0)}</span> g/min
              </span>
            )}
            {avgFloor != null && (
              <span style={{ color: '#8b949e' }}>
                Avg F: <span style={{ color: '#ffb700' }}>{avgFloor.toFixed(1)}</span>
              </span>
            )}
            {totalGames != null && (
              <span style={{ color: '#8b949e' }}>
                <span style={{ color: '#c9d1d9' }}>{totalGames.toLocaleString()}</span> games
              </span>
            )}
            {elapsedH != null && (
              <span style={{ color: '#3d444d' }}>{elapsedH.toFixed(1)}h</span>
            )}
          </div>
        );
      })()}

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
                ['F15 (Pre-Boss)', '+0.25'],
                ['F16 (A1 Boss)', '+0.50'],
                ['F17 (Beat A1)', '+1.00'],
                ['F25 (Mid A2)', '+0.30'],
                ['F33 (A2 Boss)', '+0.50'],
                ['F34 (Beat A2)', '+1.00'],
                ['F50 (A3 Boss)', '+0.50'],
                ['F51 (Beat A3)', '+1.50'],
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

      <div style={{
        display: 'grid',
        gridTemplateColumns: '1fr 1fr 1fr',
        gap: '0',
        flex: 1,
        overflow: 'hidden',
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

              {/* Overnight Training Status */}
              {systemStats.training_status && Object.keys(systemStats.training_status).length > 0 && (() => {
                const ts = systemStats.training_status!;
                const totalGames = ts.total_games as number | undefined;
                const totalWins = ts.total_wins as number | undefined;
                const avgFloor100 = ts.avg_floor_100 as number | undefined;
                const gamesPerMin = ts.games_per_min as number | undefined;
                const trainSteps = ts.train_steps as number | undefined;
                const replayBuffer = ts.replay_buffer as number | undefined;
                const replayBestFloor = ts.replay_best_floor as number | undefined;
                const configName = ts.config_name as string | undefined;
                const entropyCoeff = ts.entropy_coeff as number | undefined;
                const totalLoss = ts.total_loss as number | undefined;
                const policyLoss = ts.policy_loss as number | undefined;
                const valueLoss = ts.value_loss as number | undefined;
                const elapsedHours = ts.elapsed_hours as number | undefined;
                const winRate = totalGames && totalWins ? ((totalWins / totalGames) * 100) : 0;
                return (
                  <div style={{ marginTop: '8px', borderTop: '1px solid #21262d', paddingTop: '8px' }}>
                    <SectionHeader
                      right={configName ? (
                        <span style={{ color: '#a78bfa', fontSize: '8px' }}>{configName}</span>
                      ) : undefined}
                    >
                      Overnight Training
                    </SectionHeader>

                    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '2px 12px', fontSize: '9px' }}>
                      {totalGames != null && (
                        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                          <span style={{ color: '#8b949e' }}>Games</span>
                          <span style={{ color: '#c9d1d9' }}>
                            {totalGames.toLocaleString()}
                            {winRate > 0 && (
                              <span style={{ color: '#00ff41', marginLeft: '4px' }}>
                                ({winRate.toFixed(1)}%)
                              </span>
                            )}
                          </span>
                        </div>
                      )}
                      {avgFloor100 != null && (
                        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                          <span style={{ color: '#8b949e' }}>Avg Floor</span>
                          <span style={{ color: '#ffb700', fontWeight: 600 }}>{avgFloor100.toFixed(1)}</span>
                        </div>
                      )}
                      {gamesPerMin != null && (
                        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                          <span style={{ color: '#8b949e' }}>g/min</span>
                          <span style={{ color: '#c9d1d9' }}>{gamesPerMin.toFixed(0)}</span>
                        </div>
                      )}
                      {trainSteps != null && (
                        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                          <span style={{ color: '#8b949e' }}>Train Steps</span>
                          <span style={{ color: '#c9d1d9' }}>{trainSteps.toLocaleString()}</span>
                        </div>
                      )}
                      {replayBuffer != null && (
                        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                          <span style={{ color: '#8b949e' }}>Replay</span>
                          <span style={{ color: '#c9d1d9' }}>
                            {replayBuffer.toLocaleString()}
                            {replayBestFloor != null && (
                              <span style={{ color: '#ffb700', marginLeft: '4px' }}>
                                (best F{replayBestFloor})
                              </span>
                            )}
                          </span>
                        </div>
                      )}
                      {entropyCoeff != null && (
                        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                          <span style={{ color: '#8b949e' }}>Entropy</span>
                          <span style={{ color: '#c9d1d9' }}>{entropyCoeff.toFixed(3)}</span>
                        </div>
                      )}
                      {totalLoss != null && (
                        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                          <span style={{ color: '#8b949e' }}>Loss</span>
                          <span style={{ color: '#ff8c00' }}>{totalLoss.toFixed(4)}</span>
                        </div>
                      )}
                      {policyLoss != null && (
                        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                          <span style={{ color: '#8b949e' }}>P.Loss</span>
                          <span style={{ color: '#c9d1d9' }}>{policyLoss.toFixed(4)}</span>
                        </div>
                      )}
                      {valueLoss != null && (
                        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                          <span style={{ color: '#8b949e' }}>V.Loss</span>
                          <span style={{ color: '#c9d1d9' }}>{valueLoss.toFixed(4)}</span>
                        </div>
                      )}
                    </div>

                    {elapsedHours != null && (
                      <div style={{ fontSize: '8px', color: '#3d444d', marginTop: '4px', textAlign: 'right' }}>
                        Running {elapsedHours.toFixed(1)}h
                      </div>
                    )}
                  </div>
                );
              })()}
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
            right={filteredEpisodes.length > 0 ? (
              <span style={{ color: '#3d444d' }}>{filteredEpisodes.length} runs</span>
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
                markers={sparklineMarkers}
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
              markers={sparklineMarkers}
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
                      fontFamily: "'JetBrains Mono', monospace",
                    }}
                  >
                    {/* Floor - big number */}
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
                    {/* Details column */}
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
      </>
      )}
    </div>
  );
};
