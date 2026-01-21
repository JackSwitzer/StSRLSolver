/**
 * TreeView Component
 *
 * Main D3.js-based horizontal tree visualization for decision trees.
 * Features:
 * - Horizontal layout (decisions flow left-to-right)
 * - Collapsible branches (click to expand/collapse)
 * - Lazy loading of children on expand
 * - Automatic pruning of low-probability branches
 * - Color coding based on EV (green=good, red=bad, gray=neutral)
 */

import React, { useRef, useEffect, useState, useCallback, useMemo } from 'react';
import * as d3 from 'd3';
import {
  DecisionNode,
  TreeViewProps,
  getEVColor,
  getEVBackgroundColor,
  formatEV,
  formatWinProbability,
  DECISION_TYPE_LABELS,
  DECISION_TYPE_COLORS,
} from './types';
import TreeControls from './TreeControls';

const NODE_WIDTH = 160;
const NODE_HEIGHT = 60;
const HORIZONTAL_SPACING = 200;
const VERTICAL_SPACING = 80;

interface D3HierarchyNode extends d3.HierarchyPointNode<DecisionNode> {
  x0?: number;
  y0?: number;
}

export const TreeView: React.FC<TreeViewProps> = ({
  root,
  onNodeClick,
  onNodeExpand,
  onNodeCollapse,
  width = 1200,
  height = 800,
  pruneThreshold = 0.05,
}) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const gRef = useRef<SVGGElement>(null);
  const [treeData, setTreeData] = useState<DecisionNode>(root);
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [currentPruneThreshold, setCurrentPruneThreshold] = useState(pruneThreshold);

  // Apply pruning to the tree data
  const prunedTreeData = useMemo(() => {
    const pruneNode = (node: DecisionNode): DecisionNode => {
      const prunedChildren = node.children
        .map((child) => {
          if (child.winProbability < currentPruneThreshold) {
            return { ...child, isPruned: true, children: [] };
          }
          return pruneNode(child);
        })
        .filter((child) => !child.isPruned || child.winProbability >= currentPruneThreshold * 0.5);

      return { ...node, children: prunedChildren };
    };
    return pruneNode(treeData);
  }, [treeData, currentPruneThreshold]);

  // Handle node click
  const handleNodeClick = useCallback(
    (node: DecisionNode) => {
      setSelectedNode(node.id);
      onNodeClick?.(node);

      // Toggle expansion
      setTreeData((prev) => {
        const toggleExpansion = (n: DecisionNode): DecisionNode => {
          if (n.id === node.id) {
            const newExpanded = !n.isExpanded;
            if (newExpanded) {
              onNodeExpand?.(n);
            } else {
              onNodeCollapse?.(n);
            }
            return { ...n, isExpanded: newExpanded };
          }
          return { ...n, children: n.children.map(toggleExpansion) };
        };
        return toggleExpansion(prev);
      });
    },
    [onNodeClick, onNodeExpand, onNodeCollapse]
  );

  // Expand all nodes
  const handleExpandAll = useCallback(() => {
    setTreeData((prev) => {
      const expandAll = (n: DecisionNode): DecisionNode => ({
        ...n,
        isExpanded: true,
        children: n.children.map(expandAll),
      });
      return expandAll(prev);
    });
  }, []);

  // Collapse all nodes
  const handleCollapseAll = useCallback(() => {
    setTreeData((prev) => {
      const collapseAll = (n: DecisionNode): DecisionNode => ({
        ...n,
        isExpanded: false,
        children: n.children.map(collapseAll),
      });
      return collapseAll(prev);
    });
  }, []);

  // Reset view
  const handleResetView = useCallback(() => {
    setTreeData(root);
    setSelectedNode(null);
    if (svgRef.current && gRef.current) {
      d3.select(svgRef.current)
        .transition()
        .duration(500)
        .call(
          d3.zoom<SVGSVGElement, unknown>().transform as any,
          d3.zoomIdentity.translate(80, height / 2)
        );
    }
  }, [root, height]);

  // D3 rendering
  useEffect(() => {
    if (!svgRef.current || !gRef.current) return;

    const svg = d3.select(svgRef.current);
    const g = d3.select(gRef.current);

    // Clear previous content
    g.selectAll('*').remove();

    // Create hierarchy from pruned data, only including expanded children
    const getVisibleChildren = (node: DecisionNode): DecisionNode[] | undefined => {
      if (!node.isExpanded || node.children.length === 0) return undefined;
      return node.children;
    };

    const hierarchyRoot = d3.hierarchy(prunedTreeData, getVisibleChildren);

    // Create tree layout (horizontal)
    const treeLayout = d3.tree<DecisionNode>().nodeSize([VERTICAL_SPACING, HORIZONTAL_SPACING]);

    const treeNodes = treeLayout(hierarchyRoot) as D3HierarchyNode;

    // Get all nodes and links
    const nodes = treeNodes.descendants();
    const links = treeNodes.links();

    // Draw links
    const linkGenerator = d3
      .linkHorizontal<d3.HierarchyPointLink<DecisionNode>, d3.HierarchyPointNode<DecisionNode>>()
      .x((d) => d.y)
      .y((d) => d.x);

    g.selectAll('.link')
      .data(links)
      .join('path')
      .attr('class', 'link')
      .attr('d', linkGenerator)
      .attr('fill', 'none')
      .attr('stroke', (d) => {
        const ev = d.target.data.ev;
        return getEVColor(ev);
      })
      .attr('stroke-opacity', 0.4)
      .attr('stroke-width', 2);

    // Draw nodes
    const nodeGroups = g
      .selectAll('.node')
      .data(nodes)
      .join('g')
      .attr('class', 'node')
      .attr('transform', (d) => `translate(${d.y - NODE_WIDTH / 2}, ${d.x - NODE_HEIGHT / 2})`)
      .style('cursor', 'pointer')
      .on('click', (event, d) => {
        event.stopPropagation();
        handleNodeClick(d.data);
      });

    // Node background
    nodeGroups
      .append('rect')
      .attr('width', NODE_WIDTH)
      .attr('height', NODE_HEIGHT)
      .attr('rx', 6)
      .attr('fill', (d) => getEVBackgroundColor(d.data.ev, 0.15))
      .attr('stroke', (d) =>
        d.data.id === selectedNode ? '#d4a857' : DECISION_TYPE_COLORS[d.data.type]
      )
      .attr('stroke-width', (d) => (d.data.id === selectedNode ? 2 : 1.5));

    // Type indicator bar
    nodeGroups
      .append('rect')
      .attr('x', 0)
      .attr('y', 0)
      .attr('width', 4)
      .attr('height', NODE_HEIGHT)
      .attr('rx', 2)
      .attr('fill', (d) => DECISION_TYPE_COLORS[d.data.type]);

    // Type label
    nodeGroups
      .append('text')
      .attr('x', 12)
      .attr('y', 14)
      .attr('font-size', 9)
      .attr('fill', (d) => DECISION_TYPE_COLORS[d.data.type])
      .attr('font-family', 'Cinzel, serif')
      .text((d) => DECISION_TYPE_LABELS[d.data.type]);

    // Action name
    nodeGroups
      .append('text')
      .attr('x', 12)
      .attr('y', 30)
      .attr('font-size', 11)
      .attr('fill', '#e8e4d9')
      .attr('font-family', 'Crimson Text, serif')
      .attr('font-weight', 600)
      .text((d) => {
        const action = d.data.action;
        return action.length > 18 ? action.slice(0, 16) + '...' : action;
      });

    // EV badge background
    nodeGroups
      .append('rect')
      .attr('x', NODE_WIDTH - 50)
      .attr('y', 38)
      .attr('width', 42)
      .attr('height', 18)
      .attr('rx', 3)
      .attr('fill', (d) => getEVBackgroundColor(d.data.ev, 0.3));

    // EV badge text
    nodeGroups
      .append('text')
      .attr('x', NODE_WIDTH - 29)
      .attr('y', 51)
      .attr('text-anchor', 'middle')
      .attr('font-size', 11)
      .attr('fill', (d) => getEVColor(d.data.ev))
      .attr('font-family', 'Cinzel, serif')
      .attr('font-weight', 600)
      .text((d) => formatEV(d.data.ev));

    // Win probability
    nodeGroups
      .append('text')
      .attr('x', 12)
      .attr('y', 52)
      .attr('font-size', 9)
      .attr('fill', '#9a9486')
      .attr('font-family', 'Crimson Text, serif')
      .text((d) => `Win: ${formatWinProbability(d.data.winProbability)}`);

    // Expand/collapse indicator for nodes with children
    const nodesWithChildren = nodeGroups.filter((d) => d.data.children.length > 0);

    nodesWithChildren
      .append('circle')
      .attr('cx', NODE_WIDTH - 8)
      .attr('cy', NODE_HEIGHT / 2)
      .attr('r', 8)
      .attr('fill', '#1a1a25')
      .attr('stroke', '#3a3a4a')
      .attr('stroke-width', 1);

    nodesWithChildren
      .append('text')
      .attr('x', NODE_WIDTH - 8)
      .attr('y', NODE_HEIGHT / 2 + 4)
      .attr('text-anchor', 'middle')
      .attr('font-size', 12)
      .attr('fill', '#d4a857')
      .text((d) => (d.data.isExpanded ? '−' : '+'));

    // Child count for collapsed nodes
    nodesWithChildren
      .filter((d) => !d.data.isExpanded && d.data.children.length > 1)
      .append('text')
      .attr('x', NODE_WIDTH - 8)
      .attr('y', NODE_HEIGHT / 2 + 18)
      .attr('text-anchor', 'middle')
      .attr('font-size', 8)
      .attr('fill', '#6b7280')
      .attr('font-family', 'Cinzel, serif')
      .text((d) => d.data.children.length);

    // Setup zoom and pan
    const zoom = d3
      .zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.2, 3])
      .on('zoom', (event) => {
        g.attr('transform', event.transform);
      });

    svg.call(zoom);

    // Initial transform to center the tree
    svg.call(zoom.transform, d3.zoomIdentity.translate(80, height / 2));
  }, [prunedTreeData, selectedNode, handleNodeClick, height]);

  // Update tree data when root prop changes
  useEffect(() => {
    setTreeData(root);
  }, [root]);

  return (
    <div className="tree-view-container" style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <TreeControls
        onExpandAll={handleExpandAll}
        onCollapseAll={handleCollapseAll}
        onResetView={handleResetView}
        pruneThreshold={currentPruneThreshold}
        onPruneThresholdChange={setCurrentPruneThreshold}
      />
      <div
        style={{
          background: 'linear-gradient(180deg, #0a0a0f 0%, #12121a 100%)',
          border: '1px solid rgba(212, 168, 87, 0.2)',
          borderRadius: '8px',
          overflow: 'hidden',
        }}
      >
        <svg
          ref={svgRef}
          width={width}
          height={height}
          style={{ display: 'block' }}
        >
          <defs>
            {/* Glow filter for selected nodes */}
            <filter id="glow" x="-50%" y="-50%" width="200%" height="200%">
              <feGaussianBlur stdDeviation="4" result="coloredBlur" />
              <feMerge>
                <feMergeNode in="coloredBlur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>
          <g ref={gRef} />
        </svg>
      </div>
    </div>
  );
};

export default TreeView;
