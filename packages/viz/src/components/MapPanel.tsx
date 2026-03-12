import { useMemo } from 'react';

// ---- Types ----

export interface MapNode {
  x: number;
  y: number;
  type: string;
  edges: Array<{ dx: number; dy: number }>;
  key?: boolean;
}

export interface MapData {
  act: number;
  nodes: MapNode[];
  position: { x: number; y: number };
  visited: Array<{ act: number; x: number; y: number }>;
  available?: Array<{ x: number; y: number; type: string; score: number }>;
}

interface MapPanelProps {
  mapData: MapData | null;
  agentName?: string;
}

// ---- Constants ----

const COLS = 7;
const ROWS = 15;
const COL_SPACING = 40;
const ROW_SPACING = 38;
const PAD_X = 30;
const PAD_TOP = 50;   // extra room for boss node + act label
const PAD_BOTTOM = 20;

const SVG_W = PAD_X * 2 + (COLS - 1) * COL_SPACING;
const SVG_H = PAD_TOP + ROWS * ROW_SPACING + PAD_BOTTOM;

const NODE_R = 12;

const TYPE_COLORS: Record<string, string> = {
  M: '#ff4444',
  E: '#ff8c00',
  R: '#00ff41',
  $: '#ffb700',
  '?': '#00e5ff',
  T: '#ffd700',
  B: '#b040ff',
};

const TYPE_LABELS: Record<string, string> = {
  M: 'M',
  E: 'E',
  R: 'R',
  $: '$',
  '?': '?',
  T: 'T',
  B: 'B',
};

// ---- Helpers ----

/** Convert grid (x, y) to SVG pixel coords. Row 0 at bottom, row 14 at top. */
function gridToSvg(x: number, y: number): { cx: number; cy: number } {
  return {
    cx: PAD_X + x * COL_SPACING,
    cy: PAD_TOP + (ROWS - 1 - y) * ROW_SPACING,
  };
}

function formatScore(score: number): string {
  const sign = score >= 0 ? '+' : '';
  return `${sign}${score.toFixed(1)}`;
}

function scoreColor(score: number): string {
  if (score > 0) return '#00ff41';
  if (score < 0) return '#ff4444';
  return '#8b949e';
}

// ---- Component ----

export const MapPanel = ({ mapData, agentName }: MapPanelProps) => {
  // Pre-compute lookup sets for O(1) checks
  const { visitedSet, availableMap } = useMemo(() => {
    if (!mapData) return { visitedSet: new Set<string>(), availableMap: new Map<string, number>() };

    const vs = new Set<string>();
    for (const v of mapData.visited) {
      if (v.act === mapData.act) vs.add(`${v.x},${v.y}`);
    }

    const am = new Map<string, number>();
    if (mapData.available) {
      for (const a of mapData.available) {
        am.set(`${a.x},${a.y}`, a.score);
      }
    }

    return { visitedSet: vs, availableMap: am };
  }, [mapData]);

  if (!mapData) {
    return (
      <div style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        height: '100%',
        fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
        fontSize: '12px',
        color: '#3d444d',
        background: '#0a0a0f',
      }}>
        No map data
      </div>
    );
  }

  const { act, nodes, position } = mapData;

  return (
    <div style={{
      width: '100%',
      height: '100%',
      background: '#0a0a0f',
      display: 'flex',
      flexDirection: 'column',
      fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
      overflow: 'hidden',
    }}>
      {/* Header */}
      <div style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '8px 12px 4px',
        flexShrink: 0,
      }}>
        <span style={{ fontSize: '11px', fontWeight: 700, color: '#c9d1d9' }}>
          ACT {act} MAP
        </span>
        {agentName && (
          <span style={{
            fontSize: '9px',
            color: '#8b949e',
            background: '#21262d',
            padding: '1px 6px',
          }}>
            {agentName}
          </span>
        )}
      </div>

      {/* SVG Map */}
      <div style={{ flex: 1, overflow: 'auto', padding: '0 4px 8px' }}>
        <svg
          viewBox={`0 0 ${SVG_W} ${SVG_H}`}
          style={{ width: '100%', height: '100%' }}
          preserveAspectRatio="xMidYMid meet"
        >
          <defs>
            {/* Glow filter for current position */}
            <filter id="glow-current" x="-50%" y="-50%" width="200%" height="200%">
              <feGaussianBlur stdDeviation="3" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>

            {/* Pulse animation for available nodes */}
            <filter id="glow-available" x="-50%" y="-50%" width="200%" height="200%">
              <feGaussianBlur stdDeviation="2" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>

          {/* Edges */}
          {nodes.map((node) =>
            node.edges.map((edge, ei) => {
              const from = gridToSvg(node.x, node.y);
              const to = gridToSvg(edge.dx, edge.dy);
              const fromVisited = visitedSet.has(`${node.x},${node.y}`);
              const toVisited = visitedSet.has(`${edge.dx},${edge.dy}`);
              const bothVisited = fromVisited && toVisited;
              const isAvailableEdge =
                (node.x === position.x && node.y === position.y) &&
                availableMap.has(`${edge.dx},${edge.dy}`);

              return (
                <line
                  key={`e-${node.x}-${node.y}-${ei}`}
                  x1={from.cx}
                  y1={from.cy}
                  x2={to.cx}
                  y2={to.cy}
                  stroke={
                    isAvailableEdge
                      ? '#00ff4180'
                      : bothVisited
                        ? '#00ff4130'
                        : '#ffffff18'
                  }
                  strokeWidth={isAvailableEdge ? 1.5 : 1}
                  strokeDasharray={isAvailableEdge ? '4 2' : undefined}
                />
              );
            })
          )}

          {/* Nodes */}
          {nodes.map((node) => {
            const { cx, cy } = gridToSvg(node.x, node.y);
            const isCurrent = node.x === position.x && node.y === position.y;
            const visited = visitedSet.has(`${node.x},${node.y}`);
            const avail = availableMap.get(`${node.x},${node.y}`);
            const isAvailable = avail !== undefined;
            const color = TYPE_COLORS[node.type] ?? '#8b949e';
            const label = TYPE_LABELS[node.type] ?? node.type;

            return (
              <g key={`n-${node.x}-${node.y}`}>
                {/* Current position glow ring */}
                {isCurrent && (
                  <circle
                    cx={cx}
                    cy={cy}
                    r={NODE_R + 4}
                    fill="none"
                    stroke="#00ff41"
                    strokeWidth={2}
                    filter="url(#glow-current)"
                    opacity={0.9}
                  >
                    <animate
                      attributeName="r"
                      values={`${NODE_R + 3};${NODE_R + 6};${NODE_R + 3}`}
                      dur="2s"
                      repeatCount="indefinite"
                    />
                    <animate
                      attributeName="opacity"
                      values="0.9;0.5;0.9"
                      dur="2s"
                      repeatCount="indefinite"
                    />
                  </circle>
                )}

                {/* Available path pulse ring */}
                {isAvailable && !isCurrent && (
                  <circle
                    cx={cx}
                    cy={cy}
                    r={NODE_R + 3}
                    fill="none"
                    stroke={color}
                    strokeWidth={1.5}
                    filter="url(#glow-available)"
                    opacity={0.7}
                  >
                    <animate
                      attributeName="opacity"
                      values="0.7;0.3;0.7"
                      dur="1.5s"
                      repeatCount="indefinite"
                    />
                  </circle>
                )}

                {/* Node circle */}
                <circle
                  cx={cx}
                  cy={cy}
                  r={NODE_R}
                  fill={visited ? '#0a0a0f' : `${color}20`}
                  stroke={visited ? '#3d444d' : color}
                  strokeWidth={isCurrent ? 2 : 1}
                  opacity={visited && !isCurrent ? 0.4 : 1}
                />

                {/* Room type label */}
                <text
                  x={cx}
                  y={cy}
                  textAnchor="middle"
                  dominantBaseline="central"
                  fill={visited && !isCurrent ? '#3d444d' : color}
                  fontSize={node.type === '$' ? '11px' : '10px'}
                  fontWeight={700}
                  fontFamily="'JetBrains Mono', 'Fira Code', monospace"
                  style={{ pointerEvents: 'none' }}
                >
                  {label}
                </text>

                {/* Visited check mark */}
                {visited && !isCurrent && (
                  <text
                    x={cx + NODE_R - 2}
                    y={cy - NODE_R + 4}
                    textAnchor="middle"
                    dominantBaseline="central"
                    fill="#3d444d"
                    fontSize="7px"
                    fontFamily="'JetBrains Mono', 'Fira Code', monospace"
                    style={{ pointerEvents: 'none' }}
                  >
                    {'\u2713'}
                  </text>
                )}

                {/* Emerald key indicator */}
                {node.key && (
                  <circle
                    cx={cx + NODE_R - 1}
                    cy={cy + NODE_R - 1}
                    r={3}
                    fill="#00ff41"
                    stroke="#0a0a0f"
                    strokeWidth={1}
                  />
                )}

                {/* Score badge for available paths */}
                {isAvailable && avail !== undefined && (
                  <g>
                    <rect
                      x={cx + NODE_R + 2}
                      y={cy - 7}
                      width={avail >= 0 ? 30 : 32}
                      height={14}
                      rx={2}
                      fill="#0d1117"
                      stroke={scoreColor(avail)}
                      strokeWidth={0.5}
                      opacity={0.9}
                    />
                    <text
                      x={cx + NODE_R + 4}
                      y={cy}
                      textAnchor="start"
                      dominantBaseline="central"
                      fill={scoreColor(avail)}
                      fontSize="8px"
                      fontWeight={600}
                      fontFamily="'JetBrains Mono', 'Fira Code', monospace"
                      style={{ pointerEvents: 'none' }}
                    >
                      {formatScore(avail)}
                    </text>
                  </g>
                )}
              </g>
            );
          })}

          {/* Act label at top */}
          <text
            x={SVG_W / 2}
            y={14}
            textAnchor="middle"
            fill="#3d444d"
            fontSize="9px"
            fontFamily="'JetBrains Mono', 'Fira Code', monospace"
          >
            {'// floor ' + (ROWS - 1) + ' \u2192 boss'}
          </text>

          {/* Floor markers on left side */}
          {[0, 5, 10, 14].map((row) => {
            const { cy } = gridToSvg(0, row);
            return (
              <text
                key={`fl-${row}`}
                x={8}
                y={cy}
                textAnchor="middle"
                dominantBaseline="central"
                fill="#21262d"
                fontSize="7px"
                fontFamily="'JetBrains Mono', 'Fira Code', monospace"
              >
                {row}
              </text>
            );
          })}

          {/* Legend */}
          {Object.entries(TYPE_COLORS).map(([type, color], i) => {
            const lx = 12;
            const ly = SVG_H - 8 - (Object.keys(TYPE_COLORS).length - 1 - i) * 12;
            return (
              <g key={`leg-${type}`}>
                <circle cx={lx} cy={ly} r={4} fill={`${color}30`} stroke={color} strokeWidth={0.8} />
                <text
                  x={lx + 8}
                  y={ly}
                  dominantBaseline="central"
                  fill="#3d444d"
                  fontSize="7px"
                  fontFamily="'JetBrains Mono', 'Fira Code', monospace"
                >
                  {type === '$' ? 'Shop' : type === '?' ? 'Event' : type === 'M' ? 'Monster' : type === 'E' ? 'Elite' : type === 'R' ? 'Rest' : type === 'T' ? 'Treasure' : 'Boss'}
                </text>
              </g>
            );
          })}
        </svg>
      </div>
    </div>
  );
};
