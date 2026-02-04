# Ultra-Granular Work Units: Events

## Model-facing actions (no UI)
- [ ] Expose event choices as explicit `event_choice` actions with required params. (action: event_choice{choice_index})
- [ ] If a choice requires card selection, emit follow-up actions listing valid card indices. (action: event_choice{choice_index} + select_cards{pile:offer,card_indices})

## Action tags
Use explicit signatures on each item (see `granular-actions.md`).

## Missing handlers
- [ ] GremlinMatchGame - add handler + registry entry. (action: none{})
- [ ] GremlinWheelGame - add handler + registry entry. (action: none{})
- [ ] NoteForYourself - add definition to handler pools + handler + registry. (action: none{})

## Missing choice generators (Act 1)
- [ ] DeadAdventurer - _get_choices implementation. (action: event_choice{choice_index})
- [ ] Mushrooms - _get_choices implementation. (action: event_choice{choice_index})
- [ ] ShiningLight - _get_choices implementation. (action: event_choice{choice_index})
- [ ] Sssserpent - _get_choices implementation. (action: event_choice{choice_index})
- [ ] WingStatue - _get_choices implementation. (action: event_choice{choice_index})

## Missing choice generators (Act 2)
- [ ] Addict - _get_choices implementation. (action: event_choice{choice_index})
- [ ] Augmenter - _get_choices implementation. (action: event_choice{choice_index})
- [ ] BackToBasics - _get_choices implementation. (action: event_choice{choice_index})
- [ ] Beggar - _get_choices implementation. (action: event_choice{choice_index})
- [ ] CursedTome - _get_choices implementation. (action: event_choice{choice_index})
- [ ] ForgottenAltar - _get_choices implementation. (action: event_choice{choice_index})
- [ ] Ghosts - _get_choices implementation. (action: event_choice{choice_index})
- [ ] Nest - _get_choices implementation. (action: event_choice{choice_index})
- [ ] Vampires - _get_choices implementation. (action: event_choice{choice_index})

## Missing choice generators (Act 3)
- [ ] Falling - _get_choices implementation. (action: event_choice{choice_index})
- [ ] MoaiHead - _get_choices implementation. (action: event_choice{choice_index})
- [ ] MysteriousSphere - _get_choices implementation. (action: event_choice{choice_index})
- [ ] SecretPortal - _get_choices implementation. (action: event_choice{choice_index})
- [ ] SensoryStone - _get_choices implementation. (action: event_choice{choice_index})
- [ ] TombOfLordRedMask - _get_choices implementation. (action: event_choice{choice_index})
- [ ] WindingHalls - _get_choices implementation. (action: event_choice{choice_index})

## Missing choice generators (Shrines)
- [ ] GremlinMatchGame - _get_choices implementation. (action: event_choice{choice_index})
- [ ] GremlinWheelGame - _get_choices implementation. (action: event_choice{choice_index})

## Missing choice generators (Special)
- [ ] AccursedBlacksmith - _get_choices implementation. (action: event_choice{choice_index})
- [ ] BonfireElementals - _get_choices implementation. (action: event_choice{choice_index})
- [ ] Designer - _get_choices implementation. (action: event_choice{choice_index})
- [ ] FaceTrader - _get_choices implementation. (action: event_choice{choice_index})
- [ ] FountainOfCleansing - _get_choices implementation. (action: event_choice{choice_index})
- [ ] TheJoust - _get_choices implementation. (action: event_choice{choice_index})
- [ ] TheLab - _get_choices implementation. (action: event_choice{choice_index})
- [ ] Nloth - _get_choices implementation. (action: event_choice{choice_index})
- [ ] WeMeetAgain - _get_choices implementation. (action: event_choice{choice_index})
- [ ] WomanInBlue - _get_choices implementation. (action: event_choice{choice_index})

## Pool consistency
- [ ] Align `KnowingSkull` act/special classification across content and handler. (action: none{})
- [ ] Align `SecretPortal` act/special classification across content and handler. (action: none{})
- [ ] Ensure every `EventDefinition` has a handler + choice generator. (action: none{})
- [ ] Add tests to assert pool membership and alias normalization. (action: none{})

## Failed-tests mapping (2026-02-04)
- [ ] Add `EventHandler.SKILL_CARDS`, `POWER_CARDS`, `ATTACK_CARDS` pools used by `Falling` and derived from card types. (action: none{})
- [ ] `Falling`: removing a card should decrement the correct pool count and record `cards_removed`. (action: event_choice{choice_index})
- [ ] `LivingWall`: transform choice must never select from an empty pool (guard and fallback when pool is empty). (action: event_choice{choice_index,card_index})
