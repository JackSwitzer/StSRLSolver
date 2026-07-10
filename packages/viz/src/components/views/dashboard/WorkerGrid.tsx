import { theme } from '../../../styles/theme';
import { useWorkers } from '../../../hooks/useWorkers';

function HpBar({ hp, maxHp }: { hp: number; maxHp: number }) {
  const pct = maxHp > 0 ? (hp / maxHp) * 100 : 0;
  const color = pct > 60 ? theme.success : pct > 30 ? theme.warning : theme.danger;
  return (
    <div style={{
      width: '100%',
      height: 6,
      background: theme.bg.primary,
      borderRadius: 3,
      overflow: 'hidden',
    }}>
      <div style={{
        width: `${pct}%`,
        height: '100%',
        background: color,
        borderRadius: 3,
        transition: 'width 300ms ease',
      }} />
    </div>
  );
}

function PhaseTag({ phase }: { phase: string }) {
  const colorMap: Record<string, string> = {
    combat: theme.danger,
    map: theme.accent,
    campfire: theme.chart.blue,
    shop: theme.chart.yellow,
    event: theme.success,
    card_reward: theme.chart.purple,
    neow: theme.chart.orange,
    game_over: theme.text.muted,
  };
  const color = colorMap[phase] ?? theme.text.muted;
  return (
    <span style={{
      fontSize: 10,
      padding: '2px 6px',
      borderRadius: 4,
      background: color + '22',
      color,
      fontWeight: 600,
      textTransform: 'uppercase',
      letterSpacing: '0.5px',
    }}>
      {phase.replace('_', ' ')}
    </span>
  );
}

export function WorkerGrid() {
  const { data: workers, loading } = useWorkers();

  if (loading && !workers) {
    return (
      <div style={{ height: 220, display: 'flex', alignItems: 'center', justifyContent: 'center', color: theme.text.muted }}>
        Loading workers...
      </div>
    );
  }

  if (!workers || workers.length === 0) {
    return (
      <div style={{ height: 220, display: 'flex', alignItems: 'center', justifyContent: 'center', color: theme.text.muted }}>
        No active workers
      </div>
    );
  }

  return (
    <div style={{
      display: 'grid',
      gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))',
      gap: 8,
      maxHeight: 220,
      overflow: 'auto',
    }}>
      {workers.map(w => (
        <div
          key={w.name}
          style={{
            background: theme.bg.tertiary,
            borderRadius: 6,
            padding: '10px 12px',
            display: 'flex',
            flexDirection: 'column',
            gap: 6,
          }}
        >
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontSize: 12, fontWeight: 600, color: theme.text.primary }}>
              {w.name}
            </span>
            <PhaseTag phase={w.phase} />
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11, color: theme.text.secondary }}>
            <span>F{w.floor}</span>
            <span>{w.hp}/{w.maxHp} HP</span>
          </div>
          <HpBar hp={w.hp} maxHp={w.maxHp} />
          {w.enemy && w.phase === 'combat' && (
            <div style={{ fontSize: 10, color: theme.text.muted, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              vs {w.enemy}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
