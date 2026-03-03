import type { MCTSResultMsg } from '../types/training';

interface MCTSVizProps {
  result: MCTSResultMsg | null;
}

function formatActionId(id: string): string {
  // Shorten long action IDs for display
  if (id.length <= 20) return id;
  return id.slice(0, 18) + '...';
}

function qColor(q: number, selected: boolean): string {
  if (selected) {
    // Green hue, intensity by Q
    const g = Math.round(140 + Math.min(1, Math.max(0, q)) * 115);
    return `rgb(60, ${g}, 60)`;
  }
  // Blue hue, intensity by Q
  const b = Math.round(140 + Math.min(1, Math.max(0, q)) * 115);
  return `rgb(60, 100, ${b})`;
}

function formatMs(ms: number): string {
  if (ms < 1000) return `${ms.toFixed(0)}ms`;
  return `${(ms / 1000).toFixed(2)}s`;
}

const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '16px',
    height: '100%',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    gap: '24px',
    padding: '12px 16px',
    background: 'var(--surface)',
    borderRadius: '8px',
    border: '1px solid var(--border)',
  },
  headerStat: {
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    gap: '2px',
  },
  headerLabel: {
    fontSize: '10px',
    color: '#888',
    textTransform: 'uppercase' as const,
    letterSpacing: '0.5px',
  },
  headerValue: {
    fontSize: '18px',
    fontWeight: 700,
    color: 'var(--text)',
  },
  gauge: {
    width: '80px',
    height: '8px',
    background: '#1a1a1a',
    borderRadius: '4px',
    overflow: 'hidden',
  },
  gaugeFill: (value: number) => ({
    width: `${Math.min(100, Math.max(0, (value + 1) * 50))}%`,
    height: '100%',
    background: value > 0 ? '#44bb44' : '#cc3333',
    borderRadius: '4px',
    transition: 'width 0.3s',
  }),
  barList: {
    flex: 1,
    overflow: 'auto',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '4px',
  },
  barRow: (selected: boolean) => ({
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '6px 12px',
    borderRadius: '6px',
    background: selected ? 'rgba(68, 187, 68, 0.08)' : 'transparent',
    border: selected ? '1px solid rgba(68, 187, 68, 0.3)' : '1px solid transparent',
  }),
  actionLabel: {
    width: '160px',
    fontSize: '12px',
    fontFamily: 'monospace',
    color: 'var(--text)',
    flexShrink: 0,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap' as const,
  },
  barContainer: {
    flex: 1,
    height: '20px',
    background: '#1a1a1a',
    borderRadius: '4px',
    overflow: 'hidden',
    position: 'relative' as const,
  },
  barFill: (pct: number, color: string) => ({
    width: `${pct}%`,
    height: '100%',
    background: color,
    borderRadius: '4px',
    transition: 'width 0.3s',
  }),
  barText: {
    position: 'absolute' as const,
    right: '6px',
    top: '50%',
    transform: 'translateY(-50%)',
    fontSize: '10px',
    color: '#ccc',
    fontFamily: 'monospace',
  },
  visitCount: {
    width: '50px',
    textAlign: 'right' as const,
    fontSize: '11px',
    color: '#aaa',
    fontFamily: 'monospace',
    flexShrink: 0,
  },
  qValue: {
    width: '55px',
    textAlign: 'right' as const,
    fontSize: '11px',
    color: '#aaa',
    fontFamily: 'monospace',
    flexShrink: 0,
  },
  selectedMarker: {
    fontSize: '10px',
    color: '#44bb44',
    fontWeight: 700,
    width: '16px',
    flexShrink: 0,
  },
  empty: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100%',
    color: '#666',
    fontSize: '14px',
  },
};

export const MCTSViz = ({ result }: MCTSVizProps) => {
  if (!result) {
    return (
      <div style={styles.empty}>
        Waiting for MCTS data... Focus an agent in combat
      </div>
    );
  }

  const sortedActions = [...result.actions].sort((a, b) => b.visits - a.visits);

  return (
    <div style={styles.container}>
      {/* Header stats */}
      <div style={styles.header}>
        <div style={styles.headerStat}>
          <span style={styles.headerLabel}>Simulations</span>
          <span style={styles.headerValue}>{result.sims.toLocaleString()}</span>
        </div>
        <div style={styles.headerStat}>
          <span style={styles.headerLabel}>Elapsed</span>
          <span style={styles.headerValue}>{formatMs(result.elapsed_ms)}</span>
        </div>
        <div style={styles.headerStat}>
          <span style={styles.headerLabel}>Root Value</span>
          <span style={{ ...styles.headerValue, color: result.root_value > 0 ? '#44bb44' : '#cc3333' }}>
            {result.root_value.toFixed(3)}
          </span>
          <div style={styles.gauge}>
            <div style={styles.gaugeFill(result.root_value)} />
          </div>
        </div>
        <div style={styles.headerStat}>
          <span style={styles.headerLabel}>Actions</span>
          <span style={styles.headerValue}>{result.actions.length}</span>
        </div>
      </div>

      {/* Column headers */}
      <div style={{ display: 'flex', gap: '8px', padding: '0 12px', fontSize: '10px', color: '#666', textTransform: 'uppercase', letterSpacing: '0.5px' }}>
        <span style={{ width: '16px', flexShrink: 0 }} />
        <span style={{ width: '160px', flexShrink: 0 }}>Action</span>
        <span style={{ flex: 1 }}>Distribution</span>
        <span style={{ width: '50px', textAlign: 'right', flexShrink: 0 }}>Visits</span>
        <span style={{ width: '55px', textAlign: 'right', flexShrink: 0 }}>Q-Value</span>
      </div>

      {/* Bar chart */}
      <div style={styles.barList}>
        {sortedActions.map((action) => (
          <div key={action.id} style={styles.barRow(action.selected)}>
            <span style={styles.selectedMarker}>{action.selected ? '*' : ''}</span>
            <span style={styles.actionLabel} title={action.id}>
              {formatActionId(action.id)}
            </span>
            <div style={styles.barContainer}>
              <div style={styles.barFill(action.pct, qColor(action.q, action.selected))} />
              <span style={styles.barText}>{action.pct.toFixed(1)}%</span>
            </div>
            <span style={styles.visitCount}>{action.visits}</span>
            <span style={styles.qValue}>{action.q.toFixed(3)}</span>
          </div>
        ))}
      </div>
    </div>
  );
};
