import { useState } from 'react';
import type { MapState, MapNodeType } from '../types/game';
import { MapNodeIcon } from '../sprites';

interface MapViewProps {
  map: MapState;
}

const NODE_SPACING_Y = 48;
const MARGIN_X = 60;
const MARGIN_TOP = 40;

const NODE_TYPE_LABELS: Record<MapNodeType, string> = {
  monster: 'Monster',
  elite: 'Elite',
  boss: 'Boss',
  event: 'Event',
  shop: 'Shop',
  rest: 'Rest Site',
  treasure: 'Treasure',
};

function coordKey(pos: { x: number; y: number }): string {
  return `${pos.x},${pos.y}`;
}

function isNodeAvailable(
  node: { x: number; y: number },
  available: { x: number; y: number }[],
): boolean {
  return available.some((n) => n.x === node.x && n.y === node.y);
}

function isNodeCurrent(
  node: { x: number; y: number },
  current: { x: number; y: number } | null,
): boolean {
  if (!current) return false;
  return node.x === current.x && node.y === current.y;
}

export const MapView = ({ map }: MapViewProps) => {
  const { nodes, edges, current_node, available_next, boss_name, visited_path } = map;
  const [tooltip, setTooltip] = useState<{ label: string; px: number; py: number } | null>(null);

  // Build visited set from visited_path
  const visitedSet = new Set<string>();
  if (visited_path) {
    for (const v of visited_path) {
      visitedSet.add(coordKey(v));
    }
  }

  // Build visited edge set (edges between consecutive visited nodes)
  const visitedEdgeSet = new Set<string>();
  if (visited_path && visited_path.length > 1) {
    for (let i = 0; i < visited_path.length - 1; i++) {
      const fromKey = coordKey(visited_path[i]);
      const toKey = coordKey(visited_path[i + 1]);
      visitedEdgeSet.add(`${fromKey}->${toKey}`);
    }
  }

  const maxFloor = nodes.length;
  const maxPathWidth = Math.max(...nodes.map((floor) => floor.length), 1);

  // Responsive: use wider spacing when there's room
  const baseSpacingX = 70;
  const nodeSpacingX = maxPathWidth <= 3 ? baseSpacingX + 20 : baseSpacingX;

  const svgWidth = maxPathWidth * nodeSpacingX + MARGIN_X * 2;
  // Extra space at top for boss node
  const bossExtraSpace = 40;
  const svgHeight = (maxFloor + 1) * NODE_SPACING_Y + MARGIN_TOP * 2 + bossExtraSpace;

  function nodePos(idx: number, y: number): { px: number; py: number } {
    const floorNodes = nodes[y] || [];
    const floorWidth = floorNodes.length * nodeSpacingX;
    const offsetX = (svgWidth - floorWidth) / 2 + nodeSpacingX / 2;
    return {
      px: offsetX + idx * nodeSpacingX,
      py: svgHeight - MARGIN_TOP - y * NODE_SPACING_Y,
    };
  }

  // Build a lookup for node positions by their original x,y coordinates
  const posLookup = new Map<string, { px: number; py: number }>();
  for (const floorNodes of nodes) {
    for (let i = 0; i < floorNodes.length; i++) {
      const node = floorNodes[i];
      const pos = nodePos(i, node.y);
      posLookup.set(coordKey(node), pos);
    }
  }

  // Determine the act number from floor data (heuristic: boss at top of each 15-floor act)
  const currentFloor = current_node?.y ?? 0;
  const actNumber = Math.floor(currentFloor / 17) + 1;

  return (
    <svg
      viewBox={`0 0 ${svgWidth} ${svgHeight}`}
      width="100%"
      height="100%"
      style={{ maxHeight: '80vh' }}
    >
      {/* Title with act indicator */}
      <text x={svgWidth / 2} y="20" textAnchor="middle" fill="#e0e0e0" fontSize="14" fontWeight="bold">
        Act {actNumber} {boss_name ? `\u2014 ${boss_name}` : ''}
      </text>

      {/* Act number badge */}
      <g transform={`translate(${svgWidth / 2}, 36)`}>
        <rect x="-20" y="-8" width="40" height="14" rx="4" fill="#2a2a44" stroke="#555" strokeWidth="0.5" />
        <text textAnchor="middle" dy="3" fontSize="8" fill="#888">
          Floor {currentFloor}
        </text>
      </g>

      {/* Edges */}
      {edges.map((edge, i) => {
        const fromKey = coordKey(edge.from);
        const toKey = coordKey(edge.to);
        const from = posLookup.get(fromKey);
        const to = posLookup.get(toKey);
        if (!from || !to) return null;

        const isNextEdge =
          isNodeCurrent(edge.from, current_node) &&
          isNodeAvailable(edge.to, available_next);

        const isVisitedEdge = visitedEdgeSet.has(`${fromKey}->${toKey}`);

        let stroke = '#333';
        let strokeWidth = 1;
        let opacity = 0.4;

        if (isNextEdge) {
          stroke = '#e94560';
          strokeWidth = 2.5;
          opacity = 0.9;
        } else if (isVisitedEdge) {
          stroke = '#ffd700';
          strokeWidth = 2;
          opacity = 0.7;
        }

        return (
          <line
            key={`edge-${i}`}
            x1={from.px}
            y1={from.py}
            x2={to.px}
            y2={to.py}
            stroke={stroke}
            strokeWidth={strokeWidth}
            opacity={opacity}
            strokeLinecap="round"
          />
        );
      })}

      {/* Nodes */}
      {nodes.flatMap((floorNodes) =>
        floorNodes.map((node) => {
          const pos = posLookup.get(coordKey(node));
          if (!pos) return null;

          const isCurrent = isNodeCurrent(node, current_node);
          const isAvailable = isNodeAvailable(node, available_next);
          const isVisited = visitedSet.has(coordKey(node));
          const isBoss = node.type === 'boss';

          return (
            <g
              key={`node-${node.x}-${node.y}`}
              onMouseEnter={() => setTooltip({ label: NODE_TYPE_LABELS[node.type] || node.type, px: pos.px, py: pos.py })}
              onMouseLeave={() => setTooltip(null)}
              style={{ cursor: 'pointer' }}
            >
              {/* Visited dim ring */}
              {isVisited && !isCurrent && (
                <circle cx={pos.px} cy={pos.py} r="16" fill="none" stroke="#ffd700" strokeWidth="1" opacity="0.35" />
              )}
              {/* Current node pulse */}
              {isCurrent && (
                <circle cx={pos.px} cy={pos.py} r="18" fill="none" stroke="#ffd700" strokeWidth="2" opacity="0.7" className="pulse-ring">
                  <animate attributeName="opacity" values="0.4;0.9;0.4" dur="2s" repeatCount="indefinite" />
                  <animate attributeName="r" values="16;20;16" dur="2s" repeatCount="indefinite" />
                </circle>
              )}
              {/* Available next pulse */}
              {isAvailable && !isCurrent && (
                <circle cx={pos.px} cy={pos.py} r="16" fill="none" stroke="#e94560" strokeWidth="1.5" opacity="0.6">
                  <animate attributeName="r" values="14;18;14" dur="1.5s" repeatCount="indefinite" />
                  <animate attributeName="opacity" values="0.6;0.2;0.6" dur="1.5s" repeatCount="indefinite" />
                </circle>
              )}
              {/* Boss special glow */}
              {isBoss && (
                <>
                  <circle cx={pos.px} cy={pos.py} r="22" fill="none" stroke="#ff2222" strokeWidth="1" opacity="0.3">
                    <animate attributeName="r" values="20;26;20" dur="3s" repeatCount="indefinite" />
                    <animate attributeName="opacity" values="0.15;0.4;0.15" dur="3s" repeatCount="indefinite" />
                  </circle>
                  <circle cx={pos.px} cy={pos.py} r="18" fill="#ff0000" opacity="0.05" />
                </>
              )}
              <MapNodeIcon
                type={node.type}
                x={pos.px}
                y={pos.py}
                active={isCurrent || isAvailable}
              />
            </g>
          );
        }),
      )}

      {/* Tooltip */}
      {tooltip && (
        <g transform={`translate(${tooltip.px}, ${tooltip.py - 22})`} style={{ pointerEvents: 'none' }}>
          <rect
            x="-30"
            y="-14"
            width="60"
            height="18"
            rx="4"
            fill="#1a1a2e"
            stroke="#555"
            strokeWidth="1"
            opacity="0.95"
          />
          <text textAnchor="middle" dy="-2" fontSize="9" fill="#e0e0e0">
            {tooltip.label}
          </text>
        </g>
      )}
    </svg>
  );
};
