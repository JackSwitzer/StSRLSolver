import { useMemo } from 'react';
import type { AgentEpisodeMsg } from '../types/training';

interface TopRunsProps {
  episodes: AgentEpisodeMsg[];
  maxRuns?: number;
  onSelectEpisode?: (ep: AgentEpisodeMsg) => void;
}

const container: React.CSSProperties = {
  background: '#161b22',
  border: '1px solid #30363d',
  borderRadius: 6,
  padding: '12px 0',
  fontFamily: 'inherit',
  fontSize: 12,
  color: '#c9d1d9',
};

const header: React.CSSProperties = {
  padding: '0 14px 8px',
  fontSize: 11,
  color: '#8b949e',
  textTransform: 'uppercase',
  letterSpacing: '0.5px',
  borderBottom: '1px solid #21262d',
  marginBottom: 4,
};

const rowBase: React.CSSProperties = {
  display: 'grid',
  gridTemplateColumns: '28px 48px 90px 1fr 60px 50px',
  alignItems: 'center',
  padding: '5px 14px',
  cursor: 'pointer',
  transition: 'background 0.1s',
  fontVariantNumeric: 'tabular-nums',
  gap: 4,
};

const colHeader: React.CSSProperties = {
  ...rowBase,
  cursor: 'default',
  padding: '4px 14px 6px',
  fontSize: 10,
  color: '#484f58',
  textTransform: 'uppercase',
  letterSpacing: '0.5px',
};

const rankStyle: React.CSSProperties = {
  color: '#484f58',
  fontWeight: 600,
  textAlign: 'center',
};

const floorStyle: React.CSSProperties = {
  fontWeight: 700,
  textAlign: 'center',
};

const seedStyle: React.CSSProperties = {
  color: '#8b949e',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
  fontFamily: 'inherit',
  fontSize: 11,
};

const enemyStyle: React.CSSProperties = {
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
};

const dimStyle: React.CSSProperties = {
  color: '#8b949e',
  textAlign: 'right',
};

const empty: React.CSSProperties = {
  padding: '20px 14px',
  color: '#484f58',
  textAlign: 'center',
  fontSize: 12,
};

function formatDuration(sec: number): string {
  if (sec < 60) return `${sec.toFixed(0)}s`;
  return `${Math.floor(sec / 60)}m${Math.floor(sec % 60).toString().padStart(2, '0')}s`;
}

export const TopRuns = ({ episodes, maxRuns = 5, onSelectEpisode }: TopRunsProps) => {
  const sorted = useMemo(() => {
    return [...episodes]
      .sort((a, b) => {
        if (b.floors_reached !== a.floors_reached) return b.floors_reached - a.floors_reached;
        return a.duration - b.duration;
      })
      .slice(0, maxRuns);
  }, [episodes, maxRuns]);

  return (
    <div style={container}>
      <div style={header}>Top Runs</div>
      {sorted.length === 0 ? (
        <div style={empty}>No episodes yet</div>
      ) : (
        <>
          <div style={colHeader}>
            <span>#</span>
            <span style={{ textAlign: 'center' }}>Floor</span>
            <span>Seed</span>
            <span>Death Enemy</span>
            <span style={{ textAlign: 'right' }}>HP</span>
            <span style={{ textAlign: 'right' }}>Time</span>
          </div>
          {sorted.map((ep, i) => {
            const isHighFloor = ep.floors_reached >= 16;
            const lastCombat = ep.combats?.[ep.combats.length - 1];
            return (
              <div
                key={`${ep.seed}-${ep.episode}`}
                style={{
                  ...rowBase,
                  borderLeft: isHighFloor ? '2px solid #00ff41' : '2px solid transparent',
                }}
                onClick={() => onSelectEpisode?.(ep)}
                onMouseEnter={(e) => {
                  (e.currentTarget as HTMLDivElement).style.background = '#1c2128';
                }}
                onMouseLeave={(e) => {
                  (e.currentTarget as HTMLDivElement).style.background = 'transparent';
                }}
              >
                <span style={rankStyle}>{i + 1}</span>
                <span
                  style={{
                    ...floorStyle,
                    color: isHighFloor ? '#00ff41' : '#c9d1d9',
                  }}
                >
                  F{ep.floors_reached}
                </span>
                <span style={seedStyle} title={ep.seed}>
                  {ep.seed}
                </span>
                <span style={enemyStyle} title={ep.death_enemy ?? undefined}>
                  {ep.death_enemy ?? (lastCombat?.enemy || '--')}
                </span>
                <span style={dimStyle}>
                  {ep.won ? (
                    <span style={{ color: '#00ff41' }}>WIN</span>
                  ) : (
                    `${ep.hp_remaining} HP`
                  )}
                </span>
                <span style={dimStyle}>{formatDuration(ep.duration)}</span>
              </div>
            );
          })}
        </>
      )}
    </div>
  );
};
