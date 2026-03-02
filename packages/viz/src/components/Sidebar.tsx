import type { RunState } from '../types/game';

interface SidebarProps {
  run: RunState;
}

export const Sidebar = ({ run }: SidebarProps) => {
  const { hp, max_hp, gold, floor, act, deck, relics, potions, ascension } = run;
  const hpRatio = hp / max_hp;
  const hpColor = hpRatio > 0.6 ? '#44bb44' : hpRatio > 0.3 ? '#ccaa22' : '#cc3333';

  return (
    <div className="sidebar">
      {/* Run Info */}
      <div className="sidebar-section">
        <div className="sidebar-header">Run Info</div>
        <div className="sidebar-row">
          <span>Floor</span>
          <span>{floor}</span>
        </div>
        <div className="sidebar-row">
          <span>Act</span>
          <span>{act}</span>
        </div>
        <div className="sidebar-row">
          <span>Ascension</span>
          <span>{ascension}</span>
        </div>
      </div>

      {/* HP */}
      <div className="sidebar-section">
        <div className="sidebar-header">HP</div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <div className="hp-bar-container">
            <div
              className="hp-bar-fill"
              style={{ width: `${hpRatio * 100}%`, background: hpColor }}
            />
          </div>
          <span style={{ fontSize: '12px', color: hpColor, fontWeight: 'bold', whiteSpace: 'nowrap' }}>
            {hp}/{max_hp}
          </span>
        </div>
      </div>

      {/* Gold */}
      <div className="sidebar-section">
        <div className="sidebar-header">Gold</div>
        <div style={{ color: '#ffd700', fontWeight: 'bold', fontSize: '16px' }}>
          {gold}
        </div>
      </div>

      {/* Relics */}
      <div className="sidebar-section">
        <div className="sidebar-header">Relics ({relics.length})</div>
        <div className="relic-row">
          {relics.map((relic) => (
            <div key={relic.id} className="relic-icon" title={relic.name}>
              <svg viewBox="0 0 20 20" width="24" height="24">
                <rect x="2" y="2" width="16" height="16" rx="3" fill="#3a3a5a" stroke="#666" strokeWidth="1" />
                <text x="10" y="10" textAnchor="middle" dominantBaseline="central" fill="#ccc" fontSize="8">
                  {relic.name.charAt(0)}
                </text>
              </svg>
              {relic.counter !== undefined && relic.counter > 0 && (
                <span className="relic-counter">{relic.counter}</span>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Potions */}
      <div className="sidebar-section">
        <div className="sidebar-header">Potions</div>
        <div className="potion-row">
          {potions.map((potion, i) => (
            <div key={i} className="potion-slot" title={potion.name || 'Empty'}>
              <svg viewBox="0 0 20 24" width="20" height="24">
                {potion.id ? (
                  <>
                    <path d="M7,8 L7,4 L13,4 L13,8 Q18,16 14,20 L6,20 Q2,16 7,8 Z" fill="#4466aa" opacity="0.8" />
                    <rect x="6" y="2" width="8" height="3" rx="1" fill="#666" />
                  </>
                ) : (
                  <>
                    <path d="M7,8 L7,4 L13,4 L13,8 Q18,16 14,20 L6,20 Q2,16 7,8 Z" fill="none" stroke="#444" strokeWidth="1" strokeDasharray="2,2" />
                    <rect x="6" y="2" width="8" height="3" rx="1" fill="#333" />
                  </>
                )}
              </svg>
              {potion.name && (
                <span className="potion-label">{potion.name.slice(0, 8)}</span>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Deck */}
      <div className="sidebar-section">
        <div className="sidebar-header">Deck ({deck.length})</div>
        <div className="deck-list">
          {deck.map((card, i) => (
            <div key={`${card.id}-${i}`} className="deck-card-row">
              <span className={`card-type-dot card-type-${card.type}`} />
              <span className="deck-card-name">
                {card.name}
                {card.upgraded && <span className="upgraded-plus">+</span>}
              </span>
              <span className="deck-card-cost">{card.cost >= 0 ? card.cost : 'X'}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
