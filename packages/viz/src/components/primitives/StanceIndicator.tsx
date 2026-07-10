import type { Stance } from '../../types/engine';
import { STANCE_COLORS } from '../../types/engine';

interface StanceIndicatorProps {
  stance: Stance;
  size?: 'sm' | 'md';
}

const SIZE_MAP = {
  sm: { dot: 8, fontSize: 0, gap: 0, padding: '2px 4px' },
  md: { dot: 10, fontSize: 11, gap: 6, padding: '3px 8px' },
} as const;

export function StanceIndicator({ stance, size = 'md' }: StanceIndicatorProps) {
  const color = STANCE_COLORS[stance];
  const { dot, fontSize, gap, padding } = SIZE_MAP[size];
  const showLabel = size !== 'sm';

  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap,
        padding,
        borderRadius: 12,
        background: `${color}18`,
        border: `1px solid ${color}44`,
      }}
    >
      <span
        style={{
          display: 'inline-block',
          width: dot,
          height: dot,
          borderRadius: '50%',
          background: color,
          boxShadow: `0 0 4px ${color}88`,
        }}
      />
      {showLabel && (
        <span
          style={{
            fontSize,
            fontWeight: 600,
            color,
            textTransform: 'capitalize',
          }}
        >
          {stance}
        </span>
      )}
    </span>
  );
}
