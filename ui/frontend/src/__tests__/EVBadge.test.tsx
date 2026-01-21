/**
 * Tests for EVBadge component
 */

import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '../test-utils';
import { EVBadge } from '../components/EVDisplay/EVBadge';

describe('EVBadge', () => {
  describe('rendering', () => {
    it('renders EV value', () => {
      render(<EVBadge ev={0.15} />);

      expect(screen.getByText('+0.15')).toBeInTheDocument();
    });

    it('renders negative EV value', () => {
      render(<EVBadge ev={-0.25} />);

      expect(screen.getByText('-0.25')).toBeInTheDocument();
    });

    it('renders zero EV value', () => {
      render(<EVBadge ev={0} />);

      expect(screen.getByText('+0.00')).toBeInTheDocument();
    });

    it('respects showSign=false prop', () => {
      render(<EVBadge ev={0.15} showSign={false} />);

      expect(screen.getByText('0.15')).toBeInTheDocument();
    });
  });

  describe('color coding', () => {
    it('uses green color for positive EV > 0.1', () => {
      render(<EVBadge ev={0.15} />);

      const badge = screen.getByText('+0.15');
      expect(badge).toHaveStyle({ color: '#22c55e' });
    });

    it('uses red color for negative EV < -0.1', () => {
      render(<EVBadge ev={-0.25} />);

      const badge = screen.getByText('-0.25');
      expect(badge).toHaveStyle({ color: '#ef4444' });
    });

    it('uses gray color for neutral EV (between -0.1 and 0.1)', () => {
      render(<EVBadge ev={0.05} />);

      const badge = screen.getByText('+0.05');
      expect(badge).toHaveStyle({ color: '#6b7280' });
    });

    it('treats exactly 0.1 as neutral', () => {
      render(<EVBadge ev={0.1} />);

      const badge = screen.getByText('+0.10');
      expect(badge).toHaveStyle({ color: '#6b7280' });
    });

    it('treats exactly -0.1 as neutral', () => {
      render(<EVBadge ev={-0.1} />);

      const badge = screen.getByText('-0.10');
      expect(badge).toHaveStyle({ color: '#6b7280' });
    });
  });

  describe('sizes', () => {
    it('renders small size', () => {
      render(<EVBadge ev={0.5} size="sm" />);

      const badge = screen.getByText('+0.50');
      expect(badge).toHaveStyle({ fontSize: '0.75rem' });
    });

    it('renders medium size by default', () => {
      render(<EVBadge ev={0.5} />);

      const badge = screen.getByText('+0.50');
      expect(badge).toHaveStyle({ fontSize: '0.875rem' });
    });

    it('renders large size', () => {
      render(<EVBadge ev={0.5} size="lg" />);

      const badge = screen.getByText('+0.50');
      expect(badge).toHaveStyle({ fontSize: '1rem' });
    });
  });

  describe('tooltip with breakdown', () => {
    const breakdown = {
      baseEV: 0.5,
      hpDelta: -0.1,
      goldDelta: 0.05,
      deckImprovement: 0.15,
      relicValue: 0.1,
      floorProgress: 0.1,
      riskPenalty: 0.05,
    };

    it('shows tooltip on hover when breakdown provided', () => {
      render(<EVBadge ev={0.75} breakdown={breakdown} />);

      const badge = screen.getByText('+0.75');
      fireEvent.mouseEnter(badge.parentElement!);

      expect(screen.getByText('EV Breakdown')).toBeInTheDocument();
    });

    it('hides tooltip on mouse leave', () => {
      render(<EVBadge ev={0.75} breakdown={breakdown} />);

      const badge = screen.getByText('+0.75');
      fireEvent.mouseEnter(badge.parentElement!);
      expect(screen.getByText('EV Breakdown')).toBeInTheDocument();

      fireEvent.mouseLeave(badge.parentElement!);
      expect(screen.queryByText('EV Breakdown')).not.toBeInTheDocument();
    });

    it('displays HP delta in tooltip', () => {
      render(<EVBadge ev={0.75} breakdown={breakdown} />);

      const badge = screen.getByText('+0.75');
      fireEvent.mouseEnter(badge.parentElement!);

      expect(screen.getByText('HP Delta')).toBeInTheDocument();
      expect(screen.getByText('-0.10')).toBeInTheDocument();
    });

    it('displays gold delta in tooltip', () => {
      render(<EVBadge ev={0.75} breakdown={breakdown} />);

      const badge = screen.getByText('+0.75');
      fireEvent.mouseEnter(badge.parentElement!);

      expect(screen.getByText('Gold Delta')).toBeInTheDocument();
      expect(screen.getByText('+0.05')).toBeInTheDocument();
    });

    it('displays deck value in tooltip', () => {
      render(<EVBadge ev={0.75} breakdown={breakdown} />);

      const badge = screen.getByText('+0.75');
      fireEvent.mouseEnter(badge.parentElement!);

      expect(screen.getByText('Deck Value')).toBeInTheDocument();
      expect(screen.getByText('+0.15')).toBeInTheDocument();
    });

    it('displays relic value in tooltip', () => {
      render(<EVBadge ev={0.75} breakdown={breakdown} />);

      const badge = screen.getByText('+0.75');
      fireEvent.mouseEnter(badge.parentElement!);

      expect(screen.getByText('Relic Value')).toBeInTheDocument();
      // +0.10 appears twice (relicValue and floorProgress), use getAllByText
      const values = screen.getAllByText('+0.10');
      expect(values.length).toBe(2);
    });

    it('displays floor progress in tooltip', () => {
      render(<EVBadge ev={0.75} breakdown={breakdown} />);

      const badge = screen.getByText('+0.75');
      fireEvent.mouseEnter(badge.parentElement!);

      expect(screen.getByText('Floor Progress')).toBeInTheDocument();
    });

    it('displays risk penalty in tooltip', () => {
      render(<EVBadge ev={0.75} breakdown={breakdown} />);

      const badge = screen.getByText('+0.75');
      fireEvent.mouseEnter(badge.parentElement!);

      expect(screen.getByText('Risk Penalty')).toBeInTheDocument();
      expect(screen.getByText('-0.05')).toBeInTheDocument();
    });

    it('displays total EV in tooltip', () => {
      render(<EVBadge ev={0.75} breakdown={breakdown} />);

      const badge = screen.getByText('+0.75');
      fireEvent.mouseEnter(badge.parentElement!);

      expect(screen.getByText('Total EV')).toBeInTheDocument();
    });

    it('does not show tooltip when no breakdown provided', () => {
      render(<EVBadge ev={0.75} />);

      const badge = screen.getByText('+0.75');
      fireEvent.mouseEnter(badge.parentElement!);

      expect(screen.queryByText('EV Breakdown')).not.toBeInTheDocument();
    });
  });

  describe('custom className', () => {
    it('applies custom className', () => {
      render(<EVBadge ev={0.5} className="custom-class" />);

      const container = document.querySelector('.ev-badge');
      expect(container).toHaveClass('custom-class');
    });
  });

  describe('edge cases', () => {
    it('handles very large positive EV', () => {
      render(<EVBadge ev={99.99} />);

      expect(screen.getByText('+99.99')).toBeInTheDocument();
    });

    it('handles very large negative EV', () => {
      render(<EVBadge ev={-99.99} />);

      expect(screen.getByText('-99.99')).toBeInTheDocument();
    });

    it('handles partial breakdown', () => {
      const partialBreakdown = {
        baseEV: 0.5,
        hpDelta: -0.1,
      };

      render(<EVBadge ev={0.4} breakdown={partialBreakdown} />);

      const badge = screen.getByText('+0.40');
      fireEvent.mouseEnter(badge.parentElement!);

      expect(screen.getByText('HP Delta')).toBeInTheDocument();
      expect(screen.queryByText('Gold Delta')).not.toBeInTheDocument();
    });
  });
});
