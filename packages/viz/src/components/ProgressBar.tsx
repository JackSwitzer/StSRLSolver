import type { PathResult } from '../types/conquerer';

/** Max floors per act boundary (approximate for A20). */
const ACT_BOUNDARIES = [
  { end: 17, color: '#3366cc', label: 'Act 1' },
  { end: 34, color: '#44aa44', label: 'Act 2' },
  { end: 51, color: '#cc3333', label: 'Act 3' },
  { end: 55, color: '#daa520', label: 'Act 4' },
] as const;

const TOTAL_FLOORS = 55;

interface ProgressBarProps {
  path: PathResult;
  /** Whether this is the best path. */
  isBest: boolean;
}

export const ProgressBar = ({ path, isBest }: ProgressBarProps) => {
  const pct = (path.floors_reached / TOTAL_FLOORS) * 100;

  return (
    <div className="conquerer-progress-row">
      <span className="conquerer-progress-label" title={path.strategy}>
        #{path.path_id}
      </span>
      <div className="conquerer-progress-track">
        {/* Act color segments (background) */}
        {ACT_BOUNDARIES.map((act, i) => {
          const prevEnd = i > 0 ? ACT_BOUNDARIES[i - 1].end : 0;
          const left = (prevEnd / TOTAL_FLOORS) * 100;
          const width = ((act.end - prevEnd) / TOTAL_FLOORS) * 100;
          return (
            <div
              key={act.label}
              className="conquerer-progress-act-segment"
              style={{ left: `${left}%`, width: `${width}%`, background: act.color }}
              title={act.label}
            />
          );
        })}
        {/* Filled overlay up to floor reached */}
        <div
          className="conquerer-progress-fill"
          style={{
            width: `${pct}%`,
            boxShadow: isBest ? '0 0 6px #daa520' : undefined,
          }}
        />
        {/* Current position marker */}
        {path.floors_reached > 0 && path.floors_reached < TOTAL_FLOORS && (
          <div
            className="conquerer-progress-marker"
            style={{ left: `${pct}%` }}
          />
        )}
      </div>
      <span className="conquerer-progress-floor">
        F{path.floors_reached}
      </span>
    </div>
  );
};
