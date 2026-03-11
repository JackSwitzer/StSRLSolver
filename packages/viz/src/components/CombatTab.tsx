import type { CombatState } from '../types/game';
import { CombatView } from './CombatView';

interface CombatTabProps {
  combat: CombatState | null;
  phase: string;
  lastAction?: string;
}

export const CombatTab = ({ combat, phase, lastAction }: CombatTabProps) => {
  if (!combat) {
    return (
      <div style={{ padding: '12px', fontSize: '11px', color: '#8b949e' }}>
        {phase === 'COMBAT' ? 'Waiting for combat state...' : `Agent is in ${phase} phase — no combat active`}
      </div>
    );
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
      {lastAction && (
        <div style={{
          padding: '3px 10px',
          borderBottom: '1px solid #21262d',
          fontSize: '10px',
          color: '#8b949e',
          flexShrink: 0,
        }}>
          Last: <span style={{ color: '#ffb700' }}>{lastAction}</span>
        </div>
      )}
      <div style={{ flex: 1, overflow: 'auto' }}>
        <CombatView combat={combat} compact={true} />
      </div>
    </div>
  );
};
