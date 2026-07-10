import { type ReactNode } from 'react';
import { theme } from '../../styles/theme';
import { SparkLine } from '../charts/SparkLine';

interface MetricCardProps {
  label: string;
  value: string | number;
  unit?: string;
  trend?: number[];
  trendColor?: string;
  children?: ReactNode;
}

export function MetricCard({ label, value, unit, trend, trendColor }: MetricCardProps) {
  return (
    <div
      style={{
        position: 'relative',
        background: theme.bg.secondary,
        border: `1px solid ${theme.border}`,
        borderRadius: 8,
        padding: 16,
        overflow: 'hidden',
        minWidth: 0,
      }}
    >
      {/* Sparkline background */}
      {trend && trend.length > 1 && (
        <div
          style={{
            position: 'absolute',
            right: 8,
            bottom: 8,
            opacity: 0.4,
            pointerEvents: 'none',
          }}
        >
          <SparkLine values={trend} color={trendColor} width={80} height={32} />
        </div>
      )}

      {/* Label */}
      <div
        style={{
          fontSize: 12,
          fontWeight: 500,
          color: theme.text.secondary,
          marginBottom: 6,
          textTransform: 'uppercase',
          letterSpacing: '0.5px',
        }}
      >
        {label}
      </div>

      {/* Value */}
      <div
        style={{
          fontSize: 28,
          fontWeight: 700,
          color: theme.text.primary,
          letterSpacing: '-0.5px',
          lineHeight: 1.1,
          position: 'relative',
        }}
      >
        {value}
        {unit && (
          <span
            style={{
              fontSize: 14,
              fontWeight: 500,
              color: theme.text.secondary,
              marginLeft: 4,
            }}
          >
            {unit}
          </span>
        )}
      </div>
    </div>
  );
}
