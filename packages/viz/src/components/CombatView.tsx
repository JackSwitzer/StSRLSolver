import { useState, useCallback } from 'react';
import type { CombatState, CardInstance, EnemyState } from '../types/game';
import { PlayerSprite, EnemySprite, IntentIcon, BlockShield } from '../sprites';
import { HPBar } from './HPBar';
import { CardHand } from './CardHand';

interface CombatViewProps {
  combat: CombatState;
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

/** Rough damage preview: accounts for strength, wrath, vulnerable on target. */
function estimateDamage(card: CardInstance, combat: CombatState, _target: EnemyState | null): number | null {
  if (card.type !== 'attack') return null;

  // Parse base damage from description (e.g. "Deal 6 damage" or "Deal 3 damage 4 times")
  const desc = card.description || '';
  const match = desc.match(/Deal (\d+) damage/i);
  if (!match) return null;

  let base = parseInt(match[1], 10);
  const hitsMatch = desc.match(/(\d+) times/i);
  const hits = hitsMatch ? parseInt(hitsMatch[1], 10) : 1;

  // Apply strength
  const str = combat.player.powers.find((p) => p.id === 'strength');
  if (str) base += str.amount;

  // Apply weak (player debuff)
  const weak = combat.player.powers.find((p) => p.id === 'weakened' || p.id === 'weak');
  if (weak && weak.amount > 0) base = Math.floor(base * 0.75);

  // Wrath doubles damage
  if (combat.stance === 'wrath') base = Math.floor(base * 2);
  if (combat.stance === 'divinity') base = Math.floor(base * 3);

  // Vulnerable on target
  if (_target) {
    const vuln = _target.powers.find((p) => p.id === 'vulnerable' || p.id === 'vuln');
    if (vuln && vuln.amount > 0) base = Math.floor(base * 1.5);
  }

  return Math.max(0, base) * hits;
}

export const CombatView = ({ combat }: CombatViewProps) => {
  const { player, enemies, hand, energy, max_energy, turn, stance, draw_pile_count, discard_pile_count, exhaust_pile_count } = combat;
  const [hoveredCard, setHoveredCard] = useState<CardInstance | null>(null);

  const onCardHover = useCallback((card: CardInstance | null) => {
    setHoveredCard(card);
  }, []);

  const svgWidth = 700;
  const svgHeight = 400;
  const playerX = 120;
  const playerY = 200;
  const enemyStartX = 420;
  const enemySpacing = 120;

  // Compute damage preview for first enemy when hovering an attack card
  const primaryTarget = enemies.length > 0 ? enemies[0] : null;
  const damagePreview = hoveredCard ? estimateDamage(hoveredCard, combat, primaryTarget) : null;

  const stanceColor = STANCE_COLORS[stance] || STANCE_COLORS.neutral;
  const stanceBg = STANCE_BG[stance] || STANCE_BG.neutral;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <svg viewBox={`0 0 ${svgWidth} ${svgHeight}`} width="100%" style={{ flex: 1 }}>
        {/* Background gradient */}
        <defs>
          <linearGradient id="combatBg" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#1a1a2e" />
            <stop offset="100%" stopColor="#0f0f1a" />
          </linearGradient>
          {/* Stance glow filter */}
          <filter id="stanceGlow">
            <feGaussianBlur stdDeviation="2" result="blur" />
            <feMerge>
              <feMergeNode in="blur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>
        <rect width={svgWidth} height={svgHeight} fill="url(#combatBg)" />

        {/* Turn counter - top center */}
        <g transform={`translate(${svgWidth / 2}, 18)`}>
          <rect x="-32" y="-12" width="64" height="22" rx="6" fill="#2a2a44" stroke="#555" strokeWidth="0.5" />
          <text textAnchor="middle" dy="4" fill="#e0e0e0" fontSize="11" fontWeight="600">
            Turn {turn}
          </text>
        </g>

        {/* Stance indicator badge - below player */}
        <g transform={`translate(${playerX}, ${playerY + 50})`}>
          <rect
            x="-28"
            y="-9"
            width="56"
            height="18"
            rx="9"
            fill={stanceBg}
            stroke={stanceColor}
            strokeWidth="1.5"
            className="stance-badge"
          />
          <text textAnchor="middle" dy="4" fill={stanceColor} fontSize="9" fontWeight="bold">
            {STANCE_LABELS[stance] || stance}
          </text>
        </g>

        {/* Energy orbs display */}
        <g transform={`translate(${playerX}, ${playerY + 76})`}>
          {Array.from({ length: max_energy }, (_, i) => {
            const filled = i < energy;
            const ox = (i - (max_energy - 1) / 2) * 18;
            return (
              <g key={`orb-${i}`} transform={`translate(${ox}, 0)`}>
                <circle
                  r="7"
                  fill={filled ? '#ffd700' : '#1a1a2e'}
                  stroke={filled ? '#ffee88' : '#444'}
                  strokeWidth={filled ? 1.5 : 1}
                  opacity={filled ? 1 : 0.5}
                />
                {filled && (
                  <circle r="3" fill="#fff8dc" opacity="0.4" />
                )}
              </g>
            );
          })}
          {/* Energy text overlay */}
          <text textAnchor="middle" dy="22" fontSize="10" fill="#ffd700" fontWeight="bold">
            {energy}/{max_energy}
          </text>
        </g>

        {/* Player */}
        <g transform={`translate(${playerX}, ${playerY})`}>
          <PlayerSprite stance={stance} />
          {/* Player HP bar */}
          <g transform="translate(-40, 32)">
            <HPBar hp={player.hp} maxHp={player.max_hp} block={player.block} />
          </g>
          {/* Block shield */}
          <BlockShield x={-30} y={-10} block={player.block} />
          {/* Powers row */}
          {player.powers.length > 0 && (
            <g transform="translate(0, -55)">
              {player.powers.map((power, i) => {
                const px = (i - (player.powers.length - 1) / 2) * 30;
                const isDebuff = ['weakened', 'weak', 'vulnerable', 'vuln', 'frail'].includes(power.id);
                const badgeColor = isDebuff ? '#882244' : '#224488';
                const textColor = isDebuff ? '#ff8888' : '#88ccff';
                return (
                  <g key={power.id} transform={`translate(${px}, 0)`}>
                    <rect x="-13" y="-9" width="26" height="18" rx="4" fill={badgeColor} stroke="#555" strokeWidth="0.5" />
                    <text textAnchor="middle" dy="-1" fontSize="7" fill={textColor} fontWeight="bold">
                      {power.amount}
                    </text>
                    <text textAnchor="middle" dy="7" fontSize="5" fill="#aaa">
                      {power.name.length > 5 ? power.name.slice(0, 5) : power.name}
                    </text>
                  </g>
                );
              })}
            </g>
          )}
        </g>

        {/* Enemies */}
        {enemies.map((enemy, i) => {
          const ex = enemyStartX + i * enemySpacing;
          const ey = 180;

          // Damage preview on this enemy if hovering attack card
          const previewOnThis = i === 0 && damagePreview !== null;

          return (
            <g key={enemy.id} transform={`translate(${ex}, ${ey})`}>
              {/* Enemy name */}
              <text textAnchor="middle" y={-55} fill="#ccc" fontSize="10" fontWeight="600">
                {enemy.name}
              </text>

              {/* Intent display */}
              <g transform="translate(0, -40)">
                <IntentIcon intent={enemy.intent.type} x={0} y={0} />
                {enemy.intent.damage !== undefined && (
                  <text textAnchor="middle" y={14} fill="#ff6666" fontSize="9" fontWeight="bold">
                    {enemy.intent.damage}{enemy.intent.hits && enemy.intent.hits > 1 ? `x${enemy.intent.hits}` : ''}
                  </text>
                )}
              </g>

              {/* Enemy sprite */}
              <EnemySprite size={enemy.size} />

              {/* Damage preview overlay */}
              {previewOnThis && (
                <g className="damage-preview">
                  <text textAnchor="middle" y={-70} fill="#ff4444" fontSize="14" fontWeight="bold" opacity="0.85">
                    -{damagePreview}
                  </text>
                </g>
              )}

              {/* Block shield */}
              <BlockShield x={-24} y={-8} block={enemy.block} />

              {/* HP bar */}
              <g transform="translate(-40, 36)">
                <HPBar hp={enemy.hp} maxHp={enemy.max_hp} block={enemy.block} />
              </g>

              {/* Powers */}
              {enemy.powers.length > 0 && (
                <g transform="translate(0, 54)">
                  {enemy.powers.map((power, pi) => {
                    const ppx = (pi - (enemy.powers.length - 1) / 2) * 26;
                    const isDebuff = ['weakened', 'weak', 'vulnerable', 'vuln', 'frail'].includes(power.id);
                    const badgeColor = isDebuff ? '#882244' : '#442222';
                    const textColor = isDebuff ? '#ff8888' : '#ffaa88';
                    return (
                      <g key={power.id} transform={`translate(${ppx}, 0)`}>
                        <rect x="-11" y="-7" width="22" height="14" rx="3" fill={badgeColor} stroke="#555" strokeWidth="0.5" />
                        <text textAnchor="middle" dy="0" fontSize="6" fill={textColor} fontWeight="bold">
                          {power.amount}
                        </text>
                        <text textAnchor="middle" dy="6" fontSize="5" fill="#aaa">
                          {power.name.length > 4 ? power.name.slice(0, 4) : power.name}
                        </text>
                      </g>
                    );
                  })}
                </g>
              )}
            </g>
          );
        })}

        {/* Draw pile count - bottom left */}
        <g transform="translate(20, 370)">
          <rect x="-6" y="-12" width="60" height="20" rx="4" fill="#2a2a44" stroke="#555" strokeWidth="0.5" />
          <text fill="#88aacc" fontSize="9" fontWeight="600">
            Draw: {draw_pile_count}
          </text>
        </g>

        {/* Discard pile count - bottom center */}
        <g transform={`translate(${svgWidth / 2 - 30}, 370)`}>
          <rect x="-6" y="-12" width="72" height="20" rx="4" fill="#2a2a44" stroke="#555" strokeWidth="0.5" />
          <text fill="#aa8888" fontSize="9" fontWeight="600">
            Discard: {discard_pile_count}
          </text>
        </g>

        {/* Exhaust pile count - bottom right */}
        <g transform={`translate(${svgWidth - 90}, 370)`}>
          <rect x="-6" y="-12" width="72" height="20" rx="4" fill="#2a2a44" stroke="#555" strokeWidth="0.5" />
          <text fill="#888" fontSize="9" fontWeight="600">
            Exhaust: {exhaust_pile_count}
          </text>
        </g>
      </svg>

      {/* Card hand below the combat SVG */}
      <CardHand cards={hand} energy={energy} onCardHover={onCardHover} />
    </div>
  );
};
