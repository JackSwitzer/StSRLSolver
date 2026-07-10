import { useState, useMemo } from 'react';
import { theme } from '../../../styles/theme';
import { useEpisodeList } from '../../../hooks/useEpisodes';
import { useEnemyStats, useDeathStats, useCardPlayStats, usePathPreferences } from '../../../hooks/useAggregates';
import { ROOM_COLORS } from '../../../types/engine';
import type { EnemyStats } from '../../../hooks/useAggregates';
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Cell,
} from 'recharts';

type EnemySortKey = 'name' | 'fights' | 'avgHpLost' | 'avgTurns' | 'potionRate' | 'avgSolverMs';
type SortDir = 'asc' | 'desc';

function SectionCard({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div style={{
      background: theme.bg.secondary,
      border: `1px solid ${theme.border}`,
      borderRadius: 8,
      padding: 16,
    }}>
      <div style={{ fontSize: 14, fontWeight: 600, color: theme.text.primary, marginBottom: 12 }}>
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

export function AnalysisView() {
  const { data: episodes, loading } = useEpisodeList();
  const enemyStats = useEnemyStats(episodes);
  const deathStats = useDeathStats(episodes);
  const cardPlayStats = useCardPlayStats(episodes);
  const pathPrefs = usePathPreferences(episodes);

  const [enemySortKey, setEnemySortKey] = useState<EnemySortKey>('avgHpLost');
  const [enemySortDir, setEnemySortDir] = useState<SortDir>('desc');

  const sortedEnemies = useMemo(() => {
    return [...enemyStats].sort((a, b) => {
      let cmp = 0;
      switch (enemySortKey) {
        case 'name': cmp = a.name.localeCompare(b.name); break;
        case 'fights': cmp = a.fights - b.fights; break;
        case 'avgHpLost': cmp = a.avgHpLost - b.avgHpLost; break;
        case 'avgTurns': cmp = a.avgTurns - b.avgTurns; break;
        case 'potionRate': cmp = a.potionRate - b.potionRate; break;
        case 'avgSolverMs': cmp = a.avgSolverMs - b.avgSolverMs; break;
      }
      return enemySortDir === 'asc' ? cmp : -cmp;
    });
  }, [enemyStats, enemySortKey, enemySortDir]);

  function handleEnemySort(key: EnemySortKey) {
    if (key === enemySortKey) {
      setEnemySortDir(d => d === 'asc' ? 'desc' : 'asc');
    } else {
      setEnemySortKey(key);
      setEnemySortDir('desc');
    }
  }

  function EnemySortHeader({ k, label, width }: { k: EnemySortKey; label: string; width?: number }) {
    const active = enemySortKey === k;
    return (
      <th
        onClick={() => handleEnemySort(k)}
        style={{
          cursor: 'pointer',
          color: active ? theme.text.primary : theme.text.secondary,
          fontSize: 11,
          fontWeight: 600,
          userSelect: 'none',
          width,
          padding: '6px 10px',
          borderBottom: `1px solid ${theme.border}`,
          textAlign: k === 'name' ? 'left' : 'right',
        }}
      >
        {label} {active ? (enemySortDir === 'asc' ? '\u25B2' : '\u25BC') : ''}
      </th>
    );
  }

  if (loading && !episodes) {
    return (
      <div style={{ color: theme.text.muted, padding: 40, textAlign: 'center' }}>
        Loading episode data for analysis...
      </div>
    );
  }

  if (!episodes || episodes.length === 0) {
    return (
      <div style={{ color: theme.text.muted, padding: 40, textAlign: 'center' }}>
        No episodes to analyze. Run some training games first.
      </div>
    );
  }

  const top20Cards = cardPlayStats.slice(0, 20);
  const top15Deaths = deathStats.slice(0, 15);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      {/* Summary bar */}
      <div style={{
        display: 'flex',
        gap: 16,
        fontSize: 13,
        color: theme.text.secondary,
      }}>
        <span>
          Analyzing <span style={{ color: theme.text.primary, fontWeight: 600 }}>{episodes.length}</span> episodes
        </span>
        <span>
          Wins: <span style={{ color: theme.success, fontWeight: 600 }}>{episodes.filter(e => e.won).length}</span>
        </span>
        <span>
          Unique enemies: <span style={{ color: theme.text.primary, fontWeight: 600 }}>{enemyStats.length}</span>
        </span>
      </div>

      {/* Enemy stats table */}
      <SectionCard title="Enemy Stats (by avg HP lost)">
        <div style={{ maxHeight: 400, overflow: 'auto' }}>
          <table>
            <thead>
              <tr>
                <EnemySortHeader k="name" label="Enemy" />
                <EnemySortHeader k="fights" label="Fights" width={60} />
                <EnemySortHeader k="avgHpLost" label="Avg HP Lost" width={90} />
                <EnemySortHeader k="avgTurns" label="Avg Turns" width={80} />
                <EnemySortHeader k="potionRate" label="Potion Rate" width={90} />
                <EnemySortHeader k="avgSolverMs" label="Solver (ms)" width={90} />
              </tr>
            </thead>
            <tbody>
              {sortedEnemies.map((e: EnemyStats) => (
                <tr
                  key={e.name}
                  style={{ borderBottom: `1px solid ${theme.border}` }}
                  onMouseEnter={ev => { (ev.currentTarget as HTMLElement).style.background = theme.bg.hover; }}
                  onMouseLeave={ev => { (ev.currentTarget as HTMLElement).style.background = 'transparent'; }}
                >
                  <td style={{ fontSize: 12, color: theme.text.primary, padding: '6px 10px', fontWeight: 500 }}>
                    {e.name}
                  </td>
                  <td style={{ fontSize: 12, color: theme.text.secondary, padding: '6px 10px', textAlign: 'right' }}>
                    {e.fights}
                  </td>
                  <td style={{ fontSize: 12, color: theme.danger, padding: '6px 10px', textAlign: 'right', fontWeight: 600 }}>
                    {e.avgHpLost.toFixed(1)}
                  </td>
                  <td style={{ fontSize: 12, color: theme.text.secondary, padding: '6px 10px', textAlign: 'right' }}>
                    {e.avgTurns.toFixed(1)}
                  </td>
                  <td style={{ fontSize: 12, color: theme.text.secondary, padding: '6px 10px', textAlign: 'right' }}>
                    {(e.potionRate * 100).toFixed(0)}%
                  </td>
                  <td style={{ fontSize: 12, color: theme.text.muted, padding: '6px 10px', textAlign: 'right' }}>
                    {e.avgSolverMs.toFixed(0)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </SectionCard>

      {/* Charts row */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16 }}>
        {/* Death chart */}
        <SectionCard title="Deaths by Enemy">
          {top15Deaths.length > 0 ? (
            <ResponsiveContainer width="100%" height={Math.max(200, top15Deaths.length * 28 + 40)}>
              <BarChart data={top15Deaths} layout="vertical" margin={{ left: 80 }}>
                <CartesianGrid strokeDasharray="3 3" stroke={theme.border} horizontal={false} />
                <XAxis
                  type="number"
                  tick={{ fill: theme.text.muted, fontSize: 11 }}
                  stroke={theme.border}
                />
                <YAxis
                  type="category"
                  dataKey="enemy"
                  tick={{ fill: theme.text.secondary, fontSize: 11 }}
                  stroke={theme.border}
                  width={80}
                />
                <Tooltip {...chartTooltipStyle} />
                <Bar dataKey="count" name="Deaths" radius={[0, 4, 4, 0]}>
                  {top15Deaths.map((_, i) => (
                    <Cell key={i} fill={theme.chart.red} />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          ) : (
            <div style={{ height: 200, display: 'flex', alignItems: 'center', justifyContent: 'center', color: theme.text.muted }}>
              No deaths recorded
            </div>
          )}
        </SectionCard>

        {/* Path preferences */}
        <SectionCard title="Path Preferences">
          {pathPrefs.length > 0 ? (
            <ResponsiveContainer width="100%" height={250}>
              <BarChart data={pathPrefs}>
                <CartesianGrid strokeDasharray="3 3" stroke={theme.border} />
                <XAxis
                  dataKey="roomType"
                  tick={{ fill: theme.text.secondary, fontSize: 11 }}
                  stroke={theme.border}
                />
                <YAxis
                  tick={{ fill: theme.text.muted, fontSize: 11 }}
                  stroke={theme.border}
                />
                <Tooltip
                  {...chartTooltipStyle}
                  formatter={(value: unknown, _name: unknown, props: unknown) => {
                    const v = value as number;
                    const p = (props as { payload: { pct: number } }).payload;
                    return [`${v} (${(p.pct * 100).toFixed(1)}%)`, 'Chosen'];
                  }}
                />
                <Bar dataKey="count" name="Chosen" radius={[4, 4, 0, 0]}>
                  {pathPrefs.map((entry, i) => (
                    <Cell
                      key={i}
                      fill={ROOM_COLORS[entry.roomType as keyof typeof ROOM_COLORS] ?? theme.chart.blue}
                    />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          ) : (
            <div style={{ height: 250, display: 'flex', alignItems: 'center', justifyContent: 'center', color: theme.text.muted }}>
              No path choice data
            </div>
          )}
        </SectionCard>
      </div>

      {/* Card frequency */}
      <SectionCard title="Top 20 Most Played Cards">
        {top20Cards.length > 0 ? (
          <ResponsiveContainer width="100%" height={350}>
            <BarChart data={top20Cards} margin={{ bottom: 60 }}>
              <CartesianGrid strokeDasharray="3 3" stroke={theme.border} />
              <XAxis
                dataKey="card"
                tick={{ fill: theme.text.secondary, fontSize: 10, angle: -45, textAnchor: 'end' }}
                stroke={theme.border}
                interval={0}
                height={80}
              />
              <YAxis
                tick={{ fill: theme.text.muted, fontSize: 11 }}
                stroke={theme.border}
              />
              <Tooltip
                {...chartTooltipStyle}
                formatter={(value: unknown, name: unknown) => [
                  (value as number).toLocaleString(),
                  (name as string) === 'playCount' ? 'Total Plays' : 'Episodes',
                ]}
              />
              <Bar dataKey="playCount" name="Total Plays" fill={theme.chart.blue} radius={[4, 4, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        ) : (
          <div style={{ height: 200, display: 'flex', alignItems: 'center', justifyContent: 'center', color: theme.text.muted }}>
            No card play data
          </div>
        )}
      </SectionCard>
    </div>
  );
}
