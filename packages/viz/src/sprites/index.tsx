import React from 'react';

// ---------------------------------------------------------------------------
// Stance colors shared across sprites and combat view
// ---------------------------------------------------------------------------

const STANCE_COLORS: Record<string, string> = {
  neutral: '#888',
  calm: '#4488ff',
  wrath: '#ff4444',
  divinity: '#ffdd00',
};

// ---------------------------------------------------------------------------
// Watcher Sprite (viewBox 0 0 120 160)
// ---------------------------------------------------------------------------

export const WatcherSprite: React.FC<{ stance?: string; size?: number }> = ({
  stance = 'neutral',
  size = 120,
}) => {
  const glowColor =
    { neutral: 'none', calm: '#4488ff', wrath: '#ff4444', divinity: '#ffdd00' }[stance] || 'none';

  return (
    <svg viewBox="0 0 120 160" width={size} height={size * (160 / 120)}>
      {/* Seated Base */}
      <path d="M10 140 Q 60 155 110 140 L 115 130 Q 60 145 5 130 Z" fill="#3B0062" />
      <path d="M25 130 Q 60 140 95 130 L 80 60 L 40 60 Z" fill="#5A0D9D" />
      {/* Sash Detail */}
      <path d="M56 60 L 64 60 L 66 132 L 54 132 Z" fill="#FFD700" opacity="0.4" />
      {/* Braided Hair */}
      <path d="M38 65 Q 25 95 35 120" fill="none" stroke="#D2B48C" strokeWidth="4" strokeLinecap="round" />
      <path d="M82 65 Q 95 95 85 120" fill="none" stroke="#D2B48C" strokeWidth="4" strokeLinecap="round" />
      {/* Hands in Mudra */}
      <path d="M45 110 Q 35 120 50 125" fill="none" stroke="#E0AC69" strokeWidth="3" strokeLinecap="round" />
      <path d="M75 110 Q 85 120 70 125" fill="none" stroke="#E0AC69" strokeWidth="3" strokeLinecap="round" />
      {/* Hood */}
      <path d="M30 70 C 30 15, 90 15, 90 70 Q 90 88 60 88 Q 30 88 30 70" fill="#200040" />
      {/* Face Shadow */}
      <path d="M48 55 Q 60 45 72 55 Q 70 78 60 80 Q 50 78 48 55" fill="#0D001A" />
      {/* Glowing Third Eye */}
      <ellipse cx="60" cy="58" rx="3.5" ry="5.5" fill="#FFD700" opacity="0.6" />
      <ellipse cx="60" cy="58" rx="2" ry="3.5" fill="#FFD700" />
      <circle cx="60" cy="57" r="1" fill="#FFFFFF" opacity="0.8" />
      {/* Stance glow overlay */}
      {glowColor !== 'none' && (
        <circle cx="60" cy="80" r="50" fill={glowColor} opacity="0.15" />
      )}
    </svg>
  );
};

// ---------------------------------------------------------------------------
// Small Enemy Sprite (viewBox 0 0 100 100)
// ---------------------------------------------------------------------------

export const SmallEnemySprite: React.FC<{ size?: number }> = ({ size = 80 }) => (
  <svg viewBox="0 0 100 100" width={size} height={size}>
    {/* Body Shadow */}
    <ellipse cx="50" cy="85" rx="35" ry="10" fill="rgba(0,0,0,0.2)" />
    {/* Main Body */}
    <path
      d="M 15 80 Q 10 40 50 15 Q 90 40 85 80 Q 50 90 15 80 Z"
      fill="#5D2E2E"
      stroke="#3A1D1D"
      strokeWidth="2"
    />
    {/* Back Ridges */}
    <path d="M 30 25 Q 50 10 70 25" fill="none" stroke="#8B4513" strokeWidth="3" strokeLinecap="round" />
    <path d="M 25 40 Q 50 25 75 40" fill="none" stroke="#8B4513" strokeWidth="3" strokeLinecap="round" />
    {/* Mouth Interior */}
    <path d="M 25 65 Q 50 85 75 65 Q 50 55 25 65" fill="#2A0B0B" />
    {/* Teeth - Top Row */}
    <path d="M 30 62 L 35 70 L 40 63" fill="#E8E8E8" />
    <path d="M 45 61 L 50 72 L 55 61" fill="#E8E8E8" />
    <path d="M 60 63 L 65 70 L 70 62" fill="#E8E8E8" />
    {/* Teeth - Bottom Row */}
    <path d="M 35 78 L 40 70 L 45 77" fill="#E8E8E8" />
    <path d="M 55 77 L 60 70 L 65 78" fill="#E8E8E8" />
    {/* Eyes */}
    <ellipse cx="35" cy="45" rx="6" ry="3" fill="#FF4500" transform="rotate(-15 35 45)" />
    <ellipse cx="65" cy="45" rx="6" ry="3" fill="#FF4500" transform="rotate(15 65 45)" />
    {/* Pupils */}
    <circle cx="35" cy="45" r="2" fill="#FFFF00" />
    <circle cx="65" cy="45" r="2" fill="#FFFF00" />
    {/* Brow Ridges */}
    <path d="M 28 40 Q 35 35 42 42" fill="none" stroke="#3A1D1D" strokeWidth="2" />
    <path d="M 58 42 Q 65 35 72 40" fill="none" stroke="#3A1D1D" strokeWidth="2" />
  </svg>
);

// ---------------------------------------------------------------------------
// Elite Enemy Sprite (viewBox 0 0 120 140)
// ---------------------------------------------------------------------------

export const EliteEnemySprite: React.FC<{ size?: number }> = ({ size = 100 }) => (
  <svg viewBox="0 0 120 140" width={size} height={size * (140 / 120)}>
    {/* Shadow */}
    <ellipse cx="60" cy="132" rx="40" ry="6" fill="rgba(0,0,0,0.4)" />
    {/* Heavy Legs */}
    <path d="M35 130 L45 80 H75 L85 130 H65 L60 100 L55 130 Z" fill="#2c2c2c" stroke="#1a1a1a" strokeWidth="2" />
    {/* Main Armored Torso */}
    <path d="M20 90 L100 90 L110 40 L85 15 L35 15 L10 40 Z" fill="#3d3d3d" stroke="#1a1a1a" strokeWidth="2" />
    {/* Chest Plate Detail */}
    <path d="M40 30 H80 L75 65 H45 Z" fill="#4a4a4a" stroke="#222" strokeWidth="1" />
    <path d="M50 40 H70 L68 55 H52 Z" fill="#333" />
    {/* Spiked Pauldrons */}
    <path d="M10 45 C-5 20 40 10 45 25 L30 60 Z" fill="#2a2a2a" stroke="#111" strokeWidth="1.5" />
    <path d="M110 45 C125 20 80 10 75 25 L90 60 Z" fill="#2a2a2a" stroke="#111" strokeWidth="1.5" />
    {/* Shoulder Spikes */}
    <path d="M5 25 L15 10 L25 20" fill="none" stroke="#1a1a1a" strokeWidth="3" strokeLinejoin="round" />
    <path d="M115 25 L105 10 L95 20" fill="none" stroke="#1a1a1a" strokeWidth="3" strokeLinejoin="round" />
    {/* Helmet */}
    <path d="M46 5 H74 L80 25 H40 Z" fill="#1a1a1a" stroke="#000" strokeWidth="2" />
    <path d="M40 25 L50 35 H70 L80 25" fill="#1a1a1a" stroke="#000" strokeWidth="1.5" />
    {/* Glowing Eyes */}
    <rect x="52" y="16" width="6" height="3" rx="1" fill="#ff0000" />
    <rect x="62" y="16" width="6" height="3" rx="1" fill="#ff0000" />
    <circle cx="55" cy="17.5" r="1" fill="#fff" opacity="0.8" />
    <circle cx="65" cy="17.5" r="1" fill="#fff" opacity="0.8" />
    {/* Armored Gauntlets */}
    <path d="M15 70 L5 110 L20 105 Z" fill="#333" stroke="#111" strokeWidth="2" />
    <path d="M105 70 L115 110 L100 105 Z" fill="#333" stroke="#111" strokeWidth="2" />
    {/* Battle Damage Scratches */}
    <line x1="85" y1="40" x2="95" y2="55" stroke="#222" strokeWidth="1" opacity="0.5" />
    <line x1="88" y1="42" x2="98" y2="57" stroke="#222" strokeWidth="1" opacity="0.5" />
  </svg>
);

// ---------------------------------------------------------------------------
// Boss Sprite (viewBox 0 0 160 160)
// ---------------------------------------------------------------------------

export const BossSprite: React.FC<{ size?: number }> = ({ size = 120 }) => (
  <svg viewBox="0 0 160 160" width={size} height={size}>
    {/* Outer Orbiting Flames */}
    <g fill="#4deeea" opacity="0.7">
      <circle cx="80" cy="25" r="8" opacity="0.7" />
      <circle cx="128" cy="52" r="8" opacity="0.5" />
      <circle cx="128" cy="108" r="8" opacity="0.6" />
      <circle cx="80" cy="135" r="8" opacity="0.5" />
      <circle cx="32" cy="108" r="8" opacity="0.7" />
      <circle cx="32" cy="52" r="8" opacity="0.6" />
    </g>
    {/* Main Hexaghost Body */}
    <path
      d="M80 35 L120 55 L120 105 L80 125 L40 105 L40 55 Z"
      fill="#1a1a2e"
      stroke="#4deeea"
      strokeWidth="2"
    />
    {/* Central Core Eye */}
    <path d="M65 80 Q80 68 95 80 Q80 92 65 80" fill="#4deeea" opacity="0.8" />
    <circle cx="80" cy="80" r="4" fill="#fff" />
    {/* Internal Sigils */}
    <g stroke="#4deeea" strokeWidth="1" fill="none" opacity="0.5">
      <path d="M65 60 L95 60 M65 100 L95 100 M80 50 L80 110" />
    </g>
  </svg>
);

// ---------------------------------------------------------------------------
// Heart Sprite (viewBox 0 0 140 140)
// ---------------------------------------------------------------------------

export const HeartSprite: React.FC<{ size?: number }> = ({ size = 120 }) => (
  <svg viewBox="0 0 140 140" width={size} height={size}>
    <defs>
      <radialGradient id="heartCore" cx="50%" cy="50%" r="50%">
        <stop offset="0%" stopColor="#ff4d4d" />
        <stop offset="70%" stopColor="#880000" />
        <stop offset="100%" stopColor="#1a0000" />
      </radialGradient>
    </defs>
    {/* Ominous Aura */}
    <circle cx="70" cy="70" r="55" fill="#ff0000" opacity="0.15" />
    {/* Heart Mass */}
    <path
      d="M70,115 C35,100 20,75 20,50 C20,30 45,15 65,35 C68,38 70,42 70,42 C70,42 72,38 75,35 C95,15 120,30 120,50 C120,75 105,100 70,115 Z"
      fill="#120000"
      stroke="#330000"
      strokeWidth="2"
    />
    {/* Veins */}
    <path d="M40,45 Q30,65 45,90" fill="none" stroke="#4a0000" strokeWidth="2" strokeLinecap="round" opacity="0.6" />
    <path d="M100,45 Q110,65 95,90" fill="none" stroke="#4a0000" strokeWidth="2" strokeLinecap="round" opacity="0.6" />
    <path d="M70,45 L70,100" fill="none" stroke="#4a0000" strokeWidth="1.5" opacity="0.4" />
    {/* Top Arteries */}
    <path d="M55,30 Q45,10 30,15" fill="none" stroke="#0a0000" strokeWidth="4" strokeLinecap="round" />
    <path d="M85,30 Q95,10 110,15" fill="none" stroke="#0a0000" strokeWidth="4" strokeLinecap="round" />
    <path d="M70,25 V5" fill="none" stroke="#0a0000" strokeWidth="5" strokeLinecap="round" />
    {/* Glowing Core */}
    <circle cx="70" cy="60" r="14" fill="url(#heartCore)" />
    <circle cx="70" cy="60" r="6" fill="#ff8080" opacity="0.5" />
  </svg>
);

// ---------------------------------------------------------------------------
// Potion Sprite (viewBox 0 0 40 60)
// ---------------------------------------------------------------------------

export const PotionSprite: React.FC<{ size?: number; color?: string }> = ({
  size = 40,
  color = '#43E8D8',
}) => {
  // Derive a darker shade for the liquid bottom
  const darkerColor = color.replace(/^#/, '');
  const r = Math.max(0, parseInt(darkerColor.substring(0, 2), 16) - 30);
  const g = Math.max(0, parseInt(darkerColor.substring(2, 4), 16) - 30);
  const b = Math.max(0, parseInt(darkerColor.substring(4, 6), 16) - 30);
  const darkColor = `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;

  return (
    <svg viewBox="0 0 40 60" width={size} height={size * (60 / 40)}>
      {/* Cork */}
      <path d="M14 5h12v6H14z" fill="#8B5A2B" />
      <path d="M13 2h14v4H13z" fill="#A0522D" />
      {/* Neck */}
      <path d="M14 11h12v8H14z" fill="#B0E0E6" fillOpacity="0.3" stroke="#88C0D0" strokeWidth="1.5" />
      {/* Bottle Body */}
      <path
        d="M20 18C10 18 4 25 4 38s6 20 16 20 16-7 16-20-6-20-16-20z"
        fill="#E5E9F0"
        fillOpacity="0.2"
        stroke="#88C0D0"
        strokeWidth="2"
      />
      {/* Liquid */}
      <path d="M20 22C12 22 7 28 7 38s5 17 13 17 13-7 13-17-5-16-13-16z" fill={color} />
      <path
        d={`M20 22C12 22 7 28 7 38c0 5 2 10 6 13 3-8 14-8 17 0 3-3 3-8 3-13 0-10-5-16-13-16z`}
        fill={darkColor}
      />
      {/* Highlights */}
      <ellipse cx="14" cy="30" rx="3" ry="6" fill="#FFFFFF" fillOpacity="0.4" transform="rotate(15 14 30)" />
      <circle cx="28" cy="45" r="1.5" fill="#FFFFFF" fillOpacity="0.6" />
      <circle cx="24" cy="48" r="1" fill="#FFFFFF" fillOpacity="0.6" />
    </svg>
  );
};

// ---------------------------------------------------------------------------
// Legacy PlayerSprite (inline SVG <g> element, used inside a parent <svg>)
// Kept for backward compatibility with CombatView which places it in an SVG
// ---------------------------------------------------------------------------

export const PlayerSprite = ({ stance }: { stance: string }) => {
  const glowColor = STANCE_COLORS[stance] || STANCE_COLORS.neutral;

  return (
    <g>
      {/* Glow aura */}
      <ellipse cx="0" cy="0" rx="28" ry="36" fill={glowColor} opacity="0.08" />
      {/* Body / robe */}
      <path d="M-16,-10 Q-18,-22 -8,-28 L0,-32 L8,-28 Q18,-22 16,-10 L12,22 L-12,22 Z" fill="#3a3a5a" />
      {/* Hood */}
      <path d="M-10,-28 Q0,-40 10,-28 Q12,-20 8,-16 L0,-14 L-8,-16 Q-12,-20 -10,-28 Z" fill="#2a2a44" />
      {/* Face shadow */}
      <ellipse cx="0" cy="-22" rx="5" ry="4" fill="#1a1a2e" />
      {/* Eyes */}
      <circle cx="-2" cy="-23" r="1" fill={glowColor} opacity="0.9" />
      <circle cx="2" cy="-23" r="1" fill={glowColor} opacity="0.9" />
      {/* Stance glow ring */}
      <ellipse cx="0" cy="22" rx="18" ry="4" fill="none" stroke={glowColor} strokeWidth="1.5" opacity="0.5" />
      {/* Stance inner glow */}
      {stance !== 'neutral' && (
        <ellipse cx="0" cy="22" rx="14" ry="3" fill={glowColor} opacity="0.15" />
      )}
    </g>
  );
};

// ---------------------------------------------------------------------------
// Legacy EnemySprite (inline SVG <g> element, used inside a parent <svg>)
// ---------------------------------------------------------------------------

const ENEMY_SIZES = {
  small: { w: 24, h: 28 },
  medium: { w: 36, h: 42 },
  large: { w: 52, h: 58 },
};

export const EnemySprite = ({
  size,
  color = '#8b4444',
}: {
  size: 'small' | 'medium' | 'large';
  color?: string;
}) => {
  const { w, h } = ENEMY_SIZES[size];
  const hw = w / 2;
  const hh = h / 2;

  return (
    <g>
      {/* Body */}
      <rect x={-hw} y={-hh} width={w} height={h} rx="4" fill={color} />
      {/* Dark inner */}
      <rect x={-hw + 3} y={-hh + 3} width={w - 6} height={h - 6} rx="2" fill="#1a1a1a" opacity="0.3" />
      {/* Eyes */}
      <circle cx={-hw / 3} cy={-hh / 3} r={w / 10} fill="#ff3333" opacity="0.8" />
      <circle cx={hw / 3} cy={-hh / 3} r={w / 10} fill="#ff3333" opacity="0.8" />
      {/* Mouth / jaw */}
      <path
        d={`M${-hw / 3},${hh / 4} Q0,${hh / 2} ${hw / 3},${hh / 4}`}
        fill="none"
        stroke="#ff3333"
        strokeWidth="1.5"
        opacity="0.6"
      />
    </g>
  );
};

// ---------------------------------------------------------------------------
// Map node icons (unchanged)
// ---------------------------------------------------------------------------

interface NodeIconProps {
  x: number;
  y: number;
  active?: boolean;
}

const MonsterIcon = ({ x, y, active }: NodeIconProps) => (
  <g transform={`translate(${x},${y})`}>
    <circle r="12" fill={active ? '#e94560' : '#3a3a5a'} stroke={active ? '#ff6b81' : '#555'} strokeWidth="2" />
    {/* Sword icon */}
    <line x1="-4" y1="5" x2="4" y2="-5" stroke="#ccc" strokeWidth="2" strokeLinecap="round" />
    <line x1="2" y1="-5" x2="6" y2="-3" stroke="#ccc" strokeWidth="1.5" strokeLinecap="round" />
  </g>
);

const EliteIcon = ({ x, y, active }: NodeIconProps) => (
  <g transform={`translate(${x},${y})`}>
    <circle r="12" fill={active ? '#e94560' : '#5a3a3a'} stroke={active ? '#ff6b81' : '#885555'} strokeWidth="2" />
    {/* Shield icon */}
    <path d="M-4,-5 L0,-7 L4,-5 L4,2 Q0,7 -4,2 Z" fill="#ffd700" opacity="0.8" />
  </g>
);

const BossIcon = ({ x, y, active }: NodeIconProps) => (
  <g transform={`translate(${x},${y})`}>
    <circle r="14" fill={active ? '#cc0000' : '#4a2020'} stroke={active ? '#ff4444' : '#882222'} strokeWidth="2.5" />
    {/* Skull icon */}
    <circle cy="-2" r="5" fill="#ddd" opacity="0.8" />
    <circle cx="-2" cy="-3" r="1" fill="#333" />
    <circle cx="2" cy="-3" r="1" fill="#333" />
    <path d="M-2,1 L0,3 L2,1" fill="none" stroke="#333" strokeWidth="0.8" />
  </g>
);

const EventIcon = ({ x, y, active }: NodeIconProps) => (
  <g transform={`translate(${x},${y})`}>
    <circle r="12" fill={active ? '#4a8844' : '#2a4a2a'} stroke={active ? '#66bb66' : '#446644'} strokeWidth="2" />
    <text textAnchor="middle" dy="4" fontSize="14" fill="#e0e0e0" fontWeight="bold">
      ?
    </text>
  </g>
);

const ShopIcon = ({ x, y, active }: NodeIconProps) => (
  <g transform={`translate(${x},${y})`}>
    <circle r="12" fill={active ? '#886622' : '#443311'} stroke={active ? '#bbaa44' : '#665533'} strokeWidth="2" />
    <text textAnchor="middle" dy="4" fontSize="12" fill="#ffd700" fontWeight="bold">
      $
    </text>
  </g>
);

const RestIcon = ({ x, y, active }: NodeIconProps) => (
  <g transform={`translate(${x},${y})`}>
    <circle r="12" fill={active ? '#445588' : '#223344'} stroke={active ? '#6688bb' : '#445566'} strokeWidth="2" />
    {/* Campfire */}
    <path d="M-3,3 Q0,-5 3,3" fill="#ff8800" opacity="0.8" />
    <path d="M-1,3 Q0,-2 1,3" fill="#ffcc00" opacity="0.9" />
  </g>
);

const TreasureIcon = ({ x, y, active }: NodeIconProps) => (
  <g transform={`translate(${x},${y})`}>
    <circle r="12" fill={active ? '#886622' : '#443311'} stroke={active ? '#bbaa44' : '#665533'} strokeWidth="2" />
    {/* Chest */}
    <rect x="-5" y="-2" width="10" height="6" rx="1" fill="#cc8833" />
    <rect x="-5" y="-4" width="10" height="3" rx="1" fill="#ddaa44" />
    <circle cy="1" r="1" fill="#ffd700" />
  </g>
);

const NODE_ICON_MAP: Record<string, typeof MonsterIcon> = {
  monster: MonsterIcon,
  elite: EliteIcon,
  boss: BossIcon,
  event: EventIcon,
  shop: ShopIcon,
  rest: RestIcon,
  treasure: TreasureIcon,
};

export const MapNodeIcon = ({
  type,
  x,
  y,
  active = false,
}: {
  type: string;
  x: number;
  y: number;
  active?: boolean;
}) => {
  const Icon = NODE_ICON_MAP[type] || NODE_ICON_MAP.monster;
  return <Icon x={x} y={y} active={active} />;
};

export const IntentIcon = ({ intent, x, y }: { intent: string; x: number; y: number }) => {
  switch (intent) {
    case 'attack':
      return (
        <g transform={`translate(${x},${y})`}>
          <circle r="8" fill="#cc2222" opacity="0.8" />
          <line x1="-3" y1="3" x2="3" y2="-3" stroke="white" strokeWidth="1.5" strokeLinecap="round" />
          <line x1="1" y1="-3" x2="4" y2="-1" stroke="white" strokeWidth="1" strokeLinecap="round" />
        </g>
      );
    case 'defend':
      return (
        <g transform={`translate(${x},${y})`}>
          <circle r="8" fill="#2266cc" opacity="0.8" />
          <path d="M-3,-4 L0,-5 L3,-4 L3,1 Q0,5 -3,1 Z" fill="white" opacity="0.8" />
        </g>
      );
    case 'buff':
      return (
        <g transform={`translate(${x},${y})`}>
          <circle r="8" fill="#22aa44" opacity="0.8" />
          <line x1="0" y1="-3" x2="0" y2="3" stroke="white" strokeWidth="1.5" />
          <line x1="-3" y1="0" x2="3" y2="0" stroke="white" strokeWidth="1.5" />
        </g>
      );
    case 'debuff':
      return (
        <g transform={`translate(${x},${y})`}>
          <circle r="8" fill="#8822aa" opacity="0.8" />
          <line x1="-3" y1="0" x2="3" y2="0" stroke="white" strokeWidth="1.5" />
        </g>
      );
    default:
      return (
        <g transform={`translate(${x},${y})`}>
          <circle r="8" fill="#666" opacity="0.8" />
          <text textAnchor="middle" dy="3" fontSize="10" fill="white">
            ?
          </text>
        </g>
      );
  }
};

export const BlockShield = ({ x, y, block }: { x: number; y: number; block: number }) => {
  if (block <= 0) return null;
  return (
    <g transform={`translate(${x},${y})`}>
      <path d="M-10,-12 L0,-15 L10,-12 L10,2 Q0,12 -10,2 Z" fill="#4488cc" opacity="0.85" />
      <text textAnchor="middle" dy="0" fontSize="11" fill="white" fontWeight="bold">
        {block}
      </text>
    </g>
  );
};
