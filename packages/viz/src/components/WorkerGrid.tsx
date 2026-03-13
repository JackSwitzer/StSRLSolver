import type { AgentInfo } from '../types/training';
import { AGENT_NAMES } from '../types/training';

// ---- Helpers ----

function hpColor(ratio: number): string {
  if (ratio > 0.6) return '#00ff41';
  if (ratio > 0.3) return '#ffb700';
  return '#ff4444';
}

const PHASE_BG: Record<string, string> = {
  COMBAT: 'rgba(255,68,68,0.15)',
  EVENT: 'rgba(168,85,247,0.15)',
  REST: 'rgba(68,136,255,0.15)',
  SHOP: 'rgba(255,183,0,0.15)',
  MAP: 'rgba(139,148,158,0.08)',
  CHEST: 'rgba(255,183,0,0.10)',
  BOSS: 'rgba(255,68,68,0.25)',
  idle: 'rgba(139,148,158,0.05)',
  starting: 'rgba(139,148,158,0.05)',
  dead: 'rgba(255,68,68,0.05)',
};

const PHASE_COLOR: Record<string, string> = {
  COMBAT: '#ff4444',
  EVENT: '#a855f7',
  REST: '#4488ff',
  SHOP: '#ffb700',
  MAP: '#8b949e',
  CHEST: '#ffb700',
  BOSS: '#ff4444',
  idle: '#3d444d',
  starting: '#3d444d',
  dead: '#ff4444',
};

const PHASE_LABEL: Record<string, string> = {
  COMBAT: 'CMB',
  EVENT: 'EVT',
  REST: 'RST',
  SHOP: 'SHP',
  MAP: 'MAP',
  CHEST: 'CHT',
  BOSS: 'BOS',
  idle: 'IDL',
  starting: '...',
  dead: 'DED',
};

// ---- Component ----

interface WorkerGridProps {
  agents: AgentInfo[];
}

export const WorkerGrid = ({ agents }: WorkerGridProps) => {
  if (agents.length === 0) return null;

  // Compute columns: 4 for <=8, 8 for <=16, cap at 8
  const cols = agents.length <= 8 ? 4 : 8;

  return (
    <div>
      <div style={{
        fontSize: '9px',
        color: '#8b949e',
        textTransform: 'uppercase',
        letterSpacing: '0.8px',
        marginBottom: '6px',
        fontWeight: 600,
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
      }}>
        <span>Workers ({agents.length})</span>
        <span style={{ color: '#3d444d', textTransform: 'none' }}>
          {agents.filter(a => a.phase === 'COMBAT').length} in combat
        </span>
      </div>
      <div style={{
        display: 'grid',
        gridTemplateColumns: `repeat(${cols}, 1fr)`,
        gap: '3px',
      }}>
        {agents.map((agent, idx) => (
          <WorkerTile key={agent.id} agent={agent} index={idx} />
        ))}
      </div>
    </div>
  );
};

// ---- Worker Tile ----

const WorkerTile = ({ agent, index }: { agent: AgentInfo; index: number }) => {
  const hpRatio = agent.max_hp > 0 ? agent.hp / agent.max_hp : 0;
  const hp = hpColor(hpRatio);
  const isDead = agent.hp <= 0 || agent.status === 'dead';
  const phase = agent.phase || 'idle';
  const bg = PHASE_BG[phase] ?? PHASE_BG.idle;
  const phaseCol = PHASE_COLOR[phase] ?? PHASE_COLOR.idle;
  const phaseLabel = PHASE_LABEL[phase] ?? phase.slice(0, 3).toUpperCase();
  const name = agent.name || AGENT_NAMES[index] || `W${index}`;
  const enemyName = (agent as any).enemy_name;

  return (
    <div style={{
      padding: '4px 5px',
      background: bg,
      border: '1px solid #21262d',
      opacity: isDead ? 0.3 : 1,
      display: 'flex',
      flexDirection: 'column',
      gap: '2px',
      fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
      minWidth: 0,
      overflow: 'hidden',
    }}>
      {/* Row 1: name + phase badge */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '3px' }}>
        <span style={{
          fontSize: '8px',
          color: '#c9d1d9',
          fontWeight: 600,
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap',
          flex: 1,
        }}>
          {name}
        </span>
        <span style={{
          fontSize: '7px',
          color: phaseCol,
          fontWeight: 700,
          letterSpacing: '0.3px',
          flexShrink: 0,
        }}>
          {phaseLabel}
        </span>
      </div>

      {/* Row 2: floor + HP text */}
      <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '9px' }}>
        <span style={{ color: '#8b949e' }}>
          {agent.floor > 0 ? `F${Math.floor(agent.floor)}` : '--'}
        </span>
        <span style={{ color: hp, fontFamily: 'monospace', fontSize: '8px' }}>
          {agent.hp}/{agent.max_hp}
        </span>
      </div>

      {/* HP bar */}
      <div style={{ height: '2px', background: '#161b22', overflow: 'hidden' }}>
        <div style={{
          width: `${hpRatio * 100}%`,
          height: '100%',
          background: hp,
          transition: 'width 0.4s linear',
        }} />
      </div>

      {/* Row 3: enemy or seed */}
      <div style={{
        fontSize: '7px',
        color: '#3d444d',
        overflow: 'hidden',
        textOverflow: 'ellipsis',
        whiteSpace: 'nowrap',
      }}>
        {phase === 'COMBAT' && enemyName
          ? <span style={{ color: '#ff4444' }}>{enemyName}</span>
          : <span>{agent.seed?.slice(0, 8) || '---'}</span>
        }
      </div>
    </div>
  );
};
