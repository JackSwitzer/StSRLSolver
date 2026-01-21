/**
 * TreeNode Component
 *
 * Renders an individual decision node in the tree visualization.
 * Shows the action, EV delta, and win probability with appropriate color coding.
 */

import React from 'react';
import {
  TreeNodeProps,
  getEVColor,
  getEVBackgroundColor,
  formatEV,
  formatWinProbability,
  DECISION_TYPE_LABELS,
  DECISION_TYPE_COLORS,
} from './types';

const NODE_WIDTH = 160;
const NODE_HEIGHT = 60;

export const TreeNode: React.FC<TreeNodeProps> = ({
  node,
  x,
  y,
  onNodeClick,
  isSelected = false,
}) => {
  const evColor = getEVColor(node.ev);
  const bgColor = getEVBackgroundColor(node.ev, 0.15);
  const typeColor = DECISION_TYPE_COLORS[node.type];
  const typeLabel = DECISION_TYPE_LABELS[node.type];

  const handleClick = () => {
    onNodeClick?.(node);
  };

  // If node is pruned, render a minimal version
  if (node.isPruned) {
    return (
      <g
        transform={`translate(${x - 40}, ${y - 15})`}
        className="tree-node pruned"
        style={{ opacity: 0.5 }}
      >
        <rect
          width={80}
          height={30}
          rx={4}
          fill="#1a1a25"
          stroke="#3a3a4a"
          strokeWidth={1}
          strokeDasharray="4,2"
        />
        <text
          x={40}
          y={18}
          textAnchor="middle"
          fill="#6b7280"
          fontSize={10}
          fontFamily="Cinzel, serif"
        >
          Pruned ({formatWinProbability(node.winProbability)})
        </text>
      </g>
    );
  }

  return (
    <g
      transform={`translate(${x - NODE_WIDTH / 2}, ${y - NODE_HEIGHT / 2})`}
      className="tree-node"
      onClick={handleClick}
      style={{ cursor: 'pointer' }}
    >
      {/* Selection highlight */}
      {isSelected && (
        <rect
          x={-4}
          y={-4}
          width={NODE_WIDTH + 8}
          height={NODE_HEIGHT + 8}
          rx={10}
          fill="none"
          stroke="#d4a857"
          strokeWidth={2}
          className="selection-ring"
        />
      )}

      {/* Main node background */}
      <rect
        width={NODE_WIDTH}
        height={NODE_HEIGHT}
        rx={6}
        fill={bgColor}
        stroke={isSelected ? '#d4a857' : typeColor}
        strokeWidth={isSelected ? 2 : 1.5}
        className="node-bg"
      />

      {/* Type indicator bar */}
      <rect x={0} y={0} width={4} height={NODE_HEIGHT} rx={2} fill={typeColor} />

      {/* Type label */}
      <text
        x={12}
        y={14}
        fontSize={9}
        fill={typeColor}
        fontFamily="Cinzel, serif"
        style={{ textTransform: 'uppercase', letterSpacing: '0.5px' }}
      >
        {typeLabel}
      </text>

      {/* Action name */}
      <text
        x={12}
        y={30}
        fontSize={11}
        fill="#e8e4d9"
        fontFamily="Crimson Text, serif"
        fontWeight={600}
      >
        {node.action.length > 18 ? node.action.slice(0, 16) + '...' : node.action}
      </text>

      {/* EV badge */}
      <g transform={`translate(${NODE_WIDTH - 50}, 38)`}>
        <rect width={42} height={18} rx={3} fill={getEVBackgroundColor(node.ev, 0.3)} />
        <text
          x={21}
          y={13}
          textAnchor="middle"
          fontSize={11}
          fill={evColor}
          fontFamily="Cinzel, serif"
          fontWeight={600}
        >
          {formatEV(node.ev)}
        </text>
      </g>

      {/* Win probability */}
      <text
        x={12}
        y={52}
        fontSize={9}
        fill="#9a9486"
        fontFamily="Crimson Text, serif"
      >
        Win: {formatWinProbability(node.winProbability)}
      </text>

      {/* Expand/collapse indicator */}
      {node.children.length > 0 && (
        <g transform={`translate(${NODE_WIDTH - 16}, ${NODE_HEIGHT / 2})`}>
          <circle r={8} fill="#1a1a25" stroke="#3a3a4a" strokeWidth={1} />
          <text
            x={0}
            y={4}
            textAnchor="middle"
            fontSize={12}
            fill="#d4a857"
            fontFamily="sans-serif"
          >
            {node.isExpanded ? '−' : '+'}
          </text>
          {!node.isExpanded && node.children.length > 1 && (
            <text
              x={0}
              y={18}
              textAnchor="middle"
              fontSize={8}
              fill="#6b7280"
              fontFamily="Cinzel, serif"
            >
              {node.children.length}
            </text>
          )}
        </g>
      )}

      {/* Hover tooltip trigger area - tooltip would be handled by parent */}
      <title>
        {`${typeLabel}: ${node.action}
EV: ${formatEV(node.ev)}
Win Probability: ${formatWinProbability(node.winProbability)}
${node.metadata?.floor ? `Floor: ${node.metadata.floor}` : ''}
${node.metadata?.hp ? `HP: ${node.metadata.hp}` : ''}
Children: ${node.children.length}${node.isPruned ? ' (pruned)' : ''}`}
      </title>
    </g>
  );
};

// Memoize to prevent unnecessary re-renders
export default React.memo(TreeNode);
