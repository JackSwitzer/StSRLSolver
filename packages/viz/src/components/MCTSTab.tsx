import type { MCTSResultMsg } from '../types/training';
import { MCTSViz } from './MCTSViz';

interface MCTSTabProps {
  mcts: MCTSResultMsg | null;
}

export const MCTSTab = ({ mcts }: MCTSTabProps) => (
  <div style={{ padding: '8px 10px', height: '100%', overflow: 'auto', boxSizing: 'border-box' }}>
    <MCTSViz result={mcts} />
  </div>
);
