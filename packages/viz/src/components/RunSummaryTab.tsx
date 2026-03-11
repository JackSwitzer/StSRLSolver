import type { AgentInfo } from '../types/training';
import type { AgentEpisodeMsg } from '../types/training';
import type { CardInstance, RelicInstance, PotionSlot } from '../types/game';
import { Sparkline } from './Sparkline';

interface RunSummaryTabProps {
  agent: AgentInfo;
  episodes: AgentEpisodeMsg[];
  /** Full run state extras if available from combat WS data */
  runExtras?: {
    deck?: CardInstance[];
    relics?: RelicInstance[];
    potions?: PotionSlot[];
    gold?: number;
  };
}

const CARD_TYPE_ORDER: Array<CardInstance['type']> = ['attack', 'skill', 'power', 'status', 'curse'];
const CARD_TYPE_COLORS: Record<string, string> = {
  attack: '#cc3333',
  skill: '#3366cc',
  power: '#ccaa22',
  status: '#888888',
  curse: '#882288',
};
const CARD_TYPE_LABELS: Record<string, string> = {
  attack: 'Attacks',
  skill: 'Skills',
  power: 'Powers',
  status: 'Statuses',
  curse: 'Curses',
};

function groupCards(deck: CardInstance[]): Map<string, CardInstance[]> {
  const groups = new Map<string, CardInstance[]>();
  for (const t of CARD_TYPE_ORDER) {
    groups.set(t, []);
  }
  for (const card of deck) {
    const t = card.type || 'status';
    if (!groups.has(t)) groups.set(t, []);
    groups.get(t)!.push(card);
  }
  return groups;
}

const StatRow = ({ label, value, color = '#c9d1d9' }: { label: string; value: string; color?: string }) => (
  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline', fontSize: '10px', padding: '1px 0' }}>
    <span style={{ color: '#8b949e' }}>{label}</span>
    <span style={{ color, fontFamily: 'monospace', fontWeight: 600 }}>{value}</span>
  </div>
);

export const RunSummaryTab = ({ agent, episodes, runExtras }: RunSummaryTabProps) => {
  // HP floor history: derive from this agent's episodes (last 50)
  const agentEpisodes = episodes.filter((e) => e.agent_id === agent.id).slice(0, 50);
  const hpHistory = agentEpisodes.map((e) => e.hp_remaining);
  const floorHistory = agentEpisodes.map((e) => e.floors_reached);
  const wins = agentEpisodes.filter((e) => e.won).length;
  const bestFloor = agentEpisodes.length > 0 ? Math.max(...agentEpisodes.map((e) => e.floors_reached)) : 0;
  const avgFloor = agentEpisodes.length > 0
    ? (agentEpisodes.reduce((s, e) => s + e.floors_reached, 0) / agentEpisodes.length).toFixed(1)
    : '---';

  const deck = runExtras?.deck ?? [];
  const relics = runExtras?.relics ?? [];
  const potions = runExtras?.potions ?? [];
  const gold = runExtras?.gold;

  const cardGroups = groupCards(deck);

  return (
    <div style={{
      display: 'grid',
      gridTemplateColumns: '1fr 1fr 1fr',
      gap: '0',
      height: '100%',
      overflow: 'hidden',
    }}>
      {/* Column 1: Stats + sparklines */}
      <div style={{ borderRight: '1px solid #21262d', padding: '8px', overflow: 'auto', display: 'flex', flexDirection: 'column', gap: '8px' }}>
        <div style={{ fontSize: '9px', color: '#8b949e', textTransform: 'uppercase', letterSpacing: '0.5px' }}>
          Run Stats ({agentEpisodes.length} games)
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
          <StatRow label="Episode" value={String(agent.episode)} />
          <StatRow label="Wins" value={String(wins)} color="#00ff41" />
          <StatRow
            label="Win Rate"
            value={agentEpisodes.length > 0 ? `${((wins / agentEpisodes.length) * 100).toFixed(1)}%` : '0.0%'}
            color={wins > 0 ? '#00ff41' : '#8b949e'}
          />
          <StatRow label="Best Floor" value={bestFloor > 0 ? String(bestFloor) : '---'} />
          <StatRow label="Avg Floor" value={avgFloor} />
          <StatRow label="Current Floor" value={String(Math.floor(agent.floor))} />
          <StatRow label="Seed" value={agent.seed?.slice(0, 8) ?? '?'} />
          {gold !== undefined && <StatRow label="Gold" value={String(gold)} color="#ffb700" />}
          <StatRow label="Status" value={agent.status} />
        </div>

        {floorHistory.length >= 2 && (
          <div>
            <div style={{ fontSize: '9px', color: '#8b949e', marginBottom: '3px' }}>Floor / Game</div>
            <Sparkline data={floorHistory.slice().reverse()} width={140} height={32} color="#4488ff" />
          </div>
        )}
        {hpHistory.length >= 2 && (
          <div>
            <div style={{ fontSize: '9px', color: '#8b949e', marginBottom: '3px' }}>HP Remaining</div>
            <Sparkline data={hpHistory.slice().reverse()} width={140} height={32} color="#00ff41" />
          </div>
        )}
      </div>

      {/* Column 2: Deck */}
      <div style={{ borderRight: '1px solid #21262d', padding: '8px', overflow: 'auto' }}>
        <div style={{ fontSize: '9px', color: '#8b949e', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '6px' }}>
          Deck ({deck.length} cards)
        </div>
        {deck.length === 0 ? (
          <span style={{ fontSize: '10px', color: '#3d444d' }}>No deck data</span>
        ) : (
          CARD_TYPE_ORDER.map((type) => {
            const cards = cardGroups.get(type) ?? [];
            if (cards.length === 0) return null;
            return (
              <div key={type} style={{ marginBottom: '6px' }}>
                <div style={{
                  fontSize: '8px',
                  color: CARD_TYPE_COLORS[type],
                  textTransform: 'uppercase',
                  letterSpacing: '0.5px',
                  marginBottom: '2px',
                }}>
                  {CARD_TYPE_LABELS[type]} ({cards.length})
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '1px' }}>
                  {cards.map((card, i) => (
                    <div key={`${card.id}-${i}`} style={{
                      fontSize: '10px',
                      color: '#c9d1d9',
                      display: 'flex',
                      gap: '4px',
                    }}>
                      <span style={{ color: CARD_TYPE_COLORS[card.type], fontFamily: 'monospace' }}>
                        [{card.cost >= 0 ? card.cost : 'X'}]
                      </span>
                      <span>{card.name}{card.upgraded ? '+' : ''}</span>
                    </div>
                  ))}
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* Column 3: Relics + Potions */}
      <div style={{ padding: '8px', overflow: 'auto', display: 'flex', flexDirection: 'column', gap: '8px' }}>
        <div>
          <div style={{ fontSize: '9px', color: '#8b949e', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '4px' }}>
            Relics ({relics.length})
          </div>
          {relics.length === 0 ? (
            <span style={{ fontSize: '10px', color: '#3d444d' }}>No relic data</span>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
              {relics.map((relic) => (
                <div key={relic.id} style={{ fontSize: '10px', color: '#c9d1d9', display: 'flex', gap: '4px', alignItems: 'baseline' }}>
                  <span style={{ color: '#ffb700' }}>{relic.name}</span>
                  {relic.counter !== undefined && relic.counter !== null && (
                    <span style={{ fontSize: '9px', color: '#8b949e' }}>[{relic.counter}]</span>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        <div>
          <div style={{ fontSize: '9px', color: '#8b949e', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '4px' }}>
            Potions
          </div>
          {potions.length === 0 ? (
            <span style={{ fontSize: '10px', color: '#3d444d' }}>No potion data</span>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
              {potions.map((slot, i) => (
                <div key={i} style={{ fontSize: '10px' }}>
                  {slot.name ? (
                    <span style={{ color: '#cc88ff' }}>{slot.name}</span>
                  ) : (
                    <span style={{ color: '#3d444d' }}>[empty]</span>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
