# Wave 3 Recorder Requirements

These requirements were derived from offline intake of all 14 committed
recording bundles. They are operator work; engine agents must not edit or run
the Java harness.

## Immediate Corpus Repair

- Re-record or repair
  `-6356651387281996788-WATCHER-20260720-214546`: its gzip stream is truncated
  (33 records declared, only 3,075 of 3,108 corpus records are readable).
- Capture one pre-action checkpoint for every `RUN_START` and `RESUME`.
  Resume bundles without a save/checkpoint witness cannot be replayed.
- Keep action indices monotonic across sittings and include `sitting_id`,
  `parent_run_id`, and inherited action index.

## Initial Conditions

Emit a versioned `initial` envelope containing:

- seed, character, ascension, game version, loaded mods, daily/trial/custom
  mode flags, and final-act policy;
- profile/unlock inputs, NoteForYourself card/upgrade, boss-seen flags, and the
  resolved locked card/relic sets;
- ordered realized card/relic/event/shrine/encounter/boss pools and full map;
- ruby/emerald/sapphire key state;
- all 13 StS RNG states/counters plus ambient `MathUtils.random`
  `RandomXS128 {seed0,seed1}` and no-argument `Collections.shuffle`
  `java.util.Random` state, captured without initializing either stream.
- the process-global `TheBombPower.bombIdOffset` before the run and at every
  checkpoint where Bomb powers exist, so multiple `TheBomb0`, `TheBomb1`, ...
  identities can be compared without guessing prior process history.

For resume, include the lossless decrypted `SaveFile` witness and a stable
pre-action oracle state. `SaveFile` already exposes most persisted pools,
queues, keys, and RNG counters; ambient RNG must be emitted separately.

For new-run bundle metadata, emit optional `meta.profile` using this exact
version-1 contract. Every field inside a present profile is mandatory;
`locked_cards` and `locked_relics` must be emitted as explicit empty arrays for
an all-unlocked profile rather than omitted:

```json
{
  "v": 1,
  "note_for_yourself_card": "IronWave",
  "highest_unlocked_ascension": 20,
  "is_daily_run": false,
  "final_act_available": true,
  "bosses_seen": ["GUARDIAN", "GHOST", "SLIME", "CHAMP", "AUTOMATON", "COLLECTOR", "CROW", "DONUT", "WIZARD"],
  "locked_cards": [],
  "locked_relics": []
}
```

`locked_cards` and `locked_relics` are the resolved contents of
`UnlockTracker.lockedCards` and `UnlockTracker.lockedRelics` after refresh, not
unlock-level guesses. The Rust ordinary API keeps missing fields compatible as
all-unlocked, but the bundle comparator quarantines every recording that omits
`meta.profile`; none of the 14 legacy bundles is initialization-certified.
Use a trailing `+` in `note_for_yourself_card` for the upgraded stored card,
matching the engine's canonical card-instance string.

## Canonical Action Payloads

- Neow: emit stage (`intro|choice|continue`), mode
  (`two_choices|four_choices`), boss count, all ordered
  `{index,category,reward_id,drawback_id,label}` options, and selected payload.
  Only the semantic choice is a canonical action.
- Card rewards: emit choose, skip, Singing Bowl, and leave actions with
  `item_index`, `choice_index`, card ID, upgrades, and card-instance identity.
- Smith, Toke, shop removal, and event grids: commit at selection completion
  with deck index, card ID, upgrades/misc, and card-instance identity.
- Shops/rewards: emit authoritative offer/item indices and retain IDs as
  validators. Duplicate IDs must never be resolved by first match.
- Boss relics: emit the canonical item-select and option-select transitions,
  or define a versioned one-record compound action.
- Emit omitted `LeaveRewards`, `ShopLeave`, `LeaveChest`, `Proceed`, combat
  `Choose`/`ConfirmSelection`, potion choices, and every nested event decision.

## Causal Checkpoints

- Commit only after the action queue and screen transition settle.
- Do not attach an enemy turn to the preceding card. In the current corpus a
  `PLAY_CARD` immediately before `END_TURN` can contain the exact same
  post-end-turn state as the end-turn record.
- Do not capture `PATH` before opening-hand draw and first intent creation.
- Do not capture a lethal card before reward generation while capturing the
  following reward action afterward.
- Distilled Chaos must remain one semantic `USE_POTION` action. Suppress its
  internal `PLAY_CARD` recorder callbacks, or annotate them with authoritative
  origin/provenance. Offline compatibility recognizes at most three callbacks
  only when they use `hand_idx: -1`, are rooted immediately in the Distilled
  Chaos callback chain, and every recorded state field is identical to the
  potion's settled checkpoint; any weaker pattern remains a schema failure.
- Emit canonical decision phase rather than Java `RoomPhase`; report
  non-combat energy as `0` instead of stale `EnergyPanel.totalCount`.
- Emit key ownership and semantic Neow state on every oracle record.
- Emit the two-word `MathUtils.random` state and raw 48-bit default
  `Collections.shuffle` state on every settled checkpoint, not only at run
  start. Java shares ambient randomness with frame-driven animation, audio,
  and dialogue, so initial state cannot prove the later semantic draw cursor.

## Current Offline Prefixes

As of this request, the direct-reward A0 victory
`-5884681071377138867-WATCHER-20260720-194423` compares all represented fields
through 51/607 actions (42 direct checkpoints) and then stops at a relic
identity whose pool cannot be certified without the missing profile snapshot.
The other four A0 terminal bundles compare through 2 actions and stop on an
unrecorded Neow grid card selection. All five reports carry the initialization
quarantine; no divergence mask was created for recorder data loss.
