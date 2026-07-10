import { theme } from '../../../styles/theme';
import { useTrainingStatus } from '../../../hooks/useTrainingStatus';
import { useMetricsHistory } from '../../../hooks/useMetricsHistory';
import { WorkerGrid } from './WorkerGrid';
import {
  LineChart,
  Line,
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';

function MetricCard({ label, value, sub, color }: {
  label: string;
  value: string | number;
  sub?: string;
  color?: string;
}) {
  return (
    <div style={{
      background: theme.bg.secondary,
      border: `1px solid ${theme.border}`,
      borderRadius: 8,
      padding: '16px 20px',
      minWidth: 160,
    }}>
      <div style={{ fontSize: 12, color: theme.text.secondary, marginBottom: 4 }}>
        {label}
      </div>
      <div style={{
        fontSize: 28,
        fontWeight: 700,
        letterSpacing: '-0.5px',
        color: color ?? theme.text.primary,
      }}>
        {value}
      </div>
      {sub && (
        <div style={{ fontSize: 12, color: theme.text.muted, marginTop: 2 }}>
          {sub}
        </div>
      )}
    </div>
  );
}

function ChartCard({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div style={{
      background: theme.bg.secondary,
      border: `1px solid ${theme.border}`,
      borderRadius: 8,
      padding: 16,
    }}>
      <div style={{ fontSize: 13, color: theme.text.secondary, marginBottom: 12, fontWeight: 500 }}>
        {title}
      </div>
      {children}
    </div>
  );
}

const chartTooltipStyle = {
  contentStyle: {
    background: theme.bg.tertiary,
    border: `1px solid ${theme.border}`,
    borderRadius: 6,
    fontSize: 12,
    color: theme.text.primary,
  },
  itemStyle: { color: theme.text.primary },
  labelStyle: { color: theme.text.secondary },
};

export function DashboardView() {
  const { data: status, loading, stale } = useTrainingStatus();
  const { data: metrics } = useMetricsHistory();

  if (loading && !status) {
    return (
      <div style={{ color: theme.text.muted, padding: 40, textAlign: 'center' }}>
        Loading training status...
      </div>
    );
  }

  if (!status) {
    return (
      <div style={{ color: theme.text.muted, padding: 40, textAlign: 'center' }}>
        No training data available. Start a training run to see metrics.
      </div>
    );
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      {stale && (
        <div style={{
          background: theme.warning + '22',
          border: `1px solid ${theme.warning}44`,
          borderRadius: 6,
          padding: '8px 14px',
          fontSize: 12,
          color: theme.warning,
        }}>
          Data may be stale -- last update was more than 10 seconds ago
        </div>
      )}

      {/* Metric cards row */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(170px, 1fr))',
        gap: 12,
      }}>
        <MetricCard
          label="Total Games"
          value={status.totalGames.toLocaleString()}
          sub={`${status.gamesPerMin.toFixed(1)} games/min`}
        />
        <MetricCard
          label="Win Rate"
          value={`${(status.winRate * 100).toFixed(1)}%`}
          sub={`${status.totalWins} wins`}
          color={status.winRate > 0.5 ? theme.success : status.winRate > 0 ? theme.warning : theme.text.primary}
        />
        <MetricCard
          label="Avg Floor"
          value={status.avgFloor.toFixed(1)}
          sub={`Peak: ${status.peakFloor}`}
        />
        <MetricCard
          label="Train Steps"
          value={status.trainSteps.toLocaleString()}
          sub={`${status.elapsedHours.toFixed(1)}h elapsed`}
        />
        <MetricCard
          label="Total Loss"
          value={status.loss.total.toFixed(4)}
          sub={`P: ${status.loss.policy.toFixed(4)} V: ${status.loss.value.toFixed(4)}`}
        />
        <MetricCard
          label="Entropy"
          value={status.entropy.toFixed(3)}
        />
        <MetricCard
          label="KL Divergence"
          value={status.diagnostics.klDivergence.toFixed(4)}
          sub={`Clip: ${(status.diagnostics.clipFraction * 100).toFixed(1)}%`}
        />
        {status.gpuPercent !== null && (
          <MetricCard
            label="GPU Usage"
            value={`${status.gpuPercent.toFixed(0)}%`}
            color={status.gpuPercent > 90 ? theme.danger : theme.text.primary}
          />
        )}
      </div>

      {/* Charts grid */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: '1fr 1fr',
        gap: 16,
      }}>
        <ChartCard title="Loss Over Time">
          {metrics && metrics.length > 0 ? (
            <ResponsiveContainer width="100%" height={220}>
              <LineChart data={metrics}>
                <CartesianGrid strokeDasharray="3 3" stroke={theme.border} />
                <XAxis
                  dataKey="step"
                  tick={{ fill: theme.text.muted, fontSize: 11 }}
                  stroke={theme.border}
                />
                <YAxis
                  tick={{ fill: theme.text.muted, fontSize: 11 }}
                  stroke={theme.border}
                />
                <Tooltip {...chartTooltipStyle} />
                <Line type="monotone" dataKey="loss.total" stroke={theme.chart.red} dot={false} strokeWidth={2} name="Total" />
                <Line type="monotone" dataKey="loss.policy" stroke={theme.chart.blue} dot={false} strokeWidth={1.5} name="Policy" />
                <Line type="monotone" dataKey="loss.value" stroke={theme.chart.green} dot={false} strokeWidth={1.5} name="Value" />
              </LineChart>
            </ResponsiveContainer>
          ) : (
            <div style={{ height: 220, display: 'flex', alignItems: 'center', justifyContent: 'center', color: theme.text.muted }}>
              No metrics history yet
            </div>
          )}
        </ChartCard>

        <ChartCard title="Floor Progress">
          {metrics && metrics.length > 0 ? (
            <ResponsiveContainer width="100%" height={220}>
              <AreaChart data={metrics}>
                <CartesianGrid strokeDasharray="3 3" stroke={theme.border} />
                <XAxis
                  dataKey="games"
                  tick={{ fill: theme.text.muted, fontSize: 11 }}
                  stroke={theme.border}
                />
                <YAxis
                  tick={{ fill: theme.text.muted, fontSize: 11 }}
                  stroke={theme.border}
                  domain={[0, 55]}
                />
                <Tooltip {...chartTooltipStyle} />
                <Area type="monotone" dataKey="peakFloor" stroke={theme.chart.purple} fill={theme.chart.purple + '22'} strokeWidth={1.5} name="Peak" />
                <Area type="monotone" dataKey="avgFloor" stroke={theme.chart.blue} fill={theme.chart.blue + '22'} strokeWidth={2} name="Avg" />
              </AreaChart>
            </ResponsiveContainer>
          ) : (
            <div style={{ height: 220, display: 'flex', alignItems: 'center', justifyContent: 'center', color: theme.text.muted }}>
              No metrics history yet
            </div>
          )}
        </ChartCard>

        <ChartCard title="Entropy">
          {metrics && metrics.length > 0 ? (
            <ResponsiveContainer width="100%" height={220}>
              <LineChart data={metrics}>
                <CartesianGrid strokeDasharray="3 3" stroke={theme.border} />
                <XAxis
                  dataKey="step"
                  tick={{ fill: theme.text.muted, fontSize: 11 }}
                  stroke={theme.border}
                />
                <YAxis
                  tick={{ fill: theme.text.muted, fontSize: 11 }}
                  stroke={theme.border}
                />
                <Tooltip {...chartTooltipStyle} />
                <Line type="monotone" dataKey="entropy" stroke={theme.chart.yellow} dot={false} strokeWidth={2} />
              </LineChart>
            </ResponsiveContainer>
          ) : (
            <div style={{ height: 220, display: 'flex', alignItems: 'center', justifyContent: 'center', color: theme.text.muted }}>
              No metrics history yet
            </div>
          )}
        </ChartCard>

        <ChartCard title="Workers">
          <WorkerGrid />
        </ChartCard>
      </div>
    </div>
  );
}
