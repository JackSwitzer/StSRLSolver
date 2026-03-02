const STANCE_COLORS: Record<string, string> = {
  neutral: '#888',
  calm: '#4488ff',
  wrath: '#ff4444',
  divinity: '#ffdd00',
};

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
