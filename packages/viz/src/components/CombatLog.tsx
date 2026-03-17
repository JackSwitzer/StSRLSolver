import type { CombatSummary } from '../types/training';

interface TurnDetail {
  turn: number;
  cards: string[];
}

interface CombatLogProps {
  combat: CombatSummary & { turns_detail?: TurnDetail[] };
  floor: number;
}

// Guess card color from name
function cardColor(name: string): string {
  const lower = name.toLowerCase();
  // Attacks: anything with strike, bash, eruption, ragnarok, tantrum, etc.
  const attackWords = [
    'strike', 'bash', 'eruption', 'ragnarok', 'tantrum', 'conclude',
    'searing', 'carnage', 'bludgeon', 'pommel', 'iron wave',
    'flying sleeves', 'flurry', 'wallop', 'windmill', 'wheel kick',
    'crush joints', 'cut through', 'follow up', 'reach heaven',
    'sands of time', 'signature move', 'talk to the hand',
    'weave', 'empty fist', 'bowling bash', 'brilliance',
    'carve reality', 'consecrate', 'just lucky', 'pressure points',
    'sash whip', 'fear no evil',
  ];
  if (attackWords.some((w) => lower.includes(w))) return '#ff4444';

  // Skills: defend, block-related, vigilance, protect, etc.
  const skillWords = [
    'defend', 'vigilance', 'protect', 'meditate', 'inner peace',
    'halt', 'perseverance', 'sanctity', 'third eye', 'wave of the hand',
    'empty body', 'empty mind', 'evaluate', 'crescendo', 'tranquility',
    'swivel', 'windmill strike', 'prostrate', 'pray', 'worship',
    'collect', 'conjure blade', 'deceive reality', 'devotion',
    'establishment', 'foresight', 'foreign influence', 'judgment',
    'scrawl', 'simmering fury', 'spirit shield', 'vault',
    'wish', 'wreath of flame',
  ];
  if (skillWords.some((w) => lower.includes(w))) return '#4488ff';

  // Powers
  const powerWords = [
    'mental fortress', 'rushdown', 'like water', 'fasting',
    'battle hymn', 'deva form', 'devotion', 'establishment',
    'master reality', 'study', 'nirvana', 'omega',
    'demon form', 'barricade', 'inflame', 'metallicize',
  ];
  if (powerWords.some((w) => lower.includes(w))) return '#e3b341';

  return '#c9d1d9';
}

// Detect stance hint from card name
function stanceHint(name: string): string | null {
  const lower = name.toLowerCase();
  if (lower.includes('eruption') || lower.includes('tantrum') || lower.includes('simmering'))
    return 'Wrath';
  if (lower.includes('vigilance') || lower.includes('empty body') || lower.includes('fear no evil'))
    return 'Calm';
  if (lower.includes('worship') || lower.includes('prostrate'))
    return 'Mantra';
  if (lower.includes('crescendo'))
    return 'Wrath';
  if (lower.includes('tranquility') || lower.includes('inner peace'))
    return 'Calm';
  return null;
}

export const CombatLog: React.FC<CombatLogProps> = ({ combat, floor }) => {
  const { enemy, turns, hp_lost, damage_dealt, used_potion, stances, turns_detail } = combat;

  return (
    <div
      style={{
        background: '#161b22',
        border: '1px solid #30363d',
        borderRadius: 4,
        padding: 12,
        fontFamily: 'inherit',
        fontSize: 13,
      }}
    >
      {/* Header */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: 8,
          paddingBottom: 6,
          borderBottom: '1px solid #21262d',
        }}
      >
        <span style={{ color: '#c9d1d9', fontWeight: 600 }}>
          F{floor} vs {enemy}
        </span>
        <div style={{ display: 'flex', gap: 12, fontSize: 12, color: '#8b949e' }}>
          <span>{turns} turns</span>
          <span style={{ color: '#f85149' }}>-{hp_lost} HP</span>
          <span style={{ color: '#00ff41' }}>{damage_dealt} dmg</span>
          {used_potion && <span style={{ color: '#d2a8ff' }}>potion</span>}
        </div>
      </div>

      {/* Stances summary */}
      {stances && Object.keys(stances).length > 0 && (
        <div style={{ display: 'flex', gap: 8, marginBottom: 8, fontSize: 11 }}>
          {Object.entries(stances).map(([stance, count]) => (
            <span key={stance} style={{ color: '#8b949e' }}>
              {stance}: {count}x
            </span>
          ))}
        </div>
      )}

      {/* Turn-by-turn detail */}
      {turns_detail && turns_detail.length > 0 ? (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
          {turns_detail.map((t) => (
            <div key={t.turn} style={{ display: 'flex', gap: 6, alignItems: 'baseline' }}>
              <span
                style={{
                  color: '#484f58',
                  fontSize: 11,
                  minWidth: 28,
                  fontVariantNumeric: 'tabular-nums',
                }}
              >
                T{t.turn}
              </span>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
                {t.cards.map((card, i) => {
                  const hint = stanceHint(card);
                  return (
                    <span key={`${t.turn}-${i}`}>
                      <span style={{ color: cardColor(card), fontSize: 12 }}>{card}</span>
                      {hint && (
                        <span style={{ color: '#484f58', fontSize: 10 }}>{'\u2192'}{hint}</span>
                      )}
                      {i < t.cards.length - 1 && (
                        <span style={{ color: '#30363d' }}> / </span>
                      )}
                    </span>
                  );
                })}
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div style={{ color: '#484f58', fontSize: 12, fontStyle: 'italic' }}>
          No turn detail available
        </div>
      )}
    </div>
  );
};
