import { useState, useMemo } from 'react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';
import { theme } from '../../../styles/theme';
import { useManifest, useTrainingEvents, useTrainingMetrics, useBenchmark, useFrontier, useCorpusMatrix } from '../../../hooks/useArtifacts';
import type { BenchmarkSlice, CorpusCell } from '../../../types/artifacts';

// ---------------------------------------------------------------------------
// Shared UI primitives (match DashboardView patterns)
// ---------------------------------------------------------------------------

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

function Card({ title, children, style: extra }: {
  title?: string;
  children: React.ReactNode;
  style?: React.CSSProperties;
}) {
  return (
    <div style={{
      background: theme.bg.secondary,
      border: `1px solid ${theme.border}`,
      borderRadius: 8,
      padding: 16,
      ...extra,
    }}>
      {title && (
        <div style={{ fontSize: 13, color: theme.text.secondary, marginBottom: 12, fontWeight: 500 }}>
          {title}
        </div>
      )}
      {children}
    </div>
  );
}

const chartTooltipStyle: Record<string, unknown> = {
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

function EmptyState({ message }: { message: string }) {
  return (
    <div style={{ color: theme.text.muted, padding: 40, textAlign: 'center' }}>
      {message}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Tab definitions
// ---------------------------------------------------------------------------

const TABS = ['Overview', 'Epochs', 'Matrix', 'Benchmarks', 'Frontier'] as const;
type Tab = typeof TABS[number];

// ---------------------------------------------------------------------------
// Sub-views
// ---------------------------------------------------------------------------

function OverviewTab() {
  const { data: manifest, loading: mLoading } = useManifest();
  const { data: events } = useTrainingEvents();
  const { data: metrics } = useTrainingMetrics();

  if (mLoading && !manifest) return <EmptyState message="Loading manifest..." />;
  if (!manifest || !manifest.runId) return <EmptyState message="No v2 manifest found. Point logsDir at a v2 run." />;

  const latestPhase = events && events.length > 0
    ? events[events.length - 1].phase ?? events[events.length - 1].eventType
    : 'unknown';

  const phases = ['start', 'collect', 'reanalyze', 'epoch_complete', 'benchmark', 'complete'];
  const phaseIndex = phases.indexOf(latestPhase ?? '');
  const phasePct = phaseIndex >= 0 ? ((phaseIndex + 1) / phases.length) * 100 : 0;

  const totalCases = metrics
    ? metrics.filter(m => m.name === 'solve_probability').length
    : 0;
  const latestAccuracy = events
    ? [...events].reverse().find(e => e.accuracy !== undefined)?.accuracy
    : undefined;
  const latestThroughput = metrics
    ? [...metrics].reverse().find(m => m.name === 'epoch_throughput_examples_per_sec')?.value
    : undefined;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      {/* Manifest info */}
      <Card title="Run Manifest">
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: 12 }}>
          <div>
            <div style={{ fontSize: 11, color: theme.text.muted, marginBottom: 2 }}>Run ID</div>
            <div style={{ fontSize: 13, color: theme.text.primary, fontFamily: 'monospace' }}>
              {manifest.runId}
            </div>
          </div>
          <div>
            <div style={{ fontSize: 11, color: theme.text.muted, marginBottom: 2 }}>Git SHA</div>
            <div style={{ fontSize: 13, color: theme.text.primary, fontFamily: 'monospace' }}>
              {manifest.git.commitSha.slice(0, 8)}
              {manifest.git.dirty && (
                <span style={{ color: theme.warning, marginLeft: 4 }}>dirty</span>
              )}
            </div>
          </div>
          <div>
            <div style={{ fontSize: 11, color: theme.text.muted, marginBottom: 2 }}>Branch</div>
            <div style={{ fontSize: 13, color: theme.text.primary }}>{manifest.git.branch}</div>
          </div>
          <div>
            <div style={{ fontSize: 11, color: theme.text.muted, marginBottom: 2 }}>Config Hash</div>
            <div style={{ fontSize: 13, color: theme.text.primary, fontFamily: 'monospace' }}>
              {manifest.config.configHash.slice(0, 12)}
            </div>
          </div>
          <div>
            <div style={{ fontSize: 11, color: theme.text.muted, marginBottom: 2 }}>Host</div>
            <div style={{ fontSize: 13, color: theme.text.primary }}>{manifest.host}</div>
          </div>
          <div>
            <div style={{ fontSize: 11, color: theme.text.muted, marginBottom: 2 }}>Created</div>
            <div style={{ fontSize: 13, color: theme.text.primary }}>
              {new Date(manifest.createdAt).toLocaleString()}
            </div>
          </div>
        </div>
      </Card>

      {/* Phase indicator */}
      <Card title="Training Phase">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 12 }}>
            <span style={{ color: theme.text.primary, fontWeight: 600 }}>
              {latestPhase}
            </span>
            <span style={{ color: theme.text.muted }}>
              {events?.length ?? 0} events
            </span>
          </div>
          <div style={{
            width: '100%',
            height: 6,
            background: theme.bg.tertiary,
            borderRadius: 3,
            overflow: 'hidden',
          }}>
            <div style={{
              width: `${phasePct}%`,
              height: '100%',
              background: theme.accent,
              borderRadius: 3,
              transition: 'width 300ms ease',
            }} />
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 10, color: theme.text.muted }}>
            {phases.map(p => (
              <span key={p} style={{
                color: p === latestPhase ? theme.accent : theme.text.muted,
                fontWeight: p === latestPhase ? 600 : 400,
              }}>
                {p}
              </span>
            ))}
          </div>
        </div>
      </Card>

      {/* Key metric cards */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(170px, 1fr))',
        gap: 12,
      }}>
        <MetricCard
          label="Cases Processed"
          value={totalCases}
          sub="solve_probability entries"
        />
        <MetricCard
          label="Latest Accuracy"
          value={latestAccuracy !== undefined ? `${(latestAccuracy * 100).toFixed(1)}%` : '--'}
          color={latestAccuracy !== undefined && latestAccuracy > 0.7 ? theme.success : theme.text.primary}
        />
        <MetricCard
          label="Throughput"
          value={latestThroughput !== undefined ? `${latestThroughput.toFixed(0)}` : '--'}
          sub="examples/sec"
        />
      </div>
    </div>
  );
}

function EpochsTab() {
  const { data: metrics, loading } = useTrainingMetrics();

  const accuracyData = useMemo(() => {
    if (!metrics) return [];
    return metrics
      .filter(m => m.name === 'epoch_accuracy')
      .map((m, i) => ({ epoch: i + 1, accuracy: m.value }));
  }, [metrics]);

  const throughputData = useMemo(() => {
    if (!metrics) return [];
    return metrics
      .filter(m => m.name === 'epoch_throughput_examples_per_sec')
      .map((m, i) => ({ epoch: i + 1, throughput: m.value }));
  }, [metrics]);

  if (loading && !metrics) return <EmptyState message="Loading metrics..." />;
  if (!metrics || metrics.length === 0) return <EmptyState message="No training metrics recorded yet." />;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      <Card title="Epoch Accuracy">
        {accuracyData.length > 0 ? (
          <ResponsiveContainer width="100%" height={280}>
            <LineChart data={accuracyData}>
              <CartesianGrid strokeDasharray="3 3" stroke={theme.border} />
              <XAxis
                dataKey="epoch"
                tick={{ fill: theme.text.muted, fontSize: 11 }}
                stroke={theme.border}
                label={{ value: 'Epoch', position: 'insideBottom', offset: -4, fill: theme.text.muted, fontSize: 11 }}
              />
              <YAxis
                tick={{ fill: theme.text.muted, fontSize: 11 }}
                stroke={theme.border}
                domain={[0, 1]}
                tickFormatter={(v: number) => `${(v * 100).toFixed(0)}%`}
              />
              <Tooltip {...chartTooltipStyle} formatter={(v: unknown) => `${(Number(v) * 100).toFixed(1)}%`} />
              <Line
                type="monotone"
                dataKey="accuracy"
                stroke={theme.chart.green}
                strokeWidth={2}
                dot={{ fill: theme.chart.green, r: 4 }}
                name="Accuracy"
              />
            </LineChart>
          </ResponsiveContainer>
        ) : (
          <EmptyState message="No epoch accuracy data yet." />
        )}
      </Card>

      <Card title="Epoch Throughput">
        {throughputData.length > 0 ? (
          <ResponsiveContainer width="100%" height={280}>
            <LineChart data={throughputData}>
              <CartesianGrid strokeDasharray="3 3" stroke={theme.border} />
              <XAxis
                dataKey="epoch"
                tick={{ fill: theme.text.muted, fontSize: 11 }}
                stroke={theme.border}
                label={{ value: 'Epoch', position: 'insideBottom', offset: -4, fill: theme.text.muted, fontSize: 11 }}
              />
              <YAxis
                tick={{ fill: theme.text.muted, fontSize: 11 }}
                stroke={theme.border}
                tickFormatter={(v: number) => `${v.toFixed(0)}`}
              />
              <Tooltip {...chartTooltipStyle} formatter={(v: unknown) => `${Number(v).toFixed(1)} ex/s`} />
              <Line
                type="monotone"
                dataKey="throughput"
                stroke={theme.chart.blue}
                strokeWidth={2}
                dot={{ fill: theme.chart.blue, r: 4 }}
                name="Throughput (ex/s)"
              />
            </LineChart>
          </ResponsiveContainer>
        ) : (
          <EmptyState message="No throughput data yet." />
        )}
      </Card>
    </div>
  );
}

function solveColor(rate: number): string {
  if (rate >= 0.7) return theme.success;
  if (rate >= 0.3) return theme.warning;
  return theme.danger;
}

function MatrixTab() {
  const { data: cells, loading } = useCorpusMatrix();

  const { families, enemies, lookup } = useMemo(() => {
    if (!cells || cells.length === 0) return { families: [] as string[], enemies: [] as string[], lookup: new Map<string, CorpusCell>() };
    const fSet = new Set<string>();
    const eSet = new Set<string>();
    const lk = new Map<string, CorpusCell>();
    for (const c of cells) {
      fSet.add(c.deckFamily);
      eSet.add(c.enemy);
      lk.set(`${c.deckFamily}|${c.enemy}`, c);
    }
    return { families: [...fSet].sort(), enemies: [...eSet].sort(), lookup: lk };
  }, [cells]);

  if (loading && !cells) return <EmptyState message="Loading corpus matrix..." />;
  if (!cells || cells.length === 0) return <EmptyState message="No corpus data available." />;

  const cellStyle: React.CSSProperties = {
    padding: '8px 10px',
    fontSize: 12,
    borderBottom: `1px solid ${theme.border}`,
    borderRight: `1px solid ${theme.border}`,
  };

  return (
    <Card title="Solve Rate Matrix (Deck Family x Enemy)">
      <div style={{ overflowX: 'auto' }}>
        <table style={{ borderCollapse: 'collapse', width: '100%', minWidth: 500 }}>
          <thead>
            <tr>
              <th style={{
                ...cellStyle,
                textAlign: 'left',
                color: theme.text.secondary,
                fontWeight: 600,
                background: theme.bg.tertiary,
                position: 'sticky',
                left: 0,
                zIndex: 1,
              }}>
                Deck Family
              </th>
              {enemies.map(e => (
                <th key={e} style={{
                  ...cellStyle,
                  textAlign: 'center',
                  color: theme.text.secondary,
                  fontWeight: 600,
                  background: theme.bg.tertiary,
                  whiteSpace: 'nowrap',
                }}>
                  {e}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {families.map(f => (
              <tr key={f}>
                <td style={{
                  ...cellStyle,
                  color: theme.text.primary,
                  fontWeight: 500,
                  fontFamily: 'monospace',
                  fontSize: 11,
                  background: theme.bg.secondary,
                  position: 'sticky',
                  left: 0,
                  zIndex: 1,
                  whiteSpace: 'nowrap',
                }}>
                  {f}
                </td>
                {enemies.map(e => {
                  const cell = lookup.get(`${f}|${e}`);
                  if (!cell) {
                    return (
                      <td key={e} style={{ ...cellStyle, textAlign: 'center', color: theme.text.muted }}>
                        --
                      </td>
                    );
                  }
                  return (
                    <td key={e} style={{
                      ...cellStyle,
                      textAlign: 'center',
                      background: solveColor(cell.solveRate) + '11',
                    }}>
                      <div style={{ color: solveColor(cell.solveRate), fontWeight: 700, fontSize: 13 }}>
                        {(cell.solveRate * 100).toFixed(0)}%
                      </div>
                      <div style={{ color: theme.text.muted, fontSize: 10, marginTop: 2 }}>
                        {cell.avgHpLoss.toFixed(1)} HP
                      </div>
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </Card>
  );
}

type SortKey = keyof BenchmarkSlice;

function BenchmarksTab() {
  const { data: report, loading } = useBenchmark();
  const [sortKey, setSortKey] = useState<SortKey>('sliceName');
  const [sortDesc, setSortDesc] = useState(false);

  const sorted = useMemo(() => {
    if (!report?.slices) return [];
    const s = [...report.slices];
    s.sort((a, b) => {
      const av = a[sortKey];
      const bv = b[sortKey];
      if (typeof av === 'number' && typeof bv === 'number') {
        return sortDesc ? bv - av : av - bv;
      }
      return sortDesc
        ? String(bv).localeCompare(String(av))
        : String(av).localeCompare(String(bv));
    });
    return s;
  }, [report, sortKey, sortDesc]);

  if (loading && !report) return <EmptyState message="Loading benchmarks..." />;
  if (!report?.slices || report.slices.length === 0) return <EmptyState message="No benchmark data available." />;

  function handleSort(key: SortKey) {
    if (key === sortKey) {
      setSortDesc(!sortDesc);
    } else {
      setSortKey(key);
      setSortDesc(key !== 'sliceName');
    }
  }

  const columns: { key: SortKey; label: string; fmt: (v: unknown) => string }[] = [
    { key: 'sliceName', label: 'Slice', fmt: (v) => String(v) },
    { key: 'cases', label: 'Cases', fmt: (v) => String(v) },
    { key: 'solveRate', label: 'Solve Rate', fmt: (v) => `${((v as number) * 100).toFixed(1)}%` },
    { key: 'expectedHpLoss', label: 'HP Loss', fmt: (v) => (v as number).toFixed(1) },
    { key: 'expectedTurns', label: 'Turns', fmt: (v) => (v as number).toFixed(1) },
    { key: 'oracleTopKAgreement', label: 'Oracle Top-K', fmt: (v) => `${((v as number) * 100).toFixed(1)}%` },
    { key: 'p95ElapsedMs', label: 'p95 ms', fmt: (v) => (v as number).toFixed(0) },
    { key: 'p95RssGb', label: 'p95 RSS (GB)', fmt: (v) => (v as number).toFixed(2) },
  ];

  const thStyle: React.CSSProperties = {
    padding: '8px 10px',
    fontSize: 11,
    fontWeight: 600,
    color: theme.text.secondary,
    borderBottom: `1px solid ${theme.border}`,
    background: theme.bg.tertiary,
    cursor: 'pointer',
    userSelect: 'none',
    whiteSpace: 'nowrap',
  };
  const tdStyle: React.CSSProperties = {
    padding: '8px 10px',
    fontSize: 12,
    color: theme.text.primary,
    borderBottom: `1px solid ${theme.border}`,
  };

  return (
    <Card title="Benchmark Slices">
      <div style={{ overflowX: 'auto' }}>
        <table style={{ borderCollapse: 'collapse', width: '100%' }}>
          <thead>
            <tr>
              {columns.map(col => (
                <th
                  key={col.key}
                  style={{ ...thStyle, textAlign: col.key === 'sliceName' ? 'left' : 'right' }}
                  onClick={() => handleSort(col.key)}
                >
                  {col.label}
                  {sortKey === col.key && (
                    <span style={{ marginLeft: 4 }}>{sortDesc ? ' v' : ' ^'}</span>
                  )}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {sorted.map(slice => (
              <tr key={slice.sliceName} style={{ transition: 'background 100ms' }}>
                {columns.map(col => (
                  <td key={col.key} style={{
                    ...tdStyle,
                    textAlign: col.key === 'sliceName' ? 'left' : 'right',
                    fontFamily: col.key === 'sliceName' ? 'monospace' : 'inherit',
                    fontWeight: col.key === 'sliceName' ? 500 : 400,
                    color: col.key === 'solveRate'
                      ? solveColor(slice.solveRate)
                      : theme.text.primary,
                  }}>
                    {col.fmt(slice[col.key])}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </Card>
  );
}

function FrontierTab() {
  const { data: report, loading } = useFrontier();

  if (loading && !report) return <EmptyState message="Loading frontier data..." />;
  if (!report?.points || report.points.length === 0) return <EmptyState message="No frontier data available." />;

  const frontierSet = new Set(report.frontier);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      {/* Rankings */}
      <Card title="Frontier Rankings">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
          {report.ranking.map((label, i) => {
            const point = report.points.find(p => p.label === label);
            const isOnFrontier = frontierSet.has(label);
            return (
              <div key={label} style={{
                display: 'flex',
                alignItems: 'center',
                gap: 10,
                padding: '8px 12px',
                borderRadius: 6,
                background: isOnFrontier ? theme.success + '11' : 'transparent',
                border: `1px solid ${isOnFrontier ? theme.success + '33' : theme.border}`,
              }}>
                <span style={{
                  fontSize: 14,
                  fontWeight: 700,
                  color: theme.text.muted,
                  minWidth: 28,
                }}>
                  #{i + 1}
                </span>
                <span style={{
                  fontSize: 13,
                  fontWeight: 600,
                  color: theme.text.primary,
                  fontFamily: 'monospace',
                  flex: 1,
                }}>
                  {label}
                </span>
                {isOnFrontier && (
                  <span style={{
                    fontSize: 10,
                    fontWeight: 700,
                    padding: '2px 8px',
                    borderRadius: 4,
                    background: theme.success + '22',
                    color: theme.success,
                    textTransform: 'uppercase',
                    letterSpacing: '0.5px',
                  }}>
                    Pareto
                  </span>
                )}
                {point && (
                  <div style={{ display: 'flex', gap: 12, fontSize: 11, color: theme.text.secondary }}>
                    <span>WR: {(point.winRate * 100).toFixed(1)}%</span>
                    <span>Floor: {point.avgFloor.toFixed(1)}</span>
                    <span>{point.throughputGpm.toFixed(1)} gpm</span>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </Card>

      {/* Best by metric */}
      {report.bestByMetric && Object.keys(report.bestByMetric).length > 0 && (
        <Card title="Best by Metric">
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: 12 }}>
            {Object.entries(report.bestByMetric).map(([metric, label]) => (
              <div key={metric} style={{
                padding: '10px 14px',
                borderRadius: 6,
                background: theme.bg.tertiary,
                border: `1px solid ${theme.border}`,
              }}>
                <div style={{ fontSize: 11, color: theme.text.muted, marginBottom: 4 }}>
                  {metric}
                </div>
                <div style={{ fontSize: 13, color: theme.accent, fontWeight: 600, fontFamily: 'monospace' }}>
                  {label}
                </div>
              </div>
            ))}
          </div>
        </Card>
      )}

      {/* Groups */}
      {report.groups && report.groups.length > 0 && (
        <Card title="Frontier Groups">
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {report.groups.map((g, i) => (
              <div key={i} style={{
                padding: '10px 14px',
                borderRadius: 6,
                background: theme.bg.tertiary,
                border: `1px solid ${theme.border}`,
              }}>
                <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', marginBottom: 6 }}>
                  <span style={{ fontSize: 11, color: theme.text.muted }}>
                    deck: <span style={{ color: theme.text.primary }}>{g.key.deckFamily}</span>
                  </span>
                  <span style={{ fontSize: 11, color: theme.text.muted }}>
                    enemy: <span style={{ color: theme.text.primary }}>{g.key.enemy}</span>
                  </span>
                  <span style={{ fontSize: 11, color: theme.text.muted }}>
                    removes: <span style={{ color: theme.text.primary }}>{g.key.removeCount}</span>
                  </span>
                  <span style={{ fontSize: 11, color: theme.text.muted }}>
                    potions: <span style={{ color: theme.text.primary }}>{g.key.potionSet}</span>
                  </span>
                </div>
                <div style={{ display: 'flex', gap: 16, fontSize: 12 }}>
                  <span style={{ color: theme.text.secondary }}>
                    n={g.count}
                  </span>
                  <span style={{ color: theme.chart.green }}>
                    WR: {(g.meanWinRate * 100).toFixed(1)}%
                  </span>
                  <span style={{ color: theme.chart.blue }}>
                    Floor: {g.meanAvgFloor.toFixed(1)}
                  </span>
                  <span style={{ color: theme.chart.yellow }}>
                    {g.meanThroughputGpm.toFixed(1)} gpm
                  </span>
                </div>
                <div style={{ fontSize: 10, color: theme.text.muted, marginTop: 4 }}>
                  {g.labels.join(', ')}
                </div>
              </div>
            ))}
          </div>
        </Card>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main view
// ---------------------------------------------------------------------------

export function CorpusView() {
  const [activeTab, setActiveTab] = useState<Tab>('Overview');

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      {/* Tab bar */}
      <div style={{
        display: 'flex',
        gap: 2,
        background: theme.bg.secondary,
        borderRadius: 8,
        padding: 4,
        border: `1px solid ${theme.border}`,
      }}>
        {TABS.map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            style={{
              flex: 1,
              padding: '8px 16px',
              borderRadius: 6,
              border: 'none',
              cursor: 'pointer',
              fontSize: 13,
              fontWeight: 500,
              transition: 'background 150ms, color 150ms',
              background: activeTab === tab ? theme.bg.tertiary : 'transparent',
              color: activeTab === tab ? theme.text.primary : theme.text.secondary,
            }}
          >
            {tab}
          </button>
        ))}
      </div>

      {/* Tab content */}
      {activeTab === 'Overview' && <OverviewTab />}
      {activeTab === 'Epochs' && <EpochsTab />}
      {activeTab === 'Matrix' && <MatrixTab />}
      {activeTab === 'Benchmarks' && <BenchmarksTab />}
      {activeTab === 'Frontier' && <FrontierTab />}
    </div>
  );
}
