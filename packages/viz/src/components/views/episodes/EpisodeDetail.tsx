import { theme } from '../../../styles/theme';
import { useEpisodeDetail } from '../../../hooks/useEpisodes';
import { CombatAccordion } from './CombatAccordion';
import { CARD_TYPE_COLORS } from '../../../types/engine';
import type { CardType } from '../../../types/engine';
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';

function guessCardType(card: string): CardType {
  const lower = card.toLowerCase();
  if (lower.includes('strike') || lower.includes('crush') || lower.includes('sash')
    || lower.includes('flying') || lower.includes('reach') || lower.includes('wallop')
    || lower.includes('conclude') || lower.includes('tantrum') || lower.includes('pressure')
    || lower.includes('bowl') || lower.includes('cut') || lower.includes('flay')
    || lower.includes('ragnarok') || lower.includes('brilliance') || lower.includes('windmill'))
    return 'attack';
  if (lower.includes('defend') || lower.includes('vigilance') || lower.includes('protect')
    || lower.includes('halt') || lower.includes('empty') || lower.includes('evaluate')
    || lower.includes('meditate') || lower.includes('swivel') || lower.includes('safety'))
    return 'skill';
  if (lower.includes('eruption') || lower.includes('devotion') || lower.includes('establishment')
    || lower.includes('fasting') || lower.includes('like water') || lower.includes('mental')
    || lower.includes('nirvana') || lower.includes('worship') || lower.includes('deva'))
    return 'power';
  if (lower.includes('curse') || lower.includes('pain') || lower.includes('decay')
    || lower.includes('doubt') || lower.includes('shame') || lower.includes('regret')
    || lower.includes('injury') || lower.includes('normality') || lower.includes('writhe'))
    return 'curse';
  if (lower.includes('dazed') || lower.includes('burn') || lower.includes('wound')
    || lower.includes('void') || lower.includes('slimed'))
    return 'status';
  return 'skill';
}

export function EpisodeDetail({ seed, onClose }: { seed: string; onClose: () => void }) {
  const { episode, loading } = useEpisodeDetail(seed);

  if (loading) {
    return (
      <div style={{ padding: 40, textAlign: 'center', color: theme.text.muted }}>
        Loading episode {seed.slice(0, 10)}...
      </div>
    );
  }

  if (!episode) {
    return (
      <div style={{ padding: 40, textAlign: 'center', color: theme.text.muted }}>
        Episode not found
      </div>
    );
  }

  const hpData = episode.combats.map(c => ({
    floor: c.floor,
    hp: c.hpAfter,
    hpBefore: c.hpBefore,
    name: c.encounterName,
  }));

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16, padding: 16 }}>
      {/* Header */}
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'flex-start',
      }}>
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 4 }}>
            <span style={{
              fontSize: 11,
              fontWeight: 600,
              padding: '2px 8px',
              borderRadius: 4,
              background: episode.won ? theme.success + '22' : theme.danger + '22',
              color: episode.won ? theme.success : theme.danger,
            }}>
              {episode.won ? 'VICTORY' : 'DEFEAT'}
            </span>
            <span style={{ fontSize: 14, fontWeight: 600, color: theme.text.primary }}>
              Floor {episode.floor}
            </span>
          </div>
          <div style={{ fontSize: 12, color: theme.text.secondary, fontFamily: 'monospace' }}>
            Seed: {episode.seed}
          </div>
          <div style={{ fontSize: 12, color: theme.text.muted, marginTop: 2 }}>
            {episode.decisions} decisions in {(episode.durationMs / 1000).toFixed(1)}s
            {' | '}Reward: {episode.totalReward.toFixed(2)}
            {episode.deathEnemy && ` | Died to: ${episode.deathEnemy}`}
          </div>
        </div>
        <button
          onClick={onClose}
          style={{
            padding: '4px 10px',
            borderRadius: 4,
            background: theme.bg.tertiary,
            color: theme.text.secondary,
            fontSize: 12,
            border: `1px solid ${theme.border}`,
          }}
        >
          Close
        </button>
      </div>

      {/* HP Timeline */}
      <div style={{
        background: theme.bg.tertiary,
        borderRadius: 6,
        padding: 12,
      }}>
        <div style={{ fontSize: 12, color: theme.text.secondary, marginBottom: 8, fontWeight: 500 }}>
          HP Over Floors
        </div>
        {hpData.length > 0 ? (
          <ResponsiveContainer width="100%" height={140}>
            <AreaChart data={hpData}>
              <CartesianGrid strokeDasharray="3 3" stroke={theme.border} />
              <XAxis
                dataKey="floor"
                tick={{ fill: theme.text.muted, fontSize: 10 }}
                stroke={theme.border}
              />
              <YAxis
                tick={{ fill: theme.text.muted, fontSize: 10 }}
                stroke={theme.border}
                domain={[0, episode.maxHp]}
              />
              <Tooltip
                contentStyle={{
                  background: theme.bg.tertiary,
                  border: `1px solid ${theme.border}`,
                  borderRadius: 6,
                  fontSize: 11,
                  color: theme.text.primary,
                }}
                labelFormatter={(floor: unknown) => {
                  const f = floor as number;
                  const c = hpData.find(d => d.floor === f);
                  return c ? `F${f}: ${c.name}` : `F${f}`;
                }}
              />
              <Area
                type="monotone"
                dataKey="hp"
                stroke={theme.chart.red}
                fill={theme.chart.red + '33'}
                strokeWidth={2}
                name="HP"
              />
            </AreaChart>
          </ResponsiveContainer>
        ) : (
          <div style={{ height: 140, display: 'flex', alignItems: 'center', justifyContent: 'center', color: theme.text.muted }}>
            No combat data
          </div>
        )}
      </div>

      {/* Combat Accordion */}
      <div>
        <div style={{ fontSize: 13, color: theme.text.secondary, marginBottom: 8, fontWeight: 500 }}>
          Combats ({episode.combats.length})
        </div>
        <CombatAccordion combats={episode.combats} />
      </div>

      {/* Final Deck */}
      <div>
        <div style={{ fontSize: 13, color: theme.text.secondary, marginBottom: 8, fontWeight: 500 }}>
          Final Deck ({episode.deckFinal.length} cards)
        </div>
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
          {episode.deckFinal.map((card, i) => {
            const cardType = guessCardType(card);
            return (
              <span
                key={`${card}-${i}`}
                style={{
                  fontSize: 11,
                  padding: '3px 8px',
                  borderRadius: 4,
                  background: CARD_TYPE_COLORS[cardType] + '22',
                  color: CARD_TYPE_COLORS[cardType],
                  fontWeight: 500,
                }}
              >
                {card}
              </span>
            );
          })}
        </div>
      </div>

      {/* Relics */}
      {episode.relicsFinal.length > 0 && (
        <div>
          <div style={{ fontSize: 13, color: theme.text.secondary, marginBottom: 8, fontWeight: 500 }}>
            Relics ({episode.relicsFinal.length})
          </div>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
            {episode.relicsFinal.map((relic, i) => (
              <span
                key={`${relic}-${i}`}
                style={{
                  fontSize: 11,
                  padding: '3px 8px',
                  borderRadius: 4,
                  background: theme.chart.orange + '22',
                  color: theme.chart.orange,
                  fontWeight: 500,
                }}
              >
                {relic}
              </span>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
