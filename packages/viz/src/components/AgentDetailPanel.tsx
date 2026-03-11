import { useEffect } from 'react';
import type { AgentInfo, MCTSResultMsg, AgentEpisodeMsg } from '../types/training';
import { AGENT_NAMES } from '../types/training';
import { CombatTab } from './CombatTab';
import { RunSummaryTab } from './RunSummaryTab';
import { MCTSTab } from './MCTSTab';

export type DetailTab = 'combat' | 'run' | 'mcts';
const TABS: DetailTab[] = ['combat', 'run', 'mcts'];

interface AgentDetailPanelProps {
  agent: AgentInfo;
  combat: any | null;
  mcts: MCTSResultMsg | null;
  episodes: AgentEpisodeMsg[];
  tab: DetailTab;
  onTabChange: (t: DetailTab) => void;
  onClose: () => void;
}

export const AgentDetailPanel = ({
  agent, combat, mcts, episodes, tab, onTabChange, onClose,
}: AgentDetailPanelProps) => {
  const agentAny = agent as any;
  const hpRatio = agent.max_hp > 0 ? agent.hp / agent.max_hp : 0;
  const hpColor = hpRatio > 0.6 ? '#00ff41' : hpRatio > 0.3 ? '#ffb700' : '#ff4444';

  // Tab keyboard cycling: Tab key cycles tabs when detail panel is open
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      const tag = (e.target as HTMLElement).tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA') return;
      if (e.key === 'Tab' && !e.shiftKey) {
        // Only intercept if no modifier used for panel navigation
        // Parent MissionControl handles Tab for focus cycling — we skip here
        // The detail-panel tab cycling is handled by parent
      }
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [tab, onTabChange]);

  // Extract run extras from combat state if available
  const runExtras = combat ? {
    deck: agentAny.deck ?? combat.deck,
    relics: agentAny.relics ?? combat.relics,
    potions: agentAny.potions ?? combat.potions,
    gold: agentAny.gold ?? combat.gold,
  } : undefined;

  return (
    <div style={{
      borderTop: '1px solid #00ff41',
      background: '#161b22',
      flexShrink: 0,
      height: '280px',
      display: 'flex',
      flexDirection: 'column',
      overflow: 'hidden',
    }}>
      {/* Header bar */}
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: '10px',
        padding: '4px 10px',
        borderBottom: '1px solid #30363d',
        background: '#0d1117',
        flexShrink: 0,
      }}>
        <span style={{ fontSize: '11px', fontWeight: 700, color: '#c9d1d9' }}>
          {agent.name || AGENT_NAMES[agent.id]}
        </span>
        <span style={{ fontSize: '10px', color: '#8b949e' }}>
          F{Math.floor(agent.floor)} | {agent.phase}
        </span>
        <span style={{ fontSize: '10px', color: hpColor }}>
          {agent.hp}/{agent.max_hp} HP
        </span>
        {agentAny.stance && agentAny.stance !== 'Neutral' && (
          <span style={{ fontSize: '10px', color: '#8b949e' }}>
            [{agentAny.stance}]
          </span>
        )}
        <div style={{ flex: 1 }} />

        {/* Tab buttons */}
        {TABS.map((t) => (
          <button
            key={t}
            onClick={() => onTabChange(t)}
            style={{
              background: 'none',
              border: 'none',
              borderBottom: tab === t ? '1px solid #00ff41' : '1px solid transparent',
              color: tab === t ? '#00ff41' : '#8b949e',
              fontSize: '10px',
              padding: '2px 8px',
              cursor: 'pointer',
              textTransform: 'uppercase',
              letterSpacing: '0.5px',
              fontFamily: 'inherit',
            }}
          >
            {t}
          </button>
        ))}

        <button
          onClick={onClose}
          style={{
            background: 'none',
            border: 'none',
            color: '#8b949e',
            cursor: 'pointer',
            fontSize: '10px',
            fontFamily: 'inherit',
            padding: '2px 6px',
          }}
        >
          [ESC]
        </button>
      </div>

      {/* Tab content */}
      <div style={{ flex: 1, overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
        {tab === 'combat' && (
          <CombatTab
            combat={combat}
            phase={agent.phase}
            lastAction={agentAny.last_action}
          />
        )}
        {tab === 'run' && (
          <RunSummaryTab
            agent={agent}
            episodes={episodes}
            runExtras={runExtras}
          />
        )}
        {tab === 'mcts' && (
          <MCTSTab mcts={mcts} />
        )}
      </div>
    </div>
  );
};
