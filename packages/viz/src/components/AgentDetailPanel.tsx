import { useEffect } from 'react';
import type { AgentInfo, MCTSResultMsg, PlannerResultMsg, AgentEpisodeMsg, DeathStats } from '../types/training';
import { AGENT_NAMES } from '../types/training';
import { CombatTab } from './CombatTab';
import { RunSummaryTab } from './RunSummaryTab';
import { MCTSTab } from './MCTSTab';
import { ReplayTab } from './ReplayTab';
import { DeathMapPanel } from './DeathMapPanel';
import { DecisionLogPanel } from './DecisionLogPanel';
import { MapPanel } from './MapPanel';
import type { MapData } from './MapPanel';

export type DetailTab = 'combat' | 'run' | 'map' | 'mcts' | 'decisions' | 'replay' | 'deaths';
const TABS: DetailTab[] = ['combat', 'run', 'map', 'mcts', 'decisions', 'replay', 'deaths'];

interface AgentDetailPanelProps {
  agent: AgentInfo;
  combat: any | null;
  mapData: MapData | null;
  runState: { deck: any[]; relics: any[]; potions: any[]; gold: number } | null;
  mcts: MCTSResultMsg | null;
  planner: PlannerResultMsg | null;
  episodes: AgentEpisodeMsg[];
  deathStats: DeathStats;
  tab: DetailTab;
  onTabChange: (t: DetailTab) => void;
  onClose: () => void;
}

export const AgentDetailPanel = ({
  agent, combat, mapData, runState, mcts, planner, episodes, deathStats, tab, onTabChange, onClose,
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
        // Parent MissionControl handles Tab for focus cycling
      }
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [tab, onTabChange]);

  // Extract run extras from dedicated run state, combat state, or agent data
  const deck = runState?.deck ?? agentAny.deck ?? combat?.deck;
  const relics = runState?.relics ?? agentAny.relics ?? combat?.relics;
  const potions = runState?.potions ?? agentAny.potions ?? combat?.potions;
  const gold = runState?.gold ?? agentAny.gold ?? combat?.gold;
  const runExtras = (deck || relics || potions || gold != null) ? {
    deck, relics, potions, gold,
  } : undefined;

  return (
    <div style={{
      borderTop: '1px solid #00ff41',
      background: '#161b22',
      flex: 1,
      minHeight: 0,
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
            {t === 'decisions' ? 'decide' : t}
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
        {tab === 'map' && (
          <MapPanel mapData={mapData} agentName={agent.name || AGENT_NAMES[agent.id]} />
        )}
        {tab === 'mcts' && (
          <MCTSTab mcts={mcts} planner={planner} />
        )}
        {tab === 'decisions' && (
          <DecisionLogPanel episodes={episodes} agentId={agent.id} />
        )}
        {tab === 'replay' && (
          <ReplayTab episodes={episodes} agentId={agent.id} />
        )}
        {tab === 'deaths' && (
          <DeathMapPanel deathStats={deathStats} episodes={episodes} />
        )}
      </div>
    </div>
  );
};
