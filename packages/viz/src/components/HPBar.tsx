interface HPBarProps {
  hp: number;
  maxHp: number;
  block?: number;
  width?: number;
  height?: number;
  x?: number;
  y?: number;
}

function hpColor(ratio: number): string {
  if (ratio > 0.6) return '#44bb44';
  if (ratio > 0.3) return '#ccaa22';
  return '#cc3333';
}

export const HPBar = ({
  hp,
  maxHp,
  block = 0,
  width = 80,
  height = 10,
  x = 0,
  y = 0,
}: HPBarProps) => {
  const ratio = Math.max(0, Math.min(1, hp / maxHp));
  const barWidth = width * ratio;
  const blockRatio = Math.min(1, block / maxHp);
  const blockWidth = width * blockRatio;
  const color = hpColor(ratio);

  return (
    <g transform={`translate(${x},${y})`}>
      {/* Background */}
      <rect width={width} height={height} rx="3" fill="#1a1a1a" stroke="#333" strokeWidth="1" />
      {/* HP fill */}
      <rect width={barWidth} height={height} rx="3" fill={color} opacity="0.85" />
      {/* Block overlay */}
      {block > 0 && (
        <rect width={blockWidth} height={height} rx="3" fill="#4488cc" opacity="0.5" />
      )}
      {/* Text */}
      <text
        x={width / 2}
        y={height / 2}
        textAnchor="middle"
        dominantBaseline="central"
        fontSize="8"
        fill="white"
        fontWeight="bold"
      >
        {hp}/{maxHp}
        {block > 0 ? ` (+${block})` : ''}
      </text>
    </g>
  );
};
