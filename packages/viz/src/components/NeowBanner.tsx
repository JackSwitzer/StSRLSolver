import type { DecisionSummary } from '../types/training';

interface NeowBannerProps {
  decisions?: DecisionSummary[];
  neowChoice?: string;
}

export const NeowBanner: React.FC<NeowBannerProps> = ({ decisions, neowChoice }) => {
  const neow = decisions?.find((d) => d.type === 'neow');
  const choice = neowChoice || neow?.choice || 'Unknown';
  const detail = neow?.detail;

  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        height: 36,
        padding: '0 12px',
        background: '#161b22',
        border: '1px solid #30363d',
        borderRadius: 4,
        fontFamily: 'inherit',
        fontSize: 13,
      }}
    >
      <span style={{ color: '#ffd700', fontWeight: 600 }}>Neow:</span>
      <span style={{ color: '#c9d1d9' }}>{choice}</span>
      {detail && (
        <span style={{ color: '#8b949e', fontSize: 12 }}>({detail})</span>
      )}
    </div>
  );
};
