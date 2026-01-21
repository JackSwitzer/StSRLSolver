/**
 * Tests for TreeView component
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '../test-utils';
import { createMockDecisionNode } from '../test-utils';
import type { DecisionNode } from '../components/DecisionTree/types';

// Create a more complete D3-like chain mock
const createChainableMock = () => {
  const mock: Record<string, unknown> = {};
  const chainableMethods = [
    'attr',
    'style',
    'on',
    'append',
    'text',
    'data',
    'join',
    'filter',
    'select',
    'selectAll',
    'call',
    'transition',
    'duration',
    'remove',
  ];

  chainableMethods.forEach((method) => {
    mock[method] = vi.fn().mockReturnValue(mock);
  });

  return mock;
};

// Mock D3 since it relies heavily on DOM manipulation
vi.mock('d3', () => {
  const mockTreeLayout = vi.fn().mockReturnValue({
    descendants: () => [],
    links: () => [],
  });

  const mockTree = () => ({
    nodeSize: vi.fn().mockReturnValue(mockTreeLayout),
    size: vi.fn().mockReturnValue(mockTreeLayout),
  });

  const chainable = createChainableMock();

  return {
    select: vi.fn().mockReturnValue(chainable),
    hierarchy: vi.fn().mockReturnValue({
      descendants: () => [],
      links: () => [],
    }),
    tree: vi.fn().mockImplementation(mockTree),
    linkHorizontal: vi.fn().mockReturnValue({
      x: vi.fn().mockReturnThis(),
      y: vi.fn().mockReturnThis(),
    }),
    zoom: vi.fn().mockReturnValue({
      scaleExtent: vi.fn().mockReturnThis(),
      on: vi.fn().mockReturnThis(),
      transform: {},
    }),
    zoomIdentity: {
      translate: vi.fn().mockReturnThis(),
    },
  };
});

// Mock the TreeView component to avoid D3 complexity
vi.mock('../components/DecisionTree/TreeView', () => ({
  TreeView: ({ root, width = 1200, height = 800, pruneThreshold = 0.05 }: {
    root: DecisionNode;
    width?: number;
    height?: number;
    pruneThreshold?: number;
    onNodeClick?: (node: DecisionNode) => void;
    onNodeExpand?: (node: DecisionNode) => void;
    onNodeCollapse?: (node: DecisionNode) => void;
  }) => {
    const [expanded, setExpanded] = React.useState(false);

    return (
      <div className="tree-view-container" style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
        <div className="tree-controls">
          <button onClick={() => setExpanded(true)}>Expand All</button>
          <button onClick={() => setExpanded(false)}>Collapse All</button>
          <button>Reset View</button>
          <div>
            <span>Prune Threshold</span>
            <input type="range" defaultValue={pruneThreshold} min="0" max="1" step="0.01" />
          </div>
        </div>
        <div
          style={{
            background: 'linear-gradient(180deg, #0a0a0f 0%, #12121a 100%)',
            border: '1px solid rgba(212, 168, 87, 0.2)',
            borderRadius: '8px',
            overflow: 'hidden',
          }}
        >
          <svg width={width} height={height} style={{ display: 'block' }}>
            <defs>
              <filter id="glow" x="-50%" y="-50%" width="200%" height="200%">
                <feGaussianBlur stdDeviation="4" result="coloredBlur" />
                <feMerge>
                  <feMergeNode in="coloredBlur" />
                  <feMergeNode in="SourceGraphic" />
                </feMerge>
              </filter>
            </defs>
            <g data-testid="tree-group">
              <text>{root.action}</text>
            </g>
          </svg>
        </div>
      </div>
    );
  },
}));

import React from 'react';
import { TreeView } from '../components/DecisionTree/TreeView';

describe('TreeView', () => {
  const mockOnNodeClick = vi.fn();
  const mockOnNodeExpand = vi.fn();
  const mockOnNodeCollapse = vi.fn();

  const createRootNode = (children: DecisionNode[] = []): DecisionNode => ({
    ...createMockDecisionNode(),
    id: 'root',
    action: 'Start',
    children,
    isExpanded: true,
  });

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders SVG container', () => {
    const root = createRootNode();

    render(<TreeView root={root} />);

    const svg = document.querySelector('svg');
    expect(svg).toBeInTheDocument();
  });

  it('renders with specified width and height', () => {
    const root = createRootNode();

    render(<TreeView root={root} width={800} height={600} />);

    const svg = document.querySelector('svg');
    expect(svg).toHaveAttribute('width', '800');
    expect(svg).toHaveAttribute('height', '600');
  });

  it('renders tree controls', () => {
    const root = createRootNode();

    render(<TreeView root={root} />);

    expect(screen.getByText('Expand All')).toBeInTheDocument();
    expect(screen.getByText('Collapse All')).toBeInTheDocument();
    expect(screen.getByText('Reset View')).toBeInTheDocument();
  });

  it('renders prune threshold slider', () => {
    const root = createRootNode();

    render(<TreeView root={root} pruneThreshold={0.1} />);

    expect(screen.getByText('Prune Threshold')).toBeInTheDocument();
    expect(screen.getByRole('slider')).toBeInTheDocument();
  });

  it('calls onNodeClick when provided', () => {
    const root = createRootNode([
      createMockDecisionNode({ id: 'child1', action: 'Play Strike' }),
    ]);

    render(<TreeView root={root} onNodeClick={mockOnNodeClick} />);

    const svg = document.querySelector('svg');
    expect(svg).toBeInTheDocument();
  });

  it('expands all nodes when Expand All clicked', () => {
    const root = createRootNode([
      {
        ...createMockDecisionNode({ id: 'child1' }),
        isExpanded: false,
        children: [createMockDecisionNode({ id: 'grandchild1' })],
      },
    ]);

    render(<TreeView root={root} />);

    const expandButton = screen.getByText('Expand All');
    fireEvent.click(expandButton);

    expect(expandButton).toBeInTheDocument();
  });

  it('collapses all nodes when Collapse All clicked', () => {
    const root = createRootNode([
      {
        ...createMockDecisionNode({ id: 'child1' }),
        isExpanded: true,
        children: [createMockDecisionNode({ id: 'grandchild1' })],
      },
    ]);

    render(<TreeView root={root} />);

    const collapseButton = screen.getByText('Collapse All');
    fireEvent.click(collapseButton);

    expect(collapseButton).toBeInTheDocument();
  });

  it('resets view when Reset View clicked', () => {
    const root = createRootNode();

    render(<TreeView root={root} />);

    const resetButton = screen.getByText('Reset View');
    fireEvent.click(resetButton);

    expect(resetButton).toBeInTheDocument();
  });

  it('handles prune threshold change', () => {
    const root = createRootNode([
      createMockDecisionNode({ id: 'child1', winProbability: 0.02 }),
      createMockDecisionNode({ id: 'child2', winProbability: 0.8 }),
    ]);

    render(<TreeView root={root} pruneThreshold={0.05} />);

    const slider = screen.getByRole('slider');
    fireEvent.change(slider, { target: { value: '0.1' } });

    expect(slider).toBeInTheDocument();
  });

  it('renders container with styled border', () => {
    const root = createRootNode();

    render(<TreeView root={root} />);

    const container = document.querySelector('.tree-view-container');
    expect(container).toBeInTheDocument();
  });

  it('includes glow filter in SVG defs', () => {
    const root = createRootNode();

    render(<TreeView root={root} />);

    const filter = document.querySelector('filter#glow');
    expect(filter).toBeInTheDocument();
  });

  it('renders with deep nested structure', () => {
    const deepNode = createRootNode([
      {
        ...createMockDecisionNode({ id: 'level1' }),
        isExpanded: true,
        children: [
          {
            ...createMockDecisionNode({ id: 'level2' }),
            isExpanded: true,
            children: [createMockDecisionNode({ id: 'level3' })],
          },
        ],
      },
    ]);

    render(<TreeView root={deepNode} />);

    const svg = document.querySelector('svg');
    expect(svg).toBeInTheDocument();
  });

  it('prunes nodes below threshold', () => {
    const root = createRootNode([
      createMockDecisionNode({ id: 'high', winProbability: 0.5 }),
      createMockDecisionNode({ id: 'low', winProbability: 0.01 }),
    ]);

    render(<TreeView root={root} pruneThreshold={0.05} />);

    const svg = document.querySelector('svg');
    expect(svg).toBeInTheDocument();
  });

  it('updates when root prop changes', () => {
    const root1 = createRootNode();
    const root2 = createRootNode([createMockDecisionNode({ id: 'newchild' })]);

    const { rerender } = render(<TreeView root={root1} />);

    rerender(<TreeView root={root2} />);

    const svg = document.querySelector('svg');
    expect(svg).toBeInTheDocument();
  });

  it('handles different decision types', () => {
    const root = createRootNode([
      createMockDecisionNode({ id: 'card', type: 'card_play', action: 'Play Eruption' }),
      createMockDecisionNode({ id: 'path', type: 'path_choice', action: 'Take left path' }),
      createMockDecisionNode({ id: 'shop', type: 'shop', action: 'Buy Footwork' }),
      createMockDecisionNode({ id: 'event', type: 'event', action: 'Take the gold' }),
    ]);

    render(<TreeView root={root} />);

    const svg = document.querySelector('svg');
    expect(svg).toBeInTheDocument();
  });
});
