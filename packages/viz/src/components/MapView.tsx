import type { MapState } from '../types/game';
import { MapNodeIcon } from '../sprites';

interface MapViewProps {
  map: MapState;
}

const NODE_SPACING_X = 70;
const NODE_SPACING_Y = 48;
const MARGIN_X = 60;
const MARGIN_TOP = 40;

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
  const { nodes, edges, current_node, available_next, boss_name } = map;

  const maxFloor = nodes.length;
  const maxPathWidth = Math.max(...nodes.map((floor) => floor.length), 1);

  const svgWidth = maxPathWidth * NODE_SPACING_X + MARGIN_X * 2;
  const svgHeight = (maxFloor + 1) * NODE_SPACING_Y + MARGIN_TOP * 2;

  function nodePos(x: number, y: number): { px: number; py: number } {
    const floorNodes = nodes[y] || [];
    const floorWidth = floorNodes.length * NODE_SPACING_X;
    const offsetX = (svgWidth - floorWidth) / 2 + NODE_SPACING_X / 2;
    return {
      px: offsetX + x * NODE_SPACING_X,
      py: svgHeight - MARGIN_TOP - y * NODE_SPACING_Y,
    };
  }

  // Build a lookup for node positions by their original x,y coordinates
  const posLookup = new Map<string, { px: number; py: number }>();
  for (const floorNodes of nodes) {
    for (let i = 0; i < floorNodes.length; i++) {
      const node = floorNodes[i];
      const pos = nodePos(i, node.y);
      posLookup.set(`${node.x},${node.y}`, pos);
    }
  }

  return (
    <svg
      viewBox={`0 0 ${svgWidth} ${svgHeight}`}
      width="100%"
      height="100%"
      style={{ maxHeight: '80vh' }}
    >
      {/* Title */}
      <text x={svgWidth / 2} y="20" textAnchor="middle" fill="#e0e0e0" fontSize="14" fontWeight="bold">
        Act Map {boss_name ? `- Boss: ${boss_name}` : ''}
      </text>

      {/* Edges */}
      {edges.map((edge, i) => {
        const fromKey = `${edge.from.x},${edge.from.y}`;
        const toKey = `${edge.to.x},${edge.to.y}`;
        const from = posLookup.get(fromKey);
        const to = posLookup.get(toKey);
        if (!from || !to) return null;

        const isNextEdge =
          isNodeCurrent({ x: edge.from.x, y: edge.from.y }, current_node) &&
          isNodeAvailable({ x: edge.to.x, y: edge.to.y }, available_next);

        return (
          <line
            key={`edge-${i}`}
            x1={from.px}
            y1={from.py}
            x2={to.px}
            y2={to.py}
            stroke={isNextEdge ? '#e94560' : '#333'}
            strokeWidth={isNextEdge ? 2 : 1}
            opacity={isNextEdge ? 0.9 : 0.4}
          />
        );
      })}

      {/* Nodes */}
      {nodes.flatMap((floorNodes) =>
        floorNodes.map((node) => {
          const pos = posLookup.get(`${node.x},${node.y}`);
          if (!pos) return null;

          const isCurrent = isNodeCurrent(node, current_node);
          const isAvailable = isNodeAvailable(node, available_next);

          return (
            <g key={`node-${node.x}-${node.y}`}>
              {/* Current node highlight */}
              {isCurrent && (
                <circle cx={pos.px} cy={pos.py} r="18" fill="none" stroke="#ffd700" strokeWidth="2" opacity="0.7">
                  <animate attributeName="opacity" values="0.4;0.9;0.4" dur="2s" repeatCount="indefinite" />
                </circle>
              )}
              {/* Available pulse */}
              {isAvailable && !isCurrent && (
                <circle cx={pos.px} cy={pos.py} r="16" fill="none" stroke="#e94560" strokeWidth="1.5" opacity="0.6">
                  <animate attributeName="r" values="14;18;14" dur="1.5s" repeatCount="indefinite" />
                  <animate attributeName="opacity" values="0.6;0.2;0.6" dur="1.5s" repeatCount="indefinite" />
                </circle>
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
    </svg>
  );
};
