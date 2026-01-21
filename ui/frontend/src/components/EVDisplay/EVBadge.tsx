/**
 * EVBadge Component
 *
 * Displays an EV (Expected Value) delta with color coding.
 * - Green for positive EV (good decisions)
 * - Red for negative EV (bad decisions)
 * - Gray for neutral
 *
 * Includes a tooltip with detailed breakdown.
 */

import React, { useState } from 'react';

export interface EVBreakdown {
  baseEV: number;
  hpDelta?: number;
  goldDelta?: number;
  deckImprovement?: number;
  relicValue?: number;
  floorProgress?: number;
  riskPenalty?: number;
}

export interface EVBadgeProps {
  ev: number;
  breakdown?: EVBreakdown;
  size?: 'sm' | 'md' | 'lg';
  showSign?: boolean;
  className?: string;
}

const SIZES = {
  sm: {
    fontSize: '0.75rem',
    padding: '0.2rem 0.4rem',
    borderRadius: '3px',
  },
  md: {
    fontSize: '0.875rem',
    padding: '0.3rem 0.6rem',
    borderRadius: '4px',
  },
  lg: {
    fontSize: '1rem',
    padding: '0.4rem 0.8rem',
    borderRadius: '5px',
  },
};

export const EVBadge: React.FC<EVBadgeProps> = ({
  ev,
  breakdown,
  size = 'md',
  showSign = true,
  className = '',
}) => {
  const [showTooltip, setShowTooltip] = useState(false);

  const getColor = () => {
    if (ev > 0.1) return '#22c55e'; // green
    if (ev < -0.1) return '#ef4444'; // red
    return '#6b7280'; // gray
  };

  const getBackgroundColor = () => {
    if (ev > 0.1) return 'rgba(34, 197, 94, 0.15)';
    if (ev < -0.1) return 'rgba(239, 68, 68, 0.15)';
    return 'rgba(107, 114, 128, 0.15)';
  };

  const getBorderColor = () => {
    if (ev > 0.1) return 'rgba(34, 197, 94, 0.4)';
    if (ev < -0.1) return 'rgba(239, 68, 68, 0.4)';
    return 'rgba(107, 114, 128, 0.4)';
  };

  const formatEV = (value: number) => {
    const sign = showSign && value >= 0 ? '+' : '';
    return `${sign}${value.toFixed(2)}`;
  };

  const sizeStyles = SIZES[size];
  const color = getColor();
  const bgColor = getBackgroundColor();
  const borderColor = getBorderColor();

  return (
    <div
      className={`ev-badge ${className}`}
      style={{ position: 'relative', display: 'inline-block' }}
      onMouseEnter={() => setShowTooltip(true)}
      onMouseLeave={() => setShowTooltip(false)}
    >
      <style>{`
        .ev-badge-inner {
          font-family: 'Cinzel', serif;
          font-weight: 600;
          display: inline-flex;
          align-items: center;
          gap: 0.25rem;
          transition: all 0.2s ease;
        }

        .ev-badge-inner:hover {
          transform: scale(1.05);
        }

        .ev-tooltip {
          position: absolute;
          top: 100%;
          left: 50%;
          transform: translateX(-50%);
          margin-top: 8px;
          padding: 0.75rem 1rem;
          background: linear-gradient(180deg, #1a1a25 0%, #12121a 100%);
          border: 1px solid rgba(212, 168, 87, 0.3);
          border-radius: 6px;
          box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
          z-index: 100;
          min-width: 180px;
          pointer-events: none;
        }

        .ev-tooltip::before {
          content: '';
          position: absolute;
          bottom: 100%;
          left: 50%;
          transform: translateX(-50%);
          border: 6px solid transparent;
          border-bottom-color: rgba(212, 168, 87, 0.3);
        }

        .ev-tooltip-title {
          font-family: 'Cinzel', serif;
          font-size: 0.7rem;
          text-transform: uppercase;
          letter-spacing: 0.1em;
          color: #9a9486;
          margin-bottom: 0.5rem;
          padding-bottom: 0.5rem;
          border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        }

        .ev-tooltip-row {
          display: flex;
          justify-content: space-between;
          align-items: center;
          font-family: 'Crimson Text', serif;
          font-size: 0.85rem;
          margin-bottom: 0.25rem;
        }

        .ev-tooltip-label {
          color: #9a9486;
        }

        .ev-tooltip-value {
          font-weight: 600;
        }

        .ev-tooltip-total {
          margin-top: 0.5rem;
          padding-top: 0.5rem;
          border-top: 1px solid rgba(255, 255, 255, 0.1);
        }
      `}</style>

      <span
        className="ev-badge-inner"
        style={{
          ...sizeStyles,
          color,
          backgroundColor: bgColor,
          border: `1px solid ${borderColor}`,
          cursor: breakdown ? 'help' : 'default',
        }}
      >
        {formatEV(ev)}
      </span>

      {/* Tooltip with breakdown */}
      {showTooltip && breakdown && (
        <div className="ev-tooltip">
          <div className="ev-tooltip-title">EV Breakdown</div>

          {breakdown.hpDelta !== undefined && (
            <div className="ev-tooltip-row">
              <span className="ev-tooltip-label">HP Delta</span>
              <span
                className="ev-tooltip-value"
                style={{ color: breakdown.hpDelta >= 0 ? '#22c55e' : '#ef4444' }}
              >
                {formatEV(breakdown.hpDelta)}
              </span>
            </div>
          )}

          {breakdown.goldDelta !== undefined && (
            <div className="ev-tooltip-row">
              <span className="ev-tooltip-label">Gold Delta</span>
              <span
                className="ev-tooltip-value"
                style={{ color: breakdown.goldDelta >= 0 ? '#22c55e' : '#ef4444' }}
              >
                {formatEV(breakdown.goldDelta)}
              </span>
            </div>
          )}

          {breakdown.deckImprovement !== undefined && (
            <div className="ev-tooltip-row">
              <span className="ev-tooltip-label">Deck Value</span>
              <span
                className="ev-tooltip-value"
                style={{ color: breakdown.deckImprovement >= 0 ? '#22c55e' : '#ef4444' }}
              >
                {formatEV(breakdown.deckImprovement)}
              </span>
            </div>
          )}

          {breakdown.relicValue !== undefined && (
            <div className="ev-tooltip-row">
              <span className="ev-tooltip-label">Relic Value</span>
              <span
                className="ev-tooltip-value"
                style={{ color: breakdown.relicValue >= 0 ? '#22c55e' : '#ef4444' }}
              >
                {formatEV(breakdown.relicValue)}
              </span>
            </div>
          )}

          {breakdown.floorProgress !== undefined && (
            <div className="ev-tooltip-row">
              <span className="ev-tooltip-label">Floor Progress</span>
              <span
                className="ev-tooltip-value"
                style={{ color: breakdown.floorProgress >= 0 ? '#22c55e' : '#ef4444' }}
              >
                {formatEV(breakdown.floorProgress)}
              </span>
            </div>
          )}

          {breakdown.riskPenalty !== undefined && (
            <div className="ev-tooltip-row">
              <span className="ev-tooltip-label">Risk Penalty</span>
              <span
                className="ev-tooltip-value"
                style={{ color: '#ef4444' }}
              >
                {formatEV(-Math.abs(breakdown.riskPenalty))}
              </span>
            </div>
          )}

          <div className="ev-tooltip-row ev-tooltip-total">
            <span className="ev-tooltip-label" style={{ fontWeight: 600 }}>
              Total EV
            </span>
            <span className="ev-tooltip-value" style={{ color }}>
              {formatEV(ev)}
            </span>
          </div>
        </div>
      )}
    </div>
  );
};

export default EVBadge;
