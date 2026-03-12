import { useState, useMemo, useCallback } from 'react';
import type { AgentInfo, CombatMiniSummary } from '../types/training';
import { MiniCombatCard } from './MiniCombatCard';

// ---- Types ----

type SortKey = 'floor' | 'hp' | 'episode' | 'winrate';

interface MultiAgentViewProps {
  agents: AgentInfo[];
  combatSummaries: Record<number, CombatMiniSummary>;
  selectedIndex: number;
  onSelectAgent: (index: number) => void;
  onExpandAgent: () => void;
}

// ---- Helpers ----

function agentWinRate(a: AgentInfo): number {
  return a.episode > 0 ? a.wins / a.episode : 0;
}

function sortAgents(agents: AgentInfo[], key: SortKey): AgentInfo[] {
  const sorted = [...agents];
  switch (key) {
    case 'floor':
      sorted.sort((a, b) => b.floor - a.floor);
      break;
    case 'hp':
      sorted.sort((a, b) => {
        const ratioA = a.max_hp > 0 ? a.hp / a.max_hp : 0;
        const ratioB = b.max_hp > 0 ? b.hp / b.max_hp : 0;
        return ratioB - ratioA;
      });
      break;
    case 'episode':
      sorted.sort((a, b) => b.episode - a.episode);
      break;
    case 'winrate':
      sorted.sort((a, b) => agentWinRate(b) - agentWinRate(a));
      break;
  }
  return sorted;
}

// ---- Component ----

export const MultiAgentView = ({
  agents,
  combatSummaries,
  selectedIndex,
  onSelectAgent,
  onExpandAgent,
}: MultiAgentViewProps) => {
  const [sortKey, setSortKey] = useState<SortKey>('floor');

  const sortedAgents = useMemo(() => sortAgents(agents, sortKey), [agents, sortKey]);

  // Find the top performer by floor (for green glow)
  const topAgentId = useMemo(() => {
    if (agents.length === 0) return -1;
    let best = agents[0];
    for (const a of agents) {
      if (a.floor > best.floor) best = a;
    }
    return best.id;
  }, [agents]);

  // Map from agent id -> original index for selection callback
  const idToIndex = useMemo(() => {
    const m = new Map<number, number>();
    for (let i = 0; i < agents.length; i++) {
      m.set(agents[i].id, i);
    }
    return m;
  }, [agents]);

  const selectedAgentId = agents[selectedIndex]?.id ?? -1;

  const handleDoubleClick = useCallback(() => {
    onExpandAgent();
  }, [onExpandAgent]);

  const SORT_OPTIONS: { key: SortKey; label: string }[] = [
    { key: 'floor', label: 'Floor' },
    { key: 'hp', label: 'HP' },
    { key: 'episode', label: 'Episode' },
    { key: 'winrate', label: 'WinRate' },
  ];

  // Responsive columns: 2x4 for 8, 3x3 for 9, etc.
  const cols = agents.length <= 4 ? agents.length : agents.length <= 8 ? 4 : Math.ceil(Math.sqrt(agents.length));

  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      gap: '0',
      height: '100%',
      overflow: 'hidden',
    }}>
      {/* Sort header */}
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        padding: '5px 10px',
        background: '#161b22',
        borderBottom: '1px solid #30363d',
        flexShrink: 0,
      }}>
        <span style={{
          fontSize: '9px',
          color: '#8b949e',
          textTransform: 'uppercase',
          letterSpacing: '0.5px',
          fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
        }}>
          SORT:
        </span>
        {SORT_OPTIONS.map(({ key, label }) => (
          <button
            key={key}
            onClick={() => setSortKey(key)}
            style={{
              background: 'none',
              border: 'none',
              padding: '2px 6px',
              fontSize: '9px',
              fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
              color: sortKey === key ? '#00ff41' : '#8b949e',
              fontWeight: sortKey === key ? 700 : 400,
              cursor: 'pointer',
              textDecoration: sortKey === key ? 'underline' : 'none',
              textUnderlineOffset: '2px',
              letterSpacing: '0.3px',
            }}
          >
            {label}
          </button>
        ))}
        <div style={{ flex: 1 }} />
        <span style={{
          fontSize: '8px',
          color: '#3d444d',
          fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
        }}>
          {agents.length} agents | dbl-click to expand
        </span>
      </div>

      {/* Agent grid */}
      <div style={{
        flex: 1,
        overflow: 'auto',
        padding: '8px',
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'flex-start',
      }}>
        <div
          onDoubleClick={handleDoubleClick}
          style={{
            display: 'grid',
            gridTemplateColumns: `repeat(${cols}, 180px)`,
            gap: '6px',
          }}
        >
          {sortedAgents.map((agent) => {
            const originalIndex = idToIndex.get(agent.id) ?? 0;
            const isTopPerformer = agent.id === topAgentId && agent.floor > 0;
            return (
              <div
                key={agent.id}
                style={{
                  boxShadow: isTopPerformer ? '0 0 12px rgba(0,255,65,0.2)' : 'none',
                  borderRadius: 0,
                }}
              >
                <MiniCombatCard
                  agent={agent}
                  combatSummary={combatSummaries[agent.id] ?? null}
                  selected={agent.id === selectedAgentId}
                  onClick={() => onSelectAgent(originalIndex)}
                />
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
