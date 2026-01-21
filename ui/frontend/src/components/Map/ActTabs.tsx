/**
 * Act selection tabs
 */

import { useGameStore } from '../../store/gameStore';
import './ActTabs.css';

const ACTS = [
  { id: 1, label: 'Act I' },
  { id: 2, label: 'Act II' },
  { id: 3, label: 'Act III' },
  { id: 4, label: 'Act IV' },
];

export function ActTabs() {
  const { currentAct, setCurrentAct, isLoading } = useGameStore();

  return (
    <div className="act-tabs">
      {ACTS.map((act) => (
        <button
          key={act.id}
          className={`act-tab ${currentAct === act.id ? 'active' : ''}`}
          onClick={() => setCurrentAct(act.id)}
          disabled={isLoading}
        >
          {act.label}
        </button>
      ))}
    </div>
  );
}
