import { useState } from 'react';
import type { ConquererState, PathResult, ViewMode } from '../types/conquerer';
import { PathPanel } from './PathPanel';
import { ProgressBar } from './ProgressBar';
import { DivergenceTree } from './DivergenceTree';

// ---------------------------------------------------------------------------
// Mock data so the view renders without a server connection
// ---------------------------------------------------------------------------

function makeMockPaths(count: number): PathResult[] {
  const strategies = ['greedy', 'random_0.5', 'heuristic_1', 'mcts_64'];
  return Array.from({ length: count }, (_, i) => {
    const won = i % 3 !== 1;
    return {
      path_id: i,
      seed: 'DEMO',
      won,
      floors_reached: won ? 55 : 10 + Math.floor(Math.random() * 40),
      hp_remaining: won ? Math.floor(Math.random() * 60) + 5 : 0,
      total_reward: won ? 1.0 : 0.0,
      strategy: strategies[i % strategies.length],
    };
  });
}

const MOCK_PATHS_10 = makeMockPaths(10);

const MOCK_STATE: ConquererState = {
  seed: 'DEMO',
  paths: MOCK_PATHS_10,
  best_path_id: 4,
  win_count: MOCK_PATHS_10.filter((p) => p.won).length,
  max_floor: 55,
  active_paths: 0,
  elapsed_seconds: 42.7,
};

// ---------------------------------------------------------------------------
// Utility: sort paths
// ---------------------------------------------------------------------------

function sortPaths(paths: PathResult[]): PathResult[] {
  return [...paths].sort((a, b) => {
    if (a.won !== b.won) return a.won ? -1 : 1;
    if (a.won && b.won) return b.hp_remaining - a.hp_remaining;
    return b.floors_reached - a.floors_reached;
  });
}

// ---------------------------------------------------------------------------
// Grid config based on path count
// ---------------------------------------------------------------------------

function gridColumns(numPaths: number): number {
  if (numPaths <= 4) return 2;
  if (numPaths <= 8) return 4;
  if (numPaths <= 10) return 5;
  return 4; // 16 paths: 4x4
}

// ---------------------------------------------------------------------------
// Single View
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
  const sortedPaths = sortPaths(data.paths);
  const activeIds = new Set(data.paths.filter((_, i) => i < data.active_paths).map((p) => p.path_id));
  const selectedPath = selectedPathId !== null ? data.paths.find((p) => p.path_id === selectedPathId) : null;

  return (
    <div className="conquerer-single-layout">
      {/* Path list sidebar */}
      <div className="conquerer-path-list">
        <div className="conquerer-section-header">Paths</div>
        {sortedPaths.map((path) => (
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

      {/* Main detail area */}
      <div className="conquerer-single-main">
        {selectedPath ? (
          <div className="conquerer-single-detail">
            <div className="conquerer-single-detail-header">
              <h2>Path #{selectedPath.path_id}</h2>
              <span
                className="conquerer-single-status"
                style={{ color: selectedPath.won ? '#44bb44' : '#cc3333' }}
              >
                {selectedPath.won ? 'VICTORY' : 'DEFEAT'}
              </span>
            </div>
            <div className="conquerer-single-detail-grid">
              <div className="conquerer-detail-card">
                <span className="detail-card-label">Strategy</span>
                <span className="detail-card-value mono">{selectedPath.strategy}</span>
              </div>
              <div className="conquerer-detail-card">
                <span className="detail-card-label">Floor Reached</span>
                <span className="detail-card-value">{selectedPath.floors_reached}</span>
              </div>
              <div className="conquerer-detail-card">
                <span className="detail-card-label">HP Remaining</span>
                <span className="detail-card-value">{selectedPath.hp_remaining}</span>
              </div>
              <div className="conquerer-detail-card">
                <span className="detail-card-label">Reward</span>
                <span className="detail-card-value">{selectedPath.total_reward.toFixed(3)}</span>
              </div>
            </div>
            {/* Progress bar for this path */}
            <div style={{ marginTop: '16px' }}>
              <ProgressBar path={selectedPath} isBest={selectedPath.path_id === data.best_path_id} />
            </div>
          </div>
        ) : (
          <div className="conquerer-single-empty">
            Select a path from the list to view details
          </div>
        )}

        {/* Progress bars for all paths */}
        <div className="conquerer-progress-section" style={{ marginTop: '24px' }}>
          <div className="conquerer-section-header">All Paths Progress</div>
          {sortedPaths.map((path) => (
            <ProgressBar key={path.path_id} path={path} isBest={path.path_id === data.best_path_id} />
          ))}
        </div>
      </div>
    </div>
  );
};

// ---------------------------------------------------------------------------
// Top 3 View
// ---------------------------------------------------------------------------

const Top3View = ({
  data,
  selectedPathId,
  setSelectedPathId,
}: {
  data: ConquererState;
  selectedPathId: number | null;
  setSelectedPathId: (id: number | null) => void;
}) => {
  const sortedPaths = sortPaths(data.paths);
  const top3 = sortedPaths.slice(0, 3);
  const activeIds = new Set(data.paths.filter((_, i) => i < data.active_paths).map((p) => p.path_id));

  return (
    <div className="conquerer-top3-layout">
      <div className="conquerer-top3-panels">
        {top3.map((path) => (
          <div
            key={path.path_id}
            className={`conquerer-top3-card ${path.path_id === selectedPathId ? 'selected' : ''} ${!path.won ? 'dead' : ''}`}
            onClick={() => setSelectedPathId(selectedPathId === path.path_id ? null : path.path_id)}
          >
            <div className="conquerer-top3-card-header">
              <span className="conquerer-panel-id">#{path.path_id}</span>
              <span style={{ color: path.won ? '#44bb44' : '#cc3333', fontWeight: 700, fontSize: '11px', textTransform: 'uppercase' }}>
                {path.won ? 'Won' : 'Lost'}
              </span>
            </div>
            <div className="conquerer-top3-card-strategy">{path.strategy}</div>

            <div className="conquerer-top3-stats">
              <div className="conquerer-top3-stat">
                <span className="label">Floor</span>
                <span className="value">{path.floors_reached}</span>
              </div>
              <div className="conquerer-top3-stat">
                <span className="label">HP</span>
                <span className="value">{path.hp_remaining}</span>
              </div>
              <div className="conquerer-top3-stat">
                <span className="label">Reward</span>
                <span className="value">{path.total_reward.toFixed(2)}</span>
              </div>
            </div>

            <ProgressBar path={path} isBest={path.path_id === data.best_path_id} />

            {path.path_id === data.best_path_id && (
              <div className="conquerer-best-badge">BEST</div>
            )}
          </div>
        ))}
      </div>

      {/* Remaining paths below */}
      <div className="conquerer-top3-remaining">
        <div className="conquerer-section-header">Other Paths</div>
        <div className="conquerer-grid" style={{ gridTemplateColumns: `repeat(${Math.min(sortedPaths.length - 3, 5)}, 1fr)` }}>
          {sortedPaths.slice(3).map((path) => (
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
      </div>
    </div>
  );
};

// ---------------------------------------------------------------------------
// Grid View
// ---------------------------------------------------------------------------

const GridView = ({
  data,
  selectedPathId,
  setSelectedPathId,
}: {
  data: ConquererState;
  selectedPathId: number | null;
  setSelectedPathId: (id: number | null) => void;
}) => {
  const sortedPaths = sortPaths(data.paths);
  const activeIds = new Set(data.paths.filter((_, i) => i < data.active_paths).map((p) => p.path_id));
  const cols = gridColumns(data.paths.length);

  return (
    <div className="conquerer-grid-layout">
      <div className="conquerer-grid" style={{ gridTemplateColumns: `repeat(${cols}, 1fr)` }}>
        {sortedPaths.map((path) => (
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
    </div>
  );
};

// ---------------------------------------------------------------------------
// Scroll View
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
  const sortedPaths = sortPaths(data.paths);
  const activeIds = new Set(data.paths.filter((_, i) => i < data.active_paths).map((p) => p.path_id));

  return (
    <div className="conquerer-scroll-layout">
      {sortedPaths.map((path) => {
        const isSelected = path.path_id === selectedPathId;
        const isDead = !path.won && !activeIds.has(path.path_id);
        const hpRatio = path.hp_remaining / Math.max(1, 72 + Math.floor(path.floors_reached / 10) * 5);

        return (
          <div
            key={path.path_id}
            className={`conquerer-scroll-row ${isSelected ? 'selected' : ''} ${isDead ? 'dead' : ''}`}
            onClick={() => setSelectedPathId(isSelected ? null : path.path_id)}
          >
            <div className="conquerer-scroll-id">#{path.path_id}</div>
            <div className="conquerer-scroll-strategy">{path.strategy}</div>
            <div className="conquerer-scroll-status" style={{ color: activeIds.has(path.path_id) ? '#ccaa22' : path.won ? '#44bb44' : '#cc3333' }}>
              {activeIds.has(path.path_id) ? 'RUNNING' : path.won ? 'WON' : 'LOST'}
            </div>
            <div className="conquerer-scroll-floor">F{path.floors_reached}</div>
            <div className="conquerer-scroll-hp-bar">
              <div className="combat-hp-track" style={{ height: '8px' }}>
                <div
                  className="combat-hp-fill"
                  style={{
                    width: `${Math.max(0, Math.min(100, hpRatio * 100))}%`,
                    background: hpRatio > 0.6 ? '#44bb44' : hpRatio > 0.3 ? '#ccaa22' : '#cc3333',
                  }}
                />
              </div>
            </div>
            <div className="conquerer-scroll-hp">{path.hp_remaining} HP</div>
            <div className="conquerer-scroll-reward">{path.total_reward.toFixed(2)}</div>
            {path.path_id === data.best_path_id && (
              <div className="conquerer-best-badge small">BEST</div>
            )}
            <div className="conquerer-scroll-progress" style={{ flex: 1 }}>
              <ProgressBar path={path} isBest={path.path_id === data.best_path_id} />
            </div>
          </div>
        );
      })}
    </div>
  );
};

// ---------------------------------------------------------------------------
// Main ConquererView Component
// ---------------------------------------------------------------------------

interface ConquererViewProps {
  state?: ConquererState | null;
  /** Externally controlled view mode */
  viewMode?: ViewMode;
  /** Externally controlled num paths */
  numPaths?: number;
  /** Called when view mode changes */
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
    <div className={`conquerer-root conquerer-layout conquerer-layout--${viewMode}`}>
      {/* Stats bar */}
      <div className="conquerer-stats-bar" style={{ gridArea: 'header' }}>
        <div className="conquerer-stat">
          <span className="conquerer-stat-label">Seed</span>
          <span className="conquerer-stat-value conquerer-stat-seed">{data.seed}</span>
        </div>
        <div className="conquerer-stat">
          <span className="conquerer-stat-label">Wins</span>
          <span className="conquerer-stat-value" style={{ color: data.win_count > 0 ? '#44bb44' : '#cc3333' }}>
            {data.win_count}/{data.paths.length}
          </span>
        </div>
        <div className="conquerer-stat">
          <span className="conquerer-stat-label">Best Floor</span>
          <span className="conquerer-stat-value">{data.max_floor}</span>
        </div>
        <div className="conquerer-stat">
          <span className="conquerer-stat-label">Active</span>
          <span className="conquerer-stat-value" style={{ color: data.active_paths > 0 ? '#ccaa22' : '#888' }}>
            {data.active_paths}
          </span>
        </div>
        <div className="conquerer-stat">
          <span className="conquerer-stat-label">Elapsed</span>
          <span className="conquerer-stat-value">{data.elapsed_seconds.toFixed(1)}s</span>
        </div>

        {/* View mode selector */}
        <div className="conquerer-view-selector">
          {(['single', 'top3', 'grid', 'scroll'] as ViewMode[]).map((mode) => (
            <button
              key={mode}
              className={`conquerer-view-btn ${viewMode === mode ? 'active' : ''}`}
              onClick={() => setViewMode(mode)}
            >
              {mode === 'single' ? 'Single' : mode === 'top3' ? 'Top 3' : mode === 'grid' ? 'Grid' : 'Scroll'}
            </button>
          ))}
        </div>
      </div>

      {/* Main content area */}
      <div className="conquerer-content" style={{ gridArea: 'main' }}>
        {viewMode === 'single' && (
          <SingleView data={data} selectedPathId={selectedPathId} setSelectedPathId={setSelectedPathId} />
        )}
        {viewMode === 'top3' && (
          <Top3View data={data} selectedPathId={selectedPathId} setSelectedPathId={setSelectedPathId} />
        )}
        {viewMode === 'grid' && (
          <GridView data={data} selectedPathId={selectedPathId} setSelectedPathId={setSelectedPathId} />
        )}
        {viewMode === 'scroll' && (
          <ScrollView data={data} selectedPathId={selectedPathId} setSelectedPathId={setSelectedPathId} />
        )}
      </div>

      {/* Sidebar: divergence tree (only for single/grid) */}
      {(viewMode === 'single' || viewMode === 'grid') && (
        <div className="conquerer-sidebar" style={{ gridArea: 'sidebar' }}>
          <DivergenceTree tree={data.divergence_tree} />

          {/* Selected path detail (grid mode) */}
          {viewMode === 'grid' && selectedPathId !== null && (
            <div className="conquerer-detail">
              <div className="conquerer-section-header">Path #{selectedPathId} Detail</div>
              {(() => {
                const p = data.paths.find((x) => x.path_id === selectedPathId);
                if (!p) return <div className="conquerer-detail-empty">Not found</div>;
                return (
                  <div className="conquerer-detail-body">
                    <div className="conquerer-detail-row"><span>Strategy</span><span>{p.strategy}</span></div>
                    <div className="conquerer-detail-row"><span>Floor</span><span>{p.floors_reached}</span></div>
                    <div className="conquerer-detail-row"><span>HP</span><span>{p.hp_remaining}</span></div>
                    <div className="conquerer-detail-row"><span>Reward</span><span>{p.total_reward.toFixed(3)}</span></div>
                    <div className="conquerer-detail-row">
                      <span>Result</span>
                      <span style={{ color: p.won ? '#44bb44' : '#cc3333' }}>{p.won ? 'Victory' : 'Defeat'}</span>
                    </div>
                  </div>
                );
              })()}
            </div>
          )}
        </div>
      )}
    </div>
  );
};
