/**
 * Floor details panel showing path information and encounter predictions
 */

import { useGameStore } from '../../store/gameStore';
import { ROOM_COLORS, ROOM_SYMBOLS } from '../Map/types';
import type { RoomType } from '../../api/seedApi';
import './FloorDetails.css';

export function FloorDetails() {
  const {
    seed,
    seedValue,
    ascension,
    encounters,
    boss,
    currentAct,
    selectedPath,
    eventPredictions,
    clearPath,
  } = useGameStore();

  return (
    <div className="floor-details-panel">
      <div className="panel-title">Seed Details</div>

      {/* Seed Info */}
      {seed ? (
        <div className="seed-info">
          <div className="seed-display">{seed}</div>
          <div className="seed-value">{seedValue}</div>
          <div className="seed-ascension">Ascension {ascension}</div>
        </div>
      ) : (
        <div className="empty-state">
          <h3>No Seed Selected</h3>
          <p>Enter a seed to view predictions</p>
        </div>
      )}

      {/* Path Display */}
      <PathDisplay
        selectedPath={selectedPath}
        encounters={encounters}
        eventPredictions={eventPredictions}
        clearPath={clearPath}
      />

      {/* Encounter List */}
      {encounters && <EncounterList encounters={encounters} />}

      {/* Boss Section */}
      {boss && boss.name && <BossSection boss={boss} currentAct={currentAct} />}
    </div>
  );
}

interface PathDisplayProps {
  selectedPath: { x: number; y: number; type: RoomType }[];
  encounters: { normal: string[]; elite: string[] } | null;
  eventPredictions: Record<string, { outcome: string; event_name?: string }>;
  clearPath: () => void;
}

function PathDisplay({ selectedPath, encounters, eventPredictions, clearPath }: PathDisplayProps) {
  if (selectedPath.length === 0) {
    return (
      <div className="floor-details-card">
        <div className="empty-state">
          <h3>Click nodes to build a path</h3>
          <p>Start from floor 0 and trace your route</p>
        </div>
      </div>
    );
  }

  let normalIdx = 0;
  let eliteIdx = 0;

  return (
    <div className="floor-details-card">
      <div className="floor-header">
        <div className="floor-number" style={{ fontSize: '1rem' }}>
          Path
        </div>
        <button className="btn btn-small" onClick={clearPath}>
          Clear
        </button>
      </div>
      <div className="path-sequence">
        {selectedPath.map((p, i) => {
          let encounter = '';
          let symbol = p.type ? ROOM_SYMBOLS[p.type] : '?';
          let color = p.type ? ROOM_COLORS[p.type] : '#3d5a80';

          if (p.type === 'MONSTER' && encounters) {
            encounter = encounters.normal[normalIdx] || 'Unknown';
            normalIdx++;
          } else if (p.type === 'ELITE' && encounters) {
            encounter = encounters.elite[eliteIdx] || 'Unknown';
            eliteIdx++;
          } else if (p.type === 'REST') {
            encounter = 'Rest Site';
          } else if (p.type === 'SHOP') {
            encounter = 'Shop';
          } else if (p.type === 'TREASURE') {
            encounter = 'Treasure';
          } else if (p.type === 'EVENT') {
            const key = `${p.x},${p.y}`;
            const pred = eventPredictions[key];
            if (pred) {
              const outcome = pred.outcome;
              if (outcome === 'EVENT') {
                encounter = pred.event_name || 'Event';
                symbol = '?';
              } else if (outcome === 'MONSTER' && encounters) {
                encounter = encounters.normal[normalIdx] || 'Combat';
                normalIdx++;
                symbol = 'M';
                color = ROOM_COLORS.MONSTER;
              } else if (outcome === 'ELITE' && encounters) {
                encounter = encounters.elite[eliteIdx] || 'Elite';
                eliteIdx++;
                symbol = 'E';
                color = ROOM_COLORS.ELITE;
              } else if (outcome === 'SHOP') {
                encounter = 'Shop';
                symbol = '$';
                color = ROOM_COLORS.SHOP;
              } else if (outcome === 'TREASURE') {
                encounter = 'Treasure';
                symbol = 'T';
                color = ROOM_COLORS.TREASURE;
              }
            } else {
              encounter = '? Unknown';
            }
          }

          return (
            <div key={i} className="path-step">
              <span className="path-step-num" style={{ color }}>
                {i + 1}
              </span>
              <span className="path-step-symbol" style={{ color }}>
                {symbol}
              </span>
              <span className="path-step-encounter">{encounter}</span>
              <span className="path-step-floor">F{p.y + 1}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

interface EncounterListProps {
  encounters: { normal: string[]; elite: string[] };
}

function EncounterList({ encounters }: EncounterListProps) {
  return (
    <div className="encounter-list">
      <div className="panel-title">Encounters</div>

      <div className="encounter-section">
        {encounters.normal.map((enc, idx) => (
          <div key={idx} className="encounter-item">
            <span className="encounter-idx">{idx + 1}</span>
            <span className="encounter-name">{enc}</span>
            <span className="encounter-type">{idx < 3 ? 'Weak' : 'Strong'}</span>
          </div>
        ))}
      </div>

      <div className="panel-title" style={{ marginTop: '1rem' }}>
        Elites
      </div>
      {encounters.elite.slice(0, 5).map((enc, idx) => (
        <div key={idx} className="encounter-item">
          <span className="encounter-idx">{idx + 1}</span>
          <span className="encounter-name">{enc}</span>
        </div>
      ))}
    </div>
  );
}

interface BossSectionProps {
  boss: { name: string; hp: number; a9_hp?: number; move: string; details?: string };
  currentAct: number;
}

function BossSection({ boss, currentAct }: BossSectionProps) {
  return (
    <div className="boss-section">
      <div className="panel-title">Act {currentAct} Boss</div>
      <div className="boss-name">{boss.name}</div>
      <div className="boss-hp">HP: {boss.a9_hp || boss.hp}</div>
      <div className="boss-move">
        <strong>First Move:</strong> {boss.move}
        {boss.details && (
          <>
            <br />
            <em>{boss.details}</em>
          </>
        )}
      </div>
    </div>
  );
}
