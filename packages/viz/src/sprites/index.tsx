// SVG sprite components for the Slay the Spire training viewer.
// Ported from historical commit 7b46f0ce.

// -- Stance colors --

const STANCE_COLORS: Record<string, string> = {
  neutral: '#888',
  calm: '#4488ff',
  wrath: '#ff4444',
  divinity: '#ffdd00',
};

// -- Player (Watcher) --

export const PlayerSprite = ({ stance }: { stance: string }) => {
  const glowColor = STANCE_COLORS[stance] || STANCE_COLORS.neutral;
  return (
    <g>
      <ellipse cx="0" cy="0" rx="28" ry="36" fill={glowColor} opacity="0.08" />
      {/* Robes */}
      <path
        d="M-16,-10 Q-18,-22 -8,-28 L0,-32 L8,-28 Q18,-22 16,-10 L12,22 L-12,22 Z"
        fill="#3a3a5a"
      />
      {/* Hood */}
      <path
        d="M-10,-28 Q0,-40 10,-28 Q12,-20 8,-16 L0,-14 L-8,-16 Q-12,-20 -10,-28 Z"
        fill="#2a2a44"
      />
      {/* Face shadow */}
      <ellipse cx="0" cy="-22" rx="5" ry="4" fill="#1a1a2e" />
      {/* Eyes */}
      <circle cx="-2" cy="-23" r="1" fill={glowColor} opacity="0.9" />
      <circle cx="2" cy="-23" r="1" fill={glowColor} opacity="0.9" />
      {/* Stance glow ring */}
      <ellipse
        cx="0"
        cy="22"
        rx="18"
        ry="4"
        fill="none"
        stroke={glowColor}
        strokeWidth="1.5"
        opacity="0.5"
      />
      {stance !== 'neutral' && (
        <ellipse cx="0" cy="22" rx="14" ry="3" fill={glowColor} opacity="0.15" />
      )}
    </g>
  );
};

// -- Enemy --

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
      <rect x={-hw} y={-hh} width={w} height={h} rx="4" fill={color} />
      <rect
        x={-hw + 3}
        y={-hh + 3}
        width={w - 6}
        height={h - 6}
        rx="2"
        fill="#1a1a1a"
        opacity="0.3"
      />
      {/* Eyes */}
      <circle cx={-hw / 3} cy={-hh / 3} r={w / 10} fill="#ff3333" opacity="0.8" />
      <circle cx={hw / 3} cy={-hh / 3} r={w / 10} fill="#ff3333" opacity="0.8" />
      {/* Mouth */}
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

// -- Block Shield overlay --

export const BlockShield = ({
  x,
  y,
  block,
}: {
  x: number;
  y: number;
  block: number;
}) => {
  if (block <= 0) return null;
  return (
    <g transform={`translate(${x},${y})`}>
      <path
        d="M0,-12 C-8,-12 -12,-6 -12,-6 L-12,2 C-12,10 0,16 0,16 C0,16 12,10 12,2 L12,-6 C12,-6 8,-12 0,-12 Z"
        fill="#4488ff"
        opacity="0.8"
        stroke="#6699ff"
        strokeWidth="1"
      />
      <text
        x="0"
        y="4"
        textAnchor="middle"
        fill="white"
        fontSize="11"
        fontWeight="700"
        fontFamily="monospace"
      >
        {block}
      </text>
    </g>
  );
};

// -- Map Node Icons --

export const MonsterIcon = ({ x, y, active }: { x: number; y: number; active?: boolean }) => (
  <g transform={`translate(${x},${y})`}>
    <circle r="10" fill={active ? '#e9456044' : '#e9456022'} stroke="#e94560" strokeWidth={active ? 2 : 1} />
    <path d="M-4,-4 L4,4 M4,-4 L-4,4" stroke="#e94560" strokeWidth="2" strokeLinecap="round" />
  </g>
);

export const EliteIcon = ({ x, y, active }: { x: number; y: number; active?: boolean }) => (
  <g transform={`translate(${x},${y})`}>
    <circle r="10" fill={active ? '#ff6b3544' : '#ff6b3522'} stroke="#ff6b35" strokeWidth={active ? 2 : 1} />
    <path
      d="M0,-6 L-6,0 L-4,6 L4,6 L6,0 Z"
      fill="none"
      stroke="#ff6b35"
      strokeWidth="1.5"
      strokeLinejoin="round"
    />
  </g>
);

export const BossIcon = ({ x, y, active }: { x: number; y: number; active?: boolean }) => (
  <g transform={`translate(${x},${y})`}>
    <circle r="10" fill={active ? '#cc000044' : '#cc000022'} stroke="#cc0000" strokeWidth={active ? 2 : 1} />
    <circle cx="0" cy="-1" r="5" fill="none" stroke="#cc0000" strokeWidth="1.5" />
    <circle cx="-2" cy="-2" r="1" fill="#cc0000" />
    <circle cx="2" cy="-2" r="1" fill="#cc0000" />
    <path d="M-2,2 L-1,1 L0,2 L1,1 L2,2" stroke="#cc0000" strokeWidth="1" fill="none" />
  </g>
);

export const EventIcon = ({ x, y, active }: { x: number; y: number; active?: boolean }) => (
  <g transform={`translate(${x},${y})`}>
    <circle r="10" fill={active ? '#66bb6644' : '#66bb6622'} stroke="#66bb66" strokeWidth={active ? 2 : 1} />
    <text
      x="0"
      y="5"
      textAnchor="middle"
      fill="#66bb66"
      fontSize="14"
      fontWeight="700"
      fontFamily="serif"
    >
      ?
    </text>
  </g>
);

export const ShopIcon = ({ x, y, active }: { x: number; y: number; active?: boolean }) => (
  <g transform={`translate(${x},${y})`}>
    <circle r="10" fill={active ? '#bbaa4444' : '#bbaa4422'} stroke="#bbaa44" strokeWidth={active ? 2 : 1} />
    <text
      x="0"
      y="5"
      textAnchor="middle"
      fill="#bbaa44"
      fontSize="13"
      fontWeight="700"
      fontFamily="monospace"
    >
      $
    </text>
  </g>
);

export const RestIcon = ({ x, y, active }: { x: number; y: number; active?: boolean }) => (
  <g transform={`translate(${x},${y})`}>
    <circle r="10" fill={active ? '#4488cc44' : '#4488cc22'} stroke="#4488cc" strokeWidth={active ? 2 : 1} />
    <path
      d="M0,-5 C-2,-1 -5,1 -5,4 C-5,6 -3.5,7 0,7 C3.5,7 5,6 5,4 C5,1 2,-1 0,-5 Z"
      fill="#4488cc"
      opacity="0.6"
    />
  </g>
);

export const TreasureIcon = ({ x, y, active }: { x: number; y: number; active?: boolean }) => (
  <g transform={`translate(${x},${y})`}>
    <circle r="10" fill={active ? '#ffd70044' : '#ffd70022'} stroke="#ffd700" strokeWidth={active ? 2 : 1} />
    <rect x="-5" y="-2" width="10" height="7" rx="1" fill="none" stroke="#ffd700" strokeWidth="1.5" />
    <path d="M-5,1 L5,1" stroke="#ffd700" strokeWidth="1.5" />
    <circle cx="0" cy="3" r="1" fill="#ffd700" />
  </g>
);

// -- MapNodeIcon dispatcher --

const NODE_COMPONENTS: Record<string, typeof MonsterIcon> = {
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
  active,
}: {
  type: string;
  x: number;
  y: number;
  active?: boolean;
}) => {
  const Component = NODE_COMPONENTS[type] || EventIcon;
  return <Component x={x} y={y} active={active} />;
};

// -- IntentIcon SVG (for use inside <svg>) --

export const IntentIconSvg = ({
  kind,
  x,
  y,
}: {
  kind: string;
  x: number;
  y: number;
}) => {
  switch (kind) {
    case 'attack':
      return (
        <g transform={`translate(${x},${y})`}>
          <path
            d="M-5,-5 L5,5 M5,-5 L-5,5"
            stroke="#ff4444"
            strokeWidth="2"
            strokeLinecap="round"
          />
        </g>
      );
    case 'block':
      return (
        <g transform={`translate(${x},${y})`}>
          <path
            d="M0,-7 C-5,-7 -7,-3 -7,-3 L-7,2 C-7,7 0,10 0,10 C0,10 7,7 7,2 L7,-3 C7,-3 5,-7 0,-7 Z"
            fill="#4488ff33"
            stroke="#4488ff"
            strokeWidth="1.5"
          />
        </g>
      );
    case 'buff':
      return (
        <g transform={`translate(${x},${y})`}>
          <path
            d="M0,-6 L0,6 M-4,-2 L0,-6 L4,-2"
            stroke="#44bb44"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </g>
      );
    case 'debuff':
      return (
        <g transform={`translate(${x},${y})`}>
          <path
            d="M0,-6 L0,6 M-4,2 L0,6 L4,2"
            stroke="#8b00ff"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </g>
      );
    default:
      return (
        <g transform={`translate(${x},${y})`}>
          <text
            x="0"
            y="4"
            textAnchor="middle"
            fill="#888"
            fontSize="12"
            fontWeight="700"
          >
            ?
          </text>
        </g>
      );
  }
};
