"""
RNG-TEST-001: Determinism assertions for migrated RNG callsites.

Verifies that the same seed + same actions produce identical outcomes
for all callsites migrated from direct `random.*` to owned RNG streams.
"""

import pytest
from packages.engine.state.combat import (
    CombatState, EntityState, EnemyCombatState,
    create_combat, create_enemy,
)
from packages.engine.state.rng import Random
from packages.engine.registry import (
    RelicContext, PotionContext, RELIC_REGISTRY, POTION_REGISTRY,
    execute_relic_triggers, execute_potion_effect,
)
from packages.engine.effects.registry import EffectContext


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _make_state(**overrides):
    """Create a minimal CombatState with RNG streams attached."""
    defaults = dict(
        player_hp=60,
        player_max_hp=80,
        enemies=[create_enemy("TestA", hp=40, max_hp=40),
                 create_enemy("TestB", hp=40, max_hp=40)],
        deck=["Strike_P", "Defend_P", "Vigilance", "Eruption", "Smite"],
        relics=[],
        potions=["", "", ""],
    )
    defaults.update(overrides)
    return create_combat(**defaults)


def _attach_rng(state, seed=42):
    """Attach all per-floor RNG streams with the given seed."""
    state.card_random_rng = Random(seed)
    state.shuffle_rng = Random(seed + 1)
    state.misc_rng = Random(seed + 2)
    state.potion_rng = Random(seed + 3)
    state.card_rng = Random(seed + 4)
    return state


def _run_twice(fn, seed=42):
    """Run fn(state) twice with fresh identical states and return both results."""
    results = []
    for _ in range(2):
        state = _make_state()
        _attach_rng(state, seed)
        result = fn(state)
        results.append(result)
    return results


# ============================================================================
# Relic determinism tests
# ============================================================================

class TestWarpedTongsDeterminism:
    """Warped Tongs must pick the same card index given same seed."""

    def test_same_seed_same_card_upgraded(self):
        def run(state):
            state.hand = ["Strike_P", "Defend_P", "Vigilance"]
            state.relics = ["Warped Tongs"]
            ctx = RelicContext(state=state, relic_id="Warped Tongs")
            handler = RELIC_REGISTRY.get_handler("atTurnStart", "Warped Tongs")
            handler(ctx)
            return list(getattr(state, "combat_upgrades", set()))

        r1, r2 = _run_twice(run)
        assert r1 == r2
        assert len(r1) > 0  # actually upgraded something

    def test_advances_card_random_rng(self):
        state = _make_state()
        _attach_rng(state)
        state.hand = ["Strike_P", "Defend_P"]
        before = state.card_random_rng.counter
        ctx = RelicContext(state=state, relic_id="Warped Tongs")
        handler = RELIC_REGISTRY.get_handler("atTurnStart", "Warped Tongs")
        handler(ctx)
        assert state.card_random_rng.counter > before


class TestMarkOfPainDeterminism:
    """Mark of Pain shuffle must be deterministic."""

    def test_same_seed_same_draw_order(self):
        def run(state):
            state.draw_pile = ["Strike_P", "Defend_P", "Vigilance", "Eruption"]
            state.relics = ["Mark of Pain"]
            ctx = RelicContext(state=state, relic_id="Mark of Pain")
            handler = RELIC_REGISTRY.get_handler("atBattleStart", "Mark of Pain")
            handler(ctx)
            return state.draw_pile[:]

        r1, r2 = _run_twice(run)
        assert r1 == r2
        assert "Wound" in r1  # wounds were added


class TestEnchiridionDeterminism:
    """Enchiridion must pick the same power card given same seed."""

    def test_same_seed_same_power(self):
        def run(state):
            state.relics = ["Enchiridion"]
            ctx = RelicContext(state=state, relic_id="Enchiridion")
            handler = RELIC_REGISTRY.get_handler("atBattleStartPreDraw", "Enchiridion")
            handler(ctx)
            return state.hand[:]

        r1, r2 = _run_twice(run)
        assert r1 == r2
        assert len(r1) > 0


class TestNilrysCodexDeterminism:
    """Nilry's Codex must pick the same card given same seed."""

    def test_same_seed_same_card(self):
        def run(state):
            state.relics = ["Nilry's Codex"]
            ctx = RelicContext(state=state, relic_id="Nilry's Codex")
            handler = RELIC_REGISTRY.get_handler("onPlayerEndTurn", "Nilry's Codex")
            handler(ctx)
            return list(getattr(state, "cards_to_add_next_turn", []))

        r1, r2 = _run_twice(run)
        assert r1 == r2
        assert len(r1) > 0


class TestMummifiedHandDeterminism:
    """Mummified Hand must pick the same card index given same seed."""

    def test_same_seed_same_cost_reduction(self):
        from packages.engine.content.cards import get_card

        def run(state):
            state.hand = ["Strike_P", "Defend_P", "Vigilance"]
            state.relics = ["Mummified Hand"]
            # Create a mock card object with card_type == POWER
            card_obj = get_card("Vigilance")
            from packages.engine.content.cards import CardType
            # Use a real power card
            card_obj = get_card("Eruption")  # Not a power, but we can fake it
            class FakeCard:
                card_type = CardType.POWER
            ctx = RelicContext(state=state, relic_id="Mummified Hand", card=FakeCard())
            handler = RELIC_REGISTRY.get_handler("onPlayCard", "Mummified Hand")
            handler(ctx)
            return dict(state.card_costs)

        r1, r2 = _run_twice(run)
        assert r1 == r2
        assert len(r1) > 0  # at least one cost was modified


class TestDeadBranchDeterminism:
    """Dead Branch must add the same card given same seed."""

    def test_same_seed_same_card(self):
        def run(state):
            state.relics = ["Dead Branch"]
            ctx = RelicContext(
                state=state, relic_id="Dead Branch",
                trigger_data={"card_id": "Strike_P"},
            )
            handler = RELIC_REGISTRY.get_handler("onExhaust", "Dead Branch")
            handler(ctx)
            return state.hand[:]

        r1, r2 = _run_twice(run)
        assert r1 == r2
        assert len(r1) > 0


class TestStrangeSpoonDeterminism:
    """Strange Spoon 50% chance must be deterministic."""

    def test_same_seed_same_outcome(self):
        def run(state):
            state.relics = ["Strange Spoon"]
            state.exhaust_pile = ["Strike_P"]
            ctx = RelicContext(
                state=state, relic_id="Strange Spoon",
                trigger_data={"card_id": "Strike_P"},
            )
            handler = RELIC_REGISTRY.get_handler("onExhaust", "Strange Spoon")
            handler(ctx)
            return {
                "exhaust": state.exhaust_pile[:],
                "discard": state.discard_pile[:],
            }

        r1, r2 = _run_twice(run)
        assert r1 == r2

    def test_advances_misc_rng(self):
        state = _make_state()
        _attach_rng(state)
        state.exhaust_pile = ["Strike_P"]
        before = state.misc_rng.counter
        ctx = RelicContext(
            state=state, relic_id="Strange Spoon",
            trigger_data={"card_id": "Strike_P"},
        )
        handler = RELIC_REGISTRY.get_handler("onExhaust", "Strange Spoon")
        handler(ctx)
        assert state.misc_rng.counter > before


class TestTingshaDeterminism:
    """Tingsha must pick the same target given same seed."""

    def test_same_seed_same_target(self):
        def run(state):
            state.relics = ["Tingsha"]
            ctx = RelicContext(state=state, relic_id="Tingsha")
            handler = RELIC_REGISTRY.get_handler("onManualDiscard", "Tingsha")
            handler(ctx)
            return [e.hp for e in state.enemies]

        r1, r2 = _run_twice(run)
        assert r1 == r2


class TestSpecimenDeterminism:
    """The Specimen must transfer to the same target given same seed."""

    def test_same_seed_same_transfer_target(self):
        def run(state):
            state.relics = ["The Specimen"]
            dead = create_enemy("Dead", hp=0, max_hp=40)
            dead.statuses["Poison"] = 5
            ctx = RelicContext(
                state=state, relic_id="The Specimen",
                trigger_data={"enemy": dead},
            )
            handler = RELIC_REGISTRY.get_handler("onMonsterDeath", "The Specimen")
            handler(ctx)
            return [e.statuses.get("Poison", 0) for e in state.enemies]

        r1, r2 = _run_twice(run)
        assert r1 == r2
        assert sum(r1) > 0  # poison was transferred


# ============================================================================
# Potion determinism tests
# ============================================================================

class TestSneckoOilDeterminism:
    """SneckoOil must assign the same costs given same seed."""

    def test_same_seed_same_costs(self):
        def run(state):
            state.hand = ["Strike_P", "Defend_P", "Vigilance"]
            state.potions = ["SneckoOil", "", ""]
            result = execute_potion_effect("SneckoOil", state, target_idx=-1)
            return dict(state.card_costs)

        r1, r2 = _run_twice(run)
        assert r1 == r2

    def test_no_direct_random_fallback(self):
        """With card_random_rng attached, every cost roll uses the stream."""
        state = _make_state()
        _attach_rng(state)
        state.hand = ["Strike_P", "Defend_P", "Vigilance"]
        state.potions = ["SneckoOil", "", ""]
        before = state.card_random_rng.counter
        execute_potion_effect("SneckoOil", state, target_idx=-1)
        assert state.card_random_rng.counter > before


class TestEntropicBrewDeterminism:
    """EntropicBrew must fill the same potions given same seed."""

    def test_same_seed_same_potions(self):
        def run(state):
            state.player_class = "WATCHER"
            state.potions = ["EntropicBrew", "", ""]
            execute_potion_effect("EntropicBrew", state, target_idx=-1)
            return state.potions[:]

        r1, r2 = _run_twice(run)
        assert r1 == r2
        # Slots 1 and 2 should be filled
        assert r1[1] != ""
        assert r1[2] != ""

    def test_advances_potion_rng(self):
        state = _make_state()
        _attach_rng(state)
        state.player_class = "WATCHER"
        state.potions = ["EntropicBrew", "", ""]
        before = state.potion_rng.counter
        execute_potion_effect("EntropicBrew", state, target_idx=-1)
        assert state.potion_rng.counter > before


# ============================================================================
# Defect card determinism tests
# ============================================================================

class TestThunderStrikeDeterminism:
    """Thunder Strike random hits must be deterministic."""

    def test_same_seed_same_damage_distribution(self):
        def run(state):
            state.card_random_rng = Random(42)
            manager = _setup_orb_manager(state)
            # Channel some lightning so lightning_channeled > 0
            from packages.engine.effects.orbs import channel_orb
            for _ in range(3):
                channel_orb(state, "Lightning")

            from packages.engine.content.cards import get_card
            card = get_card("Thunder Strike")

            ctx = EffectContext(state=state, card=card, target=state.enemies[0])
            from packages.engine.effects.defect_cards import damage_per_lightning_channeled_effect
            damage_per_lightning_channeled_effect(ctx)
            return [e.hp for e in state.enemies]

        r1, r2 = _run_twice(run)
        assert r1 == r2


class TestRipAndTearDeterminism:
    """Rip and Tear random hits must be deterministic."""

    def test_same_seed_same_damage_distribution(self):
        def run(state):
            state.card_random_rng = Random(42)
            from packages.engine.content.cards import get_card
            card = get_card("Rip and Tear")
            ctx = EffectContext(state=state, card=card, target=state.enemies[0])
            from packages.engine.effects.defect_cards import rip_and_tear_effect
            rip_and_tear_effect(ctx)
            return [e.hp for e in state.enemies]

        r1, r2 = _run_twice(run)
        assert r1 == r2


class TestRebootDeterminism:
    """Reboot shuffle must be deterministic."""

    def test_same_seed_same_draw_order(self):
        def run(state):
            state.shuffle_rng = Random(42)
            state.card_random_rng = Random(43)
            state.hand = ["Strike_P", "Defend_P"]
            state.discard_pile = ["Vigilance", "Eruption"]
            state.draw_pile = ["Smite"]

            ctx = EffectContext(state=state, card=None, magic_number=4)
            from packages.engine.effects.defect_cards import reboot_effect
            reboot_effect(ctx)
            return state.hand[:], state.draw_pile[:]

        r1, r2 = _run_twice(run)
        assert r1 == r2


class TestWhiteNoiseDeterminism:
    """White Noise must pick the same power given same seed."""

    def test_same_seed_same_power(self):
        def run(state):
            state.card_random_rng = Random(42)
            ctx = EffectContext(state=state, card=None)
            from packages.engine.effects.defect_cards import white_noise_effect
            white_noise_effect(ctx)
            return state.hand[:]

        r1, r2 = _run_twice(run)
        assert r1 == r2
        assert len(r1) > 0

    def test_advances_card_random_rng(self):
        state = _make_state()
        state.card_random_rng = Random(42)
        before = state.card_random_rng.counter
        ctx = EffectContext(state=state, card=None)
        from packages.engine.effects.defect_cards import white_noise_effect
        white_noise_effect(ctx)
        assert state.card_random_rng.counter > before


# ---------------------------------------------------------------------------
# Orb manager helper for Thunder Strike test
# ---------------------------------------------------------------------------

def _setup_orb_manager(state):
    from packages.engine.effects.orbs import get_orb_manager
    return get_orb_manager(state)
