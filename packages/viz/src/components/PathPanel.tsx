import type { PathResult } from '../types/conquerer';

interface PathPanelProps {
  path: PathResult;
  isBest: boolean;
  isActive: boolean;
  isSelected?: boolean;
  onClick?: () => void;
}

function statusColor(path: PathResult, isActive: boolean): string {
  if (isActive) return '#ccaa22';
  if (path.won) return '#44bb44';
  return '#cc3333';
}

function statusLabel(path: PathResult, isActive: boolean): string {
  if (isActive) return 'RUNNING';
  if (path.won) return 'WON';
  return 'LOST';
}

function hpColor(hp: number, maxHp: number): string {
  if (maxHp === 0) return '#cc3333';
  const ratio = hp / maxHp;
  if (ratio > 0.6) return '#44bb44';
  if (ratio > 0.3) return '#ccaa22';
  return '#cc3333';
}

/** Approximate max HP for Watcher A20 at given floor. */
function estimateMaxHp(floorsReached: number): number {
  return 72 + Math.floor(floorsReached / 10) * 5;
}

export const PathPanel = ({ path, isBest, isActive, isSelected = false, onClick }: PathPanelProps) => {
  const maxHp = estimateMaxHp(path.floors_reached);
  const hpRatio = maxHp > 0 ? Math.max(0, Math.min(1, path.hp_remaining / maxHp)) : 0;
  const color = statusColor(path, isActive);
  const label = statusLabel(path, isActive);

  let borderClass = 'conquerer-panel';
  if (isSelected) borderClass += ' conquerer-panel-selected';
  if (isBest) borderClass += ' conquerer-panel-best';
  else if (path.won) borderClass += ' conquerer-panel-won';
  else if (!isActive && !path.won) borderClass += ' conquerer-panel-lost';

  return (
    <div className={borderClass} onClick={onClick} role="button" tabIndex={0}>
      {/* Header row */}
      <div className="conquerer-panel-header">
        <span className="conquerer-panel-id">#{path.path_id}</span>
        <span className="conquerer-panel-status" style={{ color }}>
          {label}
        </span>
      </div>

      {/* Strategy */}
      <div className="conquerer-panel-strategy">{path.strategy}</div>

      {/* Floor */}
      <div className="conquerer-panel-row">
        <span className="conquerer-panel-row-label">Floor</span>
        <span className="conquerer-panel-row-value">{path.floors_reached}</span>
      </div>

      {/* HP bar */}
      <div className="conquerer-panel-row">
        <span className="conquerer-panel-row-label">HP</span>
        <div className="conquerer-panel-hp-track">
          <div
            className="conquerer-panel-hp-fill"
            style={{
              width: `${hpRatio * 100}%`,
              background: hpColor(path.hp_remaining, maxHp),
            }}
          />
        </div>
        <span className="conquerer-panel-row-value" style={{ color: hpColor(path.hp_remaining, maxHp) }}>
          {path.hp_remaining}
        </span>
      </div>

      {/* Reward */}
      <div className="conquerer-panel-row">
        <span className="conquerer-panel-row-label">Reward</span>
        <span className="conquerer-panel-row-value">{path.total_reward.toFixed(2)}</span>
      </div>
    </div>
  );
};
