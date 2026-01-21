/**
 * PathTimeline Component
 *
 * Timeline chart showing HP and gold over floors for multiple paths.
 * Uses Recharts for visualization.
 */

import React, { useMemo } from 'react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';
import { Path } from '../DecisionTree/types';

export interface PathTimelineProps {
  paths: Path[];
  metric?: 'hp' | 'gold' | 'both';
  height?: number;
  className?: string;
}

// Path colors for differentiation
const PATH_COLORS = [
  { hp: '#ef4444', gold: '#fbbf24' }, // red/yellow
  { hp: '#22c55e', gold: '#84cc16' }, // green/lime
  { hp: '#3b82f6', gold: '#06b6d4' }, // blue/cyan
  { hp: '#a855f7', gold: '#ec4899' }, // purple/pink
];

// Custom tooltip
const CustomTooltip = ({ active, payload, label }: any) => {
  if (!active || !payload || !payload.length) return null;

  return (
    <div
      style={{
        background: 'linear-gradient(180deg, #1a1a25 0%, #12121a 100%)',
        border: '1px solid rgba(212, 168, 87, 0.3)',
        borderRadius: '6px',
        padding: '0.75rem 1rem',
        boxShadow: '0 4px 20px rgba(0, 0, 0, 0.5)',
      }}
    >
      <div
        style={{
          fontFamily: 'Cinzel, serif',
          fontSize: '0.75rem',
          color: '#d4a857',
          marginBottom: '0.5rem',
          textTransform: 'uppercase',
          letterSpacing: '0.1em',
        }}
      >
        Floor {label}
      </div>
      {payload.map((entry: any, index: number) => (
        <div
          key={index}
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            gap: '1rem',
            fontFamily: 'Crimson Text, serif',
            fontSize: '0.85rem',
            marginBottom: '0.25rem',
          }}
        >
          <span style={{ color: '#9a9486' }}>{entry.name}</span>
          <span style={{ color: entry.color, fontWeight: 600 }}>
            {entry.value !== null ? entry.value : '-'}
          </span>
        </div>
      ))}
    </div>
  );
};

export const PathTimeline: React.FC<PathTimelineProps> = ({
  paths,
  metric = 'both',
  height = 350,
  className = '',
}) => {
  // Transform path data into chart format
  const chartData = useMemo(() => {
    if (paths.length === 0) return [];

    // Find all unique floor numbers
    const allFloors = new Set<number>();
    paths.forEach((path) => {
      path.floors.forEach((floor) => allFloors.add(floor.floor));
    });

    const floorNumbers = Array.from(allFloors).sort((a, b) => a - b);

    // Create data points for each floor
    return floorNumbers.map((floorNum) => {
      const dataPoint: Record<string, number | null> = { floor: floorNum };

      paths.forEach((path, pathIndex) => {
        const floor = path.floors.find((f) => f.floor === floorNum);
        dataPoint[`hp_${pathIndex}`] = floor?.hp ?? null;
        dataPoint[`gold_${pathIndex}`] = floor?.gold ?? null;
      });

      return dataPoint;
    });
  }, [paths]);

  // Calculate domains
  const { maxHP, maxGold } = useMemo(() => {
    let maxHP = 0;
    let maxGold = 0;

    paths.forEach((path) => {
      path.floors.forEach((floor) => {
        if (floor.maxHp > maxHP) maxHP = floor.maxHp;
        if (floor.gold > maxGold) maxGold = floor.gold;
      });
    });

    return {
      maxHP: Math.ceil(maxHP * 1.1),
      maxGold: Math.ceil(maxGold * 1.1),
    };
  }, [paths]);

  if (paths.length === 0) {
    return (
      <div className={`path-timeline ${className}`}>
        <style>{`
          .path-timeline-empty {
            padding: 2rem;
            text-align: center;
            color: #6b7280;
            font-family: 'Crimson Text', serif;
            font-style: italic;
          }
        `}</style>
        <div className="path-timeline-empty">
          Select paths to view timeline comparison
        </div>
      </div>
    );
  }

  return (
    <div className={`path-timeline ${className}`}>
      <style>{`
        .path-timeline {
          background: linear-gradient(180deg, rgba(26, 26, 37, 0.95) 0%, rgba(18, 18, 26, 0.95) 100%);
          border: 1px solid rgba(212, 168, 87, 0.2);
          border-radius: 8px;
          padding: 1rem;
        }

        .path-timeline-title {
          font-family: 'Cinzel', serif;
          font-size: 0.85rem;
          color: #d4a857;
          text-transform: uppercase;
          letter-spacing: 0.1em;
          margin-bottom: 1rem;
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .path-timeline-legend {
          display: flex;
          gap: 1rem;
          flex-wrap: wrap;
        }

        .path-legend-item {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          font-family: 'Crimson Text', serif;
          font-size: 0.8rem;
          color: #9a9486;
        }

        .path-legend-color {
          width: 12px;
          height: 12px;
          border-radius: 2px;
        }

        .recharts-cartesian-grid-horizontal line,
        .recharts-cartesian-grid-vertical line {
          stroke: rgba(255, 255, 255, 0.05);
        }

        .recharts-legend-item-text {
          font-family: 'Crimson Text', serif !important;
          font-size: 0.75rem !important;
          color: #9a9486 !important;
        }
      `}</style>

      <div className="path-timeline-title">
        <span>Resource Timeline</span>
        <div className="path-timeline-legend">
          {paths.map((path, i) => (
            <div key={path.id} className="path-legend-item">
              <div
                className="path-legend-color"
                style={{
                  background: `linear-gradient(135deg, ${PATH_COLORS[i % PATH_COLORS.length].hp}, ${PATH_COLORS[i % PATH_COLORS.length].gold})`,
                }}
              />
              <span>{path.name}</span>
            </div>
          ))}
        </div>
      </div>

      <ResponsiveContainer width="100%" height={height}>
        <LineChart data={chartData} margin={{ top: 10, right: 30, left: 0, bottom: 0 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="rgba(255, 255, 255, 0.05)" />

          <XAxis
            dataKey="floor"
            stroke="#3a3a4a"
            tick={{ fill: '#6b7280', fontFamily: 'Crimson Text, serif', fontSize: 12 }}
            axisLine={{ stroke: '#3a3a4a' }}
            label={{
              value: 'Floor',
              position: 'insideBottom',
              offset: -5,
              fill: '#9a9486',
              fontFamily: 'Cinzel, serif',
              fontSize: 10,
            }}
          />

          {/* HP Y-axis (left) */}
          {(metric === 'hp' || metric === 'both') && (
            <YAxis
              yAxisId="hp"
              orientation="left"
              stroke="#3a3a4a"
              domain={[0, maxHP]}
              tick={{ fill: '#ef4444', fontFamily: 'Crimson Text, serif', fontSize: 11 }}
              axisLine={{ stroke: '#3a3a4a' }}
              label={{
                value: 'HP',
                angle: -90,
                position: 'insideLeft',
                fill: '#ef4444',
                fontFamily: 'Cinzel, serif',
                fontSize: 10,
              }}
            />
          )}

          {/* Gold Y-axis (right) */}
          {(metric === 'gold' || metric === 'both') && (
            <YAxis
              yAxisId="gold"
              orientation="right"
              stroke="#3a3a4a"
              domain={[0, maxGold]}
              tick={{ fill: '#eab308', fontFamily: 'Crimson Text, serif', fontSize: 11 }}
              axisLine={{ stroke: '#3a3a4a' }}
              label={{
                value: 'Gold',
                angle: 90,
                position: 'insideRight',
                fill: '#eab308',
                fontFamily: 'Cinzel, serif',
                fontSize: 10,
              }}
            />
          )}

          <Tooltip content={<CustomTooltip />} />

          {/* HP Lines */}
          {(metric === 'hp' || metric === 'both') &&
            paths.map((path, i) => (
              <Line
                key={`hp_${path.id}`}
                yAxisId="hp"
                type="monotone"
                dataKey={`hp_${i}`}
                name={`${path.name} HP`}
                stroke={PATH_COLORS[i % PATH_COLORS.length].hp}
                strokeWidth={2}
                dot={{ fill: PATH_COLORS[i % PATH_COLORS.length].hp, strokeWidth: 0, r: 3 }}
                activeDot={{ fill: PATH_COLORS[i % PATH_COLORS.length].hp, strokeWidth: 2, stroke: '#fff', r: 5 }}
                connectNulls
              />
            ))}

          {/* Gold Lines */}
          {(metric === 'gold' || metric === 'both') &&
            paths.map((path, i) => (
              <Line
                key={`gold_${path.id}`}
                yAxisId="gold"
                type="monotone"
                dataKey={`gold_${i}`}
                name={`${path.name} Gold`}
                stroke={PATH_COLORS[i % PATH_COLORS.length].gold}
                strokeWidth={2}
                strokeDasharray="5 5"
                dot={{ fill: PATH_COLORS[i % PATH_COLORS.length].gold, strokeWidth: 0, r: 3 }}
                activeDot={{ fill: PATH_COLORS[i % PATH_COLORS.length].gold, strokeWidth: 2, stroke: '#fff', r: 5 }}
                connectNulls
              />
            ))}
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
};

export default PathTimeline;
