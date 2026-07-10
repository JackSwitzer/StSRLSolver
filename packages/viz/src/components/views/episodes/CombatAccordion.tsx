import { useState } from 'react';
import { theme } from '../../../styles/theme';
import { ROOM_COLORS, STANCE_COLORS, CARD_TYPE_COLORS } from '../../../types/engine';
import type { Combat, Turn } from '../../../types/episode';
import type { CardType, EnemyState } from '../../../types/engine';

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

function EnemyHpBar({ enemy }: { enemy: EnemyState }) {
  const pct = enemy.maxHp > 0 ? (enemy.hp / enemy.maxHp) * 100 : 0;
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 11 }}>
      <span style={{ color: theme.text.secondary, minWidth: 80, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
        {enemy.name}
      </span>
      <div style={{
        flex: 1,
        height: 5,
        background: theme.bg.primary,
        borderRadius: 3,
        overflow: 'hidden',
        minWidth: 40,
      }}>
        <div style={{
          width: `${pct}%`,
          height: '100%',
          background: theme.danger,
          borderRadius: 3,
        }} />
      </div>
      <span style={{ color: theme.text.muted, fontSize: 10, minWidth: 50, textAlign: 'right' }}>
        {enemy.hp}/{enemy.maxHp}
      </span>
      {enemy.block > 0 && (
        <span style={{ color: theme.chart.blue, fontSize: 10 }}>
          [{enemy.block}]
        </span>
      )}
    </div>
  );
}

function TurnDetail({ turn }: { turn: Turn }) {
  return (
    <div style={{
      padding: '8px 12px',
      borderBottom: `1px solid ${theme.border}`,
      background: theme.bg.primary + '88',
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6 }}>
        <span style={{ fontSize: 11, fontWeight: 600, color: theme.text.secondary }}>
          Turn {turn.turn}
        </span>
        <span style={{ fontSize: 10, color: theme.text.muted }}>
          E: {turn.energyUsed}/{turn.energyUsed + turn.energyLeft}
        </span>
        <span style={{ fontSize: 10, color: theme.text.muted }}>
          HP: {turn.playerHp}
        </span>
        {turn.playerBlock > 0 && (
          <span style={{ fontSize: 10, color: theme.chart.blue }}>
            Block: {turn.playerBlock}
          </span>
        )}
        {turn.stance !== 'neutral' && (
          <span style={{
            fontSize: 10,
            padding: '1px 5px',
            borderRadius: 3,
            background: STANCE_COLORS[turn.stance] + '22',
            color: STANCE_COLORS[turn.stance],
            fontWeight: 600,
          }}>
            {turn.stance}
          </span>
        )}
        {turn.unplayedPlayable > 0 && (
          <span style={{ fontSize: 10, color: theme.warning }}>
            {turn.unplayedPlayable} unplayed
          </span>
        )}
      </div>

      {/* Cards played */}
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 3, marginBottom: 6 }}>
        {turn.cardsPlayed.length > 0 ? (
          turn.cardsPlayed.map((card, i) => {
            const ct = guessCardType(card);
            return (
              <span
                key={`${card}-${i}`}
                style={{
                  fontSize: 10,
                  padding: '2px 6px',
                  borderRadius: 3,
                  background: CARD_TYPE_COLORS[ct] + '22',
                  color: CARD_TYPE_COLORS[ct],
                  fontWeight: 500,
                }}
              >
                {card}
              </span>
            );
          })
        ) : (
          <span style={{ fontSize: 10, color: theme.text.muted, fontStyle: 'italic' }}>
            No cards played
          </span>
        )}
      </div>

      {/* Enemies */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
        {turn.enemies.map((enemy, i) => (
          <EnemyHpBar key={`${enemy.id}-${i}`} enemy={enemy} />
        ))}
      </div>
    </div>
  );
}

function CombatRow({ combat }: { combat: Combat }) {
  const [open, setOpen] = useState(false);
  const hpLost = combat.hpBefore - combat.hpAfter;
  const roomColor = ROOM_COLORS[combat.roomType] ?? theme.text.muted;

  return (
    <div style={{
      background: theme.bg.tertiary,
      borderRadius: 6,
      overflow: 'hidden',
      border: `1px solid ${theme.border}`,
    }}>
      <div
        onClick={() => setOpen(!open)}
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '8px 12px',
          cursor: 'pointer',
          userSelect: 'none',
        }}
        onMouseEnter={e => { (e.currentTarget as HTMLElement).style.background = theme.bg.hover; }}
        onMouseLeave={e => { (e.currentTarget as HTMLElement).style.background = 'transparent'; }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <span style={{ fontSize: 10, color: theme.text.muted, width: 14, textAlign: 'center' }}>
            {open ? '\u25BC' : '\u25B6'}
          </span>
          <span style={{
            fontSize: 10,
            padding: '1px 5px',
            borderRadius: 3,
            background: roomColor + '22',
            color: roomColor,
            fontWeight: 600,
            textTransform: 'uppercase',
          }}>
            {combat.roomType}
          </span>
          <span style={{ fontSize: 12, fontWeight: 600, color: theme.text.primary }}>
            F{combat.floor}: {combat.encounterName}
          </span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <span style={{ fontSize: 11, color: hpLost > 0 ? theme.danger : theme.success }}>
            {hpLost > 0 ? `-${hpLost}` : '+0'} HP
          </span>
          <span style={{ fontSize: 11, color: theme.text.muted }}>
            {combat.turns.length}T
          </span>
          <span style={{ fontSize: 11, color: theme.text.muted }}>
            {combat.cardsPlayed}C
          </span>
          {combat.potionsUsed > 0 && (
            <span style={{ fontSize: 11, color: theme.chart.green }}>
              {combat.potionsUsed}P
            </span>
          )}
        </div>
      </div>

      {open && (
        <div>
          {combat.turns.map(turn => (
            <TurnDetail key={turn.turn} turn={turn} />
          ))}
          {combat.turns.length === 0 && (
            <div style={{ padding: '12px', textAlign: 'center', color: theme.text.muted, fontSize: 11 }}>
              No turn data recorded
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export function CombatAccordion({ combats }: { combats: Combat[] }) {
  if (combats.length === 0) {
    return (
      <div style={{ color: theme.text.muted, padding: 16, textAlign: 'center', fontSize: 12 }}>
        No combats recorded
      </div>
    );
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
      {combats.map((combat, i) => (
        <CombatRow key={`${combat.floor}-${i}`} combat={combat} />
      ))}
    </div>
  );
}
