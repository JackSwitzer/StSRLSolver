import { useState, useCallback, useEffect, useMemo } from 'react';
import type { AgentInfo } from '../types/training';
import { AGENT_NAMES } from '../types/training';
import { useTrainingState } from '../hooks/useTrainingState';
import { AgentCard } from './AgentCard';
import { Sparkline } from './Sparkline';
import { ControlPanel } from './ControlPanel';
import { EventFeed } from './EventFeed';
import { AgentDetailPanel } from './AgentDetailPanel';
import type { DetailTab } from './AgentDetailPanel';

// ---- Types ----

// ---- Helpers ----

function winRatePct(stats: { win_rate?: number } | null): string {
  if (!stats || !stats.win_rate) return '0.0%';
  return `${(stats.win_rate * 100).toFixed(1)}%`;
}

// Rolling win rate from recent history
function rollingWinRate(winHistory: number[], window = 50): number {
  const slice = winHistory.slice(-window);
  if (slice.length === 0) return 0;
  return slice.reduce((a, b) => a + b, 0) / slice.length;
}

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

const ProgressBar = ({ value, max, color = '#00ff41', height = 4 }: {
  value: number; max: number; color?: string; height?: number;
}) => {
  const pct = max > 0 ? Math.min(100, (value / max) * 100) : 0;
  return (
    <div style={{ background: '#21262d', height, overflow: 'hidden' }}>
      <div style={{ width: `${pct}%`, height: '100%', background: color, transition: 'width 0.3s linear' }} />
    </div>
  );
};

const SysBar = ({ label, value, max, color = '#4488ff' }: {
  label: string; value: number; max: number; color?: string;
}) => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
    <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '9px', color: '#8b949e' }}>
      <span>{label}</span>
      <span style={{ color: '#c9d1d9' }}>{value.toFixed(0)}/{max}</span>
    </div>
    <ProgressBar value={value} max={max} color={color} height={3} />
  </div>
);

// ---- Main MissionControl ----

export const MissionControl = () => {
  const {
    state, connected, toggleFocus, clearFocus, selectAgent,
    nextFocused, prevFocused, stopTraining, resumeTraining, sendControl, sendMsg,
  } = useTrainingState();

  const [showControl, setShowControl] = useState(false);
  const [showDetail, setShowDetail] = useState(false);
  const [detailTab, setDetailTab] = useState<DetailTab>('combat');
  const [numAgents, _setNumAgents] = useState(8);

  const { stats, agents, episodes, focusedAgentIds, selectedAgentIndex,
          combatStates, floorHistory, winHistory, systemStats, mctsResult } = state;

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

  // Rolling win rate
  const recentWR = rollingWinRate(winHistory, 50);

  // Keyboard handling
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      const tag = (e.target as HTMLElement).tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA') return;

      const key = e.key;
      const n = displayAgents.length;
      const cols = Math.ceil(Math.sqrt(n));

      if (key === 'c' || key === 'C') {
        e.preventDefault();
        setShowControl((v) => !v);
        return;
      }
      if (key === 'Escape' || key === 'q' || key === 'Q') {
        e.preventDefault();
        if (showControl) { setShowControl(false); return; }
        if (showDetail) { setShowDetail(false); return; }
        clearFocus();
        return;
      }
      if (key === ' ') {
        e.preventDefault();
        if (stats) stopTraining(); else resumeTraining();
        return;
      }
      if (key === 'Enter' || key === 'e' || key === 'E') {
        e.preventDefault();
        setShowDetail((v) => !v);
        return;
      }
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
      if (key === 'Tab') {
        e.preventDefault();
        if (showDetail) {
          // Cycle detail tabs when detail panel is open
          const tabs: DetailTab[] = ['combat', 'run', 'mcts'];
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
    displayAgents.length, selectedAgentIndex, showControl, showDetail, detailTab,
    stats, selectAgent, clearFocus, stopTraining, resumeTraining, nextFocused, prevFocused, setDetailTab,
  ]);

  // Mocked system stats if server doesn't send them
  const cpu = systemStats?.cpu_pct ?? 0;
  const ramUsed = systemStats?.ram_used_gb ?? 0;
  const ramTotal = systemStats?.ram_total_gb ?? 16;
  const workers = systemStats?.workers ?? agents.length;

  const isRunning = !!stats && !state.paused;
  const totalEpisodes = stats?.total_episodes ?? 0;
  const epsPerMin = stats?.eps_per_min ?? 0;
  const avgFloor = stats?.avg_floor ?? 0;
  const mctsMs = stats?.mcts_avg_ms ?? 0;

  // Derived sparkline: smooth out floor history
  const floorSparkData = floorHistory.length > 0 ? floorHistory : [];
  // Rolling 10-episode win rate for sparkline
  const winSparkData = useMemo(() => {
    if (winHistory.length < 2) return [];
    const out: number[] = [];
    const window = 20;
    for (let i = 0; i < winHistory.length; i++) {
      const slice = winHistory.slice(Math.max(0, i - window + 1), i + 1);
      out.push(slice.reduce((a, b) => a + b, 0) / slice.length * 100);
    }
    return out;
  }, [winHistory]);

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
        gap: '16px',
        padding: '5px 12px',
        background: '#161b22',
        borderBottom: '1px solid #30363d',
        flexShrink: 0,
      }}>
        <span style={{ fontSize: '12px', fontWeight: 700, color: '#00ff41', letterSpacing: '1px' }}>
          STS RL MISSION CONTROL
        </span>

        {/* Status dot */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
          <div style={{
            width: '6px', height: '6px',
            background: connected ? '#00ff41' : '#ff4444',
            borderRadius: '50%',
            boxShadow: connected ? '0 0 6px #00ff41' : 'none',
          }} />
          <span style={{ fontSize: '9px', color: '#8b949e' }}>{connected ? 'CONNECTED' : 'OFFLINE'}</span>
        </div>

        {/* Key metrics */}
        <div style={{ display: 'flex', gap: '20px', alignItems: 'center', flex: 1 }}>
          <StatBlock label="Agents" value={String(agents.length || numAgents)} />
          <StatBlock
            label="Games/hr"
            value={epsPerMin > 0 ? String(Math.round(epsPerMin * 60)) : '---'}
            color="#c9d1d9"
          />
          <StatBlock label="Avg Floor" value={avgFloor > 0 ? avgFloor.toFixed(1) : '---'} />
          <StatBlock label="MCTS" value={mctsMs > 0 ? `${mctsMs.toFixed(0)}ms` : '---'} />
          <StatBlock
            label="Win Rate"
            value={winRatePct(stats)}
            color={(stats?.win_rate ?? 0) > 0 ? '#00ff41' : '#8b949e'}
          />
          <StatBlock label="Episodes" value={totalEpisodes > 0 ? totalEpisodes.toLocaleString() : '---'} />
        </div>

        {/* Play/pause + Control */}
        <div style={{ display: 'flex', gap: '6px', alignItems: 'center' }}>
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
            {isRunning ? '[PAUSE]' : '[START]'}
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
            [C] CTRL
          </button>
        </div>
      </header>

      {/* ===== CHARTS + STATS ROW ===== */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: '1fr 1fr 1fr 1fr',
        gap: '0',
        borderBottom: '1px solid #30363d',
        flexShrink: 0,
        background: '#161b22',
      }}>
        {/* Floor trend sparkline */}
        <div style={{ padding: '8px 12px', borderRight: '1px solid #30363d' }}>
          <div style={{ fontSize: '9px', color: '#8b949e', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '4px' }}>
            Avg Floor / Episode
          </div>
          <Sparkline data={floorSparkData} width={200} height={36} color="#4488ff" />
          <div style={{ fontSize: '9px', color: '#8b949e', marginTop: '2px' }}>
            cur: <span style={{ color: '#4488ff' }}>
              {floorSparkData.length > 0 ? floorSparkData[floorSparkData.length - 1].toFixed(0) : '---'}
            </span>
          </div>
        </div>

        {/* Win rate sparkline */}
        <div style={{ padding: '8px 12px', borderRight: '1px solid #30363d' }}>
          <div style={{ fontSize: '9px', color: '#8b949e', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '4px' }}>
            Rolling Win Rate (20ep)
          </div>
          <Sparkline data={winSparkData} width={200} height={36} color="#00ff41" />
          <div style={{ fontSize: '9px', color: '#8b949e', marginTop: '2px' }}>
            cur: <span style={{ color: '#00ff41' }}>{(recentWR * 100).toFixed(1)}%</span>
          </div>
        </div>

        {/* Win rate + progress */}
        <div style={{ padding: '8px 12px', borderRight: '1px solid #30363d', display: 'flex', flexDirection: 'column', gap: '6px' }}>
          <div style={{ fontSize: '9px', color: '#8b949e', textTransform: 'uppercase', letterSpacing: '0.5px' }}>Win Rate</div>
          <div style={{ fontSize: '20px', fontWeight: 700, color: '#00ff41' }}>
            {winRatePct(stats)}
          </div>
          <ProgressBar
            value={Math.round((stats?.win_rate ?? 0) * 100)}
            max={100}
            color="#00ff41"
            height={4}
          />
          <div style={{ fontSize: '9px', color: '#8b949e' }}>
            target: <span style={{ color: '#ffb700' }}>96%</span>
            {' | '}
            eps: <span style={{ color: '#c9d1d9' }}>{totalEpisodes.toLocaleString()}</span>
          </div>
        </div>

        {/* System stats */}
        <div style={{ padding: '8px 12px', display: 'flex', flexDirection: 'column', gap: '5px' }}>
          <div style={{ fontSize: '9px', color: '#8b949e', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '1px' }}>System</div>
          <SysBar label="CPU" value={cpu} max={100} color="#4488ff" />
          <SysBar label="RAM (GB)" value={ramUsed} max={ramTotal} color="#ffb700" />
          <div style={{ fontSize: '9px', color: '#8b949e' }}>
            Workers: <span style={{ color: '#c9d1d9' }}>{workers}</span>
            {' | '}
            MCTS: <span style={{ color: '#c9d1d9' }}>{mctsMs.toFixed(0)}ms</span>
          </div>
        </div>
      </div>

      {/* ===== AGENTS GRID ===== */}
      <div style={{ flex: 1, overflow: 'auto', padding: '8px' }}>
        <div style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))',
          gap: '4px',
        }}>
          {displayAgents.map((agent, idx) => (
            <AgentCard
              key={agent.id}
              agent={agent}
              index={idx}
              selected={selectedAgentIndex === idx}
              focused={focusedAgentIds.includes(agent.id)}
              onSelect={() => selectAgent(idx)}
              onToggleFocus={() => toggleFocus(agent.id)}
            />
          ))}
        </div>
      </div>

      {/* ===== AGENT DETAIL PANEL ===== */}
      {showDetail && selectedAgent && (
        <AgentDetailPanel
          agent={selectedAgent}
          combat={selectedCombat}
          mcts={mctsResult}
          episodes={episodes}
          tab={detailTab}
          onTabChange={setDetailTab}
          onClose={() => setShowDetail(false)}
        />
      )}

      {/* ===== EVENT FEED ===== */}
      <EventFeed episodes={episodes} />

      {/* ===== FOOTER ===== */}
      <footer style={{
        display: 'flex',
        alignItems: 'center',
        gap: '12px',
        padding: '3px 10px',
        background: '#161b22',
        borderTop: '1px solid #30363d',
        flexShrink: 0,
        fontSize: '9px',
        color: '#8b949e',
      }}>
        <KbdHint keys="WASD" label="navigate" />
        <KbdHint keys="Enter/E" label="detail" />
        <KbdHint keys="Space" label="play/pause" />
        <KbdHint keys="C" label="control" />
        <KbdHint keys="Esc" label="back" />
        <KbdHint keys="Tab" label="cycle" />
        <div style={{ flex: 1 }} />
        {focusedAgentIds.length > 0 && (
          <span style={{ color: '#00ff41' }}>{focusedAgentIds.length} focused</span>
        )}
        <span style={{ color: '#3d444d' }}>STS RL v0.1</span>
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
            onStop={stopTraining}
            isRunning={isRunning}
            sendControl={sendControl}
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
