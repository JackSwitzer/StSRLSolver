/**
 * PathSelector Component
 *
 * Allows users to save and load paths for comparison.
 * Features:
 * - Save current path with a name
 * - Load previously saved paths
 * - Delete saved paths
 * - Select paths for comparison
 */

import React, { useState } from 'react';
import { Path, FloorState } from '../DecisionTree/types';

export interface PathSelectorProps {
  savedPaths: Path[];
  currentPath?: FloorState[];
  selectedPaths: string[]; // IDs of selected paths for comparison
  onSavePath: (name: string, floors: FloorState[]) => void;
  onDeletePath: (id: string) => void;
  onSelectPath: (id: string) => void;
  onDeselectPath: (id: string) => void;
  maxComparisons?: number;
  className?: string;
}

export const PathSelector: React.FC<PathSelectorProps> = ({
  savedPaths,
  currentPath,
  selectedPaths,
  onSavePath,
  onDeletePath,
  onSelectPath,
  onDeselectPath,
  maxComparisons = 3,
  className = '',
}) => {
  const [newPathName, setNewPathName] = useState('');
  const [isNaming, setIsNaming] = useState(false);

  const handleSave = () => {
    if (!currentPath || currentPath.length === 0) return;
    if (!newPathName.trim()) return;

    onSavePath(newPathName.trim(), currentPath);
    setNewPathName('');
    setIsNaming(false);
  };

  const handleToggleSelect = (id: string) => {
    if (selectedPaths.includes(id)) {
      onDeselectPath(id);
    } else if (selectedPaths.length < maxComparisons) {
      onSelectPath(id);
    }
  };

  const formatDate = (date: Date) => {
    return new Date(date).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className={`path-selector ${className}`}>
      <style>{`
        .path-selector {
          background: linear-gradient(180deg, rgba(26, 26, 37, 0.95) 0%, rgba(18, 18, 26, 0.95) 100%);
          border: 1px solid rgba(212, 168, 87, 0.2);
          border-radius: 8px;
          padding: 1rem;
        }

        .path-selector-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 1rem;
        }

        .path-selector-title {
          font-family: 'Cinzel', serif;
          font-size: 0.85rem;
          color: #d4a857;
          text-transform: uppercase;
          letter-spacing: 0.1em;
        }

        .path-selector-save-btn {
          font-family: 'Cinzel', serif;
          font-size: 0.7rem;
          font-weight: 600;
          padding: 0.4rem 0.8rem;
          background: rgba(34, 197, 94, 0.15);
          border: 1px solid rgba(34, 197, 94, 0.4);
          border-radius: 4px;
          color: #22c55e;
          cursor: pointer;
          transition: all 0.2s ease;
          text-transform: uppercase;
        }

        .path-selector-save-btn:hover:not(:disabled) {
          background: rgba(34, 197, 94, 0.25);
        }

        .path-selector-save-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .path-name-input {
          display: flex;
          gap: 0.5rem;
          margin-bottom: 1rem;
        }

        .path-name-input input {
          flex: 1;
          font-family: 'Crimson Text', serif;
          font-size: 0.9rem;
          padding: 0.5rem 0.75rem;
          background: rgba(26, 26, 37, 0.8);
          border: 1px solid rgba(212, 168, 87, 0.3);
          border-radius: 4px;
          color: #e8e4d9;
        }

        .path-name-input input:focus {
          outline: none;
          border-color: #d4a857;
        }

        .path-name-input input::placeholder {
          color: #6b7280;
        }

        .path-list {
          display: flex;
          flex-direction: column;
          gap: 0.5rem;
          max-height: 300px;
          overflow-y: auto;
        }

        .path-item {
          display: flex;
          align-items: center;
          gap: 0.75rem;
          padding: 0.75rem;
          background: rgba(26, 26, 37, 0.5);
          border: 1px solid rgba(255, 255, 255, 0.08);
          border-radius: 6px;
          cursor: pointer;
          transition: all 0.2s ease;
        }

        .path-item:hover {
          background: rgba(26, 26, 37, 0.8);
          border-color: rgba(212, 168, 87, 0.3);
        }

        .path-item.selected {
          background: rgba(212, 168, 87, 0.1);
          border-color: rgba(212, 168, 87, 0.4);
        }

        .path-checkbox {
          width: 18px;
          height: 18px;
          border: 2px solid rgba(212, 168, 87, 0.4);
          border-radius: 3px;
          display: flex;
          align-items: center;
          justify-content: center;
          flex-shrink: 0;
          transition: all 0.2s ease;
        }

        .path-checkbox.checked {
          background: #d4a857;
          border-color: #d4a857;
        }

        .path-checkbox-check {
          color: #0a0a0f;
          font-size: 12px;
          font-weight: bold;
        }

        .path-info {
          flex: 1;
          min-width: 0;
        }

        .path-name {
          font-family: 'Cinzel', serif;
          font-size: 0.85rem;
          color: #e8e4d9;
          margin-bottom: 0.25rem;
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }

        .path-meta {
          display: flex;
          gap: 0.75rem;
          font-family: 'Crimson Text', serif;
          font-size: 0.75rem;
          color: #6b7280;
        }

        .path-ev {
          font-family: 'Cinzel', serif;
          font-size: 0.85rem;
          font-weight: 600;
        }

        .path-ev.positive {
          color: #22c55e;
        }

        .path-ev.negative {
          color: #ef4444;
        }

        .path-ev.neutral {
          color: #6b7280;
        }

        .path-delete-btn {
          padding: 0.25rem 0.5rem;
          background: transparent;
          border: 1px solid rgba(239, 68, 68, 0.3);
          border-radius: 3px;
          color: #ef4444;
          cursor: pointer;
          font-size: 0.7rem;
          opacity: 0;
          transition: all 0.2s ease;
        }

        .path-item:hover .path-delete-btn {
          opacity: 1;
        }

        .path-delete-btn:hover {
          background: rgba(239, 68, 68, 0.15);
        }

        .path-empty {
          text-align: center;
          padding: 2rem;
          color: #6b7280;
          font-family: 'Crimson Text', serif;
          font-style: italic;
        }

        .selection-count {
          font-family: 'Crimson Text', serif;
          font-size: 0.8rem;
          color: #9a9486;
          margin-top: 0.75rem;
          text-align: center;
        }
      `}</style>

      <div className="path-selector-header">
        <span className="path-selector-title">Saved Paths</span>
        <button
          className="path-selector-save-btn"
          onClick={() => setIsNaming(true)}
          disabled={!currentPath || currentPath.length === 0}
        >
          + Save Current
        </button>
      </div>

      {isNaming && (
        <div className="path-name-input">
          <input
            type="text"
            placeholder="Enter path name..."
            value={newPathName}
            onChange={(e) => setNewPathName(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleSave()}
            autoFocus
          />
          <button
            className="path-selector-save-btn"
            onClick={handleSave}
            disabled={!newPathName.trim()}
          >
            Save
          </button>
          <button
            className="path-selector-save-btn"
            style={{
              background: 'transparent',
              borderColor: 'rgba(239, 68, 68, 0.4)',
              color: '#ef4444',
            }}
            onClick={() => {
              setIsNaming(false);
              setNewPathName('');
            }}
          >
            Cancel
          </button>
        </div>
      )}

      <div className="path-list">
        {savedPaths.length === 0 ? (
          <div className="path-empty">
            No saved paths yet. Build a path and save it for comparison.
          </div>
        ) : (
          savedPaths.map((path) => {
            const isSelected = selectedPaths.includes(path.id);
            const evClass = path.finalEV > 0.1 ? 'positive' : path.finalEV < -0.1 ? 'negative' : 'neutral';

            return (
              <div
                key={path.id}
                className={`path-item ${isSelected ? 'selected' : ''}`}
                onClick={() => handleToggleSelect(path.id)}
              >
                <div className={`path-checkbox ${isSelected ? 'checked' : ''}`}>
                  {isSelected && <span className="path-checkbox-check">check</span>}
                </div>

                <div className="path-info">
                  <div className="path-name">{path.name}</div>
                  <div className="path-meta">
                    <span>{path.floors.length} floors</span>
                    <span>{formatDate(path.createdAt)}</span>
                  </div>
                </div>

                <div className={`path-ev ${evClass}`}>
                  {path.finalEV >= 0 ? '+' : ''}{path.finalEV.toFixed(2)}
                </div>

                <button
                  className="path-delete-btn"
                  onClick={(e) => {
                    e.stopPropagation();
                    onDeletePath(path.id);
                  }}
                >
                  Delete
                </button>
              </div>
            );
          })
        )}
      </div>

      {savedPaths.length > 0 && (
        <div className="selection-count">
          {selectedPaths.length}/{maxComparisons} selected for comparison
        </div>
      )}
    </div>
  );
};

export default PathSelector;
