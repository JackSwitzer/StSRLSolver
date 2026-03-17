import { useMemo } from 'react';

// ---- Types ----

export interface RunSummary {
  id: string;
  label: string;
  totalGames: number;
  avgFloor: number;
  maxFloor: number;
  winRate: number;
  durationHours: number;
  config?: Record<string, any>;
}

interface RunHistoryProps {
  runs: RunSummary[];
  selectedId: string | null;
  onSelect: (id: string) => void;
}

// ---- Helpers ----

function fmtCount(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

function fmtDuration(hours: number): string {
  if (hours >= 24) return `${(hours / 24).toFixed(1)}d`;
  if (hours >= 1) return `${hours.toFixed(1)}h`;
  return `${Math.round(hours * 60)}m`;
}

// ---- Styles ----

const tableStyle: React.CSSProperties = {
  width: '100%',
  borderCollapse: 'collapse',
  fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
  fontSize: '11px',
};

const thStyle: React.CSSProperties = {
  textAlign: 'left',
  padding: '6px 8px',
  fontSize: '9px',
  color: '#8b949e',
  textTransform: 'uppercase',
  letterSpacing: '0.5px',
  borderBottom: '1px solid #30363d',
  fontWeight: 600,
  whiteSpace: 'nowrap',
};

const thRightStyle: React.CSSProperties = {
  ...thStyle,
  textAlign: 'right',
};

// ---- Component ----

export const RunHistory = ({ runs, selectedId, onSelect }: RunHistoryProps) => {
  const sortedRuns = useMemo(
    () => [...runs].sort((a, b) => b.totalGames - a.totalGames),
    [runs],
  );

  if (runs.length === 0) {
    return (
      <div style={{
        padding: '24px 16px',
        textAlign: 'center',
        color: '#3d444d',
        fontSize: '11px',
        fontFamily: "'JetBrains Mono', monospace",
      }}>
        No archived runs
      </div>
    );
  }

  return (
    <div style={{ overflow: 'auto' }}>
      <table style={tableStyle}>
        <thead>
          <tr>
            <th style={{ ...thStyle, width: '16px', padding: '6px 4px' }} />
            <th style={thStyle}>Label</th>
            <th style={thRightStyle}>Games</th>
            <th style={thRightStyle}>Avg Floor</th>
            <th style={thRightStyle}>Max Floor</th>
            <th style={thRightStyle}>WR%</th>
            <th style={thRightStyle}>Duration</th>
          </tr>
        </thead>
        <tbody>
          {sortedRuns.map((run) => {
            const isSelected = run.id === selectedId;
            return (
              <tr
                key={run.id}
                onClick={() => onSelect(run.id)}
                style={{
                  cursor: 'pointer',
                  background: isSelected ? 'rgba(0,255,65,0.06)' : 'transparent',
                  borderLeft: isSelected ? '2px solid #00ff41' : '2px solid transparent',
                  transition: 'background 0.15s ease',
                }}
                onMouseEnter={(e) => {
                  if (!isSelected) e.currentTarget.style.background = 'rgba(139,148,158,0.06)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = isSelected ? 'rgba(0,255,65,0.06)' : 'transparent';
                }}
              >
                {/* Radio indicator */}
                <td style={{ padding: '5px 4px', verticalAlign: 'middle' }}>
                  <div style={{
                    width: 10,
                    height: 10,
                    borderRadius: '50%',
                    border: isSelected ? '2px solid #00ff41' : '2px solid #30363d',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}>
                    {isSelected && (
                      <div style={{
                        width: 4,
                        height: 4,
                        borderRadius: '50%',
                        background: '#00ff41',
                      }} />
                    )}
                  </div>
                </td>

                {/* Label */}
                <td style={{
                  padding: '5px 8px',
                  color: isSelected ? '#00ff41' : '#c9d1d9',
                  fontWeight: isSelected ? 700 : 400,
                  whiteSpace: 'nowrap',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  maxWidth: '160px',
                }}>
                  {run.label}
                </td>

                {/* Games */}
                <td style={{ padding: '5px 8px', textAlign: 'right', color: '#c9d1d9' }}>
                  {fmtCount(run.totalGames)}
                </td>

                {/* Avg Floor */}
                <td style={{
                  padding: '5px 8px',
                  textAlign: 'right',
                  color: run.avgFloor >= 15 ? '#00ff41' : run.avgFloor >= 8 ? '#ffb700' : '#ff4444',
                  fontWeight: 600,
                }}>
                  {run.avgFloor.toFixed(1)}
                </td>

                {/* Max Floor */}
                <td style={{
                  padding: '5px 8px',
                  textAlign: 'right',
                  color: run.maxFloor >= 51 ? '#00ff41' : run.maxFloor >= 34 ? '#ffb700' : '#c9d1d9',
                }}>
                  {run.maxFloor}
                </td>

                {/* Win Rate */}
                <td style={{
                  padding: '5px 8px',
                  textAlign: 'right',
                  color: run.winRate > 0 ? '#00ff41' : '#3d444d',
                  fontWeight: run.winRate > 0 ? 700 : 400,
                }}>
                  {(run.winRate * 100).toFixed(1)}%
                </td>

                {/* Duration */}
                <td style={{
                  padding: '5px 8px',
                  textAlign: 'right',
                  color: '#8b949e',
                }}>
                  {fmtDuration(run.durationHours)}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
};
