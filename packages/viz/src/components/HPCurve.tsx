interface HPCurveProps {
  hpHistory?: number[];
  maxHp?: number;
  deathFloor?: number;
}

function hpColor(hp: number, maxHp: number): string {
  const pct = hp / maxHp;
  if (pct > 0.6) return '#3fb950';
  if (pct > 0.3) return '#d29922';
  return '#f85149';
}

export const HPCurve: React.FC<HPCurveProps> = ({
  hpHistory,
  maxHp = 80,
  deathFloor,
}) => {
  const W = 400;
  const H = 120;
  const PAD_L = 30;
  const PAD_R = 8;
  const PAD_T = 8;
  const PAD_B = 18;
  const plotW = W - PAD_L - PAD_R;
  const plotH = H - PAD_T - PAD_B;

  if (!hpHistory || hpHistory.length === 0) {
    return (
      <div
        style={{
          width: W,
          height: H,
          background: '#161b22',
          border: '1px solid #30363d',
          borderRadius: 4,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#484f58',
          fontSize: 12,
          fontStyle: 'italic',
        }}
      >
        No HP data
      </div>
    );
  }

  const n = hpHistory.length;
  const xStep = n > 1 ? plotW / (n - 1) : 0;

  // Build points
  const points = hpHistory.map((hp, i) => ({
    x: PAD_L + i * xStep,
    y: PAD_T + plotH - (Math.min(hp, maxHp) / maxHp) * plotH,
    hp,
  }));

  // Build colored line segments
  const segments: Array<{ x1: number; y1: number; x2: number; y2: number; color: string }> = [];
  for (let i = 0; i < points.length - 1; i++) {
    const p1 = points[i];
    const p2 = points[i + 1];
    // Color by the lower HP of the two endpoints
    const minHp = Math.min(p1.hp, p2.hp);
    segments.push({
      x1: p1.x,
      y1: p1.y,
      x2: p2.x,
      y2: p2.y,
      color: hpColor(minHp, maxHp),
    });
  }

  // Death floor marker
  const deathIdx =
    deathFloor != null && deathFloor > 0 && deathFloor <= n
      ? deathFloor - 1
      : null;

  // Max HP reference line Y
  const maxHpY = PAD_T;

  // Y-axis labels
  const yLabels = [0, Math.round(maxHp / 2), maxHp];

  // X-axis labels (every ~10 floors, plus last)
  const xLabels: number[] = [];
  const step = Math.max(1, Math.ceil(n / 8));
  for (let i = 0; i < n; i += step) xLabels.push(i);
  if (xLabels[xLabels.length - 1] !== n - 1) xLabels.push(n - 1);

  return (
    <svg
      width={W}
      height={H}
      style={{
        background: '#161b22',
        border: '1px solid #30363d',
        borderRadius: 4,
        display: 'block',
      }}
    >
      {/* Max HP dashed reference line */}
      <line
        x1={PAD_L}
        y1={maxHpY}
        x2={W - PAD_R}
        y2={maxHpY}
        stroke="#30363d"
        strokeWidth={1}
        strokeDasharray="4 3"
      />

      {/* Y-axis labels */}
      {yLabels.map((val) => {
        const y = PAD_T + plotH - (val / maxHp) * plotH;
        return (
          <text
            key={val}
            x={PAD_L - 4}
            y={y + 3}
            textAnchor="end"
            fill="#484f58"
            fontSize={9}
            fontFamily="inherit"
          >
            {val}
          </text>
        );
      })}

      {/* X-axis labels */}
      {xLabels.map((i) => (
        <text
          key={i}
          x={points[i].x}
          y={H - 2}
          textAnchor="middle"
          fill="#484f58"
          fontSize={9}
          fontFamily="inherit"
        >
          {i + 1}
        </text>
      ))}

      {/* HP line segments */}
      {segments.map((seg, i) => (
        <line
          key={i}
          x1={seg.x1}
          y1={seg.y1}
          x2={seg.x2}
          y2={seg.y2}
          stroke={seg.color}
          strokeWidth={1.5}
          strokeLinecap="round"
        />
      ))}

      {/* Data points (small dots) */}
      {points.map((p, i) => (
        <circle
          key={i}
          cx={p.x}
          cy={p.y}
          r={n > 30 ? 1 : 2}
          fill={hpColor(p.hp, maxHp)}
        />
      ))}

      {/* Death marker */}
      {deathIdx != null && (
        <text
          x={points[deathIdx].x}
          y={points[deathIdx].y - 4}
          textAnchor="middle"
          fill="#f85149"
          fontSize={14}
          fontWeight={700}
          fontFamily="inherit"
        >
          X
        </text>
      )}
    </svg>
  );
};
