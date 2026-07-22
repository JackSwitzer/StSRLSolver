# Oracle State Schema v2

**Schema identity:** `sts.oracle_state` major `2`, minor `2`

**Rust owner:** `packages/engine-rs/src/trace/oracle_v2.rs`

This is the language-neutral post-action state contract for Java/Rust parity.
It contains gameplay identities and values only. `CoreCheckpoint` remains the
private Rust continuation format and must never be required from a Java trace.

## Required Shape

```json
{
  "schema": {"name": "sts.oracle_state", "major": 2, "minor": 2},
  "floor": 1,
  "act": 1,
  "turn": 1,
  "phase": "COMBAT",
  "map": {"x": 0, "y": 0},
  "keys": {"ruby": false, "emerald": false, "sapphire": false},
  "player": {
    "hp": 72,
    "max_hp": 72,
    "block": 0,
    "energy": 3,
    "stance": "Neutral",
    "gold": 99,
    "powers": [{"id": "Strength", "amt": 2}],
    "orbs": []
  },
  "enemies": [{
    "id": "JawWorm",
    "idx": 0,
    "dead": false,
    "hp": 42,
    "max_hp": 42,
    "block": 0,
    "intent": {"move_id": 3, "name": "CHOMP", "dmg": 12, "hits": 1},
    "powers": [],
    "move_history": [3]
  }],
  "piles": {
    "hand": ["Miracle", "Strike_P"],
    "draw_ordered": ["Vigilance"],
    "discard": [],
    "exhaust": []
  },
  "deck": ["Strike_P", "Defend_P", "Eruption", "Vigilance"],
  "relics": [{"id": "PureWater", "counter": -1}],
  "potions": ["Potion Slot", "Potion Slot", "Potion Slot"],
  "rng": {
    "card": 0,
    "monster": 36,
    "event": 0,
    "relic": 5,
    "treasure": 0,
    "potion": 0,
    "merchant": 0,
    "monsterHp": 1,
    "ai": 1,
    "shuffle": 1,
    "cardRandom": 0,
    "misc": 2,
    "map": 97,
    "ambientMath": {
      "seed0": "0123456789abcdef",
      "seed1": "fedcba9876543210"
    },
    "javaCollections": "123456789abc",
    "rawStates": {
      "card": {"seed0": "0123456789abcdef", "seed1": "fedcba9876543210", "counter": 0},
      "monster": {"seed0": "0123456789abcdef", "seed1": "fedcba9876543210", "counter": 36},
      "event": {"seed0": "0123456789abcdef", "seed1": "fedcba9876543210", "counter": 0},
      "relic": {"seed0": "0123456789abcdef", "seed1": "fedcba9876543210", "counter": 5},
      "treasure": {"seed0": "0123456789abcdef", "seed1": "fedcba9876543210", "counter": 0},
      "potion": {"seed0": "0123456789abcdef", "seed1": "fedcba9876543210", "counter": 0},
      "merchant": {"seed0": "0123456789abcdef", "seed1": "fedcba9876543210", "counter": 0},
      "monsterHp": {"seed0": "0123456789abcdef", "seed1": "fedcba9876543210", "counter": 1},
      "ai": {"seed0": "0123456789abcdef", "seed1": "fedcba9876543210", "counter": 1},
      "shuffle": {"seed0": "0123456789abcdef", "seed1": "fedcba9876543210", "counter": 1},
      "cardRandom": {"seed0": "0123456789abcdef", "seed1": "fedcba9876543210", "counter": 0},
      "misc": {"seed0": "0123456789abcdef", "seed1": "fedcba9876543210", "counter": 2},
      "map": {"seed0": "0123456789abcdef", "seed1": "fedcba9876543210", "counter": 97},
      "neow": {"seed0": "0123456789abcdef", "seed1": "fedcba9876543210", "counter": 5}
    }
  },
  "processGlobals": {
    "theBombIdOffset": 0
  },
  "neow": null
}
```

All fields above are required except `neow`, which is present only while a
Neow decision or selected Neow witness is relevant. Unknown additive fields at
the current minor may be ignored. An unknown name, major, or future minor is an
error. A missing required RNG field is always an error; counters never default
to zero.

## Ordering And Identity

- Card IDs include `+` when upgraded. Piles and master deck preserve Java list
  order; the draw-pile front is the next card Java will draw.
- Enemy `idx`, relic order, potion slots, powers, orbs, and move history remain
  ordered. Consumers must not sort these arrays before comparison.
- Power entries use exact Java `AbstractPower.ID` strings and Java-visible
  amounts. Status-backed powers preserve serialized insertion order and are
  stably ordered by Java priority for `ApplyPowerAction` semantics. Dynamic
  `TheBomb{serial}` entries participate in that same order and expose their
  independent remaining-turn amounts; their damage payload remains causal
  checkpoint state rather than public oracle state.
- Empty potion slots use Java's stable `Potion Slot` ID.
- Stance names are `Neutral`, `Wrath`, `Calm`, or `Divinity`.
- Phase values are `NEOW`, `MAP`, `CHEST`, `COMBAT`, `REWARD`, `CAMPFIRE`,
  `SHOP`, `EVENT`, `TRANSITION`, and `GAME_OVER`.
- RNG state contains the seven persistent, five floor-local, and one map
  stream counters plus both process-global RNG generators. `rawStates` repeats each
  counted stream with its two `RandomXS128` words and includes the separate
  Neow stream; this is required because rejection retries can advance raw state
  without adding wrapper counter ticks. Signed Java `int` counter values are
  preserved in `i64`. Raw `RandomXS128` and 48-bit Java LCG
  state use fixed-width lowercase hexadecimal strings so no JSON implementation
  can round their bit patterns through a floating-point number.
- `processGlobals.theBombIdOffset` is Java's signed static constructor suffix
  for the next `TheBombPower`. It is checkpoint-causal, persists across combat
  and run resets in the same process, and increments with Java `int` wrapping.

## Neow Witness

Rust intentionally exposes the four seeded blessing categories on every run.
Index alone is therefore not an oracle. The optional Neow state is:

```json
{
  "mode": "four_choices",
  "options": [{
    "category": 0,
    "reward_id": "THREE_CARDS",
    "drawback_id": "NONE",
    "label": "Choose 1 of 3 cards to obtain."
  }],
  "selected": null
}
```

`reward_id` and `drawback_id` are exact Java `NeowRewardType` and
`NeowRewardDrawback` enum names. A transition selecting Neow retains the four
pre-action options and places the selected payload in `selected` until the next
canonical action.

The current human recorder emits three UI-level `NEOW` commits and only their
button indices. It does not yet emit option payloads or an explicit
two-choice/four-choice mode. Bundle intake must report those fields as absent;
it may not infer them from labels or silently map all three commits to engine
choices.

## Comparison Contract

RNG counters compare first, followed by every other field in deterministic
path order. Array paths use zero-based brackets, for example
`enemies[0].intent.move_id`. A missing field is reported as absent rather than
coerced to a default. During recorder migration, bundle reports count skipped
absent fields explicitly; a `match` claim is valid only for fields that both
producers emitted.

`test_oracle_state_v2` proves mandatory name/major/minor validation, round-trip
serialization, all 13 named counters and both required process-global states,
the required Bomb constructor offset,
projected semantic Neow selection,
and one-field corruption across
run identity, map/keys, player, powers/orbs, enemies/intents/history, piles,
deck, relics, potions, and RNG.

## Current Recorder Coverage

The committed record-mode bundles already emit floor, act, turn, phase, map,
player, enemies (including `dead`), ordered piles, deck, relic counters,
potions, and all 13 counters. They do not emit key ownership, semantic Neow
payloads, either process-global RNG state, or the Bomb constructor offset. Profile/unlock inputs are also
absent from `meta.json`. Bundle replay therefore reports the missing values as
unverified leaves; strict canonical-v2 intake rejects them rather than guessing
seed zero.

The Rust projection maps common static status-backed powers to Java IDs,
filters private counters, derives compound amounts, distinguishes green
`Energized` from blue `EnergizedBlue`, and emits typed Minion, BackAttack,
Stasis, Pen Nib, and independent `TheBomb{serial}` entries in canonical power
order. Java certification still requires recorder-side Bomb offset capture and
the remaining combat-start/action-queue ordering audit; fields absent from the
legacy corpus must not be credited as Java-certified.

## Legacy Recording-Bundle Adapter

`trace_replay --bundle <dir> --diff <report.json>` reads the current
`meta.json + script.jsonl + trace.jsonl.gz` bundle shape. It validates gzip,
JSON, versions, contiguous indices, declared record counts, and script/trace
action-type alignment before replay. Schema/framing failures are errors;
gameplay or action-mapping differences produce a first-divergence report.

The recorder dialect is not canonical v2. The adapter therefore applies only
these explicit translations and reports coupled or semantically unverified
actions separately from direct checkpoints:

- A fresh run's three Neow commits are treated as intro, semantic selection,
  and continue. For a `GRID`, the adapter opens the unique typed reward and
  tests every legal card choice against the recorded ordered deck. A unique
  result is inferred explicitly; equivalent duplicate removals remain marked
  identity-uncertified. Semantic Neow payload coverage remains skipped.
- Java room phases are mapped only when unambiguous. `COMPLETE` and
  `INCOMPLETE` are not coerced into decision phases.
- `EnergyPanel.totalCount` outside combat is stale UI state and is skipped.
- A `PATH` combat-entry hook fires before opening-hand and intent-display
  settlement. Encounter identity, HP, moves, powers, relics, deck, and RNG
  still compare; opening piles, energy, and transient `DEBUG` intent names do
  not.
- When a `PLAY_CARD` record and immediately following `END_TURN` record carry
  identical post-state, the card-only checkpoint is missing. The card state is
  counted as skipped and the end-turn state must match after both canonical
  actions execute.
- A lethal combat record captured before reward settlement omits the reward
  transition. Combat-result fields compare there; reward phase/RNG compare on
  the next reward record. Java's stale dead-enemy/pile state on reward screens
  is excluded from the run-owned checkpoint.
- Non-attack `intentDmg` is skipped for legacy records because Java leaves the
  previous attack value there; canonical v2 uses `-1`.
- Concatenated gzip members are all decoded, and script/trace action payloads
  must match exactly after removing the script-only `idx` field.
- Recorder IDs may identify a shop/reward candidate only when the match is
  unique. Duplicate IDs fail until the recorder emits authoritative indices.

The recorder currently omits card-reward skip/leave actions, Neow grid card
identity, Smith and shop-removal card identity, several run-level grid choices,
and lossless boss-relic staging. These are hard action-mapping divergences, not
implicit policy choices.

A bundle report is `match` only when initialization is authoritative and every
action has its own complete semantic checkpoint. Missing profile state,
inferred or unverified actions, coupled checkpoints, ignored recorder
callbacks, or skipped fields produce the explicit `uncertified` status. The CLI
writes that report and exits as an evidence error rather than returning a false
match.
