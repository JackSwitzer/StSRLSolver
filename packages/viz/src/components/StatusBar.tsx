import type { TrainingStatsMsg, SystemStatsMsg } from '../types/training';

interface StatusBarProps {
  stats: TrainingStatsMsg | null;
  connected: boolean;
  systemStats?: SystemStatsMsg | null;
}

const bar: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  height: 50,
  padding: '0 16px',
  background: '#161b22',
  borderBottom: '1px solid #30363d',
  gap: 24,
  fontFamily: 'inherit',
  fontSize: 13,
  color: '#c9d1d9',
  flexShrink: 0,
};

const metricStyle: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: 6,
  whiteSpace: 'nowrap',
};

const labelStyle: React.CSSProperties = {
  color: '#8b949e',
  fontSize: 11,
  textTransform: 'uppercase',
  letterSpacing: '0.5px',
};

const valueStyle: React.CSSProperties = {
  color: '#c9d1d9',
  fontWeight: 600,
  fontVariantNumeric: 'tabular-nums',
};

const accentValue: React.CSSProperties = {
  ...valueStyle,
  color: '#00ff41',
};

const dotBase: React.CSSProperties = {
  width: 8,
  height: 8,
  borderRadius: '50%',
  flexShrink: 0,
};

const sep: React.CSSProperties = {
  width: 1,
  height: 24,
  background: '#30363d',
  flexShrink: 0,
};

function formatUptime(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

export const StatusBar = ({ stats, connected, systemStats }: StatusBarProps) => {
  const training = systemStats?.training_status;
  const entropy = training?.entropy_coeff ?? training?.entropy;
  const loss = training?.last_loss ?? training?.loss;
  const lr = training?.lr ?? training?.learning_rate;

  return (
    <div style={bar}>
      {/* Connection indicator */}
      <div style={metricStyle}>
        <div
          style={{
            ...dotBase,
            background: connected ? '#00ff41' : '#f85149',
            boxShadow: connected
              ? '0 0 6px rgba(0,255,65,0.4)'
              : '0 0 6px rgba(248,81,73,0.4)',
          }}
        />
        <span style={{ ...labelStyle, fontSize: 12 }}>
          {connected ? 'LIVE' : 'DISCONNECTED'}
        </span>
      </div>

      <div style={sep} />

      {/* Games */}
      <div style={metricStyle}>
        <span style={labelStyle}>Games</span>
        <span style={accentValue}>{stats?.total_episodes ?? '--'}</span>
      </div>

      {/* Avg Floor */}
      <div style={metricStyle}>
        <span style={labelStyle}>Avg Floor</span>
        <span style={valueStyle}>
          {stats?.avg_floor != null ? stats.avg_floor.toFixed(1) : '--'}
        </span>
      </div>

      {/* Max Floor */}
      {stats?.max_floor != null && (
        <div style={metricStyle}>
          <span style={labelStyle}>Max</span>
          <span style={valueStyle}>F{stats.max_floor}</span>
        </div>
      )}

      {/* Win Rate */}
      <div style={metricStyle}>
        <span style={labelStyle}>WR</span>
        <span style={valueStyle}>
          {stats?.win_rate != null ? `${(stats.win_rate * 100).toFixed(1)}%` : '--'}
        </span>
      </div>

      <div style={sep} />

      {/* Games/min */}
      <div style={metricStyle}>
        <span style={labelStyle}>G/min</span>
        <span style={valueStyle}>
          {stats?.eps_per_min != null ? stats.eps_per_min.toFixed(0) : '--'}
        </span>
      </div>

      {/* Train Steps */}
      <div style={metricStyle}>
        <span style={labelStyle}>Steps</span>
        <span style={valueStyle}>{stats?.train_steps ?? '--'}</span>
      </div>

      <div style={sep} />

      {/* Entropy */}
      {entropy != null && (
        <div style={metricStyle}>
          <span style={labelStyle}>Entropy</span>
          <span style={valueStyle}>{Number(entropy).toFixed(3)}</span>
        </div>
      )}

      {/* Loss (total + breakdown on hover) */}
      {loss != null && (
        <div style={metricStyle} title={
          [
            training?.policy_loss != null ? `Policy: ${Number(training.policy_loss).toFixed(4)}` : '',
            training?.value_loss != null ? `Value: ${Number(training.value_loss).toFixed(4)}` : '',
            training?.floor_pred_loss != null ? `Floor: ${Number(training.floor_pred_loss).toFixed(4)}` : '',
            training?.act_pred_loss != null ? `Act: ${Number(training.act_pred_loss).toFixed(4)}` : '',
            training?.clip_fraction != null ? `Clip: ${Number(training.clip_fraction).toFixed(3)}` : '',
          ].filter(Boolean).join(' | ')
        }>
          <span style={labelStyle}>Loss</span>
          <span style={{
            ...valueStyle,
            color: Number(loss) < 0 ? '#f85149' : '#c9d1d9',
          }}>{Number(loss).toFixed(4)}</span>
        </div>
      )}

      {/* Learning Rate */}
      {lr != null && (
        <div style={metricStyle}>
          <span style={labelStyle}>LR</span>
          <span style={valueStyle}>{Number(lr).toExponential(1)}</span>
        </div>
      )}

      {/* Spacer */}
      <div style={{ flex: 1 }} />

      {/* Uptime */}
      {stats?.uptime != null && (
        <div style={metricStyle}>
          <span style={labelStyle}>Uptime</span>
          <span style={{ ...valueStyle, color: '#8b949e' }}>
            {formatUptime(stats.uptime)}
          </span>
        </div>
      )}
    </div>
  );
};
