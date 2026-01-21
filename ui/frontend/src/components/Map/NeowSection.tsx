/**
 * Neow's Blessing options display
 */

import { useGameStore } from '../../store/gameStore';
import './NeowSection.css';

export function NeowSection() {
  const { neowOptions, currentAct } = useGameStore();

  // Only show on Act 1
  if (currentAct !== 1) {
    return null;
  }

  if (neowOptions.length === 0) {
    return (
      <div className="neow-section">
        <div className="neow-title">Neow's Blessing</div>
        <div className="neow-options">
          <div className="empty-state">Enter a seed to reveal Neow's offerings</div>
        </div>
      </div>
    );
  }

  const handleOptionClick = (option: string) => {
    console.log('Selected Neow option:', option);
    // Future: Could trigger recalculation based on Neow choice
  };

  return (
    <div className="neow-section">
      <div className="neow-title">Neow's Blessing</div>
      <div className="neow-options">
        {neowOptions.map((opt) => (
          <div
            key={opt.slot}
            className="neow-option"
            onClick={() => handleOptionClick(opt.option)}
          >
            <div className="neow-slot">Option {opt.slot}</div>
            <div className="neow-name">{opt.name}</div>
            {opt.drawback && <div className="neow-drawback">{opt.drawback}</div>}
          </div>
        ))}
      </div>
    </div>
  );
}
