# Ultra-Granular Work Units: Rewards

## Model-facing actions (no UI)
- [ ] Expose all reward decisions as explicit action objects with required params. (action: pick_card{card_reward_index,card_index})
- [ ] If parameters are missing, return action lists instead of failing. (action: pick_card{card_reward_index,card_index})

## Action tags
Use explicit signatures on each item (see `granular-actions.md`).

## Core reward action flow
- [ ] Wire `RewardHandler.get_available_actions()` into GameRunner reward phase. (action: none{})
- [ ] Wire `RewardHandler.execute_action()` into GameRunner reward phase. (action: none{})
- [ ] Ensure gold action is included for logging, even if auto-claimed. (action: claim_gold{})

## Card rewards
- [ ] PickCardAction updates deck and resolves reward. (action: pick_card{card_reward_index,card_index})
- [ ] SkipCardAction marks reward skipped. (action: skip_card{card_reward_index})
- [ ] SingingBowlAction grants +2 max HP and resolves reward. (action: singing_bowl{card_reward_index})
- [ ] Card reward count respects Prayer Wheel and Busted Crown. (action: none{})

## Potion rewards
- [ ] ClaimPotionAction adds potion to slot and resolves reward. (action: claim_potion{potion_reward_index?})
- [ ] SkipPotionAction marks reward skipped. (action: skip_potion{potion_reward_index?})
- [ ] Enforce Sozu (no potion rewards). (action: none{})
- [ ] Enforce White Beast Statue (guaranteed potion). (action: none{})

## Relic rewards
- [ ] ClaimRelicAction adds relic and resolves reward. (action: claim_relic{relic_reward_index?})
- [ ] Elite rewards generate relics using relicRng parity. (action: none{})
- [ ] Black Star grants a second elite relic reward. (action: none{})

## Emerald key
- [ ] ClaimEmeraldKeyAction obtains key and resolves reward. (action: claim_emerald_key{})
- [ ] SkipEmeraldKeyAction resolves reward without key. (action: skip_emerald_key{})

## Boss relics
- [ ] PickBossRelicAction chooses exactly one relic and resolves reward. (action: pick_boss_relic{relic_index})
- [ ] Offer explicit skip boss relic action (or Proceed option) so models can decline. (action: skip_boss_relic{})
- [ ] Proceeding allowed when boss relic is either chosen or explicitly skipped. (action: proceed_from_rewards{})

## Proceed action
- [ ] ProceedFromRewardsAction only available when mandatory rewards resolved. (action: proceed_from_rewards{})
- [ ] Proceed action returns to map phase. (action: proceed_from_rewards{})

## Model-traversable actions (no UI)
- [ ] Reward choices should be emitted as a structured action list with required parameters (e.g., relic index or skip). (action: pick_card{card_reward_index,card_index})

## Tests
- [ ] Add tests for each action type (claim/skip/proceed). (action: none{})
- [ ] Add tests for boss relic gating + emerald key skip. (action: none{})

## Failed-tests mapping (2026-02-04)
- [ ] Expose a public `RewardHandler.execute_action(run_state, rewards, action)` API (or alias) used by tests. (action: none{})
- [ ] Ensure `execute_action` handles Claim/Skip/Proceed actions with a `{"success": True}` result payload. (action: none{})

## Skipped-test mapping (2026-02-04)
- [ ] `predict_card_reward` parity: fill expected values for known seed(s) in `tests/test_rng_parity.py` and remove the skip. (action: none{})
