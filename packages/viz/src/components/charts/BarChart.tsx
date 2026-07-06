import {
  BarChart as RechartsBarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';
import { theme } from '../../styles/theme';

interface BarChartProps {
  data: Record<string, unknown>[];
  xKey: string;
  yKey: string;
  color?: string;
  horizontal?: boolean;
  height?: number;
}

export function BarChart({
  data,
  xKey,
  yKey,
  color = theme.chart.blue,
  horizontal = false,
  height = 300,
}: BarChartProps) {
  if (horizontal) {
    return (
      <ResponsiveContainer width="100%" height={height}>
        <RechartsBarChart
          data={data}
          layout="vertical"
          margin={{ top: 8, right: 16, bottom: 8, left: 0 }}
        >
          <CartesianGrid stroke={theme.bg.tertiary} strokeDasharray="3 3" horizontal={false} />
          <XAxis
            type="number"
            stroke={theme.text.muted}
            tick={{ fill: theme.text.secondary, fontSize: 11 }}
            tickLine={{ stroke: theme.text.muted }}
          />
          <YAxis
            dataKey={xKey}
            type="category"
            stroke={theme.text.muted}
            tick={{ fill: theme.text.secondary, fontSize: 11 }}
            tickLine={{ stroke: theme.text.muted }}
            width={80}
          />
          <Tooltip
            contentStyle={{
              background: theme.bg.secondary,
              border: `1px solid ${theme.border}`,
              borderRadius: 6,
              fontSize: 12,
              color: theme.text.primary,
            }}
            cursor={{ fill: `${theme.bg.hover}88` }}
          />
          <Bar
            dataKey={yKey}
            fill={color}
            radius={[0, 4, 4, 0]}
            maxBarSize={24}
          />
        </RechartsBarChart>
      </ResponsiveContainer>
    );
  }

  return (
    <ResponsiveContainer width="100%" height={height}>
      <RechartsBarChart data={data} margin={{ top: 8, right: 16, bottom: 8, left: 0 }}>
        <CartesianGrid stroke={theme.bg.tertiary} strokeDasharray="3 3" vertical={false} />
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
          cursor={{ fill: `${theme.bg.hover}88` }}
        />
        <Bar
          dataKey={yKey}
          fill={color}
          radius={[4, 4, 0, 0]}
          maxBarSize={40}
        />
      </RechartsBarChart>
    </ResponsiveContainer>
  );
}
