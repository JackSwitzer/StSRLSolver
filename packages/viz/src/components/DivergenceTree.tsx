import { useState } from 'react';
import type { DivergenceNode, DivergenceBranch } from '../types/conquerer';

// ---------------------------------------------------------------------------
// Mock divergence tree for standalone rendering
// ---------------------------------------------------------------------------

const MOCK_TREE: DivergenceNode = {
  floor: 0,
  decision_type: 'path',
  branches: [
    {
      label: 'Left (monster)',
      path_ids: [0, 1, 2, 3, 4],
      children: [
        {
          floor: 6,
          decision_type: 'card',
          branches: [
            { label: 'Take Tantrum', path_ids: [0, 1, 2], children: [] },
            { label: 'Skip', path_ids: [3, 4], children: [] },
          ],
        },
      ],
    },
    {
      label: 'Right (elite)',
      path_ids: [5, 6, 7, 8, 9],
      children: [
        {
          floor: 3,
          decision_type: 'rest',
          branches: [
            { label: 'Rest', path_ids: [5, 6, 7], children: [] },
            { label: 'Upgrade', path_ids: [8, 9], children: [] },
          ],
        },
      ],
    },
  ],
};

// ---------------------------------------------------------------------------
// Tree node component
// ---------------------------------------------------------------------------

interface TreeNodeProps {
  node: DivergenceNode;
  depth: number;
}

const DECISION_COLORS: Record<string, string> = {
  path: '#3366cc',
  card: '#44aa44',
  rest: '#ccaa22',
  event: '#aa44aa',
  shop: '#cc6633',
};

const TreeBranch = ({ branch, depth }: { branch: DivergenceBranch; depth: number }) => {
  const [expanded, setExpanded] = useState(depth < 2);
  const hasChildren = branch.children.length > 0;

  return (
    <div className="div-tree-branch" style={{ marginLeft: depth * 12 }}>
      <div
        className="div-tree-branch-row"
        onClick={() => hasChildren && setExpanded(!expanded)}
        role={hasChildren ? 'button' : undefined}
        tabIndex={hasChildren ? 0 : undefined}
      >
        {hasChildren && (
          <span className="div-tree-toggle">{expanded ? '\u25BC' : '\u25B6'}</span>
        )}
        <span className="div-tree-branch-label">{branch.label}</span>
        <span className="div-tree-path-ids">
          [{branch.path_ids.join(', ')}]
        </span>
      </div>
      {expanded && branch.children.map((child, i) => (
        <TreeNode key={i} node={child} depth={depth + 1} />
      ))}
    </div>
  );
};

const TreeNode = ({ node, depth }: TreeNodeProps) => {
  const color = DECISION_COLORS[node.decision_type] || '#888';

  return (
    <div className="div-tree-node">
      <div className="div-tree-node-header" style={{ marginLeft: depth * 12 }}>
        <span className="div-tree-floor">F{node.floor}</span>
        <span className="div-tree-decision-type" style={{ color }}>
          {node.decision_type}
        </span>
      </div>
      {node.branches.map((branch, i) => (
        <TreeBranch key={i} branch={branch} depth={depth} />
      ))}
    </div>
  );
};

// ---------------------------------------------------------------------------
// Main sidebar component
// ---------------------------------------------------------------------------

interface DivergenceTreeProps {
  tree?: DivergenceNode;
}

export const DivergenceTree = ({ tree }: DivergenceTreeProps) => {
  const [open, setOpen] = useState(false);
  const data = tree || MOCK_TREE;

  return (
    <div className="div-tree-container">
      <button className="div-tree-header-btn" onClick={() => setOpen(!open)}>
        <span>{open ? '\u25BC' : '\u25B6'} Divergence Tree</span>
      </button>
      {open && (
        <div className="div-tree-content">
          <TreeNode node={data} depth={0} />
        </div>
      )}
    </div>
  );
};
