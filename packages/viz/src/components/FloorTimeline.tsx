import { useState } from 'react';
import type { CombatSummary, DecisionSummary } from '../types/training';

interface FloorEntry {
  floor: number;
  type: string;
  event_id?: string;
}

interface FloorTimelineProps {
  combats?: CombatSummary[];
  decisions?: DecisionSummary[];
  deathFloor?: number;
  deckChanges?: string[];
  maxFloor?: number;
  floorLog?: FloorEntry[];
  neowChoice?: string;
}

type RoomType = 'monster' | 'elite' | 'boss' | 'event' | 'rest' | 'shop' | 'unknown';

const ROOM_ICONS: Record<RoomType, { symbol: string; color: string; label: string }> = {
  monster: { symbol: '\u25CF', color: '#8b949e', label: 'Monster' },
  elite: { symbol: '\u25B2', color: '#f0883e', label: 'Elite' },
  boss: { symbol: '\u25C6', color: '#f85149', label: 'Boss' },
  event: { symbol: '\u25A0', color: '#58a6ff', label: 'Event' },
  rest: { symbol: '\u2605', color: '#3fb950', label: 'Rest' },
  shop: { symbol: '$', color: '#ffd700', label: 'Shop' },
  unknown: { symbol: '\u25CB', color: '#484f58', label: '?' },
};

function inferRoomType(floor: number, combats?: CombatSummary[], floorLog?: FloorEntry[]): RoomType {
  // Check floor_log first for non-combat room types
  const logEntry = floorLog?.find((f) => f.floor === floor);
  if (logEntry) {
    const t = logEntry.type.toLowerCase();
    if (t === 'event' || t === 'event') return 'event';
    if (t === 'rest') return 'rest';
    if (t === 'shop') return 'shop';
    if (t === 'treasure') return 'shop'; // treasure uses shop icon
  }
  const combat = combats?.find((c) => c.floor === floor);
  if (!combat && logEntry) {
    // Floor exists in log but no combat — it's a non-combat floor
    return logEntry.type.toLowerCase() as RoomType;
  }
  if (!combat) return 'unknown';
  const name = (combat.enemy ?? '').toLowerCase();
  if (combat.room_type === 'boss' || name.includes('boss') || name.includes('guardian') || name.includes('hexaghost') || name.includes('champ') || name.includes('automaton') || name.includes('collector') || name.includes('awakened') || name.includes('time eater') || name.includes('donu') || name.includes('heart')) return 'boss';
  if (combat.room_type === 'elite' || name.includes('nob') || name.includes('lagavulin') || name.includes('sentries') || name.includes('slavers') || name.includes('nemesis') || name.includes('reptomancer')) return 'elite';
  return 'monster';
}

function floorEventLabel(floor: number, floorLog?: FloorEntry[]): string | null {
  const entry = floorLog?.find((f) => f.floor === floor);
  if (entry?.event_id) return entry.event_id;
  return null;
}

function cardColor(name: string): string {
  const lower = name.toLowerCase();
  const powers = ['mental fortress', 'rushdown', 'like water', 'fasting', 'battle hymn', 'deva form', 'devotion', 'nirvana', 'omega', 'alpha', 'establishment', 'master reality', 'study'];
  if (powers.some((p) => lower.includes(p))) return '#e3b341';
  const skills = ['defend', 'vigilance', 'protect', 'meditate', 'inner peace', 'halt', 'evaluate', 'crescendo', 'tranquility', 'prostrate', 'pray', 'worship', 'collect', 'judgment', 'vault', 'wish', 'miracle', 'empty body', 'empty mind', 'third eye', 'scrawl', 'spirit shield', 'wave of the hand', 'perseverance', 'sanctity', 'foreign influence', 'foresight', 'deceive reality', 'conjure blade', 'wreath of flame', 'simmering fury'];
  if (skills.some((s) => lower.includes(s))) return '#6699ff';
  return '#ff6666'; // attacks default
}

function stanceTag(name: string): string | null {
  const lower = name.toLowerCase();
  if (lower.includes('eruption') || lower.includes('tantrum') || lower.includes('crescendo') || lower.includes('simmering')) return 'W';
  if (lower.includes('vigilance') || lower.includes('empty body') || lower.includes('fear no evil') || lower.includes('inner peace') || lower.includes('tranquility')) return 'C';
  if (lower.includes('worship') || lower.includes('prostrate')) return 'M';
  return null;
}

const STANCE_COLORS: Record<string, string> = { W: '#f85149', C: '#58a6ff', M: '#d2a038' };

// Expandable combat detail row
function CombatDetail({ combat }: { combat: CombatSummary & { turns_detail?: Array<{ turn: number; cards: string[] }> } }) {
  return (
    <div style={{ padding: '8px 0 4px 28px', borderTop: '1px solid #21262d' }}>
      <div style={{ display: 'flex', gap: 16, fontSize: 11, color: '#8b949e', marginBottom: 6 }}>
        <span>{combat.turns} turns</span>
        <span style={{ color: '#f85149' }}>-{combat.hp_lost} HP</span>
        {combat.damage_dealt > 0 && <span>{combat.damage_dealt} dmg dealt</span>}
        {combat.used_potion && <span style={{ color: '#d2a8ff' }}>potion</span>}
        {combat.stances && Object.keys(combat.stances).length > 0 && (
          <span>Stances: {Object.entries(combat.stances).map(([s, n]) => `${s}x${n}`).join(', ')}</span>
        )}
      </div>
      {combat.turns_detail && combat.turns_detail.length > 0 ? (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
          {combat.turns_detail.map((t) => (
            <div key={t.turn} style={{ display: 'flex', gap: 4, alignItems: 'baseline', fontSize: 11 }}>
              <span style={{ color: '#484f58', minWidth: 22, fontVariantNumeric: 'tabular-nums' }}>T{t.turn}</span>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: 3 }}>
                {t.cards.map((card, i) => {
                  const stance = stanceTag(card);
                  return (
                    <span key={`${t.turn}-${i}`}>
                      <span style={{ color: cardColor(card) }}>{card}</span>
                      {stance && <span style={{ color: STANCE_COLORS[stance], fontSize: 9, marginLeft: 1 }}>{stance}</span>}
                      {i < t.cards.length - 1 && <span style={{ color: '#21262d' }}> / </span>}
                    </span>
                  );
                })}
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div style={{ fontSize: 11, color: '#484f58', fontStyle: 'italic' }}>No turn detail</div>
      )}
    </div>
  );
}

export const FloorTimeline: React.FC<FloorTimelineProps> = ({
  combats,
  deathFloor,
  deckChanges,
  maxFloor,
  floorLog,
  neowChoice: _neowChoice,
}) => {
  const [expandedFloor, setExpandedFloor] = useState<number | null>(null);

  const floors = maxFloor ?? Math.max(
    ...(combats?.map((c) => c.floor) ?? [0]),
    ...(floorLog?.map((f) => f.floor) ?? [0]),
    deathFloor ?? 0,
  );

  const floorNumbers = Array.from({ length: floors }, (_, i) => i + 1);

  // Track deck additions by floor (rough: just count new cards per combat floor)
  const deckByFloor = new Map<number, string>();
  if (deckChanges && combats) {
    // Simple heuristic: assign deck_changes after starter cards to combat floors
    const starters = new Set(['Strike_P', 'Defend_P', 'Eruption', 'Vigilance']);
    const additions = deckChanges.filter((c) => !starters.has(c.replace('+', '')));
    let addIdx = 0;
    for (const combat of [...combats].sort((a, b) => a.floor - b.floor)) {
      if (addIdx < additions.length) {
        deckByFloor.set(combat.floor, `+${additions[addIdx]}`);
        addIdx++;
      }
    }
  }

  return (
    <div style={{
      background: '#161b22',
      border: '1px solid #30363d',
      borderRadius: 6,
      overflow: 'hidden',
    }}>
      {/* Column headers */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: '36px 24px 80px 1fr 60px 80px',
        padding: '6px 12px',
        fontSize: 9,
        color: '#484f58',
        textTransform: 'uppercase',
        letterSpacing: '0.5px',
        borderBottom: '1px solid #21262d',
        gap: 8,
      }}>
        <span>Floor</span>
        <span />
        <span>Type</span>
        <span>Enemy / Event</span>
        <span style={{ textAlign: 'right' }}>HP</span>
        <span>Card</span>
      </div>

      {/* Floor rows */}
      {floorNumbers.map((floor) => {
        const combat = combats?.find((c) => c.floor === floor);
        const room = inferRoomType(floor, combats, floorLog);
        const eventLabel = floorEventLabel(floor, floorLog);
        const icon = ROOM_ICONS[room];
        const isDeath = floor === deathFloor;
        const isExpanded = expandedFloor === floor;
        const cardAdded = deckByFloor.get(floor);

        return (
          <div key={floor}>
            <div
              onClick={() => setExpandedFloor(isExpanded ? null : floor)}
              style={{
                display: 'grid',
                gridTemplateColumns: '36px 24px 80px 1fr 60px 80px',
                padding: '4px 12px',
                alignItems: 'center',
                gap: 8,
                cursor: combat ? 'pointer' : 'default',
                background: isExpanded ? '#1c2128' : isDeath ? 'rgba(248,81,73,0.06)' : 'transparent',
                borderLeft: isDeath ? '2px solid #f85149' : isExpanded ? '2px solid #00ff41' : '2px solid transparent',
                transition: 'background 0.1s',
                fontSize: 12,
              }}
              onMouseEnter={(e) => {
                if (!isExpanded && !isDeath) (e.currentTarget as HTMLDivElement).style.background = '#1c2128';
              }}
              onMouseLeave={(e) => {
                if (!isExpanded && !isDeath) (e.currentTarget as HTMLDivElement).style.background = 'transparent';
              }}
            >
              {/* Floor number */}
              <span style={{
                fontWeight: 600,
                color: isDeath ? '#f85149' : '#c9d1d9',
                fontVariantNumeric: 'tabular-nums',
              }}>
                F{floor}
              </span>

              {/* Room icon */}
              <span style={{ color: icon.color, fontSize: 13, textAlign: 'center' }}>
                {isDeath ? 'X' : icon.symbol}
              </span>

              {/* Room type */}
              <span style={{ color: icon.color, fontSize: 11 }}>
                {room !== 'unknown' ? icon.label : '--'}
              </span>

              {/* Enemy / Event name */}
              <span style={{
                color: isDeath ? '#f85149' : '#c9d1d9',
                fontWeight: isDeath ? 600 : 400,
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                whiteSpace: 'nowrap',
              }}>
                {combat?.enemy ?? eventLabel ?? (room !== 'unknown' ? icon.label : '--')}
                {isDeath && ' [DEATH]'}
              </span>

              {/* HP lost */}
              <span style={{
                textAlign: 'right',
                color: combat ? (combat.hp_lost > 10 ? '#f85149' : combat.hp_lost > 0 ? '#d29922' : '#3fb950') : '#484f58',
                fontVariantNumeric: 'tabular-nums',
                fontSize: 11,
              }}>
                {combat ? (combat.hp_lost > 0 ? `-${combat.hp_lost}HP` : '0 HP') : ''}
              </span>

              {/* Card added */}
              <span style={{
                color: cardAdded ? '#00ff41' : '#484f58',
                fontSize: 11,
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                whiteSpace: 'nowrap',
              }}>
                {cardAdded ?? ''}
              </span>
            </div>

            {/* Expanded combat detail */}
            {isExpanded && combat && (
              <CombatDetail combat={combat as any} />
            )}
          </div>
        );
      })}
    </div>
  );
};
