interface DeckDisplayProps {
  cards: string[];
}

type CardType = 'attack' | 'skill' | 'power' | 'status' | 'curse';

function classifyCard(name: string): CardType {
  const lower = name.toLowerCase();

  // Status/curse
  const curses = [
    'ascender', 'clumsy', 'decay', 'doubt', 'injury', 'normality',
    'pain', 'parasite', 'regret', 'shame', 'writhe', 'necronomicurse',
    'curse of the bell',
  ];
  if (curses.some((c) => lower.includes(c))) return 'curse';

  const statuses = ['burn', 'dazed', 'slimed', 'void', 'wound'];
  if (statuses.some((s) => lower === s)) return 'status';

  // Powers
  const powers = [
    'mental fortress', 'rushdown', 'like water', 'fasting',
    'battle hymn', 'deva form', 'devotion', 'establishment',
    'master reality', 'study', 'nirvana', 'omega',
    'demon form', 'barricade', 'inflame', 'metallicize',
    'feel no pain', 'corruption', 'dark embrace', 'evolve',
    'fire breathing', 'juggernaut', 'rupture', 'combust',
    'brutality', 'berserk',
  ];
  if (powers.some((p) => lower.includes(p))) return 'power';

  // Skills
  const skills = [
    'defend', 'vigilance', 'protect', 'meditate', 'inner peace',
    'halt', 'perseverance', 'sanctity', 'third eye', 'wave of the hand',
    'empty body', 'empty mind', 'evaluate', 'crescendo', 'tranquility',
    'prostrate', 'pray', 'worship', 'collect', 'conjure blade',
    'deceive reality', 'foresight', 'foreign influence', 'judgment',
    'scrawl', 'spirit shield', 'vault', 'wish', 'wreath of flame',
    'true grit', 'shrug it off', 'impervious', 'ghostly armor',
    'flame barrier', 'power through', 'second wind', 'sentinel',
    'warcry', 'rage', 'seeing red', 'bloodletting', 'dual wield',
    'offering', 'exhume', 'limit break', 'spot weakness',
  ];
  if (skills.some((s) => lower.includes(s))) return 'skill';

  // Default to attack for anything else (Strike, Bash, etc.)
  return 'attack';
}

const TYPE_COLORS: Record<CardType, { bg: string; text: string; border: string }> = {
  attack: { bg: '#2a1515', text: '#ff6666', border: '#4a2020' },
  skill: { bg: '#151f2a', text: '#6699ff', border: '#203040' },
  power: { bg: '#2a2515', text: '#e3b341', border: '#403a15' },
  status: { bg: '#1a1a1a', text: '#6e7681', border: '#30363d' },
  curse: { bg: '#1a1a1a', text: '#6e7681', border: '#30363d' },
};

function countCards(cards: string[]): Array<{ name: string; count: number }> {
  const counts = new Map<string, number>();
  for (const card of cards) {
    counts.set(card, (counts.get(card) ?? 0) + 1);
  }
  // Sort: powers first, then skills, then attacks, then status/curse
  const order: Record<CardType, number> = { power: 0, skill: 1, attack: 2, status: 3, curse: 4 };
  return Array.from(counts.entries())
    .map(([name, count]) => ({ name, count }))
    .sort((a, b) => {
      const ta = order[classifyCard(a.name)];
      const tb = order[classifyCard(b.name)];
      if (ta !== tb) return ta - tb;
      return a.name.localeCompare(b.name);
    });
}

export const DeckDisplay: React.FC<DeckDisplayProps> = ({ cards }) => {
  const grouped = countCards(cards);

  return (
    <div
      style={{
        display: 'flex',
        flexWrap: 'wrap',
        gap: 4,
        padding: 8,
        background: '#161b22',
        border: '1px solid #30363d',
        borderRadius: 4,
      }}
    >
      {grouped.length === 0 && (
        <span style={{ color: '#484f58', fontSize: 12, fontStyle: 'italic' }}>
          No cards
        </span>
      )}
      {grouped.map(({ name, count }) => {
        const type = classifyCard(name);
        const colors = TYPE_COLORS[type];
        return (
          <span
            key={name}
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: 4,
              padding: '2px 8px',
              background: colors.bg,
              border: `1px solid ${colors.border}`,
              borderRadius: 3,
              fontSize: 12,
              color: colors.text,
              lineHeight: '18px',
              maxWidth: 120,
              whiteSpace: 'nowrap',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
            }}
            title={`${count}x ${name}`}
          >
            {count > 1 && (
              <span style={{ color: '#8b949e', fontSize: 10, fontWeight: 600 }}>
                {count}x
              </span>
            )}
            {name}
          </span>
        );
      })}
    </div>
  );
};
