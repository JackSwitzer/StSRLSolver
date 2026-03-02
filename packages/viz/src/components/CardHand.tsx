import type { CardInstance } from '../types/game';

interface CardHandProps {
  cards: CardInstance[];
  energy: number;
}

const CARD_TYPE_COLORS: Record<string, string> = {
  attack: '#cc3333',
  skill: '#3366cc',
  power: '#ccaa22',
  status: '#666666',
  curse: '#882288',
};

const CARD_WIDTH = 80;
const CARD_HEIGHT = 110;

export const CardHand = ({ cards, energy }: CardHandProps) => {
  const totalWidth = Math.min(cards.length * (CARD_WIDTH + 8), 600);
  const cardSpacing = cards.length > 1 ? totalWidth / cards.length : CARD_WIDTH + 8;
  const startX = (totalWidth - cardSpacing * (cards.length - 1)) / 2;

  return (
    <div style={{ display: 'flex', justifyContent: 'center', padding: '8px 0', minHeight: CARD_HEIGHT + 20 }}>
      <svg
        viewBox={`0 0 ${Math.max(totalWidth + 40, 200)} ${CARD_HEIGHT + 20}`}
        width={Math.max(totalWidth + 40, 200)}
        height={CARD_HEIGHT + 20}
      >
        {cards.map((card, i) => {
          const cx = 20 + startX + i * cardSpacing;
          const cy = 5;
          const playable = card.playable !== false && card.cost <= energy;
          const typeColor = CARD_TYPE_COLORS[card.type] || '#444';
          const angle = cards.length > 1 ? (i - (cards.length - 1) / 2) * 3 : 0;

          return (
            <g
              key={`${card.id}-${i}`}
              transform={`translate(${cx}, ${cy}) rotate(${angle}, ${CARD_WIDTH / 2}, ${CARD_HEIGHT})`}
              opacity={playable ? 1 : 0.5}
              style={{ cursor: playable ? 'pointer' : 'default' }}
            >
              {/* Card background */}
              <rect
                width={CARD_WIDTH}
                height={CARD_HEIGHT}
                rx="6"
                fill="#1e1e2e"
                stroke={playable ? '#ffd700' : '#444'}
                strokeWidth={playable ? 2 : 1}
              />
              {/* Type color bar */}
              <rect y="2" x="2" width={CARD_WIDTH - 4} height="4" rx="2" fill={typeColor} />
              {/* Card name */}
              <text
                x={CARD_WIDTH / 2}
                y="24"
                textAnchor="middle"
                fill="#e0e0e0"
                fontSize="9"
                fontWeight="bold"
              >
                {card.name.length > 12 ? card.name.slice(0, 11) + '..' : card.name}
              </text>
              {/* Upgraded indicator */}
              {card.upgraded && (
                <text x={CARD_WIDTH / 2} y="34" textAnchor="middle" fill="#44cc44" fontSize="7">
                  +
                </text>
              )}
              {/* Cost orb */}
              <circle cx="12" cy="12" r="10" fill="#1a1a2e" stroke={typeColor} strokeWidth="1.5" />
              <text x="12" y="12" textAnchor="middle" dominantBaseline="central" fill="#e0e0e0" fontSize="10" fontWeight="bold">
                {card.cost >= 0 ? card.cost : 'X'}
              </text>
              {/* Card type label */}
              <text
                x={CARD_WIDTH / 2}
                y={CARD_HEIGHT - 10}
                textAnchor="middle"
                fill="#888"
                fontSize="7"
                textTransform="uppercase"
              >
                {card.type}
              </text>
              {/* Description (truncated) */}
              {card.description && (
                <foreignObject x="4" y="38" width={CARD_WIDTH - 8} height="58">
                  <div
                    style={{
                      fontSize: '7px',
                      color: '#aaa',
                      textAlign: 'center',
                      overflow: 'hidden',
                      lineHeight: '1.3',
                      wordBreak: 'break-word',
                    }}
                  >
                    {card.description}
                  </div>
                </foreignObject>
              )}
            </g>
          );
        })}
      </svg>
    </div>
  );
};
