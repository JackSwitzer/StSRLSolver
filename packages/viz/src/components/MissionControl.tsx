import { useState, useCallback, useEffect, useMemo, useRef } from 'react';
import type { AgentInfo, CombatMiniSummary } from '../types/training';
import { AGENT_NAMES } from '../types/training';
import { useTrainingState } from '../hooks/useTrainingState';
import { AgentCard } from './AgentCard';
import { MultiAgentView } from './MultiAgentView';
import { ControlPanel } from './ControlPanel';
import { StatsOverviewPanel } from './StatsOverviewPanel';
import { AgentDetailPanel } from './AgentDetailPanel';
import { CombatFeedView } from './CombatFeedView';
import { StatsView } from './StatsView';
import { TrainingMetricsView } from './TrainingMetricsView';
import type { DetailTab } from './AgentDetailPanel';

// ---- Types ----

type TopView = 'dashboard' | 'feed' | 'stats_view' | 'training_view';

// ---- Sub-components ----

const StatBlock = ({ label, value, color = '#c9d1d9', sub }: {
  label: string; value: string; color?: string; sub?: string;
}) => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '1px' }}>
    <span style={{ fontSize: '8px', color: '#8b949e', textTransform: 'uppercase', letterSpacing: '0.5px' }}>{label}</span>
    <span style={{ fontSize: '13px', fontWeight: 700, color, fontFamily: 'monospace' }}>{value}</span>
    {sub && <span style={{ fontSize: '8px', color: '#8b949e' }}>{sub}</span>}
  </div>
);

const ViewTab = ({ label, shortcut, active, onClick }: {
  label: string; shortcut: string; active: boolean; onClick: () => void;
}) => (
  <button
    onClick={onClick}
    style={{
      background: active ? 'rgba(0,255,65,0.1)' : 'transparent',
      border: active ? '1px solid #00ff41' : '1px solid #30363d',
      color: active ? '#00ff41' : '#8b949e',
      padding: '2px 8px',
      fontSize: '9px',
      cursor: 'pointer',
      fontFamily: 'inherit',
      letterSpacing: '0.3px',
      display: 'flex',
      alignItems: 'center',
      gap: '4px',
    }}
  >
    <span style={{
      fontSize: '8px',
      padding: '0 2px',
      background: active ? 'rgba(0,255,65,0.15)' : '#21262d',
      border: `1px solid ${active ? '#00ff41' : '#30363d'}`,
      color: active ? '#00ff41' : '#c9d1d9',
    }}>{shortcut}</span>
    {label}
  </button>
);

// ---- Main MissionControl ----

export const MissionControl = () => {
  const {
    state, connected, toggleFocus, clearFocus, selectAgent,
    nextFocused, prevFocused, stopTraining, resumeTraining, sendCommand, sendControl, sendMsg,
  } = useTrainingState();

  const [showControl, setShowControl] = useState(false);
  const [showDetail, setShowDetail] = useState(false);
  const [detailTab, setDetailTab] = useState<DetailTab>('run');
  const [numAgents, _setNumAgents] = useState(8);
  const [viewMode, setViewMode] = useState<'grid' | 'live'>('grid');
  const [topView, setTopView] = useState<TopView>('dashboard');

  const { stats, agents, episodes, focusedAgentIds, selectedAgentIndex,
          combatStates, mapStates, runStates, floorHistory, lossHistory, winHistory,
          systemStats, mctsResult, plannerResult, deathStats } = state;

  // Default placeholder agents when none connected
  const displayAgents: AgentInfo[] = agents.length > 0
    ? agents
    : Array.from({ length: numAgents }, (_, i) => ({
        id: i,
        name: AGENT_NAMES[i],
        phase: 'idle',
        floor: 0,
        hp: 72,
        max_hp: 72,
        episode: 0,
        wins: 0,
        seed: '--------',
        status: 'idle' as const,
      }));

  // Selected agent (for detail panel)
  const selectedAgent = displayAgents[selectedAgentIndex] ?? null;
  const selectedCombat = selectedAgent ? combatStates[selectedAgent.id] ?? null : null;
  const selectedMap = selectedAgent ? mapStates[selectedAgent.id] ?? null : null;
  const selectedRunState = selectedAgent ? runStates[selectedAgent.id] ?? null : null;

  // Auto-focus selected agent when detail panel opens so we get combat state
  const selectedId = selectedAgent?.id ?? -1;
  const prevShowDetailRef = useRef(false);
  useEffect(() => {
    if (showDetail && !prevShowDetailRef.current && selectedId >= 0) {
      if (!focusedAgentIds.includes(selectedId)) {
        toggleFocus(selectedId);
      }
    }
    prevShowDetailRef.current = showDetail;
  }, [showDetail, selectedId]);

  // Keyboard handling
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      const tag = (e.target as HTMLElement).tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA') return;

      const key = e.key;
      const n = displayAgents.length;
      const cols = viewMode === 'live' ? Math.min(4, n) : Math.min(n, 4);

      // Escape: close overlay, then detail, then back to dashboard
      if (key === 'Escape' || key === 'q' || key === 'Q') {
        e.preventDefault();
        if (showControl) { setShowControl(false); return; }
        if (showDetail) { setShowDetail(false); return; }
        if (topView !== 'dashboard') { setTopView('dashboard'); return; }
        clearFocus();
        return;
      }

      // Space: play/pause toggle
      if (key === ' ') {
        e.preventDefault();
        if (state.paused) resumeTraining(); else stopTraining();
        return;
      }

      // View switching: F = Feed, T = Training
      // D and S have grid conflicts, handle carefully
      if (key === 'f' || key === 'F') {
        e.preventDefault();
        setTopView('feed');
        setShowDetail(false);
        return;
      }
      if (key === 't' || key === 'T') {
        e.preventDefault();
        setTopView('training_view');
        setShowDetail(false);
        return;
      }

      // C = Control panel toggle
      if (key === 'c' || key === 'C') {
        e.preventDefault();
        setShowControl((v) => !v);
        return;
      }

      // V = toggle grid/live view mode
      if (key === 'v' || key === 'V') {
        e.preventDefault();
        setViewMode((v) => v === 'grid' ? 'live' : 'grid');
        return;
      }

      // 1-8: Select agent directly
      const num = parseInt(key, 10);
      if (num >= 1 && num <= 8 && num <= n) {
        e.preventDefault();
        selectAgent(num - 1);
        return;
      }

      // [ / ]: Prev/Next agent
      if (key === '[') {
        e.preventDefault();
        selectAgent(Math.max(0, selectedAgentIndex - 1));
        return;
      }
      if (key === ']') {
        e.preventDefault();
        selectAgent(Math.min(n - 1, selectedAgentIndex + 1));
        return;
      }

      // Enter/E: toggle detail panel
      if (key === 'Enter' || key === 'e' || key === 'E') {
        e.preventDefault();
        setShowDetail((v) => !v);
        return;
      }

      // Arrow keys and WASD for grid navigation (only when in dashboard)
      if (topView === 'dashboard' && !showDetail) {
        if (key === 'ArrowUp' || key === 'w' || key === 'W') {
          e.preventDefault();
          selectAgent(Math.max(0, selectedAgentIndex - cols));
          return;
        }
        if (key === 'ArrowDown' || key === 's' || key === 'S') {
          e.preventDefault();
          selectAgent(Math.min(n - 1, selectedAgentIndex + cols));
          return;
        }
        if (key === 'ArrowLeft' || key === 'a' || key === 'A') {
          e.preventDefault();
          selectAgent(Math.max(0, selectedAgentIndex - 1));
          return;
        }
        if (key === 'ArrowRight' || key === 'd' || key === 'D') {
          e.preventDefault();
          selectAgent(Math.min(n - 1, selectedAgentIndex + 1));
          return;
        }
      }

      // When NOT in dashboard with grid nav, D = Dashboard, S = Stats
      if (topView !== 'dashboard' || showDetail) {
        if (key === 'd' || key === 'D') {
          e.preventDefault();
          setTopView('dashboard');
          setShowDetail(false);
          return;
        }
        if (key === 's' || key === 'S') {
          e.preventDefault();
          setTopView('stats_view');
          setShowDetail(false);
          return;
        }
      }

      // Tab: cycle detail tabs or focused agents
      if (key === 'Tab') {
        e.preventDefault();
        if (showDetail) {
          const tabs: DetailTab[] = ['combat', 'run', 'map', 'mcts', 'decisions', 'replay', 'deaths'];
          const cur = tabs.indexOf(detailTab);
          const next = e.shiftKey
            ? (cur - 1 + tabs.length) % tabs.length
            : (cur + 1) % tabs.length;
          setDetailTab(tabs[next]);
        } else {
          if (e.shiftKey) prevFocused(); else nextFocused();
        }
        return;
      }
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [
    displayAgents.length, selectedAgentIndex, showControl, showDetail, detailTab, viewMode, topView,
    stats, state.paused, selectAgent, clearFocus, stopTraining, resumeTraining, nextFocused, prevFocused, setDetailTab,
  ]);

  const isRunning = !!stats && !state.paused;
  const totalEpisodes = stats?.total_episodes ?? 0;
  const epsPerMin = stats?.eps_per_min ?? 0;
  const avgFloor = stats?.avg_floor ?? 0;
  const mctsMs = stats?.mcts_avg_ms ?? 0;

  // Build combat summaries map from agent data for MultiAgentView
  const combatSummaries = useMemo(() => {
    const map: Record<number, CombatMiniSummary> = {};
    for (const a of displayAgents) {
      if (a.combat_summary) map[a.id] = a.combat_summary;
    }
    return map;
  }, [displayAgents]);

  const handleStart = useCallback((config: { num_agents: number; mcts_sims: number; ascension: number }) => {
    sendMsg({ type: 'training_start', config: { ...config, seed: 'Test123' } });
    setShowControl(false);
  }, [sendMsg]);

  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      height: '100vh',
      background: '#0d1117',
      color: '#c9d1d9',
      fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
      overflow: 'hidden',
    }}>
      {/* ===== HEADER ===== */}
      <header style={{
        display: 'flex',
        alignItems: 'center',
        gap: '12px',
        padding: '5px 12px',
        background: '#161b22',
        borderBottom: '1px solid #30363d',
        flexShrink: 0,
      }}>
        <span style={{ fontSize: '12px', fontWeight: 700, color: '#00ff41', letterSpacing: '1px' }}>
          STS RL
        </span>
        {stats?.run_id && (
          <span style={{ fontSize: '9px', color: '#8b949e', fontFamily: 'monospace' }}>
            {stats.run_id}
          </span>
        )}

        {/* Status dot */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
          <div style={{
            width: '6px', height: '6px',
            background: connected ? (state.paused ? '#ffb700' : '#00ff41') : '#ff4444',
            borderRadius: '50%',
            boxShadow: connected ? (state.paused ? '0 0 6px #ffb700' : '0 0 6px #00ff41') : 'none',
            animation: state.paused ? 'pulse 1.5s ease-in-out infinite' : 'none',
          }} />
          <span style={{ fontSize: '9px', color: state.paused ? '#ffb700' : '#8b949e' }}>
            {!connected ? 'OFFLINE' : state.paused ? 'PAUSED' : 'LIVE'}
          </span>
        </div>

        {/* View tabs */}
        <div style={{ display: 'flex', gap: '3px', alignItems: 'center' }}>
          <ViewTab label="Dashboard" shortcut="D" active={topView === 'dashboard' && !showDetail} onClick={() => { setTopView('dashboard'); setShowDetail(false); }} />
          <ViewTab label="Feed" shortcut="F" active={topView === 'feed'} onClick={() => { setTopView('feed'); setShowDetail(false); }} />
          <ViewTab label="Stats" shortcut="S" active={topView === 'stats_view'} onClick={() => { setTopView('stats_view'); setShowDetail(false); }} />
          <ViewTab label="Training" shortcut="T" active={topView === 'training_view'} onClick={() => { setTopView('training_view'); setShowDetail(false); }} />
        </div>

        {/* Key metrics */}
        <div style={{ display: 'flex', gap: '16px', alignItems: 'center', flex: 1, justifyContent: 'flex-end' }}>
          <StatBlock label="Agents" value={String(agents.length || numAgents)} />
          <StatBlock
            label="G/hr"
            value={epsPerMin > 0 ? String(Math.round(epsPerMin * 60)) : '---'}
            color="#c9d1d9"
          />
          <StatBlock label="Floor" value={avgFloor > 0 ? avgFloor.toFixed(1) : '---'} />
          <StatBlock label="MCTS" value={mctsMs > 0 ? `${mctsMs.toFixed(0)}ms` : '---'} />
          <StatBlock
            label="Best"
            value={stats?.max_floor ? String(stats.max_floor) : '---'}
            color={stats?.max_floor && stats.max_floor >= 17 ? '#00ff41' : '#ffb700'}
          />
          <StatBlock label="Eps" value={totalEpisodes > 0 ? totalEpisodes.toLocaleString() : '---'} />
        </div>

        {/* Play/pause + Control */}
        <div style={{ display: 'flex', gap: '4px', alignItems: 'center' }}>
          <button
            onClick={isRunning ? stopTraining : resumeTraining}
            style={{
              background: isRunning ? '#6e1a1a' : '#1a4d2a',
              border: `1px solid ${isRunning ? '#ff4444' : '#00ff41'}`,
              color: isRunning ? '#ff4444' : '#00ff41',
              padding: '3px 10px',
              fontSize: '10px',
              cursor: 'pointer',
              letterSpacing: '0.5px',
            }}
          >
            {isRunning ? '[||]' : '[>]'}
          </button>
          <button
            onClick={() => setShowControl((v) => !v)}
            style={{
              background: '#21262d',
              border: '1px solid #30363d',
              color: '#8b949e',
              padding: '3px 8px',
              fontSize: '10px',
              cursor: 'pointer',
            }}
          >
            [C]
          </button>
        </div>
      </header>

      {/* ===== AGENTS STRIP (only on dashboard) ===== */}
      {topView === 'dashboard' && (
        <div style={{
          flexShrink: 0,
          borderBottom: '1px solid #30363d',
          padding: '4px 8px',
          background: '#161b22',
        }}>
          {viewMode === 'live' ? (
            <MultiAgentView
              agents={displayAgents}
              combatSummaries={combatSummaries}
              selectedIndex={selectedAgentIndex}
              onSelectAgent={selectAgent}
              onExpandAgent={() => setShowDetail(true)}
            />
          ) : (
            <div style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fill, minmax(170px, 1fr))',
              gap: '4px',
            }}>
              {displayAgents.map((agent, idx) => (
                <AgentCard
                  key={agent.id}
                  index={idx}
                  agent={agent}
                  selected={selectedAgentIndex === idx}
                  focused={focusedAgentIds.includes(agent.id)}
                  onSelect={() => selectAgent(idx)}
                  onToggleFocus={() => toggleFocus(agent.id)}
                />
              ))}
            </div>
          )}
        </div>
      )}

      {/* ===== MAIN CONTENT ===== */}
      {topView === 'dashboard' && showDetail && selectedAgent ? (
        <AgentDetailPanel
          agent={selectedAgent}
          combat={selectedCombat}
          mapData={selectedMap}
          runState={selectedRunState}
          mcts={mctsResult}
          planner={plannerResult}
          episodes={episodes}
          deathStats={deathStats}
          tab={detailTab}
          onTabChange={setDetailTab}
          onClose={() => setShowDetail(false)}
        />
      ) : topView === 'dashboard' ? (
        <StatsOverviewPanel
          agents={displayAgents}
          episodes={episodes}
          stats={stats}
          systemStats={systemStats}
          deathStats={deathStats}
          floorHistory={floorHistory}
          winHistory={winHistory}
        />
      ) : topView === 'feed' ? (
        <CombatFeedView
          agents={displayAgents}
          episodes={episodes}
          selectedAgentIndex={selectedAgentIndex}
          combatStates={combatStates}
        />
      ) : topView === 'stats_view' ? (
        <StatsView episodes={episodes} />
      ) : topView === 'training_view' ? (
        <TrainingMetricsView
          stats={stats}
          systemStats={systemStats}
          floorHistory={floorHistory}
          lossHistory={lossHistory}
          winHistory={winHistory}
        />
      ) : null}

      {/* ===== FOOTER ===== */}
      <footer style={{
        display: 'flex',
        alignItems: 'center',
        gap: '10px',
        padding: '3px 10px',
        background: '#161b22',
        borderTop: '1px solid #30363d',
        flexShrink: 0,
        fontSize: '9px',
        color: '#8b949e',
      }}>
        <KbdHint keys="D" label="dashboard" />
        <KbdHint keys="F" label="feed" />
        <KbdHint keys="S" label="stats" />
        <KbdHint keys="T" label="training" />
        <span style={{ color: '#30363d' }}>|</span>
        <KbdHint keys="Space" label="play/pause" />
        <KbdHint keys="1-8" label="agent" />
        <KbdHint keys="[/]" label="prev/next" />
        <KbdHint keys="Esc" label="back" />
        <div style={{ flex: 1 }} />
        {focusedAgentIds.length > 0 && (
          <span style={{ color: '#00ff41' }}>{focusedAgentIds.length} focused</span>
        )}
        <span style={{ color: '#3d444d' }}>v0.2</span>
      </footer>

      {/* ===== CONTROL PANEL OVERLAY ===== */}
      {showControl && (
        <>
          <div
            onClick={() => setShowControl(false)}
            style={{
              position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.5)', zIndex: 90,
            }}
          />
          <ControlPanel
            onClose={() => setShowControl(false)}
            onStart={handleStart}
            onPause={stopTraining}
            onResume={resumeTraining}
            onStop={() => sendCommand('stop')}
            isRunning={isRunning}
            isPaused={state.paused}
            sendControl={sendControl}
            systemStats={systemStats}
          />
        </>
      )}
    </div>
  );
};

const KbdHint = ({ keys, label }: { keys: string; label: string }) => (
  <span>
    <span style={{
      display: 'inline-block',
      padding: '0 3px',
      background: '#21262d',
      border: '1px solid #30363d',
      fontSize: '8px',
      color: '#c9d1d9',
    }}>{keys}</span>
    {' '}{label}
  </span>
);
