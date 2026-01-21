/**
 * PathDiff Component
 *
 * Side-by-side comparison table showing path differences.
 * Compares: floor, HP, gold, deck size, relics
 */

import React from 'react';
import { Path, FloorState } from '../DecisionTree/types';
import { EVBadge } from '../EVDisplay/EVBadge';

export interface PathDiffProps {
  paths: Path[];
  highlightDifferences?: boolean;
  className?: string;
}

// Path comparison colors
const PATH_COLORS = [
  '#3b82f6', // blue
  '#22c55e', // green
  '#f59e0b', // amber
  '#ec4899', // pink
];

export const PathDiff: React.FC<PathDiffProps> = ({
  paths,
  highlightDifferences = true,
  className = '',
}) => {
  if (paths.length === 0) {
    return (
      <div className={`path-diff ${className}`}>
        <div className="path-diff-empty">
          Select paths to compare
        </div>
      </div>
    );
  }

  // Find the maximum number of floors across all paths
  const maxFloors = Math.max(...paths.map((p) => p.floors.length));

  // Create a unified floor list
  const floorNumbers = Array.from({ length: maxFloors }, (_, i) => i + 1);

  // Get floor data for a path, or null if floor doesn't exist
  const getFloorData = (path: Path, floor: number): FloorState | null => {
    return path.floors.find((f) => f.floor === floor) || null;
  };

  // Check if values differ across paths for highlighting
  const valuesDiffer = (
    getter: (floor: FloorState | null) => number | undefined,
    floorNum: number
  ): boolean => {
    const values = paths.map((p) => getter(getFloorData(p, floorNum))).filter((v) => v !== undefined);
    if (values.length <= 1) return false;
    return new Set(values).size > 1;
  };

  return (
    <div className={`path-diff ${className}`}>
      <style>{`
        .path-diff {
          background: linear-gradient(180deg, rgba(26, 26, 37, 0.95) 0%, rgba(18, 18, 26, 0.95) 100%);
          border: 1px solid rgba(212, 168, 87, 0.2);
          border-radius: 8px;
          overflow: hidden;
        }

        .path-diff-header {
          display: flex;
          background: rgba(26, 26, 37, 0.8);
          border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        }

        .path-diff-header-cell {
          flex: 1;
          padding: 0.75rem 1rem;
          font-family: 'Cinzel', serif;
          font-size: 0.8rem;
          text-transform: uppercase;
          letter-spacing: 0.08em;
          text-align: center;
          border-right: 1px solid rgba(255, 255, 255, 0.05);
        }

        .path-diff-header-cell:last-child {
          border-right: none;
        }

        .path-diff-header-cell.floor-col {
          flex: 0 0 60px;
          background: rgba(212, 168, 87, 0.1);
          color: #d4a857;
        }

        .path-name-badge {
          display: inline-block;
          padding: 0.25rem 0.5rem;
          border-radius: 4px;
          font-size: 0.75rem;
          font-weight: 600;
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
          max-width: 100%;
        }

        .path-diff-body {
          max-height: 400px;
          overflow-y: auto;
        }

        .path-diff-row {
          display: flex;
          border-bottom: 1px solid rgba(255, 255, 255, 0.05);
        }

        .path-diff-row:last-child {
          border-bottom: none;
        }

        .path-diff-row:hover {
          background: rgba(212, 168, 87, 0.05);
        }

        .path-diff-cell {
          flex: 1;
          padding: 0.5rem 0.75rem;
          font-family: 'Crimson Text', serif;
          font-size: 0.85rem;
          display: flex;
          flex-direction: column;
          gap: 0.25rem;
          border-right: 1px solid rgba(255, 255, 255, 0.03);
        }

        .path-diff-cell:last-child {
          border-right: none;
        }

        .path-diff-cell.floor-col {
          flex: 0 0 60px;
          justify-content: center;
          align-items: center;
          background: rgba(212, 168, 87, 0.05);
          font-family: 'Cinzel', serif;
          font-weight: 600;
          color: #d4a857;
        }

        .path-diff-cell.empty {
          color: #3a3a4a;
          font-style: italic;
          justify-content: center;
          align-items: center;
        }

        .path-stat-row {
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .path-stat-label {
          font-size: 0.7rem;
          color: #6b7280;
          text-transform: uppercase;
        }

        .path-stat-value {
          font-weight: 600;
        }

        .path-stat-value.highlight {
          padding: 0.1rem 0.3rem;
          border-radius: 3px;
        }

        .path-stat-value.hp {
          color: #ef4444;
        }

        .path-stat-value.hp.highlight {
          background: rgba(239, 68, 68, 0.15);
        }

        .path-stat-value.gold {
          color: #eab308;
        }

        .path-stat-value.gold.highlight {
          background: rgba(234, 179, 8, 0.15);
        }

        .path-stat-value.deck {
          color: #3b82f6;
        }

        .path-stat-value.deck.highlight {
          background: rgba(59, 130, 246, 0.15);
        }

        .path-stat-value.relics {
          color: #a855f7;
        }

        .path-stat-value.relics.highlight {
          background: rgba(168, 85, 247, 0.15);
        }

        .path-room-type {
          font-family: 'Cinzel', serif;
          font-size: 0.7rem;
          text-transform: uppercase;
          letter-spacing: 0.05em;
          padding: 0.15rem 0.4rem;
          border-radius: 3px;
          display: inline-block;
          margin-bottom: 0.25rem;
        }

        .path-room-type.monster { background: rgba(139, 38, 53, 0.3); color: #8b2635; }
        .path-room-type.elite { background: rgba(212, 168, 87, 0.2); color: #d4a857; }
        .path-room-type.rest { background: rgba(45, 90, 61, 0.3); color: #2d5a3d; }
        .path-room-type.shop { background: rgba(234, 179, 8, 0.2); color: #eab308; }
        .path-room-type.event { background: rgba(61, 90, 128, 0.3); color: #3d5a80; }
        .path-room-type.treasure { background: rgba(212, 168, 87, 0.2); color: #d4a857; }
        .path-room-type.boss { background: rgba(90, 61, 110, 0.3); color: #5a3d6e; }

        .path-diff-summary {
          display: flex;
          background: rgba(26, 26, 37, 0.8);
          border-top: 1px solid rgba(255, 255, 255, 0.1);
        }

        .path-diff-summary-cell {
          flex: 1;
          padding: 0.75rem 1rem;
          text-align: center;
          border-right: 1px solid rgba(255, 255, 255, 0.05);
        }

        .path-diff-summary-cell:last-child {
          border-right: none;
        }

        .path-diff-summary-cell.floor-col {
          flex: 0 0 60px;
          font-family: 'Cinzel', serif;
          font-size: 0.7rem;
          color: #9a9486;
          display: flex;
          align-items: center;
          justify-content: center;
        }

        .path-diff-empty {
          padding: 2rem;
          text-align: center;
          color: #6b7280;
          font-family: 'Crimson Text', serif;
          font-style: italic;
        }
      `}</style>

      {/* Header with path names */}
      <div className="path-diff-header">
        <div className="path-diff-header-cell floor-col">Floor</div>
        {paths.map((path, i) => (
          <div key={path.id} className="path-diff-header-cell">
            <span
              className="path-name-badge"
              style={{
                backgroundColor: `${PATH_COLORS[i % PATH_COLORS.length]}20`,
                color: PATH_COLORS[i % PATH_COLORS.length],
                border: `1px solid ${PATH_COLORS[i % PATH_COLORS.length]}40`,
              }}
            >
              {path.name}
            </span>
          </div>
        ))}
      </div>

      {/* Body with floor comparisons */}
      <div className="path-diff-body">
        {floorNumbers.map((floorNum) => (
          <div key={floorNum} className="path-diff-row">
            <div className="path-diff-cell floor-col">{floorNum}</div>
            {paths.map((path) => {
              const floor = getFloorData(path, floorNum);
              if (!floor) {
                return (
                  <div key={path.id} className="path-diff-cell empty">
                    -
                  </div>
                );
              }

              const hpDiffers = highlightDifferences && valuesDiffer((f) => f?.hp, floorNum);
              const goldDiffers = highlightDifferences && valuesDiffer((f) => f?.gold, floorNum);
              const deckDiffers = highlightDifferences && valuesDiffer((f) => f?.deckSize, floorNum);
              const relicDiffers = highlightDifferences && valuesDiffer((f) => f?.relicCount, floorNum);

              return (
                <div key={path.id} className="path-diff-cell">
                  <span className={`path-room-type ${floor.roomType.toLowerCase()}`}>
                    {floor.roomType}
                  </span>

                  <div className="path-stat-row">
                    <span className="path-stat-label">HP</span>
                    <span className={`path-stat-value hp ${hpDiffers ? 'highlight' : ''}`}>
                      {floor.hp}/{floor.maxHp}
                    </span>
                  </div>

                  <div className="path-stat-row">
                    <span className="path-stat-label">Gold</span>
                    <span className={`path-stat-value gold ${goldDiffers ? 'highlight' : ''}`}>
                      {floor.gold}
                    </span>
                  </div>

                  <div className="path-stat-row">
                    <span className="path-stat-label">Deck</span>
                    <span className={`path-stat-value deck ${deckDiffers ? 'highlight' : ''}`}>
                      {floor.deckSize}
                    </span>
                  </div>

                  <div className="path-stat-row">
                    <span className="path-stat-label">Relics</span>
                    <span className={`path-stat-value relics ${relicDiffers ? 'highlight' : ''}`}>
                      {floor.relicCount}
                    </span>
                  </div>
                </div>
              );
            })}
          </div>
        ))}
      </div>

      {/* Summary row */}
      <div className="path-diff-summary">
        <div className="path-diff-summary-cell floor-col">Final EV</div>
        {paths.map((path) => (
          <div key={path.id} className="path-diff-summary-cell">
            <EVBadge ev={path.finalEV} size="lg" />
          </div>
        ))}
      </div>
    </div>
  );
};

export default PathDiff;
