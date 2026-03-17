import { useState, useCallback, useMemo } from 'react';

// ---- Types ----

interface CurveData {
  id: string;
  label: string;
  color: string;
  data: number[];
}

interface FloorCurveProps {
  curves: CurveData[];
  width?: number;
  height?: number;
}

// ---- Constants ----

const PAD = { top: 20, right: 12, bottom: 28, left: 36 };
const GRID_LINES = [5, 10, 15, 20];
const BG = '#0d1117';
const GRID_COLOR = '#21262d';
const LABEL_COLOR = '#8b949e';
const AXIS_COLOR = '#30363d';

// ---- Component ----

export const FloorCurve = ({ curves, width = 600, height = 200 }: FloorCurveProps) => {
  const [tooltip, setTooltip] = useState<{
    x: number;
    y: number;
    values: Array<{ label: string; color: string; value: number }>;
    gameIndex: number;
  } | null>(null);

  // Compute global bounds
  const { maxGames, maxFloor } = useMemo(() => {
    let mg = 0;
    let mf = 0;
    for (const c of curves) {
      if (c.data.length > mg) mg = c.data.length;
      for (const v of c.data) {
        if (v > mf) mf = v;
      }
    }
    // Round up max floor to next grid line
    const ceilFloor = Math.max(mf, GRID_LINES[GRID_LINES.length - 1]);
    return { maxGames: mg, maxFloor: Math.ceil(ceilFloor / 5) * 5 };
  }, [curves]);

  const plotW = width - PAD.left - PAD.right;
  const plotH = height - PAD.top - PAD.bottom;

  // Convert data to SVG coords
  const curveCoords = useMemo(() => {
    if (maxGames < 2) return [];
    return curves.map((c) => ({
      ...c,
      points: c.data.map((v, i) => ({
        x: PAD.left + (i / (maxGames - 1)) * plotW,
        y: PAD.top + plotH - (v / maxFloor) * plotH,
        v,
      })),
    }));
  }, [curves, maxGames, maxFloor, plotW, plotH]);

  // X-axis tick labels
  const xTicks = useMemo(() => {
    if (maxGames < 2) return [];
    const count = Math.min(5, maxGames);
    const step = Math.floor(maxGames / count);
    const ticks: Array<{ x: number; label: string }> = [];
    for (let i = 0; i <= maxGames - 1; i += step) {
      const x = PAD.left + (i / (maxGames - 1)) * plotW;
      let label: string;
      if (i >= 1_000_000) label = `${(i / 1_000_000).toFixed(1)}M`;
      else if (i >= 1_000) label = `${(i / 1_000).toFixed(0)}K`;
      else label = String(i);
      ticks.push({ x, label });
    }
    return ticks;
  }, [maxGames, plotW]);

  const handleMouseMove = useCallback((e: React.MouseEvent<SVGSVGElement>) => {
    if (curveCoords.length === 0 || maxGames < 2) return;
    const rect = e.currentTarget.getBoundingClientRect();
    const mx = e.clientX - rect.left;

    // Map mouse X to game index
    const rawIdx = ((mx - PAD.left) / plotW) * (maxGames - 1);
    const idx = Math.max(0, Math.min(Math.round(rawIdx), maxGames - 1));
    const snapX = PAD.left + (idx / (maxGames - 1)) * plotW;

    const values: Array<{ label: string; color: string; value: number }> = [];
    for (const cc of curveCoords) {
      if (idx < cc.points.length) {
        values.push({ label: cc.label, color: cc.color, value: cc.points[idx].v });
      }
    }

    // Position tooltip at the average Y
    const avgY = values.length > 0
      ? curveCoords.reduce((sum, cc) => {
          const pt = cc.points[idx];
          return pt ? sum + pt.y : sum;
        }, 0) / values.length
      : PAD.top + plotH / 2;

    setTooltip({ x: snapX, y: avgY, values, gameIndex: idx });
  }, [curveCoords, maxGames, plotW, plotH]);

  const handleMouseLeave = useCallback(() => setTooltip(null), []);

  if (curves.length === 0) {
    return (
      <div style={{
        width,
        height,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        color: '#3d444d',
        fontSize: '11px',
        fontFamily: "'JetBrains Mono', monospace",
      }}>
        No curve data
      </div>
    );
  }

  return (
    <div style={{ position: 'relative', display: 'inline-block' }}>
      <svg
        width={width}
        height={height}
        onMouseMove={handleMouseMove}
        onMouseLeave={handleMouseLeave}
        style={{ display: 'block', cursor: 'crosshair', background: BG }}
      >
        {/* Plot background */}
        <rect
          x={PAD.left}
          y={PAD.top}
          width={plotW}
          height={plotH}
          fill="none"
          stroke={AXIS_COLOR}
          strokeWidth={0.5}
        />

        {/* Horizontal grid lines */}
        {GRID_LINES.filter((g) => g <= maxFloor).map((g) => {
          const y = PAD.top + plotH - (g / maxFloor) * plotH;
          return (
            <g key={`grid-${g}`}>
              <line
                x1={PAD.left}
                y1={y}
                x2={PAD.left + plotW}
                y2={y}
                stroke={GRID_COLOR}
                strokeWidth={0.5}
                strokeDasharray="3,3"
              />
              <text
                x={PAD.left - 4}
                y={y + 3}
                textAnchor="end"
                fill={LABEL_COLOR}
                fontSize={8}
                fontFamily="'JetBrains Mono', monospace"
              >
                F{g}
              </text>
            </g>
          );
        })}

        {/* X-axis ticks */}
        {xTicks.map((t, i) => (
          <text
            key={`xtick-${i}`}
            x={t.x}
            y={PAD.top + plotH + 14}
            textAnchor="middle"
            fill={LABEL_COLOR}
            fontSize={8}
            fontFamily="'JetBrains Mono', monospace"
          >
            {t.label}
          </text>
        ))}

        {/* X-axis label */}
        <text
          x={PAD.left + plotW / 2}
          y={height - 2}
          textAnchor="middle"
          fill="#3d444d"
          fontSize={8}
          fontFamily="'JetBrains Mono', monospace"
        >
          games
        </text>

        {/* Curves */}
        {curveCoords.map((cc) => {
          if (cc.points.length < 2) return null;
          const polyPoints = cc.points.map((p) => `${p.x},${p.y}`).join(' ');
          // Fill area under curve
          const fillPath = `M ${cc.points[0].x},${cc.points[0].y} ${cc.points.map((p) => `L ${p.x},${p.y}`).join(' ')} L ${cc.points[cc.points.length - 1].x},${PAD.top + plotH} L ${cc.points[0].x},${PAD.top + plotH} Z`;
          return (
            <g key={cc.id}>
              <path d={fillPath} fill={cc.color} fillOpacity={0.06} />
              <polyline
                points={polyPoints}
                fill="none"
                stroke={cc.color}
                strokeWidth={1.5}
                strokeLinejoin="round"
                strokeLinecap="round"
              />
            </g>
          );
        })}

        {/* Hover crosshair */}
        {tooltip && (
          <line
            x1={tooltip.x}
            y1={PAD.top}
            x2={tooltip.x}
            y2={PAD.top + plotH}
            stroke="#8b949e"
            strokeWidth={0.5}
            strokeOpacity={0.5}
            strokeDasharray="2,2"
          />
        )}

        {/* Hover dots on each curve */}
        {tooltip && curveCoords.map((cc) => {
          const pt = cc.points[tooltip.gameIndex];
          if (!pt) return null;
          return (
            <circle
              key={`dot-${cc.id}`}
              cx={pt.x}
              cy={pt.y}
              r={3}
              fill={cc.color}
              stroke={BG}
              strokeWidth={1}
            />
          );
        })}

        {/* Legend (top-right) */}
        {curveCoords.map((cc, i) => (
          <g key={`legend-${cc.id}`}>
            <line
              x1={PAD.left + plotW - 80}
              y1={PAD.top + 8 + i * 12}
              x2={PAD.left + plotW - 68}
              y2={PAD.top + 8 + i * 12}
              stroke={cc.color}
              strokeWidth={2}
            />
            <text
              x={PAD.left + plotW - 64}
              y={PAD.top + 11 + i * 12}
              fill={cc.color}
              fontSize={8}
              fontFamily="'JetBrains Mono', monospace"
            >
              {cc.label}
            </text>
          </g>
        ))}
      </svg>

      {/* Tooltip overlay */}
      {tooltip && tooltip.values.length > 0 && (
        <div style={{
          position: 'absolute',
          top: Math.max(0, tooltip.y - 10 - tooltip.values.length * 14),
          left: Math.min(tooltip.x + 8, width - 120),
          background: '#161b22',
          border: '1px solid #30363d',
          padding: '4px 8px',
          fontSize: '10px',
          fontFamily: "'JetBrains Mono', monospace",
          pointerEvents: 'none',
          whiteSpace: 'nowrap',
          zIndex: 10,
        }}>
          <div style={{ color: '#8b949e', fontSize: '8px', marginBottom: '2px' }}>
            Game {tooltip.gameIndex}
          </div>
          {tooltip.values.map((v, i) => (
            <div key={i} style={{ color: v.color, lineHeight: '14px' }}>
              {v.label}: <span style={{ fontWeight: 700 }}>{v.value.toFixed(1)}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
