import { useState } from 'react';

interface ControlPanelProps {
  onClose: () => void;
  onStart: (config: { num_agents: number; mcts_sims: number; ascension: number }) => void;
  onPause: () => void;
  onStop: () => void;
  isRunning: boolean;
  sendControl: (config: { num_agents?: number; mcts_sims?: number; ascension?: number }) => void;
}

const Btn = ({ children, onClick, color = '#30363d', textColor = '#c9d1d9', small = false }: {
  children: React.ReactNode;
  onClick: () => void;
  color?: string;
  textColor?: string;
  small?: boolean;
}) => (
  <button
    onClick={onClick}
    style={{
      background: color,
      border: `1px solid ${color === '#30363d' ? '#484f58' : color}`,
      color: textColor,
      padding: small ? '2px 8px' : '4px 14px',
      fontSize: small ? '10px' : '11px',
      fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
      cursor: 'pointer',
      letterSpacing: '0.3px',
    }}
  >
    {children}
  </button>
);

const Stepper = ({ label, value, onChange, min = 0, max = 64, step = 1 }: {
  label: string;
  value: number;
  onChange: (v: number) => void;
  min?: number;
  max?: number;
  step?: number;
}) => (
  <div style={{ display: 'flex', alignItems: 'center', gap: '8px', justifyContent: 'space-between' }}>
    <span style={{ fontSize: '11px', color: '#8b949e', minWidth: '90px' }}>{label}</span>
    <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
      <Btn small onClick={() => onChange(Math.max(min, value - step))}>-</Btn>
      <span style={{ fontSize: '12px', color: '#00ff41', fontFamily: 'monospace', minWidth: '32px', textAlign: 'center' }}>{value}</span>
      <Btn small onClick={() => onChange(Math.min(max, value + step))}>+</Btn>
    </div>
  </div>
);

export const ControlPanel = ({ onClose, onStart, onPause, onStop, isRunning, sendControl }: ControlPanelProps) => {
  const [workers, setWorkers] = useState(8);
  const [sims, setSims] = useState(32);
  const [ascension, setAscension] = useState(20);

  function handleApply() {
    sendControl({ num_agents: workers, mcts_sims: sims, ascension });
  }

  function handleStart() {
    onStart({ num_agents: workers, mcts_sims: sims, ascension });
  }

  return (
    <div
      style={{
        position: 'fixed',
        top: '50%',
        left: '50%',
        transform: 'translate(-50%, -50%)',
        zIndex: 100,
        background: '#0d1117',
        border: '1px solid #30363d',
        padding: '16px 20px',
        minWidth: '240px',
        fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
        boxShadow: '0 8px 32px rgba(0,0,0,0.6)',
      }}
    >
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '14px' }}>
        <span style={{ fontSize: '12px', color: '#c9d1d9', letterSpacing: '1px', textTransform: 'uppercase' }}>
          Control Panel
        </span>
        <button
          onClick={onClose}
          style={{ background: 'none', border: 'none', color: '#8b949e', cursor: 'pointer', fontSize: '12px', padding: '0 2px' }}
        >
          [X]
        </button>
      </div>

      {/* Settings */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginBottom: '16px' }}>
        <Stepper label="Workers" value={workers} onChange={setWorkers} min={1} max={16} />
        <Stepper label="MCTS Sims" value={sims} onChange={setSims} min={8} max={256} step={8} />
        <Stepper label="Ascension" value={ascension} onChange={setAscension} min={0} max={20} />
      </div>

      {/* Divider */}
      <div style={{ borderTop: '1px solid #21262d', marginBottom: '12px' }} />

      {/* Actions */}
      <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap' }}>
        {isRunning ? (
          <>
            <Btn onClick={handleApply} color="#1f6feb" textColor="#ffffff">Apply</Btn>
            <Btn onClick={onPause} color="#30363d">Pause</Btn>
            <Btn onClick={onStop} color="#6e1a1a" textColor="#ff4444">Stop</Btn>
          </>
        ) : (
          <Btn onClick={handleStart} color="#1a4d2a" textColor="#00ff41">Start</Btn>
        )}
        <Btn onClick={onClose} color="#21262d" textColor="#8b949e">Close [C]</Btn>
      </div>
    </div>
  );
};
