import { useState, useMemo } from 'react';
import type { CardInstance } from '../types/game';

interface DeckViewProps {
  deck: CardInstance[];
  onClose: () => void;
}

type CardTypeFilter = 'all' | 'attack' | 'skill' | 'power' | 'status' | 'curse';

const TYPE_COLORS: Record<string, string> = {
  attack: '#cc3333',
  skill: '#3366cc',
  power: '#ccaa22',
  status: '#666666',
  curse: '#882288',
};

const TYPE_BG: Record<string, string> = {
  attack: '#2a1a1a',
  skill: '#1a1a2a',
  power: '#2a2a1a',
  status: '#1a1a1a',
  curse: '#2a1a2a',
};

const FILTER_OPTIONS: { value: CardTypeFilter; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'attack', label: 'Attacks' },
  { value: 'skill', label: 'Skills' },
  { value: 'power', label: 'Powers' },
];

export const DeckView = ({ deck, onClose }: DeckViewProps) => {
  const [search, setSearch] = useState('');
  const [typeFilter, setTypeFilter] = useState<CardTypeFilter>('all');

  const filtered = useMemo(() => {
    let cards = deck;
    if (typeFilter !== 'all') {
      cards = cards.filter((c) => c.type === typeFilter);
    }
    if (search.trim()) {
      const q = search.toLowerCase().trim();
      cards = cards.filter((c) => c.name.toLowerCase().includes(q) || (c.description || '').toLowerCase().includes(q));
    }
    return cards;
  }, [deck, typeFilter, search]);

  // Group by type for display
  const grouped = useMemo(() => {
    const groups: Record<string, CardInstance[]> = {};
    for (const card of filtered) {
      const t = card.type;
      if (!groups[t]) groups[t] = [];
      groups[t].push(card);
    }
    // Sort groups: attack, skill, power, then others
    const order = ['attack', 'skill', 'power', 'status', 'curse'];
    const sorted: [string, CardInstance[]][] = [];
    for (const t of order) {
      if (groups[t]) sorted.push([t, groups[t]]);
    }
    return sorted;
  }, [filtered]);

  // Counts by type
  const typeCounts = useMemo(() => {
    const counts: Record<string, number> = {};
    for (const card of deck) {
      counts[card.type] = (counts[card.type] || 0) + 1;
    }
    return counts;
  }, [deck]);

  return (
    <div className="deck-view-overlay">
      <div className="deck-view-panel">
        {/* Header */}
        <div className="deck-view-header">
          <h2 className="deck-view-title">
            Deck ({deck.length})
          </h2>
          <button className="deck-view-close" onClick={onClose}>
            X
          </button>
        </div>

        {/* Search + Filters */}
        <div className="deck-view-controls">
          <input
            className="deck-view-search"
            type="text"
            placeholder="Search cards..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
          <div className="deck-view-filters">
            {FILTER_OPTIONS.map((opt) => (
              <button
                key={opt.value}
                className={`deck-view-filter-btn ${typeFilter === opt.value ? 'active' : ''}`}
                style={{
                  borderColor: opt.value !== 'all' ? TYPE_COLORS[opt.value] || '#555' : undefined,
                  background: typeFilter === opt.value ? (opt.value !== 'all' ? TYPE_BG[opt.value] : '#2a2a44') : undefined,
                }}
                onClick={() => setTypeFilter(opt.value)}
              >
                {opt.label}
                {opt.value !== 'all' && typeCounts[opt.value] ? ` (${typeCounts[opt.value]})` : ''}
              </button>
            ))}
          </div>
        </div>

        {/* Card Grid */}
        <div className="deck-view-grid">
          {grouped.map(([type, cards]) => (
            <div key={type} className="deck-view-group">
              <div className="deck-view-group-header" style={{ color: TYPE_COLORS[type] || '#888' }}>
                {type.charAt(0).toUpperCase() + type.slice(1)}s ({cards.length})
              </div>
              <div className="deck-view-cards">
                {cards.map((card, i) => (
                  <div
                    key={`${card.id}-${i}`}
                    className="deck-view-card"
                    style={{ borderLeftColor: TYPE_COLORS[card.type] || '#444' }}
                  >
                    <div className="deck-view-card-top">
                      <span className="deck-view-card-cost" style={{ borderColor: TYPE_COLORS[card.type] || '#444' }}>
                        {card.cost >= 0 ? card.cost : 'X'}
                      </span>
                      <span className="deck-view-card-name">
                        {card.name}
                        {card.upgraded && <span className="upgraded-plus">+</span>}
                      </span>
                      <span className="deck-view-card-type" style={{ color: TYPE_COLORS[card.type] || '#888' }}>
                        {card.type}
                      </span>
                    </div>
                    {card.description && (
                      <div className="deck-view-card-desc">
                        {card.description}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>
          ))}
          {filtered.length === 0 && (
            <div className="deck-view-empty">No cards match the filter.</div>
          )}
        </div>
      </div>
    </div>
  );
};
