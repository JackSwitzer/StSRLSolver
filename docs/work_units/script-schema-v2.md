# Action Script Schema v2

**Status:** frozen Rust contract; Java recorder support and full-run mint pending

Schema v2 uses the simulator's canonical `GameAction` directly. There is no
trace-only action enum and no index-remapping adapter. A script is a JSON object
with this shape:

```json
{
  "schema": {"name": "sts.trace", "major": 2, "minor": 0},
  "trace_id": "watcher-a0-example",
  "seed": "WATCHER",
  "seed_long": 57554006466,
  "character": "WATCHER",
  "ascension": 0,
  "actions": [
    {"ChooseNeowOption": 1},
    {"ChoosePath": 0},
    {"CombatAction": {"PlayCard": {"card_idx": 2, "target_idx": 0}}},
    {"CombatAction": "EndTurn"}
  ]
}
```

`seed_long` is the signed Java `long` representation of the seed bits. The
display seed and signed value must decode to the same 64-bit pattern. Indices
are zero-based and always address the ordered choices in the transition's
pre-action state. Scripts do not contain stop conditions: exhausting the action
list produces an `incomplete` end record, while a terminal transition produces
`victory` or `defeat`. Actions after a terminal transition and illegal actions
are script errors.

## Run Actions

| Rust/JSON variant | JSON payload | Java decision surface |
| --- | --- | --- |
| `ChooseNeowOption` | option index | `NeowEvent` option buttons |
| `ChoosePath` | available-path index | `DungeonMapScreen` path choice |
| `OpenChest` | none | `TreasureRoom` chest open |
| `LeaveChest` | none | chest leave/skip transition |
| `SelectRewardItem` | ordered reward-item index | `CombatRewardScreen` reward row |
| `ChooseRewardOption` | `{item_index, choice_index}` | nested card/boss-relic/purge choice opened by a reward |
| `SkipRewardItem` | ordered reward-item index | skip one skippable reward row |
| `LeaveRewards` | none | leave the current reward screen |
| `Proceed` | none | explicit room/act transition confirmation |
| `CampfireRest` | none | `CampfireRestOption` |
| `CampfireUpgrade` | master-deck index | `CampfireSmithOption` card selection |
| `CampfireToke` | none | `CampfireTokeOption`; follow-up card selection is a reward choice |
| `CampfireLift` | none | `CampfireLiftOption` |
| `CampfireDig` | none | `CampfireDigOption` |
| `CampfireRecall` | none | `CampfireRecallOption` |
| `ShopBuyCard` | merchant card-slot index | `ShopScreen` colored/colorless card slots |
| `ShopBuyRelic` | merchant relic-slot index | `ShopScreen` relic slots |
| `ShopBuyPotion` | merchant potion-slot index | `ShopScreen` potion slots |
| `ShopRemoveCard` | master-deck index | `ShopScreen` purge selection |
| `ShopLeave` | none | leave `ShopScreen` |
| `EventChoice` | current event-option index | `GenericEventDialog` option buttons |
| `CombatAction` | canonical combat action | combat hand, potion, selection, or end-turn input |
| `UsePotion` | potion-slot index | legal non-combat potion use |
| `DiscardPotion` | potion-slot index | potion discard confirmation |

## Combat Actions

The `CombatAction` payload is the canonical combat `Action` enum:

| Variant | JSON payload | Meaning |
| --- | --- | --- |
| `PlayCard` | `{card_idx, target_idx}` | play a hand card; `target_idx` is `-1` when untargeted |
| `UsePotion` | `{potion_idx, target_idx}` | use a combat potion; `target_idx` is `-1` when untargeted |
| `EndTurn` | none | end the player turn |
| `Choose` | current generated-choice index | resolve an in-combat generated-card or target choice |
| `ConfirmSelection` | none | commit the active multi-selection |

## Trace Framing

Rust replay emits one `header` envelope, one `transition` envelope per accepted
action, and one `end` envelope. Each transition contains the exact canonical
action plus complete pre/post `CoreCheckpoint`s. Consecutive transitions must
form a causal chain (`previous.post == next.pre`). Replaying the same script
twice must produce byte-identical serialized envelopes.

The v2 Rust checkpoint is a deterministic continuation artifact, not yet a
claim that the Java mod can serialize Rust's private engine representation.
Cross-language certification continues to use the shared oracle state fields
and RNG counters until a language-neutral full-state projection is frozen.
Human minting therefore remains blocked on a Java v2 action adapter plus the
shared projection, not on Rust action expressibility.

The Rust CLI replays this format without a golden:

```bash
cargo run --manifest-path packages/engine-rs/Cargo.toml --bin trace_replay -- \
  --script data/traces/scripts/smoke-v2-neow-floor1.json \
  --out /tmp/smoke-v2-neow-floor1.jsonl
```

Passing `--java-trace`, `--diff`, or `--masks` with a v2 script is rejected
until the language-neutral comparison projection is implemented. This guard
prevents a Rust `CoreCheckpoint` self-replay from being misreported as Java
certification.

## Intentional Deviation

Rust always exposes all four seeded Neow options. A Java oracle recording must
record the four generated option payloads and the chosen payload, rather than
assuming that visible option index alone has the same meaning under Java's
progression-gated two-option screen.

## Compatibility

Schema v1 remains read-only for the existing smoke trace. V2 readers reject an
unknown schema name, major, or future minor. Unknown additive fields at the
current minor are ignored by Serde. The exhaustive contract test in
`test_trace_schema_v2.rs` fails when a serialized `GameAction` variant is not
listed in this document.
