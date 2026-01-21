/**
 * Seed input component with ascension selector
 */

import { useState, KeyboardEvent } from 'react';
import { useGameStore } from '../../store/gameStore';
import './SeedInput.css';

const ASCENSION_OPTIONS = [0, 1, 5, 10, 15, 17, 20];

export function SeedInput() {
  const { seed, ascension, isLoading, fetchMapData, setAscension } = useGameStore();
  const [inputValue, setInputValue] = useState(seed || 'A20WIN');

  const handleSubmit = () => {
    if (inputValue.trim()) {
      fetchMapData(inputValue.trim(), undefined, ascension);
    }
  };

  const handleKeyPress = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      handleSubmit();
    }
  };

  const handleAscensionChange = (newAscension: number) => {
    setAscension(newAscension);
    if (seed) {
      fetchMapData(seed, undefined, newAscension);
    }
  };

  return (
    <div className="seed-input-container">
      <input
        type="text"
        className="input seed-input"
        placeholder="Enter seed..."
        value={inputValue}
        onChange={(e) => setInputValue(e.target.value)}
        onKeyPress={handleKeyPress}
        disabled={isLoading}
      />
      <select
        className="select ascension-select"
        value={ascension}
        onChange={(e) => handleAscensionChange(parseInt(e.target.value))}
        disabled={isLoading}
      >
        {ASCENSION_OPTIONS.map((asc) => (
          <option key={asc} value={asc}>
            A{asc}
          </option>
        ))}
      </select>
      <button className="btn" onClick={handleSubmit} disabled={isLoading}>
        {isLoading ? 'Loading...' : 'Divine'}
      </button>
    </div>
  );
}
