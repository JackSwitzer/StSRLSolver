import { useMemo } from 'react';
import type { AgentEpisodeMsg, DecisionSummary } from '../types/training';

interface DecisionLogPanelProps {
  episodes: AgentEpisodeMsg[];
  agentId: number;
}

// ---- Constants ----

const TYPE_CONFIG: Record<string, { color: string; icon: string; label: string }> = {
  card_pick: { color: '#4488ff', icon: '+', label: 'PICK' },
  path:      { color: '#00e5ff', icon: '>', label: 'PATH' },
  rest:      { color: '#00ff41', icon: 'Z', label: 'REST' },
  shop:      { color: '#ffb700', icon: '$', label: 'SHOP' },
  event:     { color: '#cc88ff', icon: '?', label: 'EVNT' },
  neow:      { color: '#ff44ff', icon: '*', label: 'NEOW' },
  boss_relic: { color: '#ff8c00', icon: 'B', label: 'BOSS' },
  potion:    { color: '#ff44ff', icon: 'P', label: 'POT' },
};

function getTypeConfig(type: string) {
  return TYPE_CONFIG[type] ?? { color: '#8b949e', icon: '.', label: type.slice(0, 4).toUpperCase() };
}

// ---- Sub-components ----

const DecisionEntry = ({ decision, isLatest }: { decision: DecisionSummary; isLatest: boolean }) => {
  const cfg = getTypeConfig(decision.type);
  const hasAlts = decision.alternatives && decision.alternatives.length > 0;

  return (
    <div style={{
      display: 'flex',
      gap: '6px',
      fontSize: '10px',
      padding: '3px 0',
      borderLeft: `2px solid ${isLatest ? cfg.color : '#21262d'}`,
      paddingLeft: '8px',
      alignItems: 'flex-start',
    }}>
      {/* Floor */}
      <span style={{
        color: '#8b949e',
        fontFamily: 'monospace',
        width: '26px',
        flexShrink: 0,
        textAlign: 'right',
      }}>
        F{decision.floor}
      </span>

      {/* Type badge */}
      <span style={{
        color: '#0d1117',
        background: cfg.color,
        padding: '0 4px',
        fontSize: '8px',
        fontWeight: 700,
        letterSpacing: '0.3px',
        flexShrink: 0,
        lineHeight: '16px',
      }}>
        {cfg.label}
      </span>

      {/* Choice */}
      <span style={{ color: '#c9d1d9', flex: 1 }}>
        {formatChoice(decision)}
      </span>

      {/* Score */}
      {decision.score !== undefined && (
        <span style={{
          color: decision.score >= 0 ? '#00ff41' : '#ff4444',
          fontFamily: 'monospace',
          fontSize: '9px',
          flexShrink: 0,
        }}>
          {decision.score >= 0 ? '+' : ''}{decision.score.toFixed(1)}
        </span>
      )}

      {/* Alternatives count */}
      {hasAlts && (
        <span style={{
          color: '#3d444d',
          fontSize: '8px',
          flexShrink: 0,
        }}>
          /{decision.alternatives!.length}
        </span>
      )}
    </div>
  );
};

function formatChoice(d: DecisionSummary): string {
  const parts = [d.choice];
  if (d.detail) parts.push(`(${d.detail})`);
  if (d.alternatives && d.alternatives.length > 0) {
    const skipped = d.alternatives.slice(0, 3).join(', ');
    const extra = d.alternatives.length > 3 ? ` +${d.alternatives.length - 3}` : '';
    parts.push(`[skipped: ${skipped}${extra}]`);
  }
  return parts.join(' ');
}

const EpisodeHeader = ({ ep }: { ep: AgentEpisodeMsg }) => {
  const outcomeColor = ep.won ? '#00ff41' : '#ff4444';
  const outcomeText = ep.won ? 'WON' : `DIED F${ep.death_floor ?? ep.floors_reached}`;
  const deathEnemy = !ep.won && ep.death_enemy ? ` (${ep.death_enemy})` : '';

  return (
    <div style={{
      display: 'flex',
      alignItems: 'center',
      gap: '8px',
      padding: '4px 0',
      borderBottom: '1px solid #21262d',
      marginBottom: '2px',
    }}>
      <span style={{
        fontSize: '9px',
        color: '#8b949e',
        fontFamily: 'monospace',
      }}>
        EP{ep.episode}
      </span>
      <span style={{
        fontSize: '9px',
        color: outcomeColor,
        fontWeight: 700,
      }}>
        {outcomeText}{deathEnemy}
      </span>
      <span style={{ fontSize: '8px', color: '#3d444d' }}>
        {ep.seed?.slice(0, 8)} | {ep.floors_reached}F | {ep.duration.toFixed(1)}s
      </span>
    </div>
  );
};

// ---- Aggregation views ----

const DecisionTypeBreakdown = ({ episodes }: { episodes: AgentEpisodeMsg[] }) => {
  const breakdown = useMemo(() => {
    const counts: Record<string, number> = {};
    for (const ep of episodes) {
      if (!ep.decisions) continue;
      for (const d of ep.decisions) {
        counts[d.type] = (counts[d.type] ?? 0) + 1;
      }
    }
    const entries = Object.entries(counts)
      .map(([type, count]) => ({ type, count }))
      .sort((a, b) => b.count - a.count);
    const total = entries.reduce((s, e) => s + e.count, 0);
    return { entries, total };
  }, [episodes]);

  if (breakdown.entries.length === 0) return null;

  return (
    <div style={{ padding: '6px 0' }}>
      <div style={{
        fontSize: '9px',
        color: '#8b949e',
        textTransform: 'uppercase',
        letterSpacing: '0.5px',
        marginBottom: '4px',
      }}>
        Decision Breakdown ({breakdown.total})
      </div>
      {/* Stacked bar */}
      <div style={{ display: 'flex', height: '8px', overflow: 'hidden', background: '#21262d', marginBottom: '4px' }}>
        {breakdown.entries.map(({ type, count }) => {
          const cfg = getTypeConfig(type);
          const pct = breakdown.total > 0 ? (count / breakdown.total) * 100 : 0;
          return (
            <div
              key={type}
              style={{
                width: `${pct}%`,
                height: '100%',
                background: cfg.color,
                opacity: 0.8,
              }}
            />
          );
        })}
      </div>
      {/* Legend */}
      <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap', fontSize: '9px' }}>
        {breakdown.entries.map(({ type, count }) => {
          const cfg = getTypeConfig(type);
          return (
            <span key={type} style={{ color: cfg.color }}>
              {cfg.label} {count}
            </span>
          );
        })}
      </div>
    </div>
  );
};

const PopularChoices = ({ episodes }: { episodes: AgentEpisodeMsg[] }) => {
  const topPicks = useMemo(() => {
    const counts: Record<string, number> = {};
    for (const ep of episodes) {
      if (!ep.decisions) continue;
      for (const d of ep.decisions) {
        if (d.type === 'card_pick') {
          counts[d.choice] = (counts[d.choice] ?? 0) + 1;
        }
      }
    }
    return Object.entries(counts)
      .map(([card, count]) => ({ card, count }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 6);
  }, [episodes]);

  const topSkips = useMemo(() => {
    const counts: Record<string, number> = {};
    for (const ep of episodes) {
      if (!ep.decisions) continue;
      for (const d of ep.decisions) {
        if (d.type === 'card_pick' && d.alternatives) {
          for (const alt of d.alternatives) {
            counts[alt] = (counts[alt] ?? 0) + 1;
          }
        }
      }
    }
    return Object.entries(counts)
      .map(([card, count]) => ({ card, count }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 6);
  }, [episodes]);

  if (topPicks.length === 0 && topSkips.length === 0) return null;

  const maxPick = topPicks.length > 0 ? topPicks[0].count : 1;
  const maxSkip = topSkips.length > 0 ? topSkips[0].count : 1;

  return (
    <div style={{ display: 'flex', gap: '0', flex: 1, overflow: 'hidden' }}>
      {/* Most picked */}
      <div style={{ flex: 1, padding: '6px 8px 6px 0', borderRight: '1px solid #21262d' }}>
        <div style={{
          fontSize: '9px', color: '#8b949e', textTransform: 'uppercase',
          letterSpacing: '0.5px', marginBottom: '4px',
        }}>
          Most Picked
        </div>
        {topPicks.map(({ card, count }) => (
          <div key={card} style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: '10px', height: '14px' }}>
            <span style={{ width: '80px', color: '#00ff41', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', flexShrink: 0 }}>
              {card}
            </span>
            <div style={{ flex: 1, height: '6px', background: '#21262d', overflow: 'hidden' }}>
              <div style={{ width: `${(count / maxPick) * 100}%`, height: '100%', background: '#00ff41', opacity: 0.6 }} />
            </div>
            <span style={{ width: '20px', textAlign: 'right', color: '#8b949e', fontSize: '9px', flexShrink: 0 }}>{count}</span>
          </div>
        ))}
      </div>

      {/* Most skipped */}
      <div style={{ flex: 1, padding: '6px 0 6px 8px' }}>
        <div style={{
          fontSize: '9px', color: '#8b949e', textTransform: 'uppercase',
          letterSpacing: '0.5px', marginBottom: '4px',
        }}>
          Most Skipped
        </div>
        {topSkips.map(({ card, count }) => (
          <div key={card} style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: '10px', height: '14px' }}>
            <span style={{ width: '80px', color: '#ff4444', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', flexShrink: 0 }}>
              {card}
            </span>
            <div style={{ flex: 1, height: '6px', background: '#21262d', overflow: 'hidden' }}>
              <div style={{ width: `${(count / maxSkip) * 100}%`, height: '100%', background: '#ff4444', opacity: 0.6 }} />
            </div>
            <span style={{ width: '20px', textAlign: 'right', color: '#8b949e', fontSize: '9px', flexShrink: 0 }}>{count}</span>
          </div>
        ))}
      </div>
    </div>
  );
};

// ---- Main Component ----

export const DecisionLogPanel = ({ episodes, agentId }: DecisionLogPanelProps) => {
  const agentEps = useMemo(() =>
    episodes.filter((e) => e.agent_id === agentId).slice(0, 20),
    [episodes, agentId],
  );

  // Flatten all decisions for aggregate view
  const allDecisions = useMemo(() => {
    const out: Array<DecisionSummary & { episode: number; won: boolean }> = [];
    for (const ep of agentEps) {
      if (!ep.decisions) continue;
      for (const d of ep.decisions) {
        out.push({ ...d, episode: ep.episode, won: ep.won });
      }
    }
    return out;
  }, [agentEps]);

  if (agentEps.length === 0) {
    return (
      <div style={{ padding: '12px', color: '#3d444d', fontSize: '10px', textAlign: 'center' }}>
        No episode data yet
      </div>
    );
  }

  // Show latest episode's decisions in timeline, plus aggregate stats
  const latestWithDecisions = agentEps.find((e) => e.decisions && e.decisions.length > 0);

  return (
    <div style={{
      display: 'grid',
      gridTemplateColumns: '1.4fr 1fr',
      gap: '0',
      height: '100%',
      overflow: 'hidden',
      fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
    }}>
      {/* Left: Decision Timeline */}
      <div style={{ borderRight: '1px solid #21262d', display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        <div style={{
          padding: '6px 8px',
          borderBottom: '1px solid #21262d',
          fontSize: '9px',
          color: '#8b949e',
          textTransform: 'uppercase',
          letterSpacing: '0.5px',
          flexShrink: 0,
          display: 'flex',
          justifyContent: 'space-between',
        }}>
          <span>Decision Timeline</span>
          <span style={{ color: '#3d444d' }}>{allDecisions.length} total</span>
        </div>

        <div style={{ flex: 1, overflow: 'auto', padding: '4px 8px' }}>
          {latestWithDecisions ? (
            <>
              <EpisodeHeader ep={latestWithDecisions} />
              {latestWithDecisions.decisions!.map((d, i) => (
                <DecisionEntry
                  key={i}
                  decision={d}
                  isLatest={i === latestWithDecisions.decisions!.length - 1 && !latestWithDecisions.won}
                />
              ))}

              {/* Previous episodes (collapsed) */}
              {agentEps.slice(1).filter((e) => e.decisions && e.decisions.length > 0).slice(0, 3).map((ep) => (
                <div key={ep.episode} style={{ marginTop: '8px' }}>
                  <EpisodeHeader ep={ep} />
                  {ep.decisions!.slice(0, 5).map((d, i) => (
                    <DecisionEntry key={i} decision={d} isLatest={false} />
                  ))}
                  {ep.decisions!.length > 5 && (
                    <div style={{ fontSize: '9px', color: '#3d444d', paddingLeft: '36px', padding: '2px 0' }}>
                      ... +{ep.decisions!.length - 5} more
                    </div>
                  )}
                </div>
              ))}
            </>
          ) : (
            <div style={{ color: '#3d444d', fontSize: '10px', padding: '8px 0' }}>
              No decision data in recent episodes
            </div>
          )}
        </div>
      </div>

      {/* Right: Aggregate Stats */}
      <div style={{ display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        <div style={{
          padding: '6px 8px',
          borderBottom: '1px solid #21262d',
          fontSize: '9px',
          color: '#8b949e',
          textTransform: 'uppercase',
          letterSpacing: '0.5px',
          flexShrink: 0,
        }}>
          Aggregates ({agentEps.length} episodes)
        </div>

        <div style={{ flex: 1, overflow: 'auto', padding: '4px 8px' }}>
          <DecisionTypeBreakdown episodes={agentEps} />
          <PopularChoices episodes={agentEps} />
        </div>
      </div>
    </div>
  );
};
