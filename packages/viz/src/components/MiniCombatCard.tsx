import type { AgentInfo, CombatMiniSummary } from '../types/training';

// ---- Constants ----

const STANCE_COLORS: Record<string, string> = {
  Neutral: '#8b949e',
  Calm: '#4488ff',
  Wrath: '#ff4444',
  Divinity: '#ffb700',
};

const PHASE_CONFIG: Record<string, { label: string; color: string; bg: string }> = {
  COMBAT:   { label: 'COMBAT', color: '#ff4444', bg: 'rgba(255,68,68,0.15)' },
  EVENT:    { label: '?',      color: '#ffb700', bg: 'rgba(255,183,0,0.15)' },
  SHOP:     { label: '$',      color: '#4488ff', bg: 'rgba(68,136,255,0.15)' },
  REST:     { label: '+',      color: '#00ff41', bg: 'rgba(0,255,65,0.15)' },
  MAP:      { label: '~',      color: '#8b949e', bg: 'rgba(139,148,158,0.10)' },
  CHEST:    { label: 'C',      color: '#ffb700', bg: 'rgba(255,183,0,0.15)' },
  BOSS:     { label: 'BOSS',   color: '#ff4444', bg: 'rgba(255,68,68,0.20)' },
  idle:     { label: 'IDLE',   color: '#3d444d', bg: 'rgba(61,68,77,0.10)' },
  starting: { label: '...',    color: '#8b949e', bg: 'rgba(139,148,158,0.10)' },
  dead:     { label: 'DEAD',   color: '#ff4444', bg: 'rgba(255,68,68,0.10)' },
};

// ---- Helpers ----

function hpColor(ratio: number): string {
  if (ratio > 0.6) return '#00ff41';
  if (ratio > 0.3) return '#ffb700';
  return '#ff4444';
}

function hpGradient(ratio: number): string {
  if (ratio > 0.6) return 'linear-gradient(90deg, #00cc33, #00ff41)';
  if (ratio > 0.3) return 'linear-gradient(90deg, #cc8800, #ffb700)';
  return 'linear-gradient(90deg, #cc2222, #ff4444)';
}

// ---- Component ----

interface MiniCombatCardProps {
  agent: AgentInfo;
  combatSummary?: CombatMiniSummary | null;
  selected: boolean;
  onClick: () => void;
}

export const MiniCombatCard = ({ agent, combatSummary, selected, onClick }: MiniCombatCardProps) => {
  const hpRatio = agent.max_hp > 0 ? agent.hp / agent.max_hp : 0;
  const isDead = agent.hp <= 0 || agent.status === 'dead';
  const phase = PHASE_CONFIG[agent.phase] ?? PHASE_CONFIG.idle;

  // Combat info: prefer combatSummary, fall back to flat agent fields
  const cs = combatSummary;
  const enemyName = cs?.enemy_name ?? agent.enemy_name;
  const enemyHp = cs?.enemy_hp ?? agent.enemy_hp;
  const enemyMaxHp = cs?.enemy_max_hp ?? agent.enemy_max_hp;
  const stance = cs?.stance ?? agent.stance;
  const handSize = cs?.hand_size ?? agent.hand_size;
  const energy = cs?.energy ?? 0;
  const maxEnergy = cs?.max_energy ?? 3;
  const inCombat = agent.phase === 'COMBAT' || agent.phase === 'BOSS';
  const act = agent.act ?? (Math.ceil(agent.floor / 17) || 1);

  return (
    <div
      onClick={onClick}
      style={{
        width: '180px',
        padding: '7px 9px',
        background: '#161b22',
        border: `1px solid ${selected ? '#00ff41' : '#30363d'}`,
        boxShadow: selected ? '0 0 8px rgba(0,255,65,0.15)' : 'none',
        cursor: 'pointer',
        opacity: isDead ? 0.35 : 1,
        display: 'flex',
        flexDirection: 'column',
        gap: '4px',
        fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
        transition: 'border-color 0.15s, box-shadow 0.15s',
        boxSizing: 'border-box',
      }}
    >
      {/* Row 1: Name + Floor/Act badge */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <span style={{
          fontSize: '11px',
          fontWeight: 700,
          color: '#c9d1d9',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap',
          flex: 1,
        }}>
          {agent.name}
        </span>
        <span style={{
          fontSize: '9px',
          color: '#8b949e',
          background: '#21262d',
          padding: '1px 4px',
          flexShrink: 0,
          marginLeft: '4px',
        }}>
          F{Math.floor(agent.floor)} A{act}
        </span>
      </div>

      {/* Row 2: HP bar */}
      <div>
        <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '9px', marginBottom: '2px' }}>
          <span style={{ color: '#8b949e' }}>HP</span>
          <span style={{ color: hpColor(hpRatio), fontWeight: 600 }}>
            {agent.hp}/{agent.max_hp}
          </span>
        </div>
        <div style={{ height: '4px', background: '#21262d', overflow: 'hidden' }}>
          <div style={{
            width: `${hpRatio * 100}%`,
            height: '100%',
            background: hpGradient(hpRatio),
            transition: 'width 0.4s linear',
          }} />
        </div>
      </div>

      {/* Row 3: Phase pill */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '5px' }}>
        <span style={{
          fontSize: '8px',
          fontWeight: 700,
          color: phase.color,
          background: phase.bg,
          padding: '1px 5px',
          letterSpacing: '0.5px',
        }}>
          {phase.label}
        </span>

        {/* Stance dot (combat only) */}
        {inCombat && stance && stance !== 'Neutral' && (
          <span style={{
            width: '6px',
            height: '6px',
            borderRadius: '50%',
            background: STANCE_COLORS[stance] ?? '#8b949e',
            display: 'inline-block',
            boxShadow: `0 0 4px ${STANCE_COLORS[stance] ?? '#8b949e'}`,
          }} />
        )}
      </div>

      {/* Row 4: Enemy info (combat only) */}
      {inCombat && enemyName && (
        <div style={{ fontSize: '9px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{
              color: '#ff4444',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
              maxWidth: '60%',
            }}>
              {enemyName}
            </span>
            <span style={{ color: '#8b949e', fontSize: '8px', flexShrink: 0 }}>
              {enemyHp ?? '?'}/{enemyMaxHp ?? '?'}
            </span>
          </div>
          {/* Enemy HP bar */}
          <div style={{ height: '2px', background: '#21262d', overflow: 'hidden', marginTop: '2px' }}>
            <div style={{
              width: `${enemyMaxHp && enemyMaxHp > 0 ? ((enemyHp ?? 0) / enemyMaxHp) * 100 : 0}%`,
              height: '100%',
              background: '#ff4444',
              transition: 'width 0.3s linear',
            }} />
          </div>
        </div>
      )}

      {/* Row 5: Hand + Energy (combat only) */}
      {inCombat && (
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '9px', color: '#8b949e' }}>
          <span>H:{handSize ?? '?'}</span>
          {/* Energy dots */}
          <span style={{ display: 'flex', alignItems: 'center', gap: '2px' }}>
            {Array.from({ length: maxEnergy }, (_, i) => (
              <span key={i} style={{
                width: '5px',
                height: '5px',
                borderRadius: '50%',
                background: i < energy ? '#ffb700' : '#21262d',
                border: `1px solid ${i < energy ? '#ffb700' : '#30363d'}`,
                display: 'inline-block',
              }} />
            ))}
          </span>
        </div>
      )}

      {/* Row 6: Episode count */}
      <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '8px', color: '#3d444d', marginTop: '1px' }}>
        <span>
          <span style={{ color: '#00ff41' }}>{agent.wins}W</span>
          {' '}Ep{agent.episode}
        </span>
        <span>{agent.seed.slice(0, 6)}</span>
      </div>
    </div>
  );
};
