import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from 'recharts';
import { theme } from '../../styles/theme';

interface SeriesDef {
  key: string;
  color: string;
  name: string;
}

interface TimeSeriesChartProps {
  data: Record<string, unknown>[];
  xKey: string;
  series: SeriesDef[];
  height?: number;
}

export function TimeSeriesChart({
  data,
  xKey,
  series,
  height = 300,
}: TimeSeriesChartProps) {
  return (
    <ResponsiveContainer width="100%" height={height}>
      <LineChart data={data} margin={{ top: 8, right: 16, bottom: 8, left: 0 }}>
        <CartesianGrid stroke={theme.bg.tertiary} strokeDasharray="3 3" />
        <XAxis
          dataKey={xKey}
          stroke={theme.text.muted}
          tick={{ fill: theme.text.secondary, fontSize: 11 }}
          tickLine={{ stroke: theme.text.muted }}
        />
        <YAxis
          stroke={theme.text.muted}
          tick={{ fill: theme.text.secondary, fontSize: 11 }}
          tickLine={{ stroke: theme.text.muted }}
        />
        <Tooltip
          contentStyle={{
            background: theme.bg.secondary,
            border: `1px solid ${theme.border}`,
            borderRadius: 6,
            fontSize: 12,
            color: theme.text.primary,
          }}
          labelStyle={{ color: theme.text.secondary }}
          itemStyle={{ padding: '2px 0' }}
        />
        {series.length > 1 && (
          <Legend
            wrapperStyle={{ fontSize: 12, color: theme.text.secondary }}
          />
        )}
        {series.map(s => (
          <Line
            key={s.key}
            type="monotone"
            dataKey={s.key}
            name={s.name}
            stroke={s.color}
            strokeWidth={2}
            dot={false}
            activeDot={{ r: 3, fill: s.color }}
          />
        ))}
      </LineChart>
    </ResponsiveContainer>
  );
}
