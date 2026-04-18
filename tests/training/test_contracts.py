from packages.training.contracts import (
    RestrictionBuiltin,
    RestrictionPolicy,
    parse_combat_snapshot,
    parse_combat_training_state,
    parse_training_schema_versions,
)


def test_parse_training_schema_versions_roundtrip():
    payload = {
        "training_session_schema_version": 1,
        "combat_observation_schema_version": 1,
        "action_candidate_schema_version": 1,
        "gameplay_export_schema_version": 1,
        "replay_event_trace_schema_version": 1,
    }
    versions = parse_training_schema_versions(payload)
    assert versions.training_session_schema_version == 1
    assert versions.action_candidate_schema_version == 1


def test_parse_combat_training_state_builds_dense_candidates():
    payload = {
        "schema_versions": {
            "training_session_schema_version": 1,
            "combat_observation_schema_version": 1,
            "action_candidate_schema_version": 1,
            "gameplay_export_schema_version": 1,
            "replay_event_trace_schema_version": 1,
        },
        "context": {
            "runtime_scope": "combat",
            "decision_kind": "CombatAction",
            "phase_label": "Combat::PlayerTurn",
            "terminal": False,
            "floor": None,
            "ascension": None,
            "seed": None,
        },
        "observation": {
            "schema_version": 1,
            "caps": {
                "hand": 10,
                "enemies": 5,
                "player_effects": 32,
                "enemy_effects_per_enemy": 16,
                "orbs": 10,
                "relic_counters": 8,
                "choice_options": 10,
            },
            "global": {
                "turn": 1,
                "energy": 3,
                "max_energy": 3,
                "cards_played_this_turn": 0,
                "attacks_played_this_turn": 0,
                "hand_size": 1,
                "draw_pile_size": 4,
                "discard_pile_size": 0,
                "exhaust_pile_size": 0,
                "potion_slots": 3,
                "orb_slot_count": 0,
                "occupied_orb_slots": 0,
                "player_hp": 72,
                "player_max_hp": 72,
                "player_block": 0,
                "stance": "Neutral",
                "mantra": 0,
                "mantra_gained": 0,
                "skip_enemy_turn": False,
                "blasphemy_active": False,
                "combat_over": False,
                "player_won": False,
                "total_damage_dealt": 0,
                "total_damage_taken": 0,
                "total_cards_played": 0,
            },
            "player": {
                "hp": 72,
                "max_hp": 72,
                "block": 0,
                "stance": "Neutral",
                "strength": 0,
                "dexterity": 0,
                "focus": 0,
                "weak": 0,
                "vulnerable": 0,
                "frail": 0,
                "relics": ["PureWater"],
            },
            "hand": [
                {
                    "hand_index": 0,
                    "card_id": "Strike",
                    "card_name": "Strike",
                    "card_type": "Attack",
                    "target": "Enemy",
                    "cost_for_turn": 1,
                    "base_cost": -1,
                    "misc": -1,
                    "upgraded": False,
                    "free_to_play": False,
                    "retained": False,
                    "ethereal": False,
                    "runtime_only": False,
                    "x_cost": False,
                    "multi_hit": False,
                }
            ],
            "enemies": [
                {
                    "enemy_index": 0,
                    "enemy_id": "Cultist",
                    "enemy_name": "Cultist",
                    "hp": 48,
                    "max_hp": 48,
                    "block": 0,
                    "alive": True,
                    "targetable": True,
                    "back_attack": False,
                    "intent": "Attack { damage: 6, hits: 1, effects: 0 }",
                    "intent_damage": 6,
                    "intent_hits": 1,
                    "intent_block": 0,
                }
            ],
            "player_effects": [],
            "enemy_effects": [],
            "orbs": [],
            "relic_counters": [],
            "choice": {
                "active": False,
                "reason": None,
                "min_picks": 0,
                "max_picks": 0,
                "selected": [],
                "options": [],
            },
        },
        "legal_candidates": [
            {
                "schema_version": 1,
                "dense_index": 0,
                "execution_id": 0,
                "action_kind": "end_turn",
                "description": "End the current turn",
                "card": None,
                "target": None,
                "potion": None,
                "choice": None,
            },
            {
                "schema_version": 1,
                "dense_index": 1,
                "execution_id": 65537,
                "action_kind": "play_card",
                "description": "Play Strike",
                "card": {
                    "hand_index": 0,
                    "card_id": "Strike",
                    "card_name": "Strike",
                    "card_type": "Attack",
                    "cost_for_turn": 1,
                    "base_cost": -1,
                    "upgraded": False,
                    "x_cost": False,
                    "multi_hit": False,
                    "free_to_play": False,
                },
                "target": {
                    "enemy_index": 0,
                    "enemy_name": "Cultist",
                    "hp": 48,
                    "block": 0,
                    "targetable": True,
                    "back_attack": False,
                },
                "potion": None,
                "choice": None,
            },
        ],
    }
    state = parse_combat_training_state(payload)
    assert state.observation.hand[0].card_id == "Strike"
    assert state.legal_candidates[1].target is not None
    assert state.legal_candidates[1].dense_index == 1


def test_restriction_policy_serializes_builtin_values():
    policy = RestrictionPolicy((RestrictionBuiltin.NO_CARD_ADDS, RestrictionBuiltin.UPGRADE_REMOVE_ONLY))
    assert policy.to_dict() == {
        "builtins": ["NoCardAdds", "UpgradeRemoveOnly"],
    }


def test_parse_combat_snapshot_roundtrip_shape():
    payload = {
        "schema_version": 1,
        "player_hp": 66,
        "player_max_hp": 72,
        "player_block": 4,
        "energy": 3,
        "max_energy": 3,
        "turn": 1,
        "cards_played_this_turn": 0,
        "attacks_played_this_turn": 0,
        "stance": "Neutral",
        "mantra": 0,
        "mantra_gained": 0,
        "skip_enemy_turn": False,
        "blasphemy_active": False,
        "total_damage_dealt": 0,
        "total_damage_taken": 0,
        "total_cards_played": 0,
        "player_effects": [],
        "hand": [
            {
                "card_id": "Strike",
                "cost_for_turn": 1,
                "base_cost": 1,
                "misc": -1,
                "upgraded": False,
                "free_to_play": False,
                "retained": False,
                "ethereal": False,
            }
        ],
        "draw_pile": [],
        "discard_pile": [],
        "exhaust_pile": [],
        "enemies": [
            {
                "enemy_index": 0,
                "enemy_id": "Cultist",
                "enemy_name": "Cultist",
                "hp": 48,
                "max_hp": 48,
                "block": 0,
                "back_attack": False,
                "move_id": 1,
                "intent_damage": 6,
                "intent_hits": 1,
                "intent_block": 0,
                "first_turn": True,
                "is_escaping": False,
                "statuses": [],
            }
        ],
        "potions": ["FlexPotion", "", ""],
        "relics": ["PureWater"],
        "relic_counters": [{"counter_name": "ink_bottle", "value": 6}],
        "orb_slots": 0,
        "rng_seed0": 11,
        "rng_seed1": 22,
        "rng_counter": 3,
    }

    snapshot = parse_combat_snapshot(payload)

    assert snapshot.hand[0].card_id == "Strike"
    assert snapshot.enemies[0].enemy_name == "Cultist"
    assert snapshot.potions[0] == "FlexPotion"
    assert snapshot.relic_counters[0].counter_name == "ink_bottle"
