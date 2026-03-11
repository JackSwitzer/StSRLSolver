import { useState, useCallback } from 'react';
import type { CombatState, CardInstance, EnemyState } from '../types/game';
import { WatcherSprite, SmallEnemySprite, EliteEnemySprite, BossSprite, IntentIcon } from '../sprites';
import { CardHand } from './CardHand';

interface CombatViewProps {
  combat: CombatState;
  /** Optional: combat type hint for selecting enemy sprites */
  combatType?: 'normal' | 'elite' | 'boss';
  /** Compact mode for grid/multi-view layouts */
  compact?: boolean;
}

const STANCE_LABELS: Record<string, string> = {
  neutral: 'Neutral',
  calm: 'Calm',
  wrath: 'Wrath',
  divinity: 'Divinity',
};

const STANCE_COLORS: Record<string, string> = {
  neutral: '#888888',
  calm: '#4488ff',
  wrath: '#ff4444',
  divinity: '#ffdd00',
};

const STANCE_BG: Record<string, string> = {
  neutral: '#2a2a44',
  calm: '#1a2a44',
  wrath: '#441a1a',
  divinity: '#44441a',
};

/** Compact power abbreviations + descriptions for hover */
const POWER_INFO: Record<string, { abbr: string; desc: string }> = {
  strength: { abbr: 'STR', desc: 'Deals +N additional damage' },
  dexterity: { abbr: 'DEX', desc: 'Gains +N additional block' },
  weakened: { abbr: 'WK', desc: 'Deals 25% less damage' },
  weak: { abbr: 'WK', desc: 'Deals 25% less damage' },
  vulnerable: { abbr: 'VLN', desc: 'Takes 50% more damage' },
  vuln: { abbr: 'VLN', desc: 'Takes 50% more damage' },
  frail: { abbr: 'FRL', desc: 'Gains 25% less block from cards' },
  mental_fortress: { abbr: 'MF', desc: 'Gain N block on stance change' },
  anger: { abbr: 'AGR', desc: 'Gains N strength on skill use' },
  ritual: { abbr: 'RIT', desc: 'Gains N strength at end of turn' },
  metallicize: { abbr: 'MTL', desc: 'Gains N block at end of turn' },
  plated_armor: { abbr: 'PLT', desc: 'Gains N block at end of turn' },
  thorns: { abbr: 'THN', desc: 'Deals N damage on attack received' },
  artifact: { abbr: 'ART', desc: 'Negates N debuffs' },
  intangible: { abbr: 'INT', desc: 'Reduces all damage to 1' },
  barricade: { abbr: 'BAR', desc: 'Block is not removed at turn start' },
  mantra: { abbr: 'MNT', desc: 'N mantra toward Divinity (10)' },
  rushdown: { abbr: 'RSH', desc: 'Draw N cards on entering Wrath' },
  talk_to_the_hand: { abbr: 'TTH', desc: 'Target gains N block for you' },
  battle_hymn: { abbr: 'BH', desc: 'Add Smite to hand each turn' },
  foresight: { abbr: 'FS', desc: 'Scry N at start of turn' },
  like_water: { abbr: 'LW', desc: 'Gain N block in Calm at end of turn' },
  devotion: { abbr: 'DEV', desc: 'Gain N mantra at start of turn' },
  establishment: { abbr: 'EST', desc: 'Retain cards cost 1 less' },
  study: { abbr: 'STD', desc: 'Add Insight to hand at end of turn' },
  vigor: { abbr: 'VIG', desc: 'Next attack deals +N damage' },
};

function powerAbbr(id: string, name: string): string {
  const info = POWER_INFO[id];
  if (info) return info.abbr;
  // Fallback: first 3 chars uppercase
  return name.slice(0, 3).toUpperCase();
}

function powerTooltip(id: string, name: string, amount: number): string {
  const info = POWER_INFO[id];
  const desc = info ? info.desc.replace(/N/g, String(amount)) : name;
  return `${name} ${amount}\n${desc}`;
}

/** Rough damage preview: accounts for strength, wrath, vulnerable on target. */
function estimateDamage(card: CardInstance, combat: CombatState, _target: EnemyState | null): number | null {
  if (card.type !== 'attack') return null;

  const desc = card.description || '';
  const match = desc.match(/Deal (\d+) damage/i);
  if (!match) return null;

  let base = parseInt(match[1], 10);
  const hitsMatch = desc.match(/(\d+) times/i);
  const hits = hitsMatch ? parseInt(hitsMatch[1], 10) : 1;

  const str = combat.player.powers.find((p) => p.id === 'strength');
  if (str) base += str.amount;

  const weak = combat.player.powers.find((p) => p.id === 'weakened' || p.id === 'weak');
  if (weak && weak.amount > 0) base = Math.floor(base * 0.75);

  const s = (combat.stance || '').toLowerCase();
  if (s === 'wrath') base = Math.floor(base * 2);
  if (s === 'divinity') base = Math.floor(base * 3);

  if (_target) {
    const vuln = _target.powers.find((p) => p.id === 'vulnerable' || p.id === 'vuln');
    if (vuln && vuln.amount > 0) base = Math.floor(base * 1.5);
  }

  return Math.max(0, base) * hits;
}

/** Pick the right enemy sprite component based on combat type or enemy HP heuristic */
function EnemySpriteForType({ combatType, enemy, compact }: { combatType?: string; enemy: EnemyState; compact?: boolean }) {
  const baseSize = compact ? 40 : 60;

  // If explicit combat type
  if (combatType === 'boss') return <BossSprite size={baseSize + 20} />;
  if (combatType === 'elite') return <EliteEnemySprite size={baseSize + 10} />;

  // Heuristic: high HP enemies are bosses/elites
  if (enemy.max_hp >= 250) return <BossSprite size={baseSize + 20} />;
  if (enemy.max_hp >= 100 || enemy.size === 'large') return <EliteEnemySprite size={baseSize + 10} />;

  return <SmallEnemySprite size={baseSize} />;
}

export const CombatView = ({ combat, combatType, compact = false }: CombatViewProps) => {
  const { player, enemies, hand, energy, max_energy, turn, stance, draw_pile_count, discard_pile_count, exhaust_pile_count } = combat;
  const [hoveredCard, setHoveredCard] = useState<CardInstance | null>(null);

  const onCardHover = useCallback((card: CardInstance | null) => {
    setHoveredCard(card);
  }, []);

  const primaryTarget = enemies.length > 0 ? enemies[0] : null;
  const damagePreview = hoveredCard ? estimateDamage(hoveredCard, combat, primaryTarget) : null;

  const stanceLower = (stance || 'neutral').toLowerCase();
  const stanceColor = STANCE_COLORS[stanceLower] || STANCE_COLORS.neutral;
  const stanceBg = STANCE_BG[stanceLower] || STANCE_BG.neutral;

  const spriteSize = compact ? 60 : 90;

  return (
    <div className="combat-view-root" style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      {/* Turn counter */}
      <div className="combat-turn-bar">
        <span className="combat-turn-label">Turn {turn}</span>
        <div className="combat-pile-counts">
          <span className="combat-pile draw">Draw: {draw_pile_count}</span>
          <span className="combat-pile discard">Discard: {discard_pile_count}</span>
          {exhaust_pile_count > 0 && <span className="combat-pile exhaust">Exhaust: {exhaust_pile_count}</span>}
        </div>
      </div>

      {/* Combat arena */}
      <div className="combat-arena">
        {/* Player side */}
        <div className="combat-player-side">
          {/* Player powers */}
          {player.powers.length > 0 && (
            <div className="combat-powers">
              {player.powers.map((power) => {
                const isDebuff = ['weakened', 'weak', 'vulnerable', 'vuln', 'frail'].includes(power.id);
                const abbr = powerAbbr(power.id, power.name);
                const tooltip = powerTooltip(power.id, power.name, power.amount);
                return (
                  <span
                    key={power.id}
                    className={`combat-power-badge ${isDebuff ? 'debuff' : 'buff'}`}
                    title={tooltip}
                  >
                    <span className="combat-power-abbr">{abbr}</span>
                    <span className="combat-power-amt">{power.amount}</span>
                  </span>
                );
              })}
            </div>
          )}

          {/* Watcher sprite */}
          <div className="combat-sprite-container">
            <WatcherSprite stance={stance} size={spriteSize} />
            {/* Block shield overlay */}
            {player.block > 0 && (
              <div className="combat-block-overlay">
                <svg viewBox="0 0 30 30" width="28" height="28">
                  <path d="M5,3 L15,0 L25,3 L25,17 Q15,27 5,17 Z" fill="#4488cc" opacity="0.85" />
                  <text x="15" y="14" textAnchor="middle" dominantBaseline="central" fontSize="9" fill="white" fontWeight="bold">
                    {player.block}
                  </text>
                </svg>
              </div>
            )}
          </div>

          {/* HP bar */}
          <div className="combat-hp-bar-html">
            <div className="combat-hp-track">
              <div
                className="combat-hp-fill"
                style={{
                  width: `${Math.max(0, Math.min(100, (player.hp / player.max_hp) * 100))}%`,
                  background: player.hp / player.max_hp > 0.6 ? '#44bb44' : player.hp / player.max_hp > 0.3 ? '#ccaa22' : '#cc3333',
                }}
              />
              {player.block > 0 && (
                <div
                  className="combat-hp-block-overlay"
                  style={{ width: `${Math.min(100, (player.block / player.max_hp) * 100)}%` }}
                />
              )}
            </div>
            <span className="combat-hp-text">
              {player.hp}/{player.max_hp}
              {player.block > 0 && <span className="combat-hp-block-text"> (+{player.block})</span>}
            </span>
          </div>

          {/* Stance badge */}
          <div className="combat-stance-badge" style={{ background: stanceBg, borderColor: stanceColor, color: stanceColor }}>
            {STANCE_LABELS[stanceLower] || stance}
          </div>

          {/* Energy orbs */}
          <div className="combat-energy">
            {Array.from({ length: max_energy }, (_, i) => (
              <span key={i} className={`combat-energy-orb ${i < energy ? 'filled' : 'empty'}`} />
            ))}
            <span className="combat-energy-text">{energy}/{max_energy}</span>
          </div>
        </div>

        {/* Enemies side */}
        <div className="combat-enemies-side">
          {enemies.map((enemy, i) => {
            const previewOnThis = i === 0 && damagePreview !== null;
            return (
              <div key={enemy.id} className="combat-enemy-unit">
                {/* Enemy name */}
                <div className="combat-enemy-name">{enemy.name}</div>

                {/* Intent */}
                <div className="combat-enemy-intent">
                  <svg viewBox="0 0 20 20" width="18" height="18">
                    <IntentIcon intent={enemy.intent.type} x={10} y={10} />
                  </svg>
                  {enemy.intent.damage !== undefined && (
                    <span className="combat-intent-damage">
                      {enemy.intent.damage}
                      {enemy.intent.hits && enemy.intent.hits > 1 ? `x${enemy.intent.hits}` : ''}
                    </span>
                  )}
                </div>

                {/* Damage preview */}
                {previewOnThis && (
                  <div className="combat-damage-preview">-{damagePreview}</div>
                )}

                {/* Enemy sprite */}
                <div className="combat-sprite-container">
                  <EnemySpriteForType combatType={combatType} enemy={enemy} compact={compact} />
                  {enemy.block > 0 && (
                    <div className="combat-block-overlay">
                      <svg viewBox="0 0 30 30" width="24" height="24">
                        <path d="M5,3 L15,0 L25,3 L25,17 Q15,27 5,17 Z" fill="#4488cc" opacity="0.85" />
                        <text x="15" y="14" textAnchor="middle" dominantBaseline="central" fontSize="9" fill="white" fontWeight="bold">
                          {enemy.block}
                        </text>
                      </svg>
                    </div>
                  )}
                </div>

                {/* HP bar */}
                <div className="combat-hp-bar-html">
                  <div className="combat-hp-track">
                    <div
                      className="combat-hp-fill"
                      style={{
                        width: `${Math.max(0, Math.min(100, (enemy.hp / enemy.max_hp) * 100))}%`,
                        background: enemy.hp / enemy.max_hp > 0.6 ? '#44bb44' : enemy.hp / enemy.max_hp > 0.3 ? '#ccaa22' : '#cc3333',
                      }}
                    />
                  </div>
                  <span className="combat-hp-text">{enemy.hp}/{enemy.max_hp}</span>
                </div>

                {/* Powers */}
                {enemy.powers.length > 0 && (
                  <div className="combat-powers">
                    {enemy.powers.map((power) => {
                      const isDebuff = ['weakened', 'weak', 'vulnerable', 'vuln', 'frail'].includes(power.id);
                      const abbr = powerAbbr(power.id, power.name);
                      const tooltip = powerTooltip(power.id, power.name, power.amount);
                      return (
                        <span
                          key={power.id}
                          className={`combat-power-badge ${isDebuff ? 'debuff' : 'buff'}`}
                          title={tooltip}
                        >
                          <span className="combat-power-abbr">{abbr}</span>
                          <span className="combat-power-amt">{power.amount}</span>
                        </span>
                      );
                    })}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Card hand - text only in compact, full cards otherwise */}
      {compact ? (
        <div className="combat-hand-compact">
          {hand.map((card, i) => (
            <span
              key={`${card.id}-${i}`}
              className={`combat-hand-card-text ${card.playable !== false && card.cost <= energy ? 'playable' : 'unplayable'}`}
              style={{ borderLeftColor: card.type === 'attack' ? '#cc3333' : card.type === 'skill' ? '#3366cc' : '#ccaa22' }}
            >
              {card.cost >= 0 ? `[${card.cost}]` : '[X]'} {card.name}{card.upgraded ? '+' : ''}
            </span>
          ))}
        </div>
      ) : (
        <CardHand cards={hand} energy={energy} onCardHover={onCardHover} />
      )}
    </div>
  );
};
