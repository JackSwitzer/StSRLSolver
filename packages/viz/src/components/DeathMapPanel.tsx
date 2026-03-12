import { useMemo } from 'react';
import type { DeathStats, AgentEpisodeMsg } from '../types/training';
import { Sparkline } from './Sparkline';

interface DeathMapPanelProps {
  deathStats: DeathStats;
  episodes?: AgentEpisodeMsg[];
}

/** Horizontal bar for a single entry */
const Bar = ({ label, count, maxCount, color }: {
  label: string; count: number; maxCount: number; color: string;
}) => {
  const pct = maxCount > 0 ? (count / maxCount) * 100 : 0;
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '10px', height: '16px' }}>
      <span style={{ width: '60px', textAlign: 'right', color: '#8b949e', fontFamily: 'monospace', flexShrink: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
        {label}
      </span>
      <div style={{ flex: 1, background: '#21262d', height: '10px', position: 'relative' }}>
        <div style={{
          width: `${pct}%`,
          height: '100%',
          background: color,
          opacity: 0.8,
          transition: 'width 0.3s ease',
          minWidth: count > 0 ? '2px' : '0',
        }} />
      </div>
      <span style={{ width: '28px', textAlign: 'right', color: '#c9d1d9', fontFamily: 'monospace', flexShrink: 0 }}>
        {count}
      </span>
    </div>
  );
};

const SectionHeader = ({ children, right }: { children: React.ReactNode; right?: React.ReactNode }) => (
  <div style={{
    fontSize: '9px',
    color: '#8b949e',
    textTransform: 'uppercase',
    letterSpacing: '0.5px',
    marginBottom: '4px',
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  }}>
    <span>{children}</span>
    {right && <span>{right}</span>}
  </div>
);

export const DeathMapPanel = ({ deathStats, episodes = [] }: DeathMapPanelProps) => {
  const { byFloor, byEnemy, totalDeaths } = deathStats;

  // Floor histogram: floors 1-55 condensed
  const floorBars = useMemo(() => {
    const entries = Object.entries(byFloor)
      .map(([f, c]) => ({ floor: Number(f), count: c }))
      .sort((a, b) => a.floor - b.floor);
    const max = entries.reduce((m, e) => Math.max(m, e.count), 0);
    return { entries, max };
  }, [byFloor]);

  // Enemy kill leaderboard: top 10
  const enemyBars = useMemo(() => {
    const entries = Object.entries(byEnemy)
      .map(([e, c]) => ({ enemy: e, count: c }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 10);
    const max = entries.length > 0 ? entries[0].count : 0;
    return { entries, max };
  }, [byEnemy]);

  // Damage by enemy: aggregate hp_lost from combats across all episodes
  const damageByEnemy = useMemo(() => {
    const dmg: Record<string, { total: number; combats: number }> = {};
    for (const ep of episodes) {
      if (!ep.combats) continue;
      for (const c of ep.combats) {
        if (!dmg[c.enemy]) dmg[c.enemy] = { total: 0, combats: 0 };
        dmg[c.enemy].total += c.hp_lost;
        dmg[c.enemy].combats += 1;
      }
    }
    const entries = Object.entries(dmg)
      .map(([enemy, { total, combats }]) => ({ enemy, total, avg: total / combats, combats }))
      .sort((a, b) => b.total - a.total)
      .slice(0, 8);
    const maxTotal = entries.length > 0 ? entries[0].total : 0;
    return { entries, maxTotal };
  }, [episodes]);

  // Average HP at each floor across episodes
  const hpByFloor = useMemo(() => {
    const floorData: Record<number, { sum: number; count: number }> = {};
    for (const ep of episodes) {
      if (!ep.hp_history || ep.hp_history.length === 0) continue;
      for (let f = 0; f < ep.hp_history.length; f++) {
        const floor = f + 1;
        if (!floorData[floor]) floorData[floor] = { sum: 0, count: 0 };
        floorData[floor].sum += ep.hp_history[f];
        floorData[floor].count += 1;
      }
    }
    const maxFloor = Object.keys(floorData).length > 0 ? Math.max(...Object.keys(floorData).map(Number)) : 0;
    const curve: number[] = [];
    for (let f = 1; f <= maxFloor; f++) {
      const d = floorData[f];
      curve.push(d ? d.sum / d.count : 0);
    }
    return curve;
  }, [episodes]);

  if (totalDeaths === 0 && episodes.length === 0) {
    return (
      <div style={{ padding: '12px', color: '#3d444d', fontSize: '10px', textAlign: 'center' }}>
        No deaths recorded yet
      </div>
    );
  }

  return (
    <div style={{
      display: 'grid',
      gridTemplateColumns: '1fr 1fr',
      gridTemplateRows: '1fr auto',
      gap: '0',
      height: '100%',
      overflow: 'hidden',
    }}>
      {/* Top-Left: Deaths by Floor */}
      <div style={{ borderRight: '1px solid #21262d', borderBottom: '1px solid #21262d', padding: '6px 8px', overflow: 'auto' }}>
        <SectionHeader right={<span style={{ color: '#ff4444' }}>{totalDeaths} total</span>}>
          Deaths by Floor
        </SectionHeader>
        {floorBars.entries.length > 0 ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1px' }}>
            {floorBars.entries.map(({ floor, count }) => (
              <Bar key={floor} label={`F${floor}`} count={count} maxCount={floorBars.max} color="#ff4444" />
            ))}
          </div>
        ) : (
          <div style={{ color: '#3d444d', fontSize: '10px', padding: '8px 0' }}>No deaths recorded</div>
        )}
      </div>

      {/* Top-Right: Top Killers */}
      <div style={{ borderBottom: '1px solid #21262d', padding: '6px 8px', overflow: 'auto' }}>
        <SectionHeader>Top Killers</SectionHeader>
        {enemyBars.entries.length > 0 ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1px' }}>
            {enemyBars.entries.map(({ enemy, count }) => (
              <Bar
                key={enemy}
                label={enemy.length > 10 ? enemy.slice(0, 9) + '~' : enemy}
                count={count}
                maxCount={enemyBars.max}
                color="#ffb700"
              />
            ))}
          </div>
        ) : (
          <div style={{ color: '#3d444d', fontSize: '10px', padding: '8px 0' }}>No deaths recorded</div>
        )}
      </div>

      {/* Bottom-Left: Avg HP Curve */}
      <div style={{ borderRight: '1px solid #21262d', padding: '6px 8px', overflow: 'hidden' }}>
        <SectionHeader>Avg HP by Floor</SectionHeader>
        {hpByFloor.length >= 2 ? (
          <Sparkline
            data={hpByFloor}
            width={280}
            height={60}
            color="#00ff41"
            fill={true}
          />
        ) : (
          <div style={{ color: '#3d444d', fontSize: '10px', padding: '8px 0' }}>
            Need HP history data (2+ episodes)
          </div>
        )}
      </div>

      {/* Bottom-Right: Damage by Enemy */}
      <div style={{ padding: '6px 8px', overflow: 'auto' }}>
        <SectionHeader right={damageByEnemy.entries.length > 0 ? (
          <span style={{ color: '#3d444d' }}>{damageByEnemy.entries.reduce((s, e) => s + e.combats, 0)} combats</span>
        ) : undefined}>
          Damage Taken by Enemy
        </SectionHeader>
        {damageByEnemy.entries.length > 0 ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1px' }}>
            {damageByEnemy.entries.map(({ enemy, total, avg }) => (
              <div key={enemy} style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: '10px', height: '16px' }}>
                <span style={{
                  width: '60px',
                  textAlign: 'right',
                  color: '#8b949e',
                  fontFamily: 'monospace',
                  flexShrink: 0,
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: 'nowrap',
                }}>
                  {enemy.length > 10 ? enemy.slice(0, 9) + '~' : enemy}
                </span>
                <div style={{ flex: 1, background: '#21262d', height: '10px', overflow: 'hidden' }}>
                  <div style={{
                    width: `${damageByEnemy.maxTotal > 0 ? (total / damageByEnemy.maxTotal) * 100 : 0}%`,
                    height: '100%',
                    background: '#ff4444',
                    opacity: 0.7,
                    transition: 'width 0.3s ease',
                    minWidth: '2px',
                  }} />
                </div>
                <span style={{ width: '32px', textAlign: 'right', color: '#ff4444', fontFamily: 'monospace', flexShrink: 0 }}>
                  {total}
                </span>
                <span style={{ width: '28px', textAlign: 'right', color: '#3d444d', fontFamily: 'monospace', fontSize: '8px', flexShrink: 0 }}>
                  ~{avg.toFixed(0)}
                </span>
              </div>
            ))}
          </div>
        ) : (
          <div style={{ color: '#3d444d', fontSize: '10px', padding: '8px 0' }}>
            Need combat data from episodes
          </div>
        )}
      </div>
    </div>
  );
};
