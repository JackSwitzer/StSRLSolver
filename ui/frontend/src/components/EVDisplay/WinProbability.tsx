/**
 * WinProbability Component
 *
 * Displays the current win probability with visual indicators.
 * Shows:
 * - Current win percentage
 * - Visual progress bar
 * - Trend indicator (up/down from previous)
 * - Color coding based on probability level
 */

import React from 'react';

export interface WinProbabilityProps {
  probability: number; // 0 to 1
  previousProbability?: number; // For showing trend
  showBar?: boolean;
  showTrend?: boolean;
  size?: 'sm' | 'md' | 'lg';
  label?: string;
  className?: string;
}

const SIZES = {
  sm: {
    fontSize: '1rem',
    barHeight: 4,
    width: 80,
  },
  md: {
    fontSize: '1.25rem',
    barHeight: 6,
    width: 120,
  },
  lg: {
    fontSize: '1.75rem',
    barHeight: 8,
    width: 160,
  },
};

export const WinProbability: React.FC<WinProbabilityProps> = ({
  probability,
  previousProbability,
  showBar = true,
  showTrend = true,
  size = 'md',
  label = 'Win Rate',
  className = '',
}) => {
  const getColor = () => {
    if (probability >= 0.8) return '#22c55e'; // green - great
    if (probability >= 0.6) return '#84cc16'; // lime - good
    if (probability >= 0.4) return '#eab308'; // yellow - okay
    if (probability >= 0.2) return '#f97316'; // orange - risky
    return '#ef4444'; // red - danger
  };

  const getBackgroundColor = (opacity: number) => {
    const color = getColor();
    // Convert hex to rgba
    const hex = color.replace('#', '');
    const r = parseInt(hex.substring(0, 2), 16);
    const g = parseInt(hex.substring(2, 4), 16);
    const b = parseInt(hex.substring(4, 6), 16);
    return `rgba(${r}, ${g}, ${b}, ${opacity})`;
  };

  const trend = previousProbability !== undefined ? probability - previousProbability : 0;
  const showTrendIndicator = showTrend && previousProbability !== undefined && Math.abs(trend) > 0.001;

  const sizeStyles = SIZES[size];
  const percentage = (probability * 100).toFixed(1);

  return (
    <div className={`win-probability ${className}`}>
      <style>{`
        .win-probability {
          display: inline-flex;
          flex-direction: column;
          align-items: center;
          gap: 0.25rem;
        }

        .win-probability-label {
          font-family: 'Cinzel', serif;
          font-size: 0.7rem;
          text-transform: uppercase;
          letter-spacing: 0.1em;
          color: #9a9486;
        }

        .win-probability-value {
          display: flex;
          align-items: baseline;
          gap: 0.25rem;
        }

        .win-probability-number {
          font-family: 'Cinzel', serif;
          font-weight: 700;
          letter-spacing: -0.02em;
        }

        .win-probability-percent {
          font-family: 'Cinzel', serif;
          font-size: 0.6em;
          color: #9a9486;
        }

        .win-probability-trend {
          font-family: 'Cinzel', serif;
          font-size: 0.75rem;
          margin-left: 0.25rem;
        }

        .win-probability-bar {
          width: 100%;
          background: rgba(255, 255, 255, 0.1);
          border-radius: 4px;
          overflow: hidden;
        }

        .win-probability-bar-fill {
          height: 100%;
          border-radius: 4px;
          transition: width 0.5s ease, background-color 0.3s ease;
        }

        .win-probability-bar-glow {
          position: relative;
        }

        .win-probability-bar-glow::after {
          content: '';
          position: absolute;
          top: 0;
          right: 0;
          bottom: 0;
          width: 20px;
          background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.3));
          animation: shimmer 2s infinite;
        }

        @keyframes shimmer {
          0% { opacity: 0; }
          50% { opacity: 1; }
          100% { opacity: 0; }
        }
      `}</style>

      {label && <span className="win-probability-label">{label}</span>}

      <div className="win-probability-value">
        <span
          className="win-probability-number"
          style={{
            fontSize: sizeStyles.fontSize,
            color: getColor(),
          }}
        >
          {percentage}
        </span>
        <span className="win-probability-percent">%</span>

        {showTrendIndicator && (
          <span
            className="win-probability-trend"
            style={{
              color: trend > 0 ? '#22c55e' : '#ef4444',
            }}
          >
            {trend > 0 ? '+' : ''}{(trend * 100).toFixed(1)}%
          </span>
        )}
      </div>

      {showBar && (
        <div
          className="win-probability-bar"
          style={{
            width: sizeStyles.width,
            height: sizeStyles.barHeight,
          }}
        >
          <div
            className={`win-probability-bar-fill ${probability >= 0.8 ? 'win-probability-bar-glow' : ''}`}
            style={{
              width: `${probability * 100}%`,
              backgroundColor: getColor(),
              boxShadow: `0 0 8px ${getBackgroundColor(0.5)}`,
            }}
          />
        </div>
      )}
    </div>
  );
};

export default WinProbability;
