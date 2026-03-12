import type { TrainingStatsMsg, SystemStatsMsg } from '../types/training';

interface Props {
  stats: TrainingStatsMsg | null;
  systemStats: SystemStatsMsg | null;
  floorHistory: number[];
  lossHistory: number[];
  winHistory: number[];
}

function MiniChart({ values, label, color = '#60a5fa' }: { values: number[]; label: string; color?: string }) {
  if (values.length < 2) return <div className="text-gray-500 text-xs">{label}: no data</div>;
  const max = Math.max(...values, 0.001);
  const min = Math.min(...values, 0);
  const range = max - min || 1;
  const w = 200;
  const h = 40;
  const points = values.map((v, i) =>
    `${(i / (values.length - 1)) * w},${h - ((v - min) / range) * h}`
  ).join(' ');
  const latest = values[values.length - 1];

  return (
    <div>
      <div className="flex items-center justify-between mb-1">
        <span className="text-xs text-gray-400">{label}</span>
        <span className="text-xs font-mono" style={{ color }}>{
          Math.abs(latest) >= 100 ? latest.toFixed(1) :
          Math.abs(latest) >= 1 ? latest.toFixed(2) : latest.toFixed(4)
        }</span>
      </div>
      <svg width={w} height={h}>
        <polyline points={points} fill="none" stroke={color} strokeWidth="1.5" />
      </svg>
    </div>
  );
}

function Stat({ label, value, warn }: { label: string; value: string; warn?: boolean }) {
  return (
    <div className="bg-gray-800 rounded px-3 py-2">
      <div className="text-xs text-gray-400">{label}</div>
      <div className={`text-lg font-mono ${warn ? 'text-red-400' : 'text-white'}`}>{value}</div>
    </div>
  );
}

export function TrainingMetricsView({ stats, systemStats, floorHistory, lossHistory, winHistory }: Props) {
  const s = stats;

  const winRate = winHistory.length > 0
    ? (winHistory.reduce((a, b) => a + b, 0) / winHistory.length * 100).toFixed(1) + '%'
    : '-';

  return (
    <div className="h-full overflow-y-auto bg-gray-900 text-gray-100 p-4">
      <h2 className="text-lg font-bold mb-4">Training Metrics</h2>

      {s && (
        <div className="grid grid-cols-4 gap-4 mb-6">
          <Stat label="Total Episodes" value={s.total_episodes.toLocaleString()} />
          <Stat label="Eps/min" value={s.eps_per_min.toFixed(1)} />
          <Stat label="Avg Floor" value={s.avg_floor.toFixed(1)} />
          <Stat label="Win Rate" value={winRate} />
          <Stat label="Train Steps" value={(s.train_steps ?? 0).toLocaleString()} />
          <Stat label="MCTS Avg" value={`${s.mcts_avg_ms.toFixed(0)}ms`} />
          <Stat label="Uptime" value={`${Math.round(s.uptime / 60)}m`} />
          <Stat label="Max Floor" value={String(s.max_floor ?? '-')} />
        </div>
      )}

      <div className="grid grid-cols-3 gap-6 mb-6">
        <MiniChart values={floorHistory} label="Avg Floor" color="#34d399" />
        <MiniChart values={lossHistory} label="Loss" color="#f87171" />
        <MiniChart values={winHistory} label="Win Rate" color="#60a5fa" />
      </div>

      {systemStats && (
        <div className="mt-4">
          <h3 className="text-sm text-gray-400 mb-2">System</h3>
          <div className="grid grid-cols-3 gap-4">
            <Stat label="CPU" value={`${systemStats.cpu_pct.toFixed(0)}%`} />
            <Stat label="RAM" value={`${systemStats.ram_pct.toFixed(0)}%`}
                  warn={systemStats.ram_pct > 85} />
            <Stat label="Workers" value={String(systemStats.workers)} />
          </div>
        </div>
      )}
    </div>
  );
}
