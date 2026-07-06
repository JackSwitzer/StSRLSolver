import { theme } from '../../styles/theme';
import type { CardType } from '../../types/engine';
import { CARD_TYPE_COLORS } from '../../types/engine';

interface CardChipProps {
  name: string;
  type?: CardType;
  upgraded?: boolean;
  cost?: number;
  highlighted?: boolean;
}

export function CardChip({ name, type, upgraded, cost, highlighted }: CardChipProps) {
  const color = type ? CARD_TYPE_COLORS[type] : theme.text.muted;

  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: 4,
        padding: '2px 8px',
        borderRadius: 12,
        fontSize: 12,
        fontWeight: 500,
        lineHeight: '18px',
        whiteSpace: 'nowrap',
        color: theme.text.primary,
        background: `${color}22`,
        border: `1px solid ${highlighted ? color : `${color}44`}`,
        boxShadow: highlighted ? `0 0 6px ${color}66` : 'none',
        transition: 'box-shadow 200ms ease',
      }}
    >
      {cost !== undefined && (
        <span
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            justifyContent: 'center',
            width: 16,
            height: 16,
            borderRadius: '50%',
            fontSize: 10,
            fontWeight: 700,
            background: `${color}33`,
            color,
          }}
        >
          {cost}
        </span>
      )}
      <span>
        {name}
        {upgraded && (
          <span style={{ color: theme.success, fontWeight: 700 }}>+</span>
        )}
      </span>
    </span>
  );
}
