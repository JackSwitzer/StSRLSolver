import { useState, useMemo } from 'react';
import { useTrainingState } from './hooks/useTrainingState';
import { useRunData } from './hooks/useRunData';
import type { AgentEpisodeMsg } from './types/training';

// Live view
import { StatusBar } from './components/StatusBar';
import { MetricsCharts } from './components/MetricsCharts';
import { TopRuns } from './components/TopRuns';
import { WorkerList } from './components/WorkerList';
import { SystemBar } from './components/SystemBar';
import { PerformancePanel } from './components/PerformancePanel';

// Analysis view
import { RunHistory } from './components/RunHistory';
import { FloorCurve } from './components/FloorCurve';
import { DeathAnalysis } from './components/DeathAnalysis';
import { EfficiencyPanel } from './components/EfficiencyPanel';
import { ConfigDiff } from './components/ConfigDiff';
import { RewardBreakdown } from './components/RewardBreakdown';
import { CardAnalysis } from './components/CardAnalysis';

// Detail view
import { NeowBanner } from './components/NeowBanner';
import { FloorTimeline } from './components/FloorTimeline';
import { DeckDisplay } from './components/DeckDisplay';
import { HPCurve } from './components/HPCurve';

type View = 'live' | 'analysis' | 'detail';

const TAB: React.CSSProperties = {
  padding: '6px 16px',
  fontSize: 12,
  fontWeight: 600,
  cursor: 'pointer',
  border: 'none',
  background: 'transparent',
  color: '#8b949e',
  borderBottom: '2px solid transparent',
  transition: 'color 0.15s, border-color 0.15s',
  fontFamily: 'inherit',
};

const TAB_ACTIVE: React.CSSProperties = {
  ...TAB,
  color: '#00ff41',
  borderBottomColor: '#00ff41',
};

const SECTION: React.CSSProperties = {
  background: '#161b22',
  border: '1px solid #30363d',
  borderRadius: 6,
  padding: 14,
};

const TITLE: React.CSSProperties = {
  fontSize: 11,
  color: '#8b949e',
  textTransform: 'uppercase',
  letterSpacing: '0.5px',
  marginBottom: 10,
};

// --- Live View ---

function LiveView({
  state,
  connected,
  fileEpisodes,
  floorCurveData,
  onSelectEpisode,
}: {
  state: ReturnType<typeof useTrainingState>['state'];
  connected: boolean;
  fileEpisodes: AgentEpisodeMsg[];
  floorCurveData: number[];
  onSelectEpisode: (ep: AgentEpisodeMsg) => void;
}) {
  // Use WS episodes if connected, file episodes otherwise
  const episodes = connected && state.episodes.length > 0 ? state.episodes : fileEpisodes;

  // Floor curves for inline chart
  const curves = useMemo(() => {
    const result = [];
    if (floorCurveData.length > 1) {
      result.push({ id: 'history', label: 'Historical', color: '#30363d', data: floorCurveData });
    }
    if (state.floorHistory.length > 1) {
      result.push({ id: 'live', label: 'Live', color: '#00ff41', data: state.floorHistory });
    }
    return result;
  }, [floorCurveData, state.floorHistory]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      <StatusBar stats={state.stats} connected={connected} systemStats={state.systemStats} />
      <MetricsCharts
        floorHistory={state.floorHistory}
        lossHistory={state.lossHistory}
        trainStepMarkers={state.trainStepMarkers}
      />
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
        <TopRuns episodes={episodes} maxRuns={10} onSelectEpisode={onSelectEpisode} />
        <PerformancePanel episodes={episodes} />
      </div>
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
        <div style={SECTION}>
          <DeathAnalysis episodes={episodes} />
        </div>
        {curves.length > 0 && (
          <div style={SECTION}>
            <div style={TITLE}>Floor Progression</div>
            <FloorCurve curves={curves} width={500} height={140} />
          </div>
        )}
      </div>
      <WorkerList agents={state.agents} />
    </div>
  );
}

// --- Analysis View ---

function AnalysisView({
  state,
  fileEpisodes,
  floorCurveData,
  runData,
}: {
  state: ReturnType<typeof useTrainingState>['state'];
  fileEpisodes: AgentEpisodeMsg[];
  floorCurveData: number[];
  runData: ReturnType<typeof useRunData>;
}) {
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const episodes = fileEpisodes.length > 0 ? fileEpisodes : state.episodes;

  const curves = useMemo(() => {
    const result = [];
    if (floorCurveData.length > 1) {
      result.push({ id: 'weekend', label: 'v5 Weekend (500K)', color: '#00ff41', data: floorCurveData });
    }
    if (state.floorHistory.length > 1) {
      result.push({ id: 'live', label: 'Live Session', color: '#58a6ff', data: state.floorHistory });
    }
    return result;
  }, [floorCurveData, state.floorHistory]);

  // Build config diff from selected run vs current
  const configEntries = useMemo(() => {
    const entries = [];
    if (selectedRunId) {
      const cfg = runData.getRunConfig(selectedRunId);
      if (Object.keys(cfg).length > 0) {
        const run = runData.runs.find((r) => r.id === selectedRunId);
        entries.push({ label: run?.label ?? selectedRunId, config: cfg });
      }
    }
    const currentCfg = runData.getRunConfig('weekend-run');
    if (Object.keys(currentCfg).length > 0) {
      entries.push({ label: 'v5 Weekend', config: currentCfg });
    }
    return entries;
  }, [selectedRunId, runData]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      <div style={SECTION}>
        <div style={TITLE}>Run History</div>
        <RunHistory runs={runData.runs} selectedId={selectedRunId} onSelect={setSelectedRunId} />
      </div>

      <div style={SECTION}>
        <div style={TITLE}>Floor Progression (rolling avg)</div>
        <FloorCurve curves={curves} width={800} height={220} />
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
        <div style={SECTION}>
          <DeathAnalysis episodes={episodes} />
        </div>
        <div style={SECTION}>
          <EfficiencyPanel episodes={episodes} stats={state.stats} />
        </div>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
        <div style={SECTION}>
          <RewardBreakdown episodes={episodes} />
        </div>
        <div style={SECTION}>
          <CardAnalysis episodes={episodes} />
        </div>
      </div>

      {configEntries.length > 0 && (
        <div style={SECTION}>
          <ConfigDiff configs={configEntries} />
        </div>
      )}
    </div>
  );
}

// --- Detail View ---

function DetailView({
  episode,
  allEpisodes,
  onSelectEpisode,
}: {
  episode: AgentEpisodeMsg | null;
  allEpisodes: AgentEpisodeMsg[];
  onSelectEpisode: (ep: AgentEpisodeMsg) => void;
}) {
  const [page, setPage] = useState(0);
  const PAGE_SIZE = 20;

  // Sort all episodes: best first
  const sorted = useMemo(() => {
    return [...allEpisodes]
      .sort((a, b) => b.floors_reached - a.floors_reached || a.duration - b.duration);
  }, [allEpisodes]);

  // Top 10 for quick-switch tabs
  const topEpisodes = useMemo(() => sorted.slice(0, 10), [sorted]);

  // Auto-select best episode if none selected
  const active = episode ?? topEpisodes[0] ?? null;

  if (allEpisodes.length === 0) {
    return (
      <div style={{ padding: 40, textAlign: 'center', color: '#484f58', fontSize: 13 }}>
        No episode data loaded
      </div>
    );
  }

  if (!active) {
    return (
      <div style={{ padding: 40, textAlign: 'center', color: '#484f58', fontSize: 13 }}>
        Loading episodes...
      </div>
    );
  }

  // Paginated episode list
  const pageEpisodes = sorted.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE);
  const totalPages = Math.ceil(sorted.length / PAGE_SIZE);

  // Aggregate stats across all episodes
  const aggStats = useMemo(() => {
    const allCombats = allEpisodes.flatMap((e) => e.combats ?? []);
    const cardCounts = new Map<string, number>();
    const stanceCounts = { wrath: 0, calm: 0, total: 0 };

    for (const ep of allEpisodes) {
      for (const c of ep.combats ?? []) {
        if (c.stances && typeof c.stances === 'object') {
          const changes = typeof c.stances === 'number' ? c.stances : (c.stances as any).changes ?? 0;
          stanceCounts.total += changes;
        }
      }
      for (const card of ep.deck_changes ?? []) {
        const clean = card.replace(/^\+/, '');
        cardCounts.set(clean, (cardCounts.get(clean) ?? 0) + 1);
      }
    }

    const topCards = [...cardCounts.entries()]
      .sort((a, b) => b[1] - a[1])
      .slice(0, 15);

    const avgFloor = allEpisodes.length > 0
      ? allEpisodes.reduce((s, e) => s + e.floors_reached, 0) / allEpisodes.length
      : 0;
    const avgDuration = allEpisodes.length > 0
      ? allEpisodes.reduce((s, e) => s + e.duration, 0) / allEpisodes.length
      : 0;
    const avgCombats = allEpisodes.length > 0
      ? allCombats.length / allEpisodes.length
      : 0;

    return { topCards, avgFloor, avgDuration, avgCombats, totalCombats: allCombats.length };
  }, [allEpisodes]);

  return (
    <div style={{ display: 'flex', gap: 12 }}>
      {/* Left sidebar: episode browser */}
      <div style={{
        width: 280,
        flexShrink: 0,
        background: '#161b22',
        border: '1px solid #30363d',
        borderRadius: 6,
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
      }}>
        <div style={{
          padding: '8px 12px',
          fontSize: 11,
          color: '#8b949e',
          textTransform: 'uppercase',
          letterSpacing: '0.5px',
          borderBottom: '1px solid #21262d',
        }}>
          Top {sorted.length} Episodes
        </div>

        <div style={{ flex: 1, overflow: 'auto' }}>
          {pageEpisodes.map((ep, i) => {
            const isActive = ep.seed === active.seed && ep.episode === active.episode;
            const rank = page * PAGE_SIZE + i + 1;
            return (
              <div
                key={`${ep.seed}-${ep.episode}`}
                onClick={() => onSelectEpisode(ep)}
                style={{
                  display: 'grid',
                  gridTemplateColumns: '24px 36px 1fr 50px',
                  gap: 4,
                  padding: '4px 10px',
                  cursor: 'pointer',
                  background: isActive ? 'rgba(0,255,65,0.08)' : 'transparent',
                  borderLeft: isActive ? '2px solid #00ff41' : '2px solid transparent',
                  fontSize: 11,
                  alignItems: 'center',
                }}
                onMouseEnter={(e) => { if (!isActive) (e.currentTarget as HTMLDivElement).style.background = '#1c2128'; }}
                onMouseLeave={(e) => { if (!isActive) (e.currentTarget as HTMLDivElement).style.background = 'transparent'; }}
              >
                <span style={{ color: '#484f58', fontSize: 10 }}>#{rank}</span>
                <span style={{ fontWeight: 700, color: ep.floors_reached >= 16 ? '#00ff41' : '#c9d1d9' }}>
                  F{ep.floors_reached}
                </span>
                <span style={{ color: '#8b949e', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                  {ep.death_enemy ?? (ep.combats?.[ep.combats.length - 1]?.enemy) ?? '--'}
                </span>
                <span style={{ color: '#484f58', textAlign: 'right', fontSize: 10 }}>
                  {ep.duration > 0 ? `${ep.duration.toFixed(0)}s` : ''}
                </span>
              </div>
            );
          })}
        </div>

        {/* Pagination */}
        {totalPages > 1 && (
          <div style={{
            display: 'flex',
            justifyContent: 'center',
            gap: 8,
            padding: '6px',
            borderTop: '1px solid #21262d',
            fontSize: 11,
          }}>
            <button
              onClick={() => setPage(Math.max(0, page - 1))}
              disabled={page === 0}
              style={{ background: 'none', border: 'none', color: page === 0 ? '#484f58' : '#8b949e', cursor: page === 0 ? 'default' : 'pointer', fontFamily: 'inherit' }}
            >
              Prev
            </button>
            <span style={{ color: '#484f58' }}>{page + 1}/{totalPages}</span>
            <button
              onClick={() => setPage(Math.min(totalPages - 1, page + 1))}
              disabled={page >= totalPages - 1}
              style={{ background: 'none', border: 'none', color: page >= totalPages - 1 ? '#484f58' : '#8b949e', cursor: page >= totalPages - 1 ? 'default' : 'pointer', fontFamily: 'inherit' }}
            >
              Next
            </button>
          </div>
        )}

        {/* Quick stats */}
        <div style={{
          padding: '8px 12px',
          borderTop: '1px solid #21262d',
          fontSize: 10,
          color: '#484f58',
          display: 'flex',
          flexDirection: 'column',
          gap: 2,
        }}>
          <span>Avg floor: {aggStats.avgFloor.toFixed(1)} | Avg time: {aggStats.avgDuration.toFixed(1)}s</span>
          <span>Avg combats/run: {aggStats.avgCombats.toFixed(1)} | Total: {aggStats.totalCombats}</span>
        </div>
      </div>

      {/* Right: episode detail */}
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 12 }}>
        {/* Episode summary bar */}
        <div style={{
          display: 'flex',
          alignItems: 'center',
          gap: 16,
          padding: '8px 12px',
          background: '#161b22',
          border: '1px solid #30363d',
          borderRadius: 6,
          fontSize: 13,
        }}>
          <span style={{ fontWeight: 700, color: '#c9d1d9' }}>{active.seed}</span>
          <span style={{
            fontWeight: 600,
            color: active.won ? '#00ff41' : '#f85149',
          }}>
            F{active.floors_reached} {active.won ? 'WIN' : 'DEATH'}
          </span>
          {active.death_enemy && (
            <span style={{ color: '#8b949e' }}>vs {active.death_enemy}</span>
          )}
          <span style={{ color: '#484f58', fontSize: 11 }}>
            {active.duration > 0 ? `${active.duration.toFixed(1)}s` : ''}
          </span>
          {active.deck_size != null && (
            <span style={{ color: '#484f58', fontSize: 11 }}>{active.deck_size} cards</span>
          )}
          <span style={{ color: '#484f58', fontSize: 11 }}>
            {active.combats?.length ?? 0} combats
          </span>
        </div>

        <NeowBanner decisions={active.decisions} neowChoice={active.neow_choice} />

        {/* Timeline-first layout: full width floor rows */}
        <FloorTimeline
          combats={active.combats}
          deathFloor={active.death_floor ?? active.floors_reached}
          deckChanges={active.deck_changes}
          floorLog={active.floor_log}
        />

        {/* Bottom: HP curve + Deck side by side */}
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
          <div>
            <div style={TITLE}>HP Over Floors</div>
            <HPCurve
              hpHistory={active.hp_history}
              maxHp={80}
              deathFloor={active.death_floor ?? active.floors_reached}
            />
          </div>
          <div>
            <div style={TITLE}>Deck at Death ({active.deck_changes?.length ?? '?'} cards)</div>
            <DeckDisplay cards={active.deck_changes ?? []} />
          </div>
        </div>
      </div>
    </div>
  );
}

// --- Main App ---

export const App = () => {
  const { state, connected } = useTrainingState();
  const runData = useRunData(connected);
  const [view, setView] = useState<View>('live');
  const [selectedEpisode, setSelectedEpisode] = useState<AgentEpisodeMsg | null>(null);

  const handleSelectEpisode = (ep: AgentEpisodeMsg) => {
    setSelectedEpisode(ep);
    setView('detail');
  };

  // Merge episodes: file data + live WS data
  const allEpisodes = useMemo(() => {
    const eps = [...runData.currentEpisodes];
    for (const e of state.episodes) {
      if (!eps.some((x) => x.seed === e.seed && x.episode === e.episode)) {
        eps.push(e);
      }
    }
    return eps;
  }, [runData.currentEpisodes, state.episodes]);

  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      height: '100vh',
      background: '#0d1117',
      color: '#c9d1d9',
      fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
      fontSize: 13,
      overflow: 'hidden',
    }}>
      {/* Tab bar */}
      <div style={{
        display: 'flex',
        alignItems: 'center',
        height: 38,
        padding: '0 12px',
        borderBottom: '1px solid #21262d',
        gap: 2,
        flexShrink: 0,
      }}>
        {(['live', 'analysis', 'detail'] as View[]).map((v) => (
          <button
            key={v}
            onClick={() => setView(v)}
            style={view === v ? TAB_ACTIVE : TAB}
          >
            {v.charAt(0).toUpperCase() + v.slice(1)}
          </button>
        ))}
        <div style={{ flex: 1 }} />
        {runData.loading ? (
          <span style={{ fontSize: 10, color: '#484f58' }}>Loading data...</span>
        ) : (
          <span style={{ fontSize: 10, color: '#30363d' }}>
            {allEpisodes.length} episodes
          </span>
        )}
        <button
          onClick={() => runData.loadData()}
          style={{
            background: 'none',
            border: '1px solid #30363d',
            borderRadius: 4,
            color: '#484f58',
            padding: '2px 8px',
            cursor: 'pointer',
            fontSize: 10,
            fontFamily: 'inherit',
            marginLeft: 4,
          }}
        >
          Reload
        </button>
        <span style={{ fontSize: 11, color: '#484f58', letterSpacing: '1px', marginLeft: 8 }}>
          STS RL TRAINING
        </span>
      </div>

      {/* Main content */}
      <div style={{ flex: 1, overflow: 'auto', padding: 12 }}>
        {view === 'live' && (
          <LiveView
            state={state}
            connected={connected}
            fileEpisodes={allEpisodes}
            floorCurveData={runData.floorCurveData}
            onSelectEpisode={handleSelectEpisode}
          />
        )}
        {view === 'analysis' && (
          <AnalysisView
            state={state}
            fileEpisodes={allEpisodes}
            floorCurveData={runData.floorCurveData}
            runData={runData}
          />
        )}
        {view === 'detail' && (
          <DetailView
            episode={selectedEpisode}
            allEpisodes={allEpisodes}
            onSelectEpisode={setSelectedEpisode}
          />
        )}
      </div>

      {/* System bar */}
      <SystemBar stats={state.systemStats} />
    </div>
  );
};
