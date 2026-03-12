import type { AgentInfo } from '../types/training';
import { AGENT_NAMES } from '../types/training';

const STANCE_COLORS: Record<string, string> = {
  Neutral: '#8b949e',
  Calm: '#4488ff',
  Wrath: '#ff4444',
  Divinity: '#ffb700',
};

const PHASE_ICONS: Record<string, string> = {
  COMBAT: 'X',
  MAP: 'M',
  REST: 'Z',
  SHOP: '$',
  EVENT: '?',
  CHEST: 'C',
  BOSS: 'B',
  starting: '.',
  idle: '-',
  dead: 'D',
};

function hpColor(ratio: number): string {
  if (ratio > 0.6) return '#00ff41';
  if (ratio > 0.3) return '#ffb700';
  return '#ff4444';
}

interface AgentCardProps {
  agent: AgentInfo;
  index: number;
  selected: boolean;
  focused: boolean;
  onSelect: () => void;
  onToggleFocus: () => void;
}

export const AgentCard = ({ agent, index, selected, focused, onSelect, onToggleFocus }: AgentCardProps) => {
  const hpRatio = agent.max_hp > 0 ? agent.hp / agent.max_hp : 0;
  const hp = hpColor(hpRatio);
  const inCombat = agent.phase === 'COMBAT';
  const agentAny = agent as any;
  const enemyHpRatio = agentAny.enemy_max_hp > 0 ? agentAny.enemy_hp / agentAny.enemy_max_hp : 0;
  const isDead = agent.hp <= 0 || agent.status === 'dead';
  const phaseIcon = PHASE_ICONS[agent.phase] ?? '?';

  const borderColor = focused
    ? '#00ff41'
    : selected
    ? '#ffb700'
    : '#30363d';

  const bgColor = focused
    ? 'rgba(0,255,65,0.04)'
    : selected
    ? 'rgba(255,183,0,0.04)'
    : '#0d1117';

  return (
    <div
      onClick={onSelect}
      onDoubleClick={onToggleFocus}
      style={{
        padding: '6px 8px',
        background: bgColor,
        border: `1px solid ${borderColor}`,
        cursor: 'pointer',
        opacity: isDead ? 0.35 : 1,
        display: 'flex',
        flexDirection: 'column',
        gap: '3px',
        fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
        minWidth: 0,
      }}
    >
      {/* Row 1: ID + name + phase + stance */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '5px', overflow: 'hidden' }}>
        <span style={{ fontSize: '9px', color: '#8b949e', width: '16px', flexShrink: 0 }}>
          A{index + 1}
        </span>
        <span style={{ fontSize: '11px', fontWeight: 700, color: '#c9d1d9', flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
          {agent.name || AGENT_NAMES[index]}
        </span>
        <span style={{ fontSize: '9px', color: '#8b949e', flexShrink: 0 }}>
          [{phaseIcon}]
        </span>
        {agentAny.stance && agentAny.stance !== 'Neutral' && (
          <span style={{
            fontSize: '8px',
            color: STANCE_COLORS[agentAny.stance] ?? '#8b949e',
            flexShrink: 0,
            fontWeight: 700,
          }}>
            {agentAny.stance}
          </span>
        )}
      </div>

      {/* Row 2: floor + HP numbers */}
      <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '10px' }}>
        <span style={{ color: '#8b949e' }}>F{Math.floor(agent.floor)}</span>
        <span style={{ color: hp, fontFamily: 'monospace' }}>{agent.hp}/{agent.max_hp}</span>
      </div>

      {/* HP bar */}
      <div style={{ height: '3px', background: '#21262d', overflow: 'hidden' }}>
        <div style={{
          width: `${hpRatio * 100}%`,
          height: '100%',
          background: hp,
          transition: 'width 0.4s linear',
        }} />
      </div>

      {/* Enemy info (combat only) */}
      {inCombat && agentAny.enemy_name && (
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: '9px' }}>
          <span style={{ color: '#ff4444', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: '55%' }}>
            {agentAny.enemy_name}
          </span>
          <div style={{ display: 'flex', alignItems: 'center', gap: '3px', flexShrink: 0 }}>
            <div style={{ width: '36px', height: '2px', background: '#21262d', overflow: 'hidden' }}>
              <div style={{ width: `${enemyHpRatio * 100}%`, height: '100%', background: '#ff4444' }} />
            </div>
            <span style={{ color: '#8b949e', fontSize: '8px' }}>{agentAny.enemy_hp ?? '?'}</span>
          </div>
        </div>
      )}

      {/* Row 3: deck/relics/potions/gold */}
      <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '8px', color: '#8b949e' }}>
        <span>
          {agentAny.deck_size != null && <><span style={{ color: '#4488ff' }}>{agentAny.deck_size}</span>c </>}
          {agentAny.relic_count != null && <><span style={{ color: '#ffb700' }}>{agentAny.relic_count}</span>r </>}
          {agentAny.potion_count != null && <><span style={{ color: '#ff44ff' }}>{agentAny.potion_count}/{agentAny.potion_max ?? 2}</span>p</>}
        </span>
        <span>
          {agentAny.gold != null && <><span style={{ color: '#ffb700' }}>{agentAny.gold}</span>g </>}
          <span style={{ color: '#00ff41' }}>{agent.wins}W</span>
        </span>
      </div>
      {/* Row 4: turn/hand or status + episode */}
      <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '8px', color: '#8b949e' }}>
        <span>
          {inCombat
            ? `T${agentAny.turn ?? '?'} H${agentAny.hand_size ?? '?'}`
            : agent.status}
        </span>
        <span>Ep{agent.episode}</span>
      </div>
    </div>
  );
};
