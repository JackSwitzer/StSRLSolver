import type { MCTSResultMsg, PlannerResultMsg } from '../types/training';
import { MCTSViz } from './MCTSViz';

interface MCTSTabProps {
  mcts: MCTSResultMsg | null;
  planner?: PlannerResultMsg | null;
}

const PlannerStats = ({ planner }: { planner: PlannerResultMsg }) => {
  const confidenceColor = planner.confidence > 0.7 ? '#00ff41'
    : planner.confidence > 0.4 ? '#ffb700'
    : '#ff4444';

  return (
    <div style={{
      padding: '8px 12px',
      borderBottom: '1px solid #21262d',
      background: '#0d1117',
    }}>
      {/* Header */}
      <div style={{
        fontSize: '9px',
        color: '#8b949e',
        textTransform: 'uppercase',
        letterSpacing: '0.5px',
        marginBottom: '6px',
      }}>
        Planner Result
      </div>

      {/* Stats row */}
      <div style={{
        display: 'flex',
        gap: '16px',
        alignItems: 'center',
        flexWrap: 'wrap',
      }}>
        <StatBlock label="Strategy" value={planner.strategy} color="#00e5ff" />
        <StatBlock label="Lines" value={planner.lines_considered.toLocaleString()} color="#c9d1d9" />
        <StatBlock label="Turns to Kill" value={String(planner.turns_to_kill)} color="#ffb700" />
        <StatBlock label="Exp HP Loss" value={planner.expected_hp_loss.toFixed(1)} color="#ff4444" />
        <StatBlock label="Confidence" value={`${(planner.confidence * 100).toFixed(0)}%`} color={confidenceColor} />
        {planner.elapsed_ms !== undefined && (
          <StatBlock label="Time" value={`${planner.elapsed_ms.toFixed(0)}ms`} color="#8b949e" />
        )}
      </div>

      {/* Cards played */}
      {planner.cards_played.length > 0 && (
        <div style={{ marginTop: '6px' }}>
          <span style={{ fontSize: '9px', color: '#8b949e', marginRight: '6px' }}>Plan:</span>
          {planner.cards_played.map((card, i) => (
            <span key={i} style={{
              fontSize: '10px',
              color: '#c9d1d9',
              background: '#21262d',
              padding: '1px 5px',
              marginRight: '3px',
              display: 'inline-block',
            }}>
              {card}
            </span>
          ))}
        </div>
      )}
    </div>
  );
};

const StatBlock = ({ label, value, color = '#c9d1d9' }: {
  label: string; value: string; color?: string;
}) => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '1px' }}>
    <span style={{ fontSize: '8px', color: '#8b949e', textTransform: 'uppercase', letterSpacing: '0.3px' }}>{label}</span>
    <span style={{ fontSize: '12px', fontWeight: 700, color, fontFamily: 'monospace' }}>{value}</span>
  </div>
);

export const MCTSTab = ({ mcts, planner }: MCTSTabProps) => (
  <div style={{ height: '100%', overflow: 'auto', display: 'flex', flexDirection: 'column' }}>
    {planner && <PlannerStats planner={planner} />}
    <div style={{ padding: '8px 10px', flex: 1, overflow: 'auto', boxSizing: 'border-box' }}>
      <MCTSViz result={mcts} />
    </div>
  </div>
);
