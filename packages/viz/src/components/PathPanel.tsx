import { useState } from 'react';
import type { PathResult } from '../types/conquerer';
import { agentName, floorToAct } from '../types/conquerer';

interface PathPanelProps {
  path: PathResult;
  isBest: boolean;
  isActive: boolean;
  isSelected?: boolean;
  compact?: boolean;
  onClick?: () => void;
}

const ACT_COLORS = ['#3366cc', '#44aa44', '#cc3333', '#daa520'];

/** Approximate max HP for Watcher A20 at given floor. */
function estimateMaxHp(floor: number): number {
  return 72 + Math.floor(floor / 10) * 5;
}

export const PathPanel = ({ path, isBest, isActive, isSelected = false, compact = false, onClick }: PathPanelProps) => {
  const [hovered, setHovered] = useState(false);
  const maxHp = estimateMaxHp(path.floors_reached);
  const hpRatio = maxHp > 0 ? Math.max(0, Math.min(1, path.hp_remaining / maxHp)) : 0;
  const act = floorToAct(path.floors_reached);
  const actColor = ACT_COLORS[act - 1];
  const pct = Math.min(100, (path.floors_reached / 55) * 100);

  const status = isActive ? 'running' : path.won ? 'won' : 'lost';
  const statusColor = isActive ? '#ccaa22' : path.won ? '#44bb44' : '#cc3333';
  const name = agentName(path.path_id);

  let cls = 'agent-card';
  if (isSelected) cls += ' agent-card--selected';
  if (isBest) cls += ' agent-card--best';
  if (status === 'lost') cls += ' agent-card--dead';
  if (compact) cls += ' agent-card--compact';

  return (
    <div
      className={cls}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      role="button"
      tabIndex={0}
    >
      {/* Top row: number + name + status dot */}
      <div className="agent-card-header">
        <span className="agent-card-num">{path.path_id + 1}</span>
        <span className="agent-card-name">{name}</span>
        <span className="agent-card-status" style={{ background: statusColor }} title={status.toUpperCase()} />
      </div>

      {/* Floor progress mini-bar */}
      <div className="agent-card-progress">
        <div className="agent-card-progress-fill" style={{ width: `${pct}%`, background: actColor }} />
      </div>

      {/* Key stats row */}
      <div className="agent-card-stats">
        <span className="agent-card-floor">F{path.floors_reached}</span>
        <span className="agent-card-hp" style={{ color: hpRatio > 0.3 ? '#ccc' : '#cc3333' }}>
          {path.hp_remaining}hp
        </span>
        {path.won && <span className="agent-card-win">W</span>}
      </div>

      {/* Hover detail overlay */}
      {hovered && !compact && (
        <div className="agent-card-detail">
          <div className="agent-card-detail-row"><span>Strategy</span><span className="mono">{path.strategy}</span></div>
          <div className="agent-card-detail-row"><span>Floor</span><span>{path.floors_reached} (Act {act})</span></div>
          <div className="agent-card-detail-row"><span>HP</span><span>{path.hp_remaining}/{maxHp}</span></div>
          <div className="agent-card-detail-row"><span>Reward</span><span>{path.total_reward.toFixed(3)}</span></div>
          <div className="agent-card-detail-row">
            <span>Result</span>
            <span style={{ color: statusColor, fontWeight: 700 }}>
              {isActive ? 'RUNNING' : path.won ? 'VICTORY' : 'DEFEAT'}
            </span>
          </div>
          {isBest && <div className="agent-card-best-tag">BEST PATH</div>}
        </div>
      )}
    </div>
  );
};
