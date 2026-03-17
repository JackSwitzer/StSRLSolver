import type { SystemStatsMsg } from '../types/training';

interface SystemBarProps {
  stats: SystemStatsMsg | null;
}

const bar: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  height: 32,
  padding: '0 16px',
  background: '#0d1117',
  borderTop: '1px solid #21262d',
  gap: 20,
  fontFamily: 'inherit',
  fontSize: 11,
  color: '#8b949e',
  flexShrink: 0,
};

const metricStyle: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: 5,
  whiteSpace: 'nowrap',
  fontVariantNumeric: 'tabular-nums',
};

const labelStyle: React.CSSProperties = {
  color: '#484f58',
  fontSize: 10,
  textTransform: 'uppercase',
  letterSpacing: '0.3px',
};

function usageColor(pct: number): string {
  if (pct < 60) return '#3fb950';
  if (pct < 80) return '#d29922';
  return '#f85149';
}

function MiniBar({ pct }: { pct: number }) {
  const width = 40;
  const height = 4;
  return (
    <svg width={width} height={height} style={{ flexShrink: 0 }}>
      <rect x={0} y={0} width={width} height={height} rx={1} fill="#21262d" />
      <rect
        x={0}
        y={0}
        width={Math.round(width * Math.min(1, pct / 100))}
        height={height}
        rx={1}
        fill={usageColor(pct)}
      />
    </svg>
  );
}

export const SystemBar = ({ stats }: SystemBarProps) => {
  if (!stats) {
    return (
      <div style={bar}>
        <span style={labelStyle}>System</span>
        <span>--</span>
      </div>
    );
  }

  return (
    <div style={bar}>
      {/* CPU */}
      <div style={metricStyle}>
        <span style={labelStyle}>CPU</span>
        <MiniBar pct={stats.cpu_pct} />
        <span style={{ color: usageColor(stats.cpu_pct) }}>
          {stats.cpu_pct.toFixed(0)}%
        </span>
      </div>

      {/* RAM */}
      <div style={metricStyle}>
        <span style={labelStyle}>RAM</span>
        <MiniBar pct={stats.ram_pct} />
        <span style={{ color: usageColor(stats.ram_pct) }}>
          {stats.ram_used_gb.toFixed(1)}/{stats.ram_total_gb.toFixed(0)}GB
        </span>
      </div>

      {/* Swap (if present) */}
      {stats.swap_used_gb != null && stats.swap_total_gb != null && stats.swap_total_gb > 0 && (
        <div style={metricStyle}>
          <span style={labelStyle}>Swap</span>
          <span>{stats.swap_used_gb.toFixed(1)}/{stats.swap_total_gb.toFixed(0)}GB</span>
        </div>
      )}

      {/* GPU */}
      {stats.gpu_util_pct != null && (
        <div style={metricStyle}>
          <span style={labelStyle}>GPU</span>
          <MiniBar pct={stats.gpu_util_pct} />
          <span style={{ color: usageColor(stats.gpu_util_pct) }}>
            {stats.gpu_util_pct.toFixed(0)}%
          </span>
          {stats.gpu_mem_used_gb != null && (
            <span style={{ color: '#484f58', marginLeft: 2 }}>
              {stats.gpu_mem_used_gb.toFixed(1)}GB
            </span>
          )}
        </div>
      )}

      {/* Workers */}
      <div style={metricStyle}>
        <span style={labelStyle}>Workers</span>
        <span style={{ color: '#c9d1d9' }}>{stats.workers}</span>
      </div>

      {/* Paused indicator */}
      {stats.paused && (
        <div style={metricStyle}>
          <span
            style={{
              color: '#d29922',
              fontWeight: 600,
              textTransform: 'uppercase',
              fontSize: 10,
              letterSpacing: '0.5px',
            }}
          >
            PAUSED
          </span>
        </div>
      )}
    </div>
  );
};
