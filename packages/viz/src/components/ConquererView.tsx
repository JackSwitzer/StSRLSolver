import { useState } from 'react';
import type { ConquererState, PathResult, ViewMode } from '../types/conquerer';
import { agentName, floorToAct } from '../types/conquerer';
import { PathPanel } from './PathPanel';
import { ProgressBar } from './ProgressBar';
import { DivergenceTree } from './DivergenceTree';

// ---------------------------------------------------------------------------
// Mock data (16 agents)
// ---------------------------------------------------------------------------

function makeMockPaths(count: number): PathResult[] {
  const strategies = [
    'greedy', 'random_0.5', 'random_1.0', 'random_2.0',
    'heuristic_attack', 'heuristic_block', 'heuristic_balanced',
    'weighted_7', 'weighted_8', 'weighted_9',
    'mcts_32', 'mcts_64', 'mcts_128', 'mcts_256',
    'rollout_deep', 'rollout_wide',
  ];
  return Array.from({ length: count }, (_, i) => {
    const won = i % 4 !== 1 && i % 5 !== 2;
    const floor = won ? 55 : 8 + Math.floor(Math.random() * 42);
    return {
      path_id: i,
      seed: 'DEMO',
      won,
      floors_reached: floor,
      hp_remaining: won ? Math.floor(Math.random() * 55) + 5 : 0,
      total_reward: won ? 1.0 : floor / 60,
      strategy: strategies[i % strategies.length],
    };
  });
}

const MOCK_PATHS = makeMockPaths(16);

const MOCK_STATE: ConquererState = {
  seed: 'DEMO',
  paths: MOCK_PATHS,
  best_path_id: 0,
  win_count: MOCK_PATHS.filter((p) => p.won).length,
  max_floor: 55,
  active_paths: 0,
  elapsed_seconds: 87.3,
};

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

function sortPaths(paths: PathResult[]): PathResult[] {
  return [...paths].sort((a, b) => {
    if (a.won !== b.won) return a.won ? -1 : 1;
    if (a.won && b.won) return b.hp_remaining - a.hp_remaining;
    return b.floors_reached - a.floors_reached;
  });
}

// ---------------------------------------------------------------------------
// Aggregate Stats Header
// ---------------------------------------------------------------------------

const AggregateStats = ({ data }: { data: ConquererState }) => {
  const wins = data.paths.filter((p) => p.won);
  const losses = data.paths.filter((p) => !p.won);
  const avgFloor = data.paths.length > 0
    ? (data.paths.reduce((s, p) => s + p.floors_reached, 0) / data.paths.length).toFixed(1)
    : '0';
  const bestHp = wins.length > 0 ? Math.max(...wins.map((p) => p.hp_remaining)) : 0;
  const worstLossFloor = losses.length > 0 ? Math.min(...losses.map((p) => p.floors_reached)) : '-';

  return (
    <div className="conq-aggregate">
      <div className="conq-agg-item">
        <span className="conq-agg-label">Seed</span>
        <span className="conq-agg-value mono">{data.seed}</span>
      </div>
      <div className="conq-agg-item">
        <span className="conq-agg-label">Win Rate</span>
        <span className="conq-agg-value" style={{ color: data.win_count > 0 ? '#44bb44' : '#cc3333' }}>
          {data.win_count}/{data.paths.length}
        </span>
      </div>
      <div className="conq-agg-item">
        <span className="conq-agg-label">Avg Floor</span>
        <span className="conq-agg-value">{avgFloor}</span>
      </div>
      <div className="conq-agg-item">
        <span className="conq-agg-label">Best HP</span>
        <span className="conq-agg-value" style={{ color: '#44bb44' }}>{bestHp}</span>
      </div>
      <div className="conq-agg-item">
        <span className="conq-agg-label">Worst Death</span>
        <span className="conq-agg-value" style={{ color: '#cc3333' }}>F{worstLossFloor}</span>
      </div>
      {data.active_paths > 0 && (
        <div className="conq-agg-item">
          <span className="conq-agg-label">Active</span>
          <span className="conq-agg-value" style={{ color: '#ccaa22' }}>{data.active_paths}</span>
        </div>
      )}
      <div className="conq-agg-item">
        <span className="conq-agg-label">Time</span>
        <span className="conq-agg-value">{data.elapsed_seconds.toFixed(1)}s</span>
      </div>
    </div>
  );
};

// ---------------------------------------------------------------------------
// Grid View (primary: 4x4 for 16, 5x2 for 10, etc.)
// ---------------------------------------------------------------------------

function gridCols(n: number): number {
  if (n <= 4) return 2;
  if (n <= 6) return 3;
  if (n <= 8) return 4;
  if (n <= 12) return 4;
  return 4; // 16 -> 4x4
}

const GridView = ({
  data,
  selectedPathId,
  setSelectedPathId,
}: {
  data: ConquererState;
  selectedPathId: number | null;
  setSelectedPathId: (id: number | null) => void;
}) => {
  const sorted = sortPaths(data.paths);
  const activeIds = new Set(data.paths.filter((_, i) => i < data.active_paths).map((p) => p.path_id));
  const cols = gridCols(data.paths.length);

  return (
    <div className="conq-grid" style={{ gridTemplateColumns: `repeat(${cols}, 1fr)` }}>
      {sorted.map((path) => (
        <PathPanel
          key={path.path_id}
          path={path}
          isBest={path.path_id === data.best_path_id}
          isActive={activeIds.has(path.path_id)}
          isSelected={path.path_id === selectedPathId}
          onClick={() => setSelectedPathId(selectedPathId === path.path_id ? null : path.path_id)}
        />
      ))}
    </div>
  );
};

// ---------------------------------------------------------------------------
// Scroll View (compact rows)
// ---------------------------------------------------------------------------

const ScrollView = ({
  data,
  selectedPathId,
  setSelectedPathId,
}: {
  data: ConquererState;
  selectedPathId: number | null;
  setSelectedPathId: (id: number | null) => void;
}) => {
  const sorted = sortPaths(data.paths);
  const activeIds = new Set(data.paths.filter((_, i) => i < data.active_paths).map((p) => p.path_id));

  return (
    <div className="conq-scroll">
      {/* Table header */}
      <div className="conq-scroll-header">
        <span className="conq-scroll-col-num">#</span>
        <span className="conq-scroll-col-name">Agent</span>
        <span className="conq-scroll-col-status">Status</span>
        <span className="conq-scroll-col-floor">Floor</span>
        <span className="conq-scroll-col-hp">HP</span>
        <span className="conq-scroll-col-progress">Progress</span>
      </div>
      {sorted.map((path) => {
        const isActive = activeIds.has(path.path_id);
        const isSelected = path.path_id === selectedPathId;
        const status = isActive ? 'RUN' : path.won ? 'WIN' : 'LOST';
        const statusColor = isActive ? '#ccaa22' : path.won ? '#44bb44' : '#cc3333';
        const name = agentName(path.path_id);
        const pct = Math.min(100, (path.floors_reached / 55) * 100);

        return (
          <div
            key={path.path_id}
            className={`conq-scroll-row ${isSelected ? 'selected' : ''} ${!path.won && !isActive ? 'dead' : ''}`}
            onClick={() => setSelectedPathId(isSelected ? null : path.path_id)}
          >
            <span className="conq-scroll-col-num">{path.path_id + 1}</span>
            <span className="conq-scroll-col-name">{name}</span>
            <span className="conq-scroll-col-status" style={{ color: statusColor }}>{status}</span>
            <span className="conq-scroll-col-floor">F{path.floors_reached}</span>
            <span className="conq-scroll-col-hp">{path.hp_remaining}</span>
            <span className="conq-scroll-col-progress">
              <div className="conq-mini-bar">
                <div
                  className="conq-mini-bar-fill"
                  style={{ width: `${pct}%`, background: path.won ? '#44bb44' : '#cc3333' }}
                />
              </div>
            </span>
            {path.path_id === data.best_path_id && (
              <span className="conq-best-tag">BEST</span>
            )}
          </div>
        );
      })}
    </div>
  );
};

// ---------------------------------------------------------------------------
// Single / Detail View
// ---------------------------------------------------------------------------

const SingleView = ({
  data,
  selectedPathId,
  setSelectedPathId,
}: {
  data: ConquererState;
  selectedPathId: number | null;
  setSelectedPathId: (id: number | null) => void;
}) => {
  const sorted = sortPaths(data.paths);
  const activeIds = new Set(data.paths.filter((_, i) => i < data.active_paths).map((p) => p.path_id));
  const selected = selectedPathId !== null ? data.paths.find((p) => p.path_id === selectedPathId) : null;

  return (
    <div className="conq-single-layout">
      {/* Compact agent list sidebar */}
      <div className="conq-agent-list">
        {sorted.map((path) => (
          <PathPanel
            key={path.path_id}
            path={path}
            isBest={path.path_id === data.best_path_id}
            isActive={activeIds.has(path.path_id)}
            isSelected={path.path_id === selectedPathId}
            compact
            onClick={() => setSelectedPathId(selectedPathId === path.path_id ? null : path.path_id)}
          />
        ))}
      </div>

      {/* Detail panel */}
      <div className="conq-detail-panel">
        {selected ? (
          <>
            <div className="conq-detail-header">
              <span className="conq-detail-num">{selected.path_id + 1}</span>
              <h2>{agentName(selected.path_id)}</h2>
              <span
                className="conq-detail-status"
                style={{ color: selected.won ? '#44bb44' : '#cc3333' }}
              >
                {selected.won ? 'VICTORY' : 'DEFEAT'}
              </span>
            </div>
            <div className="conq-detail-grid">
              <div className="conq-detail-stat">
                <span className="label">Strategy</span>
                <span className="value mono">{selected.strategy}</span>
              </div>
              <div className="conq-detail-stat">
                <span className="label">Floor</span>
                <span className="value">{selected.floors_reached} (Act {floorToAct(selected.floors_reached)})</span>
              </div>
              <div className="conq-detail-stat">
                <span className="label">HP</span>
                <span className="value">{selected.hp_remaining}</span>
              </div>
              <div className="conq-detail-stat">
                <span className="label">Reward</span>
                <span className="value">{selected.total_reward.toFixed(3)}</span>
              </div>
            </div>
            <div style={{ marginTop: '16px' }}>
              <ProgressBar path={selected} isBest={selected.path_id === data.best_path_id} />
            </div>
          </>
        ) : (
          <div className="conq-detail-empty">
            Select an agent to view details
          </div>
        )}

        {/* All progress bars */}
        <div className="conq-all-progress">
          <div className="conq-section-header">All Agents</div>
          {sorted.map((path) => (
            <ProgressBar key={path.path_id} path={path} isBest={path.path_id === data.best_path_id} />
          ))}
        </div>
      </div>
    </div>
  );
};

// ---------------------------------------------------------------------------
// Main ConquererView
// ---------------------------------------------------------------------------

interface ConquererViewProps {
  state?: ConquererState | null;
  viewMode?: ViewMode;
  onViewModeChange?: (mode: ViewMode) => void;
}

export const ConquererView = ({ state, viewMode: externalViewMode, onViewModeChange }: ConquererViewProps) => {
  const data = state || MOCK_STATE;
  const [internalViewMode, setInternalViewMode] = useState<ViewMode>('grid');
  const [selectedPathId, setSelectedPathId] = useState<number | null>(null);

  const viewMode = externalViewMode || internalViewMode;
  const setViewMode = (mode: ViewMode) => {
    setInternalViewMode(mode);
    onViewModeChange?.(mode);
  };

  return (
    <div className="conq-root">
      {/* Aggregate stats */}
      <AggregateStats data={data} />

      {/* View mode selector */}
      <div className="conq-controls">
        {(['grid', 'scroll', 'single'] as ViewMode[]).map((mode) => (
          <button
            key={mode}
            className={`conq-view-btn ${viewMode === mode ? 'active' : ''}`}
            onClick={() => setViewMode(mode)}
          >
            {mode === 'grid' ? 'Grid' : mode === 'scroll' ? 'List' : 'Detail'}
          </button>
        ))}
      </div>

      {/* Main content */}
      <div className="conq-content">
        {viewMode === 'grid' && (
          <GridView data={data} selectedPathId={selectedPathId} setSelectedPathId={setSelectedPathId} />
        )}
        {viewMode === 'scroll' && (
          <ScrollView data={data} selectedPathId={selectedPathId} setSelectedPathId={setSelectedPathId} />
        )}
        {viewMode === 'single' && (
          <SingleView data={data} selectedPathId={selectedPathId} setSelectedPathId={setSelectedPathId} />
        )}
      </div>

      {/* Sidebar: divergence tree (grid + single only) */}
      {(viewMode === 'grid' || viewMode === 'single') && data.divergence_tree && (
        <div className="conq-sidebar">
          <DivergenceTree tree={data.divergence_tree} />
        </div>
      )}
    </div>
  );
};
