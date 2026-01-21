/**
 * ResourceProjection Component
 *
 * Displays a chart showing HP and gold projections over floors.
 * Uses Recharts for the visualization.
 */

import React, { useMemo } from 'react';
import {
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  Area,
  ComposedChart,
  ReferenceLine,
} from 'recharts';

export interface FloorResource {
  floor: number;
  hp: number;
  maxHp: number;
  gold: number;
  ev?: number;
}

export interface ResourceProjectionProps {
  data: FloorResource[];
  showHP?: boolean;
  showGold?: boolean;
  showEV?: boolean;
  height?: number;
  currentFloor?: number;
  className?: string;
}

// Custom tooltip component
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
            fontSize: '0.9rem',
            marginBottom: '0.25rem',
          }}
        >
          <span style={{ color: '#9a9486' }}>{entry.name}</span>
          <span style={{ color: entry.color, fontWeight: 600 }}>
            {entry.name === 'EV' ? (entry.value >= 0 ? '+' : '') + entry.value.toFixed(2) : entry.value}
          </span>
        </div>
      ))}
    </div>
  );
};

export const ResourceProjection: React.FC<ResourceProjectionProps> = ({
  data,
  showHP = true,
  showGold = true,
  showEV = false,
  height = 300,
  currentFloor,
  className = '',
}) => {
  // Calculate chart domain
  const { maxHP, maxGold } = useMemo(() => {
    let maxHP = 0;
    let maxGold = 0;

    data.forEach((d) => {
      if (d.maxHp > maxHP) maxHP = d.maxHp;
      if (d.gold > maxGold) maxGold = d.gold;
    });

    return {
      maxHP: Math.ceil(maxHP * 1.1),
      maxGold: Math.ceil(maxGold * 1.1),
    };
  }, [data]);

  return (
    <div className={`resource-projection ${className}`}>
      <style>{`
        .resource-projection {
          background: linear-gradient(180deg, rgba(26, 26, 37, 0.5) 0%, rgba(18, 18, 26, 0.5) 100%);
          border: 1px solid rgba(255, 255, 255, 0.08);
          border-radius: 8px;
          padding: 1rem;
        }

        .resource-projection-title {
          font-family: 'Cinzel', serif;
          font-size: 0.85rem;
          color: #d4a857;
          text-transform: uppercase;
          letter-spacing: 0.1em;
          margin-bottom: 1rem;
        }

        .recharts-cartesian-grid-horizontal line,
        .recharts-cartesian-grid-vertical line {
          stroke: rgba(255, 255, 255, 0.05);
        }

        .recharts-legend-item-text {
          font-family: 'Cinzel', serif !important;
          font-size: 0.75rem !important;
          color: #9a9486 !important;
        }

        .recharts-xaxis .recharts-cartesian-axis-tick-value,
        .recharts-yaxis .recharts-cartesian-axis-tick-value {
          font-family: 'Crimson Text', serif;
          font-size: 0.75rem;
          fill: #6b7280;
        }
      `}</style>

      <div className="resource-projection-title">Resource Projection</div>

      <ResponsiveContainer width="100%" height={height}>
        <ComposedChart
          data={data}
          margin={{ top: 10, right: 30, left: 0, bottom: 0 }}
        >
          <defs>
            {/* HP gradient */}
            <linearGradient id="hpGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="#ef4444" stopOpacity={0.3} />
              <stop offset="95%" stopColor="#ef4444" stopOpacity={0} />
            </linearGradient>
            {/* Gold gradient */}
            <linearGradient id="goldGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="#eab308" stopOpacity={0.3} />
              <stop offset="95%" stopColor="#eab308" stopOpacity={0} />
            </linearGradient>
          </defs>

          <CartesianGrid strokeDasharray="3 3" stroke="rgba(255, 255, 255, 0.05)" />

          <XAxis
            dataKey="floor"
            stroke="#3a3a4a"
            tick={{ fill: '#6b7280' }}
            axisLine={{ stroke: '#3a3a4a' }}
          />

          {/* Left Y-axis for HP */}
          {showHP && (
            <YAxis
              yAxisId="hp"
              orientation="left"
              stroke="#3a3a4a"
              domain={[0, maxHP]}
              tick={{ fill: '#ef4444' }}
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

          {/* Right Y-axis for Gold */}
          {showGold && (
            <YAxis
              yAxisId="gold"
              orientation="right"
              stroke="#3a3a4a"
              domain={[0, maxGold]}
              tick={{ fill: '#eab308' }}
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

          <Legend
            wrapperStyle={{
              fontFamily: 'Cinzel, serif',
              fontSize: '0.75rem',
            }}
          />

          {/* Current floor indicator */}
          {currentFloor !== undefined && (
            <ReferenceLine
              x={currentFloor}
              stroke="#d4a857"
              strokeDasharray="3 3"
              strokeWidth={2}
              yAxisId="hp"
            />
          )}

          {/* HP Area + Line */}
          {showHP && (
            <>
              <Area
                yAxisId="hp"
                type="monotone"
                dataKey="hp"
                stroke="transparent"
                fill="url(#hpGradient)"
                name="HP"
              />
              <Line
                yAxisId="hp"
                type="monotone"
                dataKey="hp"
                stroke="#ef4444"
                strokeWidth={2}
                dot={{ fill: '#ef4444', strokeWidth: 0, r: 3 }}
                activeDot={{ fill: '#ef4444', strokeWidth: 2, stroke: '#fff', r: 5 }}
                name="HP"
              />
              {/* Max HP line */}
              <Line
                yAxisId="hp"
                type="monotone"
                dataKey="maxHp"
                stroke="#ef4444"
                strokeWidth={1}
                strokeDasharray="5 5"
                dot={false}
                name="Max HP"
              />
            </>
          )}

          {/* Gold Area + Line */}
          {showGold && (
            <>
              <Area
                yAxisId="gold"
                type="monotone"
                dataKey="gold"
                stroke="transparent"
                fill="url(#goldGradient)"
                name="Gold"
              />
              <Line
                yAxisId="gold"
                type="monotone"
                dataKey="gold"
                stroke="#eab308"
                strokeWidth={2}
                dot={{ fill: '#eab308', strokeWidth: 0, r: 3 }}
                activeDot={{ fill: '#eab308', strokeWidth: 2, stroke: '#fff', r: 5 }}
                name="Gold"
              />
            </>
          )}

          {/* EV Line (if enabled) */}
          {showEV && (
            <Line
              yAxisId="hp" // Share axis with HP for now
              type="monotone"
              dataKey="ev"
              stroke="#22c55e"
              strokeWidth={2}
              dot={{ fill: '#22c55e', strokeWidth: 0, r: 3 }}
              activeDot={{ fill: '#22c55e', strokeWidth: 2, stroke: '#fff', r: 5 }}
              name="EV"
            />
          )}
        </ComposedChart>
      </ResponsiveContainer>
    </div>
  );
};

export default ResourceProjection;
