import type { TrainingStatsMsg, AgentEpisodeMsg, AgentInfo } from '../types/training';
import { AGENT_NAMES } from '../types/training';

interface TrainingStatsViewProps {
  stats: TrainingStatsMsg | null;
  episodes: AgentEpisodeMsg[];
  agents: AgentInfo[];
}

function formatUptime(seconds: number): string {
  if (seconds < 60) return `${Math.floor(seconds)}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${Math.floor(seconds % 60)}s`;
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  return `${h}h ${m}m`;
}

function formatDuration(seconds: number): string {
  if (seconds < 1) return `${(seconds * 1000).toFixed(0)}ms`;
  if (seconds < 60) return `${seconds.toFixed(1)}s`;
  return `${Math.floor(seconds / 60)}m ${Math.floor(seconds % 60)}s`;
}

function agentDisplayName(agentId: number): string {
  return AGENT_NAMES[agentId] ?? `Agent ${agentId}`;
}

const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '20px',
    height: '100%',
    overflow: 'auto',
  },
  bigNumbers: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))',
    gap: '12px',
  },
  bigCard: {
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    justifyContent: 'center',
    padding: '16px 12px',
    background: 'var(--surface)',
    borderRadius: '8px',
    border: '1px solid var(--border)',
    gap: '4px',
  },
  bigLabel: {
    fontSize: '10px',
    color: '#888',
    textTransform: 'uppercase' as const,
    letterSpacing: '0.5px',
  },
  bigValue: {
    fontSize: '28px',
    fontWeight: 700,
    color: 'var(--text)',
    fontFamily: 'monospace',
  },
  sectionTitle: {
    fontSize: '13px',
    fontWeight: 600,
    color: '#aaa',
    textTransform: 'uppercase' as const,
    letterSpacing: '0.5px',
    marginBottom: '4px',
  },
  table: {
    width: '100%',
    borderCollapse: 'collapse' as const,
    fontSize: '12px',
  },
  th: {
    textAlign: 'left' as const,
    padding: '6px 10px',
    borderBottom: '1px solid var(--border)',
    color: '#888',
    fontSize: '10px',
    textTransform: 'uppercase' as const,
    letterSpacing: '0.5px',
    fontWeight: 600,
  },
  td: {
    padding: '5px 10px',
    borderBottom: '1px solid rgba(42, 42, 68, 0.5)',
    color: 'var(--text)',
  },
  wonBadge: {
    display: 'inline-block',
    padding: '1px 6px',
    borderRadius: '3px',
    fontSize: '10px',
    fontWeight: 700,
  },
  twoCol: {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    gap: '16px',
  },
  section: {
    background: 'var(--surface)',
    borderRadius: '8px',
    border: '1px solid var(--border)',
    padding: '12px',
    overflow: 'auto',
    maxHeight: '400px',
  },
  empty: {
    textAlign: 'center' as const,
    color: '#666',
    padding: '24px',
    fontSize: '13px',
  },
};

export const TrainingStatsView = ({ stats, episodes, agents }: TrainingStatsViewProps) => {
  // Build per-agent leaderboard from episodes
  const agentStats = new Map<number, { name: string; episodes: number; wins: number }>();
  for (const ep of episodes) {
    const existing = agentStats.get(ep.agent_id);
    if (existing) {
      existing.episodes++;
      if (ep.won) existing.wins++;
    } else {
      agentStats.set(ep.agent_id, {
        name: agentDisplayName(ep.agent_id),
        episodes: 1,
        wins: ep.won ? 1 : 0,
      });
    }
  }
  // Also include agents from grid who may not have completed episodes yet
  for (const agent of agents) {
    if (!agentStats.has(agent.id)) {
      agentStats.set(agent.id, {
        name: agent.name || agentDisplayName(agent.id),
        episodes: agent.episode,
        wins: agent.wins,
      });
    }
  }
  const leaderboard = Array.from(agentStats.values()).sort((a, b) => b.wins - a.wins);

  const recentEps = episodes.slice(0, 20);

  return (
    <div style={styles.container}>
      {/* Big numbers */}
      <div style={styles.bigNumbers}>
        <div style={styles.bigCard}>
          <span style={styles.bigLabel}>Total Episodes</span>
          <span style={styles.bigValue}>{stats?.total_episodes?.toLocaleString() ?? '---'}</span>
        </div>
        <div style={styles.bigCard}>
          <span style={styles.bigLabel}>Win Rate</span>
          <span style={{ ...styles.bigValue, color: (stats?.win_rate ?? 0) > 0.5 ? '#44bb44' : '#ccaa22' }}>
            {stats ? `${(stats.win_rate * 100).toFixed(1)}%` : '---'}
          </span>
        </div>
        <div style={styles.bigCard}>
          <span style={styles.bigLabel}>Avg Floor</span>
          <span style={styles.bigValue}>{stats?.avg_floor?.toFixed(1) ?? '---'}</span>
        </div>
        <div style={styles.bigCard}>
          <span style={styles.bigLabel}>Eps / Min</span>
          <span style={styles.bigValue}>{stats?.eps_per_min?.toFixed(1) ?? '---'}</span>
        </div>
        <div style={styles.bigCard}>
          <span style={styles.bigLabel}>MCTS Avg</span>
          <span style={styles.bigValue}>{stats ? `${stats.mcts_avg_ms.toFixed(0)}ms` : '---'}</span>
        </div>
        <div style={styles.bigCard}>
          <span style={styles.bigLabel}>Uptime</span>
          <span style={styles.bigValue}>{stats ? formatUptime(stats.uptime) : '---'}</span>
        </div>
      </div>

      {/* Two columns: recent episodes + leaderboard */}
      <div style={styles.twoCol}>
        {/* Recent Episodes */}
        <div style={styles.section}>
          <div style={styles.sectionTitle}>Recent Episodes</div>
          {recentEps.length === 0 ? (
            <div style={styles.empty}>No episodes completed yet</div>
          ) : (
            <table style={styles.table}>
              <thead>
                <tr>
                  <th style={styles.th}>Agent</th>
                  <th style={styles.th}>Seed</th>
                  <th style={styles.th}>Result</th>
                  <th style={styles.th}>Floor</th>
                  <th style={styles.th}>Duration</th>
                </tr>
              </thead>
              <tbody>
                {recentEps.map((ep, i) => (
                  <tr key={`${ep.agent_id}-${ep.episode}-${i}`}>
                    <td style={styles.td}>{agentDisplayName(ep.agent_id)}</td>
                    <td style={{ ...styles.td, fontFamily: 'monospace', fontSize: '11px', color: '#999' }}>
                      {ep.seed.length > 10 ? ep.seed.slice(0, 10) + '...' : ep.seed}
                    </td>
                    <td style={styles.td}>
                      <span
                        style={{
                          ...styles.wonBadge,
                          background: ep.won ? 'rgba(68, 187, 68, 0.15)' : 'rgba(204, 51, 51, 0.15)',
                          color: ep.won ? '#44bb44' : '#cc3333',
                        }}
                      >
                        {ep.won ? 'WIN' : 'LOSS'}
                      </span>
                    </td>
                    <td style={styles.td}>{ep.floors_reached}</td>
                    <td style={{ ...styles.td, fontFamily: 'monospace', fontSize: '11px' }}>
                      {formatDuration(ep.duration)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        {/* Leaderboard */}
        <div style={styles.section}>
          <div style={styles.sectionTitle}>Agent Leaderboard</div>
          {leaderboard.length === 0 ? (
            <div style={styles.empty}>No agent data yet</div>
          ) : (
            <table style={styles.table}>
              <thead>
                <tr>
                  <th style={styles.th}>#</th>
                  <th style={styles.th}>Agent</th>
                  <th style={styles.th}>Episodes</th>
                  <th style={styles.th}>Wins</th>
                  <th style={styles.th}>Win %</th>
                </tr>
              </thead>
              <tbody>
                {leaderboard.map((agent, i) => {
                  const winPct = agent.episodes > 0 ? (agent.wins / agent.episodes) * 100 : 0;
                  return (
                    <tr key={agent.name}>
                      <td style={{ ...styles.td, color: '#666' }}>{i + 1}</td>
                      <td style={{ ...styles.td, fontWeight: 600 }}>{agent.name}</td>
                      <td style={styles.td}>{agent.episodes}</td>
                      <td style={{ ...styles.td, color: '#44bb44' }}>{agent.wins}</td>
                      <td style={{ ...styles.td, fontFamily: 'monospace' }}>
                        {agent.episodes > 0 ? `${winPct.toFixed(1)}%` : '--'}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          )}
        </div>
      </div>
    </div>
  );
};
