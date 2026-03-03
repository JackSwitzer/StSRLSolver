import { useState } from 'react';
import type { ConquererState, PathResult } from '../types/conquerer';
import { PathPanel } from './PathPanel';
import { ProgressBar } from './ProgressBar';
import { DivergenceTree } from './DivergenceTree';

// ---------------------------------------------------------------------------
// Mock data so the view renders without a server connection
// ---------------------------------------------------------------------------

const MOCK_PATHS: PathResult[] = [
  { path_id: 0, seed: 'DEMO', won: true,  floors_reached: 55, hp_remaining: 34, total_reward: 1.0,  strategy: 'greedy' },
  { path_id: 1, seed: 'DEMO', won: false, floors_reached: 32, hp_remaining: 0,  total_reward: 0.0,  strategy: 'random_0.5' },
  { path_id: 2, seed: 'DEMO', won: true,  floors_reached: 55, hp_remaining: 12, total_reward: 1.0,  strategy: 'heuristic_1' },
  { path_id: 3, seed: 'DEMO', won: false, floors_reached: 48, hp_remaining: 0,  total_reward: 0.0,  strategy: 'mcts_64' },
  { path_id: 4, seed: 'DEMO', won: true,  floors_reached: 55, hp_remaining: 51, total_reward: 1.0,  strategy: 'greedy' },
  { path_id: 5, seed: 'DEMO', won: false, floors_reached: 18, hp_remaining: 0,  total_reward: 0.0,  strategy: 'random_0.5' },
  { path_id: 6, seed: 'DEMO', won: true,  floors_reached: 55, hp_remaining: 8,  total_reward: 1.0,  strategy: 'heuristic_1' },
  { path_id: 7, seed: 'DEMO', won: false, floors_reached: 41, hp_remaining: 0,  total_reward: 0.0,  strategy: 'mcts_64' },
  { path_id: 8, seed: 'DEMO', won: true,  floors_reached: 55, hp_remaining: 27, total_reward: 1.0,  strategy: 'greedy' },
  { path_id: 9, seed: 'DEMO', won: false, floors_reached: 9,  hp_remaining: 0,  total_reward: 0.0,  strategy: 'random_0.5' },
];

const MOCK_STATE: ConquererState = {
  seed: 'DEMO',
  paths: MOCK_PATHS,
  best_path_id: 4,
  win_count: 5,
  max_floor: 55,
  active_paths: 0,
  elapsed_seconds: 42.7,
};

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

interface ConquererViewProps {
  state?: ConquererState | null;
}

export const ConquererView = ({ state }: ConquererViewProps) => {
  const data = state || MOCK_STATE;
  const [selectedPathId, setSelectedPathId] = useState<number | null>(null);

  // Sort paths: won first (by HP desc), then lost (by floor desc)
  const sortedPaths = [...data.paths].sort((a, b) => {
    if (a.won !== b.won) return a.won ? -1 : 1;
    if (a.won && b.won) return b.hp_remaining - a.hp_remaining;
    return b.floors_reached - a.floors_reached;
  });

  const activeIds = new Set(
    data.paths
      .filter((_, i) => i < data.active_paths)
      .map((p) => p.path_id),
  );

  return (
    <div className="conquerer-root">
      {/* Overall stats bar */}
      <div className="conquerer-stats-bar">
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
      </div>

      <div className="conquerer-body">
        {/* Main content: grid + progress bars */}
        <div className="conquerer-main">
          {/* 10-path panel grid */}
          <div className="conquerer-grid">
            {sortedPaths.map((path) => (
              <PathPanel
                key={path.path_id}
                path={path}
                isBest={path.path_id === data.best_path_id}
                isActive={activeIds.has(path.path_id)}
                onClick={() =>
                  setSelectedPathId(
                    selectedPathId === path.path_id ? null : path.path_id,
                  )
                }
              />
            ))}
          </div>

          {/* Progress bars for all paths */}
          <div className="conquerer-progress-section">
            <div className="conquerer-section-header">Floor Progress</div>
            {sortedPaths.map((path) => (
              <ProgressBar
                key={path.path_id}
                path={path}
                isBest={path.path_id === data.best_path_id}
              />
            ))}
          </div>
        </div>

        {/* Right sidebar: divergence tree */}
        <div className="conquerer-sidebar">
          <DivergenceTree tree={data.divergence_tree} />

          {/* Selected path detail */}
          {selectedPathId !== null && (
            <div className="conquerer-detail">
              <div className="conquerer-section-header">
                Path #{selectedPathId} Detail
              </div>
              {(() => {
                const p = data.paths.find((x) => x.path_id === selectedPathId);
                if (!p) return <div className="conquerer-detail-empty">Not found</div>;
                return (
                  <div className="conquerer-detail-body">
                    <div className="conquerer-detail-row">
                      <span>Strategy</span><span>{p.strategy}</span>
                    </div>
                    <div className="conquerer-detail-row">
                      <span>Floor</span><span>{p.floors_reached}</span>
                    </div>
                    <div className="conquerer-detail-row">
                      <span>HP</span><span>{p.hp_remaining}</span>
                    </div>
                    <div className="conquerer-detail-row">
                      <span>Reward</span><span>{p.total_reward.toFixed(3)}</span>
                    </div>
                    <div className="conquerer-detail-row">
                      <span>Result</span>
                      <span style={{ color: p.won ? '#44bb44' : '#cc3333' }}>
                        {p.won ? 'Victory' : 'Defeat'}
                      </span>
                    </div>
                  </div>
                );
              })()}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
