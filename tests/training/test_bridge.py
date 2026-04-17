from packages.training.bridge import (
    load_combat_snapshot,
    load_combat_training_state,
    load_training_schema_versions,
    run_combat_puct,
)
from packages.training.contracts import (
    CombatPuctConfig,
    CombatSearchStopReason,
    RestrictionBuiltin,
    RestrictionPolicy,
)


class _FakeEngine:
    def get_training_schema_versions(self):
        return {
            "training_session_schema_version": 1,
            "combat_observation_schema_version": 1,
            "action_candidate_schema_version": 1,
            "gameplay_export_schema_version": 1,
            "replay_event_trace_schema_version": 1,
        }

    def get_combat_training_state(self, restriction_policy_json=None):
        assert restriction_policy_json is not None
        return {
            "schema_versions": self.get_training_schema_versions(),
            "context": {
                "runtime_scope": "run",
                "decision_kind": "CombatAction",
                "phase_label": "Run::Combat",
                "terminal": False,
                "floor": 3,
                "ascension": 0,
                "seed": 42,
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
                    "hand_size": 0,
                    "draw_pile_size": 0,
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
                    "relics": [],
                },
                "hand": [],
                "enemies": [],
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
            "legal_candidates": [],
        }

    def get_combat_snapshot(self):
        return {
            "schema_version": 1,
            "player_hp": 72,
            "player_max_hp": 72,
            "player_block": 0,
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
            "hand": [],
            "draw_pile": [],
            "discard_pile": [],
            "exhaust_pile": [],
            "enemies": [],
            "potions": ["", "", ""],
            "relics": [],
            "relic_counters": [],
            "orb_slots": 0,
            "rng_seed0": 1,
            "rng_seed1": 2,
            "rng_counter": 0,
        }

    def run_combat_puct(self, evaluator, config_json=None):
        assert config_json is not None
        evaluation = evaluator(self.get_combat_training_state(config_json))
        assert len(evaluation["priors"]) == 0
        return {
            "chosen_action_id": None,
            "root_action_ids": [],
            "root_visits": [],
            "root_visit_shares": [],
            "root_priors": [],
            "frontier": [],
            "root_outcome": evaluation["outcome"],
            "root_total_visits": 1,
            "stable_windows": 0,
            "nodes_expanded": 1,
            "leaf_evaluations": 1,
            "max_depth_reached": 0,
            "elapsed_ms": 1,
            "stop_reason": "Converged",
        }


def test_bridge_loaders_use_engine_session_surface():
    engine = _FakeEngine()
    versions = load_training_schema_versions(engine)
    state = load_combat_training_state(
        engine,
        RestrictionPolicy((RestrictionBuiltin.NO_CARD_ADDS,)),
    )
    snapshot = load_combat_snapshot(engine)
    assert versions.gameplay_export_schema_version == 1
    assert state.context.floor == 3
    assert snapshot.player_hp == 72
    assert snapshot.rng_seed1 == 2


def test_bridge_can_parse_combat_puct_results():
    engine = _FakeEngine()
    result = run_combat_puct(
        engine,
        lambda state: {
            "priors": [],
            "outcome": {
                "solve_probability": 0.75,
                "expected_hp_loss": 3.0,
                "expected_turns": 4.0,
                "potion_cost": 0.0,
                "setup_value_delta": 0.5,
                "persistent_scaling_delta": 0.0,
            },
        },
        CombatPuctConfig(min_visits=8, visit_window=4, hard_visit_cap=16),
    )
    assert result.root_total_visits == 1
    assert result.stop_reason is CombatSearchStopReason.CONVERGED
    assert result.root_outcome.solve_probability == 0.75
