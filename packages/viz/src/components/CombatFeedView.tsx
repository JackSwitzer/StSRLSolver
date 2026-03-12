import { useMemo } from 'react';
import type { AgentInfo, AgentEpisodeMsg } from '../types/training';

interface Props {
  agents: AgentInfo[];
  episodes: AgentEpisodeMsg[];
  selectedAgentIndex: number;
  combatStates: Record<number, any>;
}

export function CombatFeedView({ agents, episodes, selectedAgentIndex, combatStates }: Props) {
  const agent = agents[selectedAgentIndex];
  const agentId = agent?.id ?? 0;
  const agentName = agent?.name ?? `Agent ${selectedAgentIndex}`;
  const combat = combatStates[agentId];

  const recentEpisodes = useMemo(() =>
    episodes.filter(e => e.agent_id === agentId).slice(-20),
    [episodes, agentId]
  );

  return (
    <div className="h-full flex flex-col bg-gray-900 text-gray-100">
      <div className="flex items-center justify-between px-4 py-2 border-b border-gray-700">
        <h2 className="text-lg font-bold">Combat Feed: {agentName}</h2>
        {combat && (
          <div className="flex gap-4 text-sm font-mono">
            <span>HP: {combat.player_hp ?? '-'}/{combat.player_max_hp ?? '-'}</span>
            <span>Block: {combat.player_block ?? 0}</span>
            <span>Energy: {combat.energy ?? 0}</span>
            <span className={
              combat.stance === 'Wrath' ? 'text-red-400' :
              combat.stance === 'Calm' ? 'text-blue-400' :
              combat.stance === 'Divinity' ? 'text-purple-400' : 'text-gray-400'
            }>{combat.stance ?? 'Neutral'}</span>
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto px-4 py-2">
        {!combat && recentEpisodes.length === 0 ? (
          <div className="text-gray-500 text-center mt-20">
            Waiting for combat data...
            <br />
            <span className="text-xs">Select an agent and wait for combat to begin</span>
          </div>
        ) : (
          <div className="space-y-2">
            {combat?.hand && (
              <div className="mb-4">
                <div className="text-xs text-gray-400 mb-1">Current Hand</div>
                <div className="flex gap-1 flex-wrap">
                  {combat.hand.map((card: any, i: number) => (
                    <span key={i} className="px-2 py-1 bg-gray-800 rounded text-xs font-mono">
                      {typeof card === 'string' ? card : card.id ?? card.name ?? '?'}
                    </span>
                  ))}
                </div>
              </div>
            )}

            {combat?.enemies && (
              <div className="mb-4">
                <div className="text-xs text-gray-400 mb-1">Enemies</div>
                <div className="space-y-1">
                  {combat.enemies.map((e: any, i: number) => (
                    <div key={i} className="flex items-center gap-2 text-sm font-mono">
                      <span className="text-red-300">{e.name ?? e.id ?? '?'}</span>
                      <span>HP: {e.hp ?? '?'}/{e.max_hp ?? '?'}</span>
                      {e.intent && <span className="text-yellow-400">[{e.intent}]</span>}
                    </div>
                  ))}
                </div>
              </div>
            )}

            <div className="text-xs text-gray-400 mb-1">Recent Episodes</div>
            <table className="w-full text-xs font-mono">
              <thead>
                <tr className="text-gray-500 border-b border-gray-700">
                  <th className="text-left py-1">Seed</th>
                  <th className="text-right">Floor</th>
                  <th className="text-right">HP</th>
                  <th className="text-right">Won</th>
                </tr>
              </thead>
              <tbody>
                {recentEpisodes.map((ep, i) => (
                  <tr key={i} className="border-b border-gray-800">
                    <td className="py-0.5">{ep.seed}</td>
                    <td className="text-right">{ep.floors_reached}</td>
                    <td className="text-right">{ep.hp_remaining}</td>
                    <td className="text-right">{ep.won ? 'W' : '-'}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
