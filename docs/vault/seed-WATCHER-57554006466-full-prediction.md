# WATCHER seed 57554006466 — smoke-golden spot checks

The historical filename is retained for links from docs/goal/UNITS.md. This is
not a full-run prediction. It is a concise index of facts in the committed
TraceLab golden and a candidate starting point for a larger corpus.

## Provenance and scope

- Script: data/traces/scripts/smoke-neow-floor1.json
- Golden: data/traces/java/smoke-neow-floor1.jsonl
- Character: WATCHER
- Ascension: 0
- Game version recorded by the golden: 2022-12-18
- Recorded actions: Neow choice 1, path choice 0, then three END_TURN actions
- End status: script_exhausted during floor-1 combat

## Committed spot checks

| Record | Floor/turn | Player | Enemy state after action | Selected RNG counters |
|---:|---|---|---|---|
| 0 | floor 0 / turn 0 | 79/79 HP, 99 gold, PureWater | no enemies | map 94, monster 38, relic 5, misc 1 |
| 1 | floor 1 / turn 1 | 79/79 HP | JawWorm 40/40; move history [1] | ai 1, monsterHp 1, shuffle 1 |
| 2 | floor 1 / turn 2 | 68/79 HP | JawWorm 40/40; next move 3, ATTACK_DEFEND, 7 damage | ai 2, shuffle 1 |
| 3 | floor 1 / turn 3 | 61/79 HP | JawWorm 40/40, 5 block; next move 2, DEFEND_BUFF | ai 3, shuffle 2 |
| 4 | floor 1 / turn 4 | 61/79 HP | JawWorm 40/40, 6 block, Strength 3; next move 1, ATTACK, 14 damage | ai 5, shuffle 2 |

The golden also records ordered draw, hand, discard, and exhaust piles, all
relic and potion slots, intents, move history, and all 13 RNG counters at each
action boundary. Read the JSONL directly for exact order-sensitive assertions.

## Corpus-candidate status

This seed is suitable for extending beyond the smoke path because its first
transition and several Jaw Worm rolls are already frozen. It does not currently
prove rewards, later map choices, events, shops, elites, bosses, later acts, or
A20 rules. Any extension requires a new deterministic script and a human-minted
golden; do not reconstruct later floors from this document.
