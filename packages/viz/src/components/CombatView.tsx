import type { CombatState } from '../types/game';
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

export const CombatView = ({ combat }: CombatViewProps) => {
  const { player, enemies, hand, energy, max_energy, turn, stance, draw_pile_count, discard_pile_count, exhaust_pile_count } = combat;

  const svgWidth = 700;
  const svgHeight = 400;
  const playerX = 120;
  const playerY = 200;
  const enemyStartX = 420;
  const enemySpacing = 120;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <svg viewBox={`0 0 ${svgWidth} ${svgHeight}`} width="100%" style={{ flex: 1 }}>
        {/* Background gradient */}
        <defs>
          <linearGradient id="combatBg" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#1a1a2e" />
            <stop offset="100%" stopColor="#0f0f1a" />
          </linearGradient>
        </defs>
        <rect width={svgWidth} height={svgHeight} fill="url(#combatBg)" />

        {/* Turn indicator */}
        <text x="10" y="20" fill="#888" fontSize="12">
          Turn {turn}
        </text>

        {/* Stance label */}
        <text x="10" y="36" fill={stance === 'wrath' ? '#ff4444' : stance === 'calm' ? '#4488ff' : stance === 'divinity' ? '#ffdd00' : '#888'} fontSize="11">
          {STANCE_LABELS[stance] || stance}
        </text>

        {/* Energy orb */}
        <g transform={`translate(${playerX}, ${playerY + 60})`}>
          <circle r="18" fill="#1a1a2e" stroke="#ffd700" strokeWidth="2" />
          <text textAnchor="middle" dy="5" fontSize="16" fill="#ffd700" fontWeight="bold">
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
          {/* Powers */}
          {player.powers.map((power, i) => (
            <g key={power.id} transform={`translate(${-40 + i * 28}, -55)`}>
              <rect x="-12" y="-8" width="24" height="16" rx="3" fill="#2a2a44" stroke="#555" strokeWidth="0.5" />
              <text textAnchor="middle" dy="4" fontSize="7" fill="#aaa">
                {power.name.slice(0, 3)}
              </text>
              <text textAnchor="middle" dy="-1" fontSize="7" fill="#ffd700" fontWeight="bold">
                {power.amount}
              </text>
            </g>
          ))}
        </g>

        {/* Enemies */}
        {enemies.map((enemy, i) => {
          const ex = enemyStartX + i * enemySpacing;
          const ey = 180;

          return (
            <g key={enemy.id} transform={`translate(${ex}, ${ey})`}>
              {/* Enemy name */}
              <text textAnchor="middle" y={-50} fill="#ccc" fontSize="10">
                {enemy.name}
              </text>
              {/* Intent */}
              <IntentIcon intent={enemy.intent.type} x={0} y={-38} />
              {enemy.intent.damage !== undefined && (
                <text textAnchor="middle" y={-25} fill="#ff6666" fontSize="9" fontWeight="bold">
                  {enemy.intent.damage}{enemy.intent.hits && enemy.intent.hits > 1 ? `x${enemy.intent.hits}` : ''}
                </text>
              )}
              {/* Enemy sprite */}
              <EnemySprite size={enemy.size} />
              {/* Block shield */}
              <BlockShield x={-24} y={-8} block={enemy.block} />
              {/* HP bar */}
              <g transform="translate(-40, 36)">
                <HPBar hp={enemy.hp} maxHp={enemy.max_hp} block={enemy.block} />
              </g>
              {/* Powers */}
              {enemy.powers.map((power, pi) => (
                <g key={power.id} transform={`translate(${-20 + pi * 24}, 54)`}>
                  <rect x="-10" y="-6" width="20" height="12" rx="2" fill="#2a2a44" stroke="#555" strokeWidth="0.5" />
                  <text textAnchor="middle" dy="3" fontSize="6" fill="#aaa">
                    {power.name.slice(0, 3)}{power.amount > 0 ? ` ${power.amount}` : ''}
                  </text>
                </g>
              ))}
            </g>
          );
        })}

        {/* Pile counts */}
        <g transform={`translate(${svgWidth - 100}, ${svgHeight - 20})`}>
          <text fill="#666" fontSize="10">
            Draw: {draw_pile_count} | Disc: {discard_pile_count} | Exh: {exhaust_pile_count}
          </text>
        </g>
      </svg>

      {/* Card hand below the combat SVG */}
      <CardHand cards={hand} energy={energy} />
    </div>
  );
};
