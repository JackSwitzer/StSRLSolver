/**
 * TreeControls Component
 *
 * Provides controls for the decision tree visualization:
 * - Expand/collapse all nodes
 * - Reset view to initial state
 * - Adjust probability pruning threshold
 */

import React from 'react';
import { TreeControlsProps } from './types';

export const TreeControls: React.FC<TreeControlsProps> = ({
  onExpandAll,
  onCollapseAll,
  onResetView,
  pruneThreshold,
  onPruneThresholdChange,
}) => {
  const handleThresholdChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseFloat(e.target.value);
    if (!isNaN(value) && value >= 0 && value <= 1) {
      onPruneThresholdChange(value);
    }
  };

  return (
    <div className="tree-controls">
      <style>{`
        .tree-controls {
          display: flex;
          align-items: center;
          gap: 1rem;
          padding: 0.75rem 1rem;
          background: linear-gradient(180deg, rgba(26, 26, 37, 0.95) 0%, rgba(18, 18, 26, 0.95) 100%);
          border: 1px solid rgba(212, 168, 87, 0.2);
          border-radius: 8px;
          flex-wrap: wrap;
        }

        .tree-controls-section {
          display: flex;
          align-items: center;
          gap: 0.5rem;
        }

        .tree-controls-label {
          font-family: 'Cinzel', serif;
          font-size: 0.75rem;
          color: #9a9486;
          text-transform: uppercase;
          letter-spacing: 0.08em;
        }

        .tree-controls-btn {
          font-family: 'Cinzel', serif;
          font-size: 0.75rem;
          font-weight: 600;
          padding: 0.4rem 0.8rem;
          background: rgba(26, 26, 37, 0.8);
          border: 1px solid rgba(212, 168, 87, 0.3);
          border-radius: 4px;
          color: #d4a857;
          cursor: pointer;
          transition: all 0.2s ease;
          text-transform: uppercase;
          letter-spacing: 0.05em;
        }

        .tree-controls-btn:hover {
          background: rgba(212, 168, 87, 0.15);
          border-color: rgba(212, 168, 87, 0.5);
        }

        .tree-controls-btn:active {
          transform: translateY(1px);
        }

        .threshold-input {
          font-family: 'IM Fell English', serif;
          font-size: 0.9rem;
          padding: 0.3rem 0.5rem;
          width: 60px;
          background: rgba(26, 26, 37, 0.8);
          border: 1px solid rgba(212, 168, 87, 0.3);
          border-radius: 4px;
          color: #e8e4d9;
          text-align: center;
        }

        .threshold-input:focus {
          outline: none;
          border-color: #d4a857;
          box-shadow: 0 0 8px rgba(212, 168, 87, 0.2);
        }

        .threshold-slider {
          width: 100px;
          height: 4px;
          -webkit-appearance: none;
          background: rgba(212, 168, 87, 0.2);
          border-radius: 2px;
          cursor: pointer;
        }

        .threshold-slider::-webkit-slider-thumb {
          -webkit-appearance: none;
          width: 14px;
          height: 14px;
          background: #d4a857;
          border-radius: 50%;
          cursor: pointer;
        }

        .threshold-slider::-moz-range-thumb {
          width: 14px;
          height: 14px;
          background: #d4a857;
          border-radius: 50%;
          cursor: pointer;
          border: none;
        }

        .divider {
          width: 1px;
          height: 24px;
          background: rgba(255, 255, 255, 0.1);
        }
      `}</style>

      {/* Expand/Collapse Controls */}
      <div className="tree-controls-section">
        <span className="tree-controls-label">View</span>
        <button className="tree-controls-btn" onClick={onExpandAll}>
          Expand All
        </button>
        <button className="tree-controls-btn" onClick={onCollapseAll}>
          Collapse All
        </button>
        <button className="tree-controls-btn" onClick={onResetView}>
          Reset
        </button>
      </div>

      <div className="divider" />

      {/* Prune Threshold */}
      <div className="tree-controls-section">
        <span className="tree-controls-label">Prune Below</span>
        <input
          type="range"
          className="threshold-slider"
          min="0"
          max="0.2"
          step="0.01"
          value={pruneThreshold}
          onChange={handleThresholdChange}
        />
        <input
          type="text"
          className="threshold-input"
          value={`${(pruneThreshold * 100).toFixed(0)}%`}
          onChange={(e) => {
            const num = parseFloat(e.target.value.replace('%', ''));
            if (!isNaN(num)) {
              onPruneThresholdChange(num / 100);
            }
          }}
        />
      </div>

      {/* Legend */}
      <div className="divider" />
      <div className="tree-controls-section">
        <span className="tree-controls-label">EV</span>
        <span style={{ color: '#22c55e', fontSize: '0.8rem', fontFamily: 'Cinzel, serif' }}>
          +Good
        </span>
        <span style={{ color: '#6b7280', fontSize: '0.8rem', fontFamily: 'Cinzel, serif' }}>
          Neutral
        </span>
        <span style={{ color: '#ef4444', fontSize: '0.8rem', fontFamily: 'Cinzel, serif' }}>
          -Bad
        </span>
      </div>
    </div>
  );
};

export default TreeControls;
