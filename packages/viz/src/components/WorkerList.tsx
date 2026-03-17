import type { AgentInfo } from '../types/training';

interface WorkerListProps {
  agents: AgentInfo[];
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

const row: React.CSSProperties = {
  display: 'grid',
  gridTemplateColumns: '90px 40px 80px 70px 1fr',
  alignItems: 'center',
  height: 28,
  padding: '0 14px',
  gap: 8,
  fontVariantNumeric: 'tabular-nums',
};

const colHeaderRow: React.CSSProperties = {
  ...row,
  height: 22,
  fontSize: 10,
  color: '#484f58',
  textTransform: 'uppercase',
  letterSpacing: '0.5px',
};

const nameCell: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: 6,
  overflow: 'hidden',
  whiteSpace: 'nowrap',
};

const nameText: React.CSSProperties = {
  fontWeight: 600,
  overflow: 'hidden',
  textOverflow: 'ellipsis',
};

const floorCell: React.CSSProperties = {
  textAlign: 'center',
  fontWeight: 600,
};

const phaseCell: React.CSSProperties = {
  color: '#8b949e',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
  fontSize: 11,
};

const enemyCell: React.CSSProperties = {
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
  fontSize: 11,
};

const emptyState: React.CSSProperties = {
  padding: '20px 14px',
  color: '#484f58',
  textAlign: 'center',
  fontSize: 12,
};

const STANCE_COLORS: Record<string, string> = {
  Calm: '#58a6ff',
  Wrath: '#f85149',
  Divinity: '#d2a038',
  Neutral: '#484f58',
};

function stanceDot(stance: string | undefined): React.CSSProperties {
  const color = STANCE_COLORS[stance ?? 'Neutral'] ?? '#484f58';
  return {
    width: 6,
    height: 6,
    borderRadius: '50%',
    background: color,
    flexShrink: 0,
    boxShadow: stance && stance !== 'Neutral' ? `0 0 4px ${color}` : 'none',
  };
}

function hpColor(ratio: number): string {
  if (ratio > 0.6) return '#3fb950';
  if (ratio > 0.3) return '#d29922';
  return '#f85149';
}

function HPBarSVG({ hp, maxHp }: { hp: number; maxHp: number }) {
  const ratio = maxHp > 0 ? Math.max(0, Math.min(1, hp / maxHp)) : 0;
  const width = 72;
  const height = 8;
  const color = hpColor(ratio);

  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
      <svg width={width} height={height} style={{ flexShrink: 0 }}>
        <rect x={0} y={0} width={width} height={height} rx={2} fill="#21262d" />
        <rect
          x={0}
          y={0}
          width={Math.round(width * ratio)}
          height={height}
          rx={2}
          fill={color}
        />
      </svg>
      <span
        style={{
          fontSize: 10,
          color: '#8b949e',
          fontVariantNumeric: 'tabular-nums',
          whiteSpace: 'nowrap',
          minWidth: 36,
        }}
      >
        {hp}/{maxHp}
      </span>
    </div>
  );
}

function statusColor(status: string): string {
  switch (status) {
    case 'playing':
      return '#c9d1d9';
    case 'won':
      return '#00ff41';
    case 'dead':
      return '#f85149';
    case 'idle':
    case 'starting':
    case 'restarting':
      return '#484f58';
    default:
      return '#8b949e';
  }
}

export const WorkerList = ({ agents }: WorkerListProps) => {
  return (
    <div style={container}>
      <div style={header}>Workers ({agents.length})</div>
      {agents.length === 0 ? (
        <div style={emptyState}>No workers</div>
      ) : (
        <>
          <div style={colHeaderRow}>
            <span>Name</span>
            <span style={{ textAlign: 'center' }}>Floor</span>
            <span>HP</span>
            <span>Phase</span>
            <span>Enemy</span>
          </div>
          {agents.map((agent) => (
            <div
              key={agent.id}
              style={{
                ...row,
                opacity: agent.status === 'idle' || agent.status === 'starting' ? 0.5 : 1,
              }}
            >
              <div style={nameCell}>
                <div style={stanceDot(agent.stance)} title={agent.stance ?? 'Neutral'} />
                <span style={{ ...nameText, color: statusColor(agent.status) }}>
                  {agent.name}
                </span>
              </div>
              <span style={floorCell}>{agent.floor}</span>
              <HPBarSVG hp={agent.hp} maxHp={agent.max_hp} />
              <span style={phaseCell} title={agent.phase}>
                {agent.phase}
              </span>
              <span style={enemyCell} title={agent.enemy_name ?? undefined}>
                {agent.enemy_name ?? '--'}
              </span>
            </div>
          ))}
        </>
      )}
    </div>
  );
};
