import { theme } from '../../styles/theme';

interface RelicBadgeProps {
  name: string;
}

export function RelicBadge({ name }: RelicBadgeProps) {
  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        padding: '2px 8px',
        borderRadius: 12,
        fontSize: 11,
        fontWeight: 500,
        lineHeight: '18px',
        whiteSpace: 'nowrap',
        color: theme.chart.yellow,
        background: `${theme.chart.yellow}12`,
        border: `1px solid ${theme.chart.yellow}44`,
      }}
    >
      {name}
    </span>
  );
}
