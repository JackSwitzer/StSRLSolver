import type { AgentEpisodeMsg, CombatSummary, DecisionSummary } from '../types/training';
import { Sparkline } from './Sparkline';

interface ReplayTabProps {
  episodes: AgentEpisodeMsg[];
  agentId: number;
}

const SectionHeader = ({ text }: { text: string }) => (
  <div style={{
    fontSize: '9px', color: '#8b949e', textTransform: 'uppercase',
    letterSpacing: '0.5px', marginBottom: '4px',
  }}>
    {text}
  </div>
);

const CombatRow = ({ combat, isLast }: { combat: CombatSummary; isLast: boolean }) => {
  const hpColor = combat.hp_lost === 0 ? '#00ff41' : combat.hp_lost > 15 ? '#ff4444' : '#ffb700';
  return (
    <div style={{
      display: 'flex', gap: '6px', fontSize: '10px', padding: '2px 0',
      borderLeft: isLast ? '2px solid #ff4444' : '2px solid #30363d',
      paddingLeft: '6px',
    }}>
      <span style={{ color: '#8b949e', fontFamily: 'monospace', width: '24px', flexShrink: 0 }}>F{combat.floor}</span>
      <span style={{ color: '#c9d1d9', flex: 1 }}>{combat.enemy}</span>
      <span style={{ color: hpColor, fontFamily: 'monospace', width: '36px', textAlign: 'right', flexShrink: 0 }}>
        -{combat.hp_lost}hp
      </span>
      <span style={{ color: '#8b949e', fontFamily: 'monospace', width: '24px', textAlign: 'right', flexShrink: 0 }}>
        {combat.turns}t
      </span>
    </div>
  );
};

const DecisionRow = ({ decision }: { decision: DecisionSummary }) => {
  const typeColor = decision.type === 'path' ? '#4488ff'
    : decision.type === 'rest' ? '#00ff41'
    : '#ffb700';
  return (
    <div style={{ display: 'flex', gap: '6px', fontSize: '10px', padding: '1px 0' }}>
      <span style={{ color: typeColor, fontFamily: 'monospace', width: '36px', flexShrink: 0, textTransform: 'uppercase' }}>
        {decision.type}
      </span>
      <span style={{ color: '#8b949e', fontFamily: 'monospace', width: '24px', flexShrink: 0 }}>F{decision.floor}</span>
      <span style={{ color: '#c9d1d9', flex: 1 }}>{decision.choice}</span>
    </div>
  );
};

const DeckChangeRow = ({ change }: { change: string }) => {
  const isAdd = change.startsWith('+');
  return (
    <span style={{
      fontSize: '10px',
      color: isAdd ? '#00ff41' : '#ff4444',
      fontFamily: 'monospace',
    }}>
      {change}
    </span>
  );
};

export const ReplayTab = ({ episodes, agentId }: ReplayTabProps) => {
  // Get the most recent episode for this agent that has rich data
  const agentEps = episodes.filter((e) => e.agent_id === agentId);
  const latest = agentEps.find((e) => e.combats && e.combats.length > 0) ?? agentEps[0];

  if (!latest) {
    return (
      <div style={{ padding: '12px', color: '#3d444d', fontSize: '10px', textAlign: 'center' }}>
        No episode data yet
      </div>
    );
  }

  const combats = latest.combats ?? [];
  const decisions = latest.decisions ?? [];
  const hpHistory = latest.hp_history ?? [];
  const deckChanges = latest.deck_changes ?? [];

  return (
    <div style={{
      display: 'grid',
      gridTemplateColumns: '1.2fr 1fr 0.8fr',
      gap: '0',
      height: '100%',
      overflow: 'hidden',
    }}>
      {/* Column 1: Combat log */}
      <div style={{ borderRight: '1px solid #21262d', padding: '6px 8px', overflow: 'auto' }}>
        <SectionHeader text={`Combats (${combats.length}) ${latest.won ? '— WON' : `— Died F${latest.death_floor ?? latest.floors_reached}`}`} />
        {combats.length === 0 ? (
          <span style={{ fontSize: '10px', color: '#3d444d' }}>No combat data</span>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column' }}>
            {combats.map((c, i) => (
              <CombatRow key={i} combat={c} isLast={!latest.won && i === combats.length - 1} />
            ))}
          </div>
        )}

        {/* HP over run sparkline */}
        {hpHistory.length >= 2 && (
          <div style={{ marginTop: '8px' }}>
            <div style={{ fontSize: '9px', color: '#8b949e', marginBottom: '3px' }}>HP over run</div>
            <Sparkline data={hpHistory} width={180} height={28} color="#00ff41" />
          </div>
        )}
      </div>

      {/* Column 2: Decisions */}
      <div style={{ borderRight: '1px solid #21262d', padding: '6px 8px', overflow: 'auto' }}>
        <SectionHeader text={`Decisions (${decisions.length})`} />
        {decisions.length === 0 ? (
          <span style={{ fontSize: '10px', color: '#3d444d' }}>No decision data</span>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column' }}>
            {decisions.map((d, i) => (
              <DecisionRow key={i} decision={d} />
            ))}
          </div>
        )}

        {/* Episode stats */}
        <div style={{ marginTop: '8px', display: 'flex', flexDirection: 'column', gap: '2px' }}>
          <SectionHeader text="Episode" />
          <div style={{ fontSize: '10px', display: 'flex', justifyContent: 'space-between' }}>
            <span style={{ color: '#8b949e' }}>Seed</span>
            <span style={{ color: '#c9d1d9', fontFamily: 'monospace' }}>{latest.seed}</span>
          </div>
          <div style={{ fontSize: '10px', display: 'flex', justifyContent: 'space-between' }}>
            <span style={{ color: '#8b949e' }}>Floor</span>
            <span style={{ color: '#4488ff', fontFamily: 'monospace' }}>{latest.floors_reached}</span>
          </div>
          <div style={{ fontSize: '10px', display: 'flex', justifyContent: 'space-between' }}>
            <span style={{ color: '#8b949e' }}>Duration</span>
            <span style={{ color: '#c9d1d9', fontFamily: 'monospace' }}>{latest.duration.toFixed(1)}s</span>
          </div>
          <div style={{ fontSize: '10px', display: 'flex', justifyContent: 'space-between' }}>
            <span style={{ color: '#8b949e' }}>Steps</span>
            <span style={{ color: '#c9d1d9', fontFamily: 'monospace' }}>{latest.total_steps}</span>
          </div>
        </div>
      </div>

      {/* Column 3: Deck changes */}
      <div style={{ padding: '6px 8px', overflow: 'auto' }}>
        <SectionHeader text={`Deck Changes (${deckChanges.length})`} />
        {deckChanges.length === 0 ? (
          <span style={{ fontSize: '10px', color: '#3d444d' }}>No changes</span>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1px' }}>
            {deckChanges.map((c, i) => (
              <DeckChangeRow key={i} change={c} />
            ))}
          </div>
        )}

        {/* Deck/Relic counts */}
        {(latest.deck_size || latest.relic_count) && (
          <div style={{ marginTop: '8px', display: 'flex', flexDirection: 'column', gap: '2px' }}>
            <SectionHeader text="Final State" />
            {latest.deck_size !== undefined && (
              <div style={{ fontSize: '10px', display: 'flex', justifyContent: 'space-between' }}>
                <span style={{ color: '#8b949e' }}>Deck</span>
                <span style={{ color: '#c9d1d9', fontFamily: 'monospace' }}>{latest.deck_size} cards</span>
              </div>
            )}
            {latest.relic_count !== undefined && (
              <div style={{ fontSize: '10px', display: 'flex', justifyContent: 'space-between' }}>
                <span style={{ color: '#8b949e' }}>Relics</span>
                <span style={{ color: '#ffb700', fontFamily: 'monospace' }}>{latest.relic_count}</span>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
