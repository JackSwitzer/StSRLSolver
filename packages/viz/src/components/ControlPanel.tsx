import { useState } from 'react';
import type { SystemStatsMsg } from '../types/training';

interface ControlPanelProps {
  onClose: () => void;
  onStart: (config: { num_agents: number; mcts_sims: number; ascension: number }) => void;
  onPause: () => void;
  onResume: () => void;
  onStop: () => void;
  isRunning: boolean;
  isPaused: boolean;
  sendControl: (params: Record<string, unknown>) => void;
  systemStats: SystemStatsMsg | null;
}

const Btn = ({ children, onClick, color = '#30363d', textColor = '#c9d1d9', small = false, disabled = false }: {
  children: React.ReactNode;
  onClick: () => void;
  color?: string;
  textColor?: string;
  small?: boolean;
  disabled?: boolean;
}) => (
  <button
    onClick={onClick}
    disabled={disabled}
    style={{
      background: disabled ? '#21262d' : color,
      border: `1px solid ${color === '#30363d' ? '#484f58' : color}`,
      color: disabled ? '#484f58' : textColor,
      padding: small ? '2px 8px' : '4px 14px',
      fontSize: small ? '10px' : '11px',
      fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
      cursor: disabled ? 'not-allowed' : 'pointer',
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

const NumInput = ({ label, value, onChange, min, max, step, unit }: {
  label: string;
  value: number;
  onChange: (v: number) => void;
  min: number;
  max: number;
  step: number;
  unit?: string;
}) => (
  <div style={{ display: 'flex', alignItems: 'center', gap: '8px', justifyContent: 'space-between' }}>
    <span style={{ fontSize: '11px', color: '#8b949e', minWidth: '110px' }}>{label}</span>
    <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
      <input
        type="number"
        value={value}
        min={min}
        max={max}
        step={step}
        onChange={(e) => {
          const v = parseFloat(e.target.value);
          if (!isNaN(v) && v >= min && v <= max) onChange(v);
        }}
        style={{
          width: '72px',
          background: '#0d1117',
          border: '1px solid #30363d',
          color: '#00ff41',
          fontSize: '11px',
          fontFamily: "'JetBrains Mono', monospace",
          padding: '2px 6px',
          textAlign: 'right',
        }}
      />
      {unit && <span style={{ fontSize: '10px', color: '#484f58' }}>{unit}</span>}
    </div>
  </div>
);

const SectionHeader = ({ children }: { children: React.ReactNode }) => (
  <div style={{
    fontSize: '10px',
    color: '#58a6ff',
    letterSpacing: '1px',
    textTransform: 'uppercase',
    marginBottom: '8px',
    borderBottom: '1px solid #21262d',
    paddingBottom: '4px',
  }}>
    {children}
  </div>
);

export const ControlPanel = ({
  onClose, onStart, onPause, onResume, onStop,
  isRunning, isPaused, sendControl, systemStats,
}: ControlPanelProps) => {
  // Server config
  const [workers, setWorkers] = useState(systemStats?.workers ?? 8);
  const [sims, setSims] = useState(32);
  const [ascension, setAscension] = useState(20);

  // Training params (hot-reload)
  const [entropy, setEntropy] = useState(0.05);
  const [lr, setLr] = useState(0.0001);
  const [temperature, setTemperature] = useState(1.0);
  const [solverBudget, setSolverBudget] = useState(50);

  // Status message
  const [status, setStatus] = useState('');

  function handleApplyServerConfig() {
    sendControl({ workers, sims, ascension });
    setStatus('Server config applied');
    setTimeout(() => setStatus(''), 3000);
  }

  function handleApplyTrainingParams() {
    sendControl({ entropy, lr, temperature, solver_budget_ms: solverBudget });
    setStatus('Training params sent (hot-reload)');
    setTimeout(() => setStatus(''), 3000);
  }

  function handleStart() {
    onStart({ num_agents: workers, mcts_sims: sims, ascension });
    setStatus('Training started');
    setTimeout(() => setStatus(''), 3000);
  }

  function handlePauseResume() {
    if (isPaused) {
      onResume();
      setStatus('Resumed');
    } else {
      onPause();
      setStatus('Paused');
    }
    setTimeout(() => setStatus(''), 3000);
  }

  function handleStop() {
    onStop();
    setStatus('Stopped');
    setTimeout(() => setStatus(''), 3000);
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
        minWidth: '480px',
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

      {/* Two-column layout */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px', marginBottom: '14px' }}>
        {/* Left: Server Config */}
        <div>
          <SectionHeader>Server Config</SectionHeader>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
            <Stepper label="Workers" value={workers} onChange={setWorkers} min={1} max={16} />
            <Stepper label="MCTS Sims" value={sims} onChange={setSims} min={8} max={256} step={8} />
            <Stepper label="Ascension" value={ascension} onChange={setAscension} min={0} max={20} />
          </div>
          <div style={{ marginTop: '10px' }}>
            <Btn onClick={handleApplyServerConfig} color="#1f6feb" textColor="#ffffff" disabled={!isRunning}>
              Apply Server Config
            </Btn>
          </div>
        </div>

        {/* Right: Training Params */}
        <div>
          <SectionHeader>Training Params</SectionHeader>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
            <NumInput label="Entropy Coeff" value={entropy} onChange={setEntropy} min={0.01} max={0.20} step={0.01} />
            <NumInput label="Learning Rate" value={lr} onChange={setLr} min={0.00001} max={0.001} step={0.0001} />
            <NumInput label="Temperature" value={temperature} onChange={setTemperature} min={0.1} max={2.0} step={0.1} />
            <NumInput label="Solver Budget" value={solverBudget} onChange={setSolverBudget} min={5} max={200} step={5} unit="ms" />
          </div>
          <div style={{ marginTop: '10px' }}>
            <Btn onClick={handleApplyTrainingParams} color="#1f6feb" textColor="#ffffff" disabled={!isRunning}>
              Apply Training Params
            </Btn>
          </div>
        </div>
      </div>

      {/* Divider */}
      <div style={{ borderTop: '1px solid #21262d', marginBottom: '12px' }} />

      {/* Actions */}
      <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap', alignItems: 'center' }}>
        {!isRunning && (
          <Btn onClick={handleStart} color="#1a4d2a" textColor="#00ff41">Start</Btn>
        )}
        {isRunning && (
          <>
            <Btn onClick={handlePauseResume} color={isPaused ? '#1a4d2a' : '#30363d'} textColor={isPaused ? '#00ff41' : '#c9d1d9'}>
              {isPaused ? 'Resume' : 'Pause'}
            </Btn>
            <Btn onClick={handleStop} color="#6e1a1a" textColor="#ff4444">Stop</Btn>
          </>
        )}
        <Btn onClick={onClose} color="#21262d" textColor="#8b949e">Close [C]</Btn>

        {/* Status indicator */}
        {status && (
          <span style={{ fontSize: '10px', color: '#58a6ff', marginLeft: '8px' }}>
            {status}
          </span>
        )}
      </div>

      {/* Current values from server */}
      {systemStats && (
        <div style={{ marginTop: '10px', fontSize: '10px', color: '#484f58' }}>
          Server: {systemStats.workers} workers | CPU {systemStats.cpu_pct.toFixed(0)}% | RAM {systemStats.ram_used_gb.toFixed(1)}/{systemStats.ram_total_gb.toFixed(1)} GB
          {systemStats.paused ? ' | PAUSED' : ''}
        </div>
      )}
    </div>
  );
};
