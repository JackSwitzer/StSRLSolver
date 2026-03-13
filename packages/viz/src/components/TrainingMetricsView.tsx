import type { TrainingStatsMsg, SystemStatsMsg, ProcessCategory } from '../types/training';

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

function Stat({ label, value, warn, sub }: { label: string; value: string; warn?: boolean; sub?: string }) {
  return (
    <div className="bg-gray-800 rounded px-3 py-2">
      <div className="text-xs text-gray-400">{label}</div>
      <div className={`text-lg font-mono ${warn ? 'text-red-400' : 'text-white'}`}>{value}</div>
      {sub && <div className="text-xs text-gray-500 font-mono">{sub}</div>}
    </div>
  );
}

function UsageBar({ pct, color = 'bg-blue-500' }: { pct: number; color?: string }) {
  return (
    <div className="w-full bg-gray-700 rounded h-2 overflow-hidden">
      <div className={`h-full rounded ${color}`}
           style={{ width: `${Math.min(100, pct)}%` }} />
    </div>
  );
}

const CAT_COLORS: Record<string, string> = {
  'RL Training': '#34d399',
  'Codex (GPT 5.4)': '#a78bfa',
  'Claude Code': '#f97316',
  'Rust Build': '#f87171',
  'Python': '#60a5fa',
  'Node/Vite': '#fbbf24',
  'macOS System': '#6b7280',
  'Other': '#4b5563',
};

function ProcessBreakdown({ processes }: { processes: Record<string, ProcessCategory> }) {
  const entries = Object.entries(processes).sort((a, b) => b[1].cpu - a[1].cpu);
  if (entries.length === 0) return null;

  const totalCpu = entries.reduce((s, [, v]) => s + v.cpu, 0);
  const totalRam = entries.reduce((s, [, v]) => s + v.ram_gb, 0);

  return (
    <div>
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-sm text-gray-400">Process Breakdown</h3>
        <span className="text-xs text-gray-500 font-mono">
          {totalCpu.toFixed(0)}% CPU / {totalRam.toFixed(1)} GB RAM
        </span>
      </div>

      {/* Stacked CPU bar */}
      <div className="w-full bg-gray-700 rounded h-3 overflow-hidden flex mb-3">
        {entries.map(([cat, vals]) => (
          <div
            key={cat}
            style={{
              width: `${Math.max(0.5, (vals.cpu / Math.max(totalCpu, 1)) * 100)}%`,
              backgroundColor: CAT_COLORS[cat] || '#4b5563',
            }}
            className="h-full"
            title={`${cat}: ${vals.cpu.toFixed(0)}% CPU`}
          />
        ))}
      </div>

      <div className="space-y-1.5">
        {entries.map(([cat, vals]) => (
          <div key={cat} className="flex items-center gap-2 text-xs font-mono">
            <div className="w-2.5 h-2.5 rounded-sm flex-shrink-0"
                 style={{ backgroundColor: CAT_COLORS[cat] || '#4b5563' }} />
            <span className="w-28 text-gray-300 truncate">{cat}</span>
            <span className="w-16 text-right text-gray-400">{vals.cpu.toFixed(0)}% CPU</span>
            <span className="w-16 text-right text-gray-500">{vals.ram_gb.toFixed(1)} GB</span>
            <span className="w-8 text-right text-gray-600">({vals.count})</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function CpuCoreGrid({ perCpu }: { perCpu: number[] }) {
  if (perCpu.length === 0) return null;
  return (
    <div>
      <h3 className="text-sm text-gray-400 mb-2">Per-Core CPU ({perCpu.length} cores)</h3>
      <div className="flex gap-1 flex-wrap">
        {perCpu.map((pct, i) => (
          <div key={i} className="flex flex-col items-center gap-0.5" title={`Core ${i}: ${pct.toFixed(0)}%`}>
            <div className="w-5 h-12 bg-gray-700 rounded overflow-hidden flex flex-col justify-end">
              <div
                className={`w-full rounded-t ${pct > 80 ? 'bg-red-500' : pct > 50 ? 'bg-yellow-500' : 'bg-green-500'}`}
                style={{ height: `${pct}%` }}
              />
            </div>
            <span className="text-[9px] text-gray-500">{i}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

export function TrainingMetricsView({ stats, systemStats, floorHistory, lossHistory, winHistory }: Props) {
  const s = stats;
  const sys = systemStats;

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

      {sys && (
        <div className="space-y-6">
          <div>
            <h3 className="text-sm text-gray-400 mb-2">System Resources</h3>
            <div className="grid grid-cols-4 gap-4 mb-3">
              <div>
                <Stat label="CPU" value={`${sys.cpu_pct.toFixed(0)}%`}
                      warn={sys.cpu_pct > 90}
                      sub={`${sys.cpu_cores ?? '?'} cores`} />
                <UsageBar pct={sys.cpu_pct} color={sys.cpu_pct > 90 ? 'bg-red-500' : sys.cpu_pct > 70 ? 'bg-yellow-500' : 'bg-green-500'} />
              </div>
              <div>
                <Stat label="RAM" value={`${sys.ram_used_gb} / ${sys.ram_total_gb} GB`}
                      warn={sys.ram_pct > 85}
                      sub={`${sys.ram_pct.toFixed(0)}%`} />
                <UsageBar pct={sys.ram_pct} color={sys.ram_pct > 85 ? 'bg-red-500' : sys.ram_pct > 70 ? 'bg-yellow-500' : 'bg-blue-500'} />
              </div>
              <div>
                <Stat label="GPU" value={sys.gpu_available ? (sys.gpu_name ?? 'Available') : 'N/A'}
                      sub={sys.gpu_mem_used_gb ? `${sys.gpu_mem_used_gb} GB used` : undefined} />
                {sys.gpu_mem_used_gb != null && sys.gpu_mem_used_gb > 0 && (
                  <UsageBar pct={(sys.gpu_mem_used_gb / Math.max(sys.ram_total_gb * 0.75, 1)) * 100} color="bg-purple-500" />
                )}
              </div>
              <div>
                <Stat label="Swap" value={`${sys.swap_used_gb ?? 0} / ${sys.swap_total_gb ?? 0} GB`}
                      warn={(sys.swap_used_gb ?? 0) > 2} />
                <Stat label="Workers" value={String(sys.workers)} />
              </div>
            </div>
          </div>

          <CpuCoreGrid perCpu={sys.per_cpu ?? []} />

          {sys.processes && <ProcessBreakdown processes={sys.processes} />}

          {sys.paused && (
            <div className="bg-yellow-900/30 border border-yellow-600 rounded px-3 py-2 text-yellow-300 text-sm">
              Training is paused
            </div>
          )}
        </div>
      )}
    </div>
  );
}
