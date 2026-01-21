/**
 * Interactive map canvas component
 * Renders the dungeon map with clickable nodes for path building
 */

import { useRef, useEffect, useCallback } from 'react';
import { useGameStore } from '../../store/gameStore';
import { ROOM_COLORS, ROOM_SYMBOLS, NodePosition, RenderableNode } from './types';
import type { MapEdge, RoomType } from '../../api/seedApi';
import './MapCanvas.css';

const NODE_RADIUS = 18;
const BOSS_NODE_RADIUS = NODE_RADIUS * 1.3;
const PADDING = 40;
const BOSS_Y = 16; // Boss is rendered at y=16 in the visual

export function MapCanvas() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const nodesRef = useRef<RenderableNode[]>([]);

  const {
    mapData,
    selectedPath,
    addToPath,
    removeFromPath,
    canAddToPath,
    isNodeOnPath,
    getPathIndex,
  } = useGameStore();

  // Calculate node position on canvas
  const getNodePos = useCallback(
    (
      nodeX: number,
      nodeY: number,
      canvasWidth: number,
      canvasHeight: number,
      mapWidth: number,
      mapHeight: number
    ): NodePosition => {
      const cellWidth = (canvasWidth - PADDING * 2) / mapWidth;
      const cellHeight = (canvasHeight - PADDING * 2) / (mapHeight + 2);

      return {
        x: PADDING + nodeX * cellWidth + cellWidth / 2,
        y: canvasHeight - PADDING - nodeY * cellHeight - cellHeight / 2,
      };
    },
    []
  );

  // Check if edge is on selected path
  const isEdgeOnPath = useCallback(
    (edge: MapEdge): boolean => {
      for (let i = 0; i < selectedPath.length - 1; i++) {
        if (
          selectedPath[i].x === edge.src_x &&
          selectedPath[i].y === edge.src_y &&
          selectedPath[i + 1].x === edge.dst_x &&
          selectedPath[i + 1].y === edge.dst_y
        ) {
          return true;
        }
      }

      // Check boss edge
      if (selectedPath.length > 0) {
        const last = selectedPath[selectedPath.length - 1];
        if (edge.is_boss && edge.src_x === last.x && edge.src_y === last.y) {
          return true;
        }
      }

      return false;
    },
    [selectedPath]
  );

  // Draw the map
  const drawMap = useCallback(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container || !mapData) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Set canvas size based on container
    const rect = container.getBoundingClientRect();
    const dpr = window.devicePixelRatio;
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    canvas.style.width = `${rect.width}px`;
    canvas.style.height = `${rect.height}px`;
    ctx.scale(dpr, dpr);

    const width = rect.width;
    const height = rect.height;

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    const { nodes, edges } = mapData;
    nodesRef.current = [];

    // Draw non-path edges first
    edges.forEach((edge) => {
      if (isEdgeOnPath(edge)) return;

      const src = getNodePos(edge.src_x, edge.src_y, width, height, mapData.width, mapData.height);
      const dst = edge.is_boss
        ? getNodePos(3, BOSS_Y, width, height, mapData.width, mapData.height)
        : getNodePos(edge.dst_x, edge.dst_y, width, height, mapData.width, mapData.height);

      ctx.strokeStyle = 'rgba(212, 168, 87, 0.2)';
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(src.x, src.y);
      ctx.lineTo(dst.x, dst.y);
      ctx.stroke();
    });

    // Draw path edges (highlighted)
    edges.forEach((edge) => {
      if (!isEdgeOnPath(edge)) return;

      const src = getNodePos(edge.src_x, edge.src_y, width, height, mapData.width, mapData.height);
      const dst = edge.is_boss
        ? getNodePos(3, BOSS_Y, width, height, mapData.width, mapData.height)
        : getNodePos(edge.dst_x, edge.dst_y, width, height, mapData.width, mapData.height);

      // Glow effect
      ctx.strokeStyle = 'rgba(212, 168, 87, 0.6)';
      ctx.lineWidth = 6;
      ctx.lineCap = 'round';
      ctx.beginPath();
      ctx.moveTo(src.x, src.y);
      ctx.lineTo(dst.x, dst.y);
      ctx.stroke();

      // Main line
      ctx.strokeStyle = '#d4a857';
      ctx.lineWidth = 3;
      ctx.beginPath();
      ctx.moveTo(src.x, src.y);
      ctx.lineTo(dst.x, dst.y);
      ctx.stroke();
    });

    // Draw nodes
    nodes.forEach((node) => {
      if (!node.type && !node.has_edges) return;

      const pos = getNodePos(node.x, node.y, width, height, mapData.width, mapData.height);
      const color = node.type ? ROOM_COLORS[node.type as RoomType] : '#3a3a4a';
      const symbol = node.type ? ROOM_SYMBOLS[node.type as RoomType] : '';
      const onPath = isNodeOnPath(node.x, node.y);

      // Store for click detection
      nodesRef.current.push({
        x: node.x,
        y: node.y,
        type: node.type as RoomType | null,
        symbol,
        hasEdges: node.has_edges,
        position: pos,
        radius: NODE_RADIUS,
      });

      // Glow effect
      const gradient = ctx.createRadialGradient(pos.x, pos.y, 0, pos.x, pos.y, NODE_RADIUS * 1.5);
      gradient.addColorStop(0, onPath ? color + '80' : color + '40');
      gradient.addColorStop(1, 'transparent');
      ctx.fillStyle = gradient;
      ctx.beginPath();
      ctx.arc(pos.x, pos.y, NODE_RADIUS * 1.5, 0, Math.PI * 2);
      ctx.fill();

      // Node circle
      ctx.fillStyle = onPath ? color + '30' : '#1a1a25';
      ctx.strokeStyle = color;
      ctx.lineWidth = onPath ? 3 : 2;
      ctx.beginPath();
      ctx.arc(pos.x, pos.y, NODE_RADIUS, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();

      // Path order number
      if (onPath) {
        const pathIdx = getPathIndex(node.x, node.y);
        ctx.fillStyle = '#fff';
        ctx.font = 'bold 10px Cinzel, serif';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText((pathIdx + 1).toString(), pos.x, pos.y - NODE_RADIUS - 8);
      }

      // Symbol
      ctx.fillStyle = color;
      ctx.font = '600 14px Cinzel, serif';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillText(symbol, pos.x, pos.y + 1);
    });

    // Draw boss node
    const bossPos = getNodePos(3, BOSS_Y, width, height, mapData.width, mapData.height);
    const bossColor = ROOM_COLORS.BOSS;
    const bossOnPath = selectedPath.length > 0 && selectedPath[selectedPath.length - 1].y === 14;

    ctx.fillStyle = bossOnPath ? bossColor + '30' : '#1a1a25';
    ctx.strokeStyle = bossColor;
    ctx.lineWidth = bossOnPath ? 4 : 3;
    ctx.beginPath();
    ctx.arc(bossPos.x, bossPos.y, BOSS_NODE_RADIUS, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();

    ctx.fillStyle = bossColor;
    ctx.font = '700 16px Cinzel, serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText('B', bossPos.x, bossPos.y + 1);
  }, [mapData, selectedPath, isEdgeOnPath, getNodePos, isNodeOnPath, getPathIndex]);

  // Handle canvas click
  const handleClick = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      const canvas = canvasRef.current;
      if (!canvas || !mapData) return;

      const rect = canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      // Find clicked node
      for (const node of nodesRef.current) {
        const dx = x - node.position.x;
        const dy = y - node.position.y;
        const dist = Math.sqrt(dx * dx + dy * dy);

        if (dist <= node.radius + 5) {
          const pathIdx = getPathIndex(node.x, node.y);

          if (pathIdx >= 0) {
            // Click on existing path node - remove from that point
            removeFromPath(pathIdx);
          } else if (canAddToPath({ x: node.x, y: node.y }) && node.type) {
            // Add to path
            addToPath({ x: node.x, y: node.y, type: node.type });
          }
          return;
        }
      }
    },
    [mapData, getPathIndex, removeFromPath, canAddToPath, addToPath]
  );

  // Redraw on data change
  useEffect(() => {
    drawMap();
  }, [drawMap]);

  // Resize handler
  useEffect(() => {
    const handleResize = () => {
      drawMap();
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, [drawMap]);

  return (
    <div className="map-container" ref={containerRef}>
      <canvas ref={canvasRef} className="map-canvas" onClick={handleClick} />
    </div>
  );
}
