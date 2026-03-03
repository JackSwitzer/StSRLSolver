import { useState, useCallback, useMemo } from 'react';
import type { ScreenMode, AgentInfo } from '../types/training';
import { AGENT_NAMES } from '../types/training';
import { useTrainingState } from '../hooks/useTrainingState';
import { useKeyboardNav } from '../hooks/useKeyboardNav';
import { CombatView } from './CombatView';
import { MCTSViz } from './MCTSViz';
import { TrainingStatsView } from './TrainingStatsView';

const COLS = 4;
const SCREENS: ScreenMode[] = ['grid', 'combat', 'map', 'mcts', 'stats'];

const STANCE_COLORS: Record<string, string> = {
  Neutral: '#888', Calm: '#4488ff', Wrath: '#ff4444', Divinity: '#ffdd00',
};

// ---------------------------------------------------------------------------
// AgentCard with live combat info
// ---------------------------------------------------------------------------

const AgentCard = ({
  agent, index, selected, focused, onSelect, onToggleFocus,
}: {
  agent: AgentInfo; index: number; selected: boolean; focused: boolean;
  onSelect: () => void; onToggleFocus: () => void;
}) => {
  const hpRatio = agent.max_hp > 0 ? agent.hp / agent.max_hp : 0;
  const hpColor = hpRatio > 0.6 ? '#44bb44' : hpRatio > 0.3 ? '#ccaa22' : '#cc3333';
  const inCombat = agent.phase === 'COMBAT';
  const enemyHpRatio = (agent as any).enemy_max_hp > 0 ? (agent as any).enemy_hp / (agent as any).enemy_max_hp : 0;

  return (
    <div
      onClick={onSelect}
      onDoubleClick={onToggleFocus}
      style={{
        padding: '10px 12px',
        background: focused ? 'rgba(68,187,68,0.08)' : selected ? 'rgba(233,69,96,0.06)' : 'var(--surface)',
        border: focused ? '2px solid #44bb44' : selected ? '2px solid var(--accent)' : '1px solid var(--border)',
        borderRadius: '8px', cursor: 'pointer',
        opacity: agent.hp > 0 ? 1 : 0.4,
        transition: 'all 0.15s', display: 'flex', flexDirection: 'column', gap: '4px',
      }}
    >
      {/* Row 1: number + name + stance dot */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
        <span style={{ fontSize: '10px', color: '#555', fontFamily: 'monospace', width: '14px' }}>{index + 1}</span>
        <span style={{ fontSize: '13px', fontWeight: 700, flex: 1 }}>{agent.name || AGENT_NAMES[index]}</span>
        {inCombat && (agent as any).stance && (
          <span style={{
            width: '7px', height: '7px', borderRadius: '50%',
            background: STANCE_COLORS[(agent as any).stance] || '#888',
          }} title={(agent as any).stance} />
        )}
        <span style={{
          width: '7px', height: '7px', borderRadius: '50%',
          background: agent.status === 'playing' ? '#44bb44' : agent.status === 'restarting' ? '#ccaa22' : '#555',
        }} />
      </div>

      {/* Row 2: floor + HP */}
      <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '11px' }}>
        <span style={{ color: '#aaa' }}>F{Math.floor(agent.floor)}</span>
        <span style={{ fontFamily: 'monospace', color: hpColor }}>{agent.hp}/{agent.max_hp}</span>
      </div>

      {/* HP bar */}
      <div style={{ height: '4px', background: '#111', borderRadius: '2px', overflow: 'hidden' }}>
        <div style={{ width: `${hpRatio * 100}%`, height: '100%', background: hpColor, borderRadius: '2px', transition: 'width 0.5s' }} />
      </div>

      {/* Combat info (when fighting) */}
      {inCombat && (agent as any).enemy_name && (
        <div style={{ fontSize: '10px', color: '#999', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ color: '#cc6666' }}>{(agent as any).enemy_name}</span>
          <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
            <div style={{ width: '40px', height: '3px', background: '#111', borderRadius: '2px', overflow: 'hidden' }}>
              <div style={{ width: `${enemyHpRatio * 100}%`, height: '100%', background: '#cc3333', transition: 'width 0.5s' }} />
            </div>
            <span style={{ fontFamily: 'monospace', fontSize: '9px' }}>{(agent as any).enemy_hp}</span>
          </div>
        </div>
      )}

      {/* Row 3: phase + wins + episode */}
      <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '10px', color: '#666' }}>
        <span style={{ textTransform: 'uppercase', letterSpacing: '0.3px' }}>
          {inCombat ? `T${(agent as any).turn || '?'} H${(agent as any).hand_size || '?'}` : agent.phase}
        </span>
        <span><span style={{ color: '#44bb44' }}>{agent.wins}W</span> Ep{agent.episode}</span>
      </div>
    </div>
  );
};

// ---------------------------------------------------------------------------
// Focus tabs
// ---------------------------------------------------------------------------

const FocusTabs = ({
  ids, agents, activeIdx, onSelect, onRemove,
}: {
  ids: number[]; agents: AgentInfo[]; activeIdx: number;
  onSelect: (i: number) => void; onRemove: (id: number) => void;
}) => {
  if (ids.length === 0) return null;
  return (
    <div style={{ display: 'flex', gap: '4px', padding: '6px 0', flexWrap: 'wrap' }}>
      {ids.map((id, i) => {
        const a = agents.find((x) => x.id === id);
        const name = a?.name || AGENT_NAMES[id] || `#${id}`;
        const active = i === activeIdx;
        return (
          <button key={id} onClick={() => onSelect(i)} style={{
            padding: '3px 10px', fontSize: '11px', border: 'none', borderRadius: '4px', cursor: 'pointer',
            background: active ? '#44bb44' : 'var(--border)', color: active ? '#000' : '#aaa',
            fontWeight: active ? 700 : 400, display: 'flex', alignItems: 'center', gap: '6px',
          }}>
            {name}
            <span onClick={(e) => { e.stopPropagation(); onRemove(id); }} style={{ cursor: 'pointer', opacity: 0.6, fontSize: '9px' }}>x</span>
          </button>
        );
      })}
    </div>
  );
};

// ---------------------------------------------------------------------------
// TrainingView
// ---------------------------------------------------------------------------

export const TrainingView = () => {
  const { state, connected, toggleFocus, clearFocus, selectAgent, nextFocused, prevFocused, stopTraining, resumeTraining } = useTrainingState();
  const [screenMode, setScreenMode] = useState<ScreenMode>('grid');

  const numAgents = Math.max(state.agents.length, 4);

  const handleFocus = useCallback(() => {
    const agent = state.agents[state.selectedAgentIndex];
    if (agent) {
      toggleFocus(agent.id);
      setScreenMode('combat');
    }
  }, [state.agents, state.selectedAgentIndex, toggleFocus]);

  const handleUnfocus = useCallback(() => {
    clearFocus();
    setScreenMode('grid');
  }, [clearFocus]);

  useKeyboardNav({
    numAgents, columns: COLS,
    selectedIndex: state.selectedAgentIndex,
    screenMode,
    onScreenChange: setScreenMode,
    onAgentChange: selectAgent,
    onFocus: handleFocus,
    onUnfocus: handleUnfocus,
    onNextFocused: nextFocused,
    onPrevFocused: prevFocused,
  });

  const { stats, focusedAgentIds, activeFocusIndex } = state;
  const activeAgentId = focusedAgentIds[activeFocusIndex] ?? null;
  const activeAgent = activeAgentId !== null ? state.agents.find((a) => a.id === activeAgentId) : null;
  const activeCombat = activeAgentId !== null ? state.combatStates?.[activeAgentId] ?? null : null;

  // Focus presets
  const sortedByPerf = useMemo(() =>
    [...state.agents].sort((a, b) => (b.wins - a.wins) || (b.floor - a.floor)),
    [state.agents],
  );

  const focusTop = useCallback((n: number) => {
    clearFocus();
    sortedByPerf.slice(0, n).forEach((a) => toggleFocus(a.id));
    setScreenMode('combat');
  }, [sortedByPerf, clearFocus, toggleFocus]);

  const defaultAgents: AgentInfo[] = state.agents.length > 0
    ? state.agents
    : Array.from({ length: 4 }, (_, i) => ({
        id: i, name: AGENT_NAMES[i], phase: 'starting', floor: 0,
        hp: 72, max_hp: 72, episode: 0, wins: 0, seed: 'Test123', status: 'idle' as const,
      }));

  const isRunning = !!stats;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', background: 'var(--bg)', color: 'var(--text)' }}>
      {/* Header */}
      <header style={{
        display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        padding: '6px 12px', background: 'var(--surface)', borderBottom: '1px solid var(--border)', flexShrink: 0,
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <span style={{ fontSize: '13px', fontWeight: 700 }}>STS RL</span>
          <button onClick={isRunning ? stopTraining : resumeTraining} style={{
            padding: '3px 10px', fontSize: '10px', border: 'none', borderRadius: '3px', cursor: 'pointer',
            background: isRunning ? '#cc3333' : '#2a7a2a', color: '#fff', fontWeight: 600,
          }}>
            {isRunning ? 'Stop' : 'Start'}
          </button>

          {/* Screen tabs */}
          <div style={{ display: 'flex', gap: '2px' }}>
            {SCREENS.map((mode, i) => (
              <button key={mode} onClick={() => setScreenMode(mode)} style={{
                padding: '3px 8px', fontSize: '10px', border: 'none', borderRadius: '3px', cursor: 'pointer',
                background: screenMode === mode ? 'var(--accent)' : 'var(--border)',
                color: screenMode === mode ? '#fff' : '#999', fontWeight: screenMode === mode ? 700 : 400,
              }}>
                {i + 1}:{mode}
              </button>
            ))}
          </div>

          {/* Focus presets */}
          <div style={{ display: 'flex', gap: '2px', marginLeft: '8px' }}>
            <button onClick={() => { clearFocus(); setScreenMode('grid'); }} style={{
              padding: '2px 6px', fontSize: '9px', border: '1px solid var(--border)', borderRadius: '3px',
              cursor: 'pointer', background: 'transparent', color: '#888',
            }}>All</button>
            <button onClick={() => focusTop(1)} style={{
              padding: '2px 6px', fontSize: '9px', border: '1px solid var(--border)', borderRadius: '3px',
              cursor: 'pointer', background: 'transparent', color: '#888',
            }}>Top 1</button>
            <button onClick={() => focusTop(2)} style={{
              padding: '2px 6px', fontSize: '9px', border: '1px solid var(--border)', borderRadius: '3px',
              cursor: 'pointer', background: 'transparent', color: '#888',
            }}>Top 2</button>
          </div>
        </div>

        {/* Stats */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px', fontSize: '11px', color: '#888' }}>
          <span>Eps <b style={{ color: 'var(--text)' }}>{stats?.total_episodes ?? 0}</b></span>
          <span>WR <b style={{ color: (stats?.win_rate ?? 0) > 0 ? '#44bb44' : '#888' }}>
            {stats ? `${(stats.win_rate * 100).toFixed(1)}%` : '0%'}
          </b></span>
          <span>Floor <b style={{ color: 'var(--text)' }}>{stats?.avg_floor?.toFixed(1) ?? '0'}</b></span>
          <span>MCTS <b style={{ color: 'var(--text)' }}>{stats?.mcts_avg_ms?.toFixed(0) ?? '0'}ms</b></span>
          <span style={{ width: '7px', height: '7px', borderRadius: '50%', background: connected ? '#44bb44' : '#cc3333' }} />
        </div>
      </header>

      {/* Body */}
      <div style={{ flex: 1, overflow: 'auto', padding: '10px' }}>
        {/* Focus tabs when agents selected */}
        {focusedAgentIds.length > 0 && screenMode !== 'grid' && (
          <FocusTabs ids={focusedAgentIds} agents={defaultAgents} activeIdx={activeFocusIndex}
            onSelect={(i) => selectAgent(i)} onRemove={(id) => toggleFocus(id)} />
        )}

        {screenMode === 'grid' && (
          <div style={{ display: 'grid', gridTemplateColumns: `repeat(${COLS}, 1fr)`, gap: '8px', maxWidth: '1100px', margin: '0 auto' }}>
            {defaultAgents.map((agent, idx) => (
              <AgentCard key={agent.id} agent={agent} index={idx}
                selected={state.selectedAgentIndex === idx}
                focused={focusedAgentIds.includes(agent.id)}
                onSelect={() => selectAgent(idx)}
                onToggleFocus={() => { toggleFocus(agent.id); setScreenMode('combat'); }}
              />
            ))}
          </div>
        )}

        {screenMode === 'combat' && (
          activeAgent ? (
            <div>
              <div style={{ marginBottom: '8px', display: 'flex', alignItems: 'center', gap: '12px', fontSize: '12px' }}>
                <b style={{ fontSize: '14px' }}>{activeAgent.name}</b>
                <span style={{ color: '#888' }}>Floor {Math.floor(activeAgent.floor)} | {activeAgent.phase}</span>
                {focusedAgentIds.length > 1 && (
                  <>
                    <button onClick={prevFocused} style={{ background: 'var(--border)', border: 'none', color: '#aaa', borderRadius: '3px', padding: '2px 8px', cursor: 'pointer' }}>Prev</button>
                    <button onClick={nextFocused} style={{ background: 'var(--border)', border: 'none', color: '#aaa', borderRadius: '3px', padding: '2px 8px', cursor: 'pointer' }}>Next</button>
                  </>
                )}
              </div>
              {activeCombat ? (
                <CombatView combat={activeCombat} />
              ) : (
                <div style={{ color: '#666', fontSize: '13px', textAlign: 'center', marginTop: '40px' }}>
                  {activeAgent.phase === 'COMBAT'
                    ? 'Loading combat state...'
                    : `${activeAgent.name} is in ${activeAgent.phase} phase`}
                </div>
              )}
            </div>
          ) : (
            <div style={{ color: '#666', fontSize: '13px', textAlign: 'center', marginTop: '40px' }}>
              Select agent with WASD in grid, press Enter or double-click to focus
            </div>
          )
        )}

        {screenMode === 'map' && (
          <div style={{ color: '#666', fontSize: '13px', textAlign: 'center', marginTop: '40px' }}>
            {activeAgent ? `${activeAgent.name} -- Map view coming soon` : 'Focus an agent first'}
          </div>
        )}

        {screenMode === 'mcts' && <MCTSViz result={state.mctsResult} />}

        {screenMode === 'stats' && (
          <TrainingStatsView stats={state.stats} episodes={state.episodes} agents={defaultAgents} />
        )}
      </div>

      {/* Footer */}
      <footer style={{
        display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '16px',
        padding: '5px 12px', background: 'var(--surface)', borderTop: '1px solid var(--border)',
        flexShrink: 0, fontSize: '10px', color: '#555',
      }}>
        <span><Kbd>WASD</Kbd> navigate</span>
        <span><Kbd>Enter</Kbd> focus</span>
        <span><Kbd>Esc</Kbd> back</span>
        <span><Kbd>1-5</Kbd> screens</span>
        <span><Kbd>Tab</Kbd> cycle agents</span>
        {focusedAgentIds.length > 0 && <span style={{ color: '#44bb44' }}>{focusedAgentIds.length} focused</span>}
      </footer>
    </div>
  );
};

const Kbd = ({ children }: { children: React.ReactNode }) => (
  <span style={{
    display: 'inline-block', padding: '1px 4px', background: '#2a2a44',
    borderRadius: '3px', fontFamily: 'monospace', color: '#aaa', border: '1px solid #3a3a5a', fontSize: '10px',
  }}>{children}</span>
);
