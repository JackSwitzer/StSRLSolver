import { useState, useCallback } from 'react';

export interface SparklineMarker {
  index: number;  // data index where the marker appears
  label: string;  // e.g. "T1", "T2"
}

interface SparklineProps {
  data: number[];
  width?: number;
  height?: number;
  color?: string;
  fill?: boolean;
  label?: string;
  markers?: SparklineMarker[];
}

export const Sparkline = ({ data, width = 200, height = 40, color = '#00ff41', fill = true, label, markers }: SparklineProps) => {
  const [tooltip, setTooltip] = useState<{ x: number; y: number; value: number } | null>(null);

  const points = data.length < 2 ? [] : data;

  const minVal = points.length > 0 ? Math.min(...points) : 0;
  const maxVal = points.length > 0 ? Math.max(...points) : 1;
  const range = maxVal - minVal || 1;

  const pad = 2;
  const w = width - pad * 2;
  const h = height - pad * 2;

  const coords = points.map((v, i) => {
    const x = pad + (i / (points.length - 1)) * w;
    const y = pad + h - ((v - minVal) / range) * h;
    return { x, y, v };
  });

  const polyPoints = coords.map((c) => `${c.x},${c.y}`).join(' ');

  // Fill path: go down to bottom-right, across to bottom-left, close
  const fillPath = coords.length > 1
    ? `M ${coords[0].x},${coords[0].y} ${coords.map((c) => `L ${c.x},${c.y}`).join(' ')} L ${coords[coords.length - 1].x},${height - pad} L ${coords[0].x},${height - pad} Z`
    : '';

  const handleMouseMove = useCallback((e: React.MouseEvent<SVGSVGElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const mx = e.clientX - rect.left;
    // Find closest data point
    if (coords.length === 0) return;
    let closest = coords[0];
    let minDist = Math.abs(mx - coords[0].x);
    for (const c of coords) {
      const dist = Math.abs(mx - c.x);
      if (dist < minDist) { minDist = dist; closest = c; }
    }
    setTooltip({ x: closest.x, y: closest.y, value: closest.v });
  }, [coords]);

  const handleMouseLeave = useCallback(() => setTooltip(null), []);

  // Compute marker x-positions from data indices
  const markerCoords = (markers ?? [])
    .filter((m) => m.index >= 0 && m.index < coords.length)
    .map((m) => ({ x: coords[m.index].x, label: m.label }));

  return (
    <div style={{ position: 'relative', display: 'inline-block' }}>
      {label && (
        <div style={{ fontSize: '9px', color: '#8b949e', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '2px' }}>
          {label}
        </div>
      )}
      <svg
        width={width}
        height={height}
        onMouseMove={handleMouseMove}
        onMouseLeave={handleMouseLeave}
        style={{ display: 'block', cursor: 'crosshair' }}
      >
        {fill && fillPath && (
          <path d={fillPath} fill={color} fillOpacity={0.10} stroke="none" />
        )}
        {/* Training step markers - vertical dashed lines */}
        {markerCoords.map((m, i) => (
          <g key={`marker-${i}`}>
            <line
              x1={m.x}
              y1={pad}
              x2={m.x}
              y2={height - pad}
              stroke="#a78bfa"
              strokeWidth={0.75}
              strokeDasharray="2,2"
              strokeOpacity={0.6}
            />
            <text
              x={m.x}
              y={height - 1}
              textAnchor="middle"
              fill="#a78bfa"
              fontSize={6}
              fontFamily="'JetBrains Mono', monospace"
              opacity={0.7}
            >
              {m.label}
            </text>
          </g>
        ))}
        {coords.length > 1 && (
          <polyline
            points={polyPoints}
            fill="none"
            stroke={color}
            strokeWidth={1.5}
            strokeLinejoin="round"
            strokeLinecap="round"
          />
        )}
        {tooltip && (
          <>
            <line x1={tooltip.x} y1={pad} x2={tooltip.x} y2={height - pad} stroke={color} strokeWidth={0.5} strokeOpacity={0.4} />
            <circle cx={tooltip.x} cy={tooltip.y} r={2.5} fill={color} />
          </>
        )}
        {points.length === 0 && (
          <text x={width / 2} y={height / 2 + 4} textAnchor="middle" fill="#3a3a3a" fontSize={10}>no data</text>
        )}
      </svg>
      {tooltip && (
        <div style={{
          position: 'absolute',
          top: -20,
          left: tooltip.x,
          transform: 'translateX(-50%)',
          background: '#0d1117',
          border: '1px solid #30363d',
          padding: '1px 5px',
          fontSize: '10px',
          color: color,
          fontFamily: 'monospace',
          whiteSpace: 'nowrap',
          pointerEvents: 'none',
        }}>
          {typeof tooltip.value === 'number' && !Number.isInteger(tooltip.value)
            ? tooltip.value.toFixed(3)
            : tooltip.value}
        </div>
      )}
    </div>
  );
};
