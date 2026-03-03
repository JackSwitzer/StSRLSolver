"""
CRD-DE-001: Defect Card Behavior Verification & Closure

Verifies all 68 Defect card effects against Java source implementations.
Tests cover:
- Effect handler registration (every card's effects are in the registry)
- Behavioral scenarios for each card
- Upgraded vs base value differences
"""

import pytest

from packages.engine.content.cards import (
    Card, CardType, CardRarity, CardTarget, CardColor,
    get_card, DEFECT_CARDS,
)
from packages.engine.state.combat import (
    CombatState, EntityState, EnemyCombatState,
    create_player, create_enemy, create_combat,
)
from packages.engine.state.rng import Random
from packages.engine.effects.orbs import (
    OrbManager, OrbType, Orb, get_orb_manager, channel_orb,
    channel_random_orb, evoke_orb, evoke_all_orbs,
)
from packages.engine.effects.registry import (
    execute_effect, EffectContext, get_effect_handler,
)
from packages.engine.effects.defect_cards import DEFECT_CARD_EFFECTS


# =============================================================================
# Fixtures
# =============================================================================

@pytest.fixture
def combat():
    """Standard single-enemy combat."""
    player = create_player(hp=70, max_hp=70)
    enemy = create_enemy(id="TestEnemy", hp=50, max_hp=50, move_damage=10)
    state = CombatState(
        player=player,
        energy=3,
        max_energy=3,
        enemies=[enemy],
        hand=[],
        draw_pile=["Strike_B"] * 5 + ["Defend_B"] * 5,
        discard_pile=[],
        exhaust_pile=[],
    )
    return state


@pytest.fixture
def multi_combat():
    """Combat with 3 enemies."""
    player = create_player(hp=70, max_hp=70)
    enemies = [
        create_enemy(id="E1", hp=30, move_damage=5),
        create_enemy(id="E2", hp=30, move_damage=8),
        create_enemy(id="E3", hp=30, move_damage=6),
    ]
    state = CombatState(
        player=player,
        energy=3,
        max_energy=3,
        enemies=enemies,
        hand=[],
        draw_pile=["Strike_B"] * 10,
        discard_pile=[],
    )
    return state


def make_ctx(state, card_id=None, target=None, upgraded=False, magic=0,
             extra=None):
    """Helper to construct an EffectContext."""
    card = get_card(card_id, upgraded=upgraded) if card_id else None
    tgt = state.enemies[0] if target is None and state.enemies else target
    return EffectContext(
        state=state,
        card=card,
        target=tgt,
        target_idx=0,
        is_upgraded=upgraded,
        magic_number=card.magic_number if card and card.magic_number > 0 else magic,
        extra_data=extra or {},
    )


# =============================================================================
# 1. Effect Handler Registration
# =============================================================================

class TestEffectRegistration:
    """Every Defect card effect string must resolve to a handler."""

    def test_all_defect_effects_are_registered(self):
        """All effect strings referenced in DEFECT_CARD_EFFECTS must resolve."""
        missing = []
        for card_id, effects in DEFECT_CARD_EFFECTS.items():
            for eff in effects:
                handler = get_effect_handler(eff)
                if handler is None:
                    missing.append(f"{card_id}: {eff}")
        assert not missing, f"Unregistered effects: {missing}"

    def test_all_defect_card_data_effects_are_registered(self):
        """All effect strings in card data must resolve."""
        # Also check cards.py effect lists
        missing = []
        for card_id, card in DEFECT_CARDS.items():
            for eff in card.effects:
                handler = get_effect_handler(eff)
                if handler is None:
                    # Some effects are handled by executor fallback, skip those
                    from packages.engine.effects.executor import EffectExecutor
                    if eff not in EffectExecutor._NOOP_EFFECTS and eff not in EffectExecutor._EFFECT_HANDLERS:
                        if not eff.startswith("draw_") and not eff.startswith("gain_"):
                            missing.append(f"{card_id}: {eff}")
        assert not missing, f"Unregistered card data effects: {missing}"


# =============================================================================
# 2. Orb Channeling Cards
# =============================================================================

class TestOrbChannelingBehavior:
    """Behavioral tests for orb channeling cards."""

    def test_zap_channels_lightning(self, combat):
        ctx = make_ctx(combat, "Zap")
        execute_effect("channel_lightning", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.get_orb_count() == 1
        assert mgr.orbs[0].orb_type == OrbType.LIGHTNING

    def test_ball_lightning_channels_lightning(self, combat):
        card = get_card("Ball Lightning")
        assert "channel_lightning" in card.effects
        ctx = make_ctx(combat, "Ball Lightning")
        execute_effect("channel_lightning", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.lightning_channeled == 1

    def test_cold_snap_channels_frost(self, combat):
        ctx = make_ctx(combat, "Cold Snap")
        execute_effect("channel_frost", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.frost_channeled == 1
        assert mgr.orbs[0].orb_type == OrbType.FROST

    def test_coolheaded_channels_frost_and_draws(self, combat):
        card = get_card("Coolheaded")
        assert "channel_frost" in card.effects
        assert "draw_cards" in card.effects

    def test_coolheaded_upgraded_draws_2(self):
        card = get_card("Coolheaded", upgraded=True)
        assert card.magic_number == 2

    def test_darkness_channels_dark(self, combat):
        ctx = make_ctx(combat, "Darkness")
        execute_effect("channel_dark", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.dark_channeled == 1
        assert mgr.orbs[0].orb_type == OrbType.DARK

    def test_darkness_upgraded_triggers_dark_impulse(self, combat):
        """Darkness+ channels Dark then triggers DarkImpulseAction on all Dark orbs."""
        mgr = get_orb_manager(combat)
        channel_orb(combat, "Dark")  # Pre-existing Dark orb
        initial_accumulated = mgr.orbs[0].accumulated_damage  # 6

        ctx = make_ctx(combat, "Darkness", upgraded=True)
        execute_effect("channel_dark", ctx)
        execute_effect("darkness_trigger_dark_orbs", ctx)

        # We now have 2 Dark orbs. The first got 2 passive triggers (start+end).
        # Each passive adds 6+focus=6, so first orb accumulated += 12
        assert mgr.orbs[0].accumulated_damage == initial_accumulated + 12

    def test_darkness_base_no_dark_impulse(self, combat):
        """Base Darkness does NOT trigger DarkImpulseAction."""
        mgr = get_orb_manager(combat)
        channel_orb(combat, "Dark")
        initial = mgr.orbs[0].accumulated_damage

        ctx = make_ctx(combat, "Darkness", upgraded=False)
        execute_effect("darkness_trigger_dark_orbs", ctx)
        # Should be unchanged since not upgraded
        assert mgr.orbs[0].accumulated_damage == initial

    def test_doom_and_gloom_channels_dark(self, combat):
        card = get_card("Doom and Gloom")
        assert "channel_dark" in card.effects
        assert card.target == CardTarget.ALL_ENEMY

    def test_fusion_channels_plasma(self, combat):
        ctx = make_ctx(combat, "Fusion")
        execute_effect("channel_plasma", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.plasma_channeled == 1

    def test_glacier_channels_2_frost(self, combat):
        ctx = make_ctx(combat, "Glacier")
        execute_effect("channel_2_frost", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.frost_channeled == 2
        assert mgr.get_orb_count() == 2

    def test_chaos_channels_random_orb(self, combat):
        combat.card_random_rng = Random(42)
        ctx = make_ctx(combat, "Chaos")
        execute_effect("channel_random_orb", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.get_orb_count() == 1

    def test_chaos_upgraded_channels_2(self, combat):
        combat.card_random_rng = Random(42)
        ctx = make_ctx(combat, "Chaos", upgraded=True)
        execute_effect("channel_random_orb", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.get_orb_count() == 2

    def test_chill_channels_frost_per_enemy(self, multi_combat):
        ctx = make_ctx(multi_combat, "Chill")
        execute_effect("channel_frost_per_enemy", ctx)
        mgr = get_orb_manager(multi_combat)
        assert mgr.frost_channeled == 3

    def test_rainbow_channels_all_three(self, combat):
        ctx = make_ctx(combat, "Rainbow")
        execute_effect("channel_lightning_frost_dark", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.lightning_channeled == 1
        assert mgr.frost_channeled == 1
        assert mgr.dark_channeled == 1
        assert mgr.get_orb_count() == 3

    def test_rainbow_upgraded_no_exhaust(self):
        """Java: Rainbow+ removes exhaust."""
        card = get_card("Rainbow", upgraded=True)
        assert card.exhaust is False

    def test_meteor_strike_channels_3_plasma(self, combat):
        ctx = make_ctx(combat, "Meteor Strike")
        execute_effect("channel_3_plasma", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.plasma_channeled == 3

    def test_tempest_channels_x_lightning(self, combat):
        combat.energy = 2
        ctx = make_ctx(combat, "Tempest", extra={"x_cost": 2})
        execute_effect("channel_x_lightning", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.lightning_channeled == 2

    def test_tempest_upgraded_channels_x_plus_1(self, combat):
        """Java: Tempest+ channels X+1 Lightning orbs."""
        ctx = make_ctx(combat, "Tempest", upgraded=True, extra={"x_cost": 2})
        execute_effect("channel_x_lightning", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.lightning_channeled == 3

    def test_electrodynamics_channels_lightning_magic(self, combat):
        ctx = make_ctx(combat, "Electrodynamics", magic=2)
        execute_effect("channel_lightning_magic", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.lightning_channeled == 2

    def test_electrodynamics_upgraded_channels_3(self, combat):
        ctx = make_ctx(combat, "Electrodynamics", upgraded=True)
        execute_effect("channel_lightning_magic", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.lightning_channeled == 3


# =============================================================================
# 3. Orb Evoke Cards
# =============================================================================

class TestOrbEvokeBehavior:
    """Behavioral tests for orb evoke cards."""

    def test_dualcast_evokes_twice(self, combat):
        channel_orb(combat, "Frost")
        ctx = make_ctx(combat, "Dualcast")
        execute_effect("evoke_orb_twice", ctx)
        # 5 block * 2 = 10
        assert combat.player.block == 10
        assert get_orb_manager(combat).get_orb_count() == 0

    def test_multi_cast_evokes_x_times(self, combat):
        channel_orb(combat, "Frost")
        ctx = make_ctx(combat, "Multi-Cast", extra={"x_cost": 3})
        execute_effect("evoke_first_orb_x_times", ctx)
        # 5 block * 3 = 15
        assert combat.player.block == 15

    def test_multi_cast_upgraded_evokes_x_plus_1(self, combat):
        """Java: Multi-Cast+ evokes X+1 times."""
        channel_orb(combat, "Frost")
        ctx = make_ctx(combat, "Multi-Cast", upgraded=True, extra={"x_cost": 2})
        execute_effect("evoke_first_orb_x_times", ctx)
        # 5 block * 3 = 15
        assert combat.player.block == 15

    def test_recursion_evokes_then_channels_same(self, combat):
        channel_orb(combat, "Lightning")
        mgr = get_orb_manager(combat)
        before_hp = combat.enemies[0].hp

        ctx = make_ctx(combat, "Redo")
        execute_effect("evoke_then_channel_same_orb", ctx)

        # Should have evoked Lightning (dealing damage) then channeled Lightning
        assert combat.enemies[0].hp < before_hp
        assert mgr.get_orb_count() == 1
        assert mgr.orbs[0].orb_type == OrbType.LIGHTNING

    def test_fission_base_removes_orbs_no_resources(self, combat):
        """Java: Fission (base) removes all orbs, no energy/draw."""
        channel_orb(combat, "Lightning")
        channel_orb(combat, "Frost")
        initial_energy = combat.energy

        ctx = make_ctx(combat, "Fission", upgraded=False)
        execute_effect("remove_orbs_gain_energy_and_draw", ctx)

        mgr = get_orb_manager(combat)
        assert mgr.get_orb_count() == 0
        assert combat.energy == initial_energy  # No energy gain for base

    def test_fission_upgraded_gains_resources(self, combat):
        """Java: Fission+ removes all orbs, gain 1 energy and draw 1 per orb."""
        channel_orb(combat, "Lightning")
        channel_orb(combat, "Frost")
        initial_energy = combat.energy

        ctx = make_ctx(combat, "Fission", upgraded=True)
        execute_effect("remove_orbs_gain_energy_and_draw", ctx)

        mgr = get_orb_manager(combat)
        assert mgr.get_orb_count() == 0
        assert combat.energy == initial_energy + 2  # +1 per orb


# =============================================================================
# 4. Focus Manipulation Cards
# =============================================================================

class TestFocusManipulation:

    def test_defragment_gains_focus(self, combat):
        ctx = make_ctx(combat, "Defragment")
        execute_effect("gain_focus", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.focus == 1

    def test_defragment_upgraded_gains_2(self, combat):
        ctx = make_ctx(combat, "Defragment", upgraded=True)
        execute_effect("gain_focus", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.focus == 2

    def test_consume_gains_focus_loses_slot(self, combat):
        ctx = make_ctx(combat, "Consume")
        execute_effect("gain_focus_lose_orb_slot", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.focus == 2
        assert mgr.max_slots == 2  # 3 - 1

    def test_consume_upgraded_gains_3_focus(self, combat):
        ctx = make_ctx(combat, "Consume", upgraded=True)
        execute_effect("gain_focus_lose_orb_slot", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.focus == 3

    def test_biased_cognition_gains_focus_and_debuff(self, combat):
        ctx = make_ctx(combat, "Biased Cognition")
        execute_effect("gain_focus_lose_focus_each_turn", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.focus == 4
        assert combat.player.statuses.get("BiasedCognition", 0) == 1

    def test_biased_cognition_upgraded(self, combat):
        ctx = make_ctx(combat, "Biased Cognition", upgraded=True)
        execute_effect("gain_focus_lose_focus_each_turn", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.focus == 5

    def test_hyperbeam_loses_focus(self, combat):
        ctx = make_ctx(combat, "Hyperbeam")
        execute_effect("lose_focus", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.focus == -3

    def test_reprogram_trades_focus_for_str_dex(self, combat):
        ctx = make_ctx(combat, "Reprogram")
        execute_effect("lose_focus_gain_strength_dex", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.focus == -1
        assert combat.player.statuses.get("Strength", 0) == 1
        assert combat.player.statuses.get("Dexterity", 0) == 1

    def test_reprogram_upgraded_2(self, combat):
        ctx = make_ctx(combat, "Reprogram", upgraded=True)
        execute_effect("lose_focus_gain_strength_dex", ctx)
        assert combat.player.statuses.get("Strength", 0) == 2
        assert combat.player.statuses.get("Dexterity", 0) == 2


# =============================================================================
# 5. Orb Slot Cards
# =============================================================================

class TestOrbSlots:

    def test_capacitor_adds_slots(self, combat):
        ctx = make_ctx(combat, "Capacitor")
        execute_effect("increase_orb_slots", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.max_slots == 5  # 3 + 2

    def test_capacitor_upgraded_adds_3(self, combat):
        ctx = make_ctx(combat, "Capacitor", upgraded=True)
        execute_effect("increase_orb_slots", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.max_slots == 6  # 3 + 3


# =============================================================================
# 6. Orb Counting Cards
# =============================================================================

class TestOrbCountingCards:

    def test_barrage_damage_per_orb(self, combat):
        channel_orb(combat, "Lightning")
        channel_orb(combat, "Frost")
        card = get_card("Barrage")
        ctx = make_ctx(combat, "Barrage")
        before_hp = combat.enemies[0].hp

        execute_effect("damage_per_orb", ctx)

        # 4 damage x 2 orbs
        assert combat.enemies[0].hp == before_hp - 8

    def test_compile_driver_draws_per_unique_orb(self, combat):
        channel_orb(combat, "Lightning")
        channel_orb(combat, "Lightning")
        channel_orb(combat, "Frost")
        ctx = make_ctx(combat, "Compile Driver")

        execute_effect("draw_per_unique_orb", ctx)
        # 2 unique types -> draw 2
        assert len(ctx.cards_drawn) == 2

    def test_blizzard_damage_per_frost(self, multi_combat):
        mgr = get_orb_manager(multi_combat)
        channel_orb(multi_combat, "Frost")
        channel_orb(multi_combat, "Frost")
        channel_orb(multi_combat, "Frost")

        ctx = make_ctx(multi_combat, "Blizzard")
        before_hps = [e.hp for e in multi_combat.enemies]

        execute_effect("damage_per_frost_channeled", ctx)

        # 3 frost * 2 damage_per = 6 damage to ALL enemies
        for i, enemy in enumerate(multi_combat.enemies):
            assert enemy.hp == before_hps[i] - 6

    def test_blizzard_upgraded_3_per_frost(self, multi_combat):
        channel_orb(multi_combat, "Frost")
        channel_orb(multi_combat, "Frost")

        ctx = make_ctx(multi_combat, "Blizzard", upgraded=True)
        before_hps = [e.hp for e in multi_combat.enemies]

        execute_effect("damage_per_frost_channeled", ctx)

        # 2 frost * 3 = 6 to all
        for i, enemy in enumerate(multi_combat.enemies):
            assert enemy.hp == before_hps[i] - 6

    def test_thunder_strike_hits_per_lightning(self, combat):
        combat.card_random_rng = Random(42)
        channel_orb(combat, "Lightning")
        channel_orb(combat, "Lightning")
        channel_orb(combat, "Lightning")

        ctx = make_ctx(combat, "Thunder Strike")
        before_hp = combat.enemies[0].hp

        execute_effect("damage_per_lightning_channeled", ctx)

        # 3 lightning channeled, 7 damage each hit to random enemy
        assert combat.enemies[0].hp == before_hp - 21


# =============================================================================
# 7. Power Cards
# =============================================================================

class TestDefectPowerCards:

    def test_storm_applies_power(self, combat):
        ctx = make_ctx(combat, "Storm")
        execute_effect("channel_lightning_on_power_play", ctx)
        assert combat.player.statuses.get("Storm", 0) == 1

    def test_static_discharge_applies_power(self, combat):
        ctx = make_ctx(combat, "Static Discharge")
        execute_effect("channel_lightning_on_damage", ctx)
        assert combat.player.statuses.get("StaticDischarge", 0) == 1

    def test_static_discharge_upgraded_2(self, combat):
        ctx = make_ctx(combat, "Static Discharge", upgraded=True)
        execute_effect("channel_lightning_on_damage", ctx)
        assert combat.player.statuses.get("StaticDischarge", 0) == 2

    def test_heatsinks_applies_power(self, combat):
        ctx = make_ctx(combat, "Heatsinks")
        execute_effect("draw_on_power_play", ctx)
        assert combat.player.statuses.get("Heatsinks", 0) == 1

    def test_heatsinks_upgraded_2(self, combat):
        ctx = make_ctx(combat, "Heatsinks", upgraded=True)
        execute_effect("draw_on_power_play", ctx)
        assert combat.player.statuses.get("Heatsinks", 0) == 2

    def test_echo_form_applies_power(self, combat):
        ctx = make_ctx(combat, "Echo Form")
        execute_effect("play_first_card_twice", ctx)
        assert combat.player.statuses.get("EchoForm", 0) == 1

    def test_echo_form_base_is_ethereal(self):
        card = get_card("Echo Form")
        assert card.ethereal is True

    def test_echo_form_upgraded_not_ethereal(self):
        """Java: Echo Form+ removes ethereal."""
        card = get_card("Echo Form", upgraded=True)
        assert card.ethereal is False

    def test_creative_ai_applies_power(self, combat):
        ctx = make_ctx(combat, "Creative AI")
        execute_effect("add_random_power_each_turn", ctx)
        assert combat.player.statuses.get("CreativeAI", 0) == 1

    def test_machine_learning_applies_power(self, combat):
        ctx = make_ctx(combat, "Machine Learning")
        execute_effect("draw_extra_each_turn", ctx)
        assert combat.player.statuses.get("MachineLearning", 0) == 1

    def test_hello_world_applies_power(self, combat):
        ctx = make_ctx(combat, "Hello World")
        execute_effect("add_common_card_each_turn", ctx)
        assert combat.player.statuses.get("HelloWorld", 0) == 1

    def test_self_repair_applies_power(self, combat):
        ctx = make_ctx(combat, "Self Repair")
        execute_effect("heal_at_end_of_combat", ctx)
        assert combat.player.statuses.get("SelfRepair", 0) == 7

    def test_self_repair_upgraded_10(self, combat):
        ctx = make_ctx(combat, "Self Repair", upgraded=True)
        execute_effect("heal_at_end_of_combat", ctx)
        assert combat.player.statuses.get("SelfRepair", 0) == 10

    def test_buffer_applies_power(self, combat):
        ctx = make_ctx(combat, "Buffer")
        execute_effect("prevent_next_hp_loss", ctx)
        assert combat.player.statuses.get("Buffer", 0) == 1

    def test_buffer_upgraded_2(self, combat):
        ctx = make_ctx(combat, "Buffer", upgraded=True)
        execute_effect("prevent_next_hp_loss", ctx)
        assert combat.player.statuses.get("Buffer", 0) == 2

    def test_amplify_applies_power(self, combat):
        ctx = make_ctx(combat, "Amplify")
        execute_effect("next_power_plays_twice", ctx)
        assert combat.player.statuses.get("Amplify", 0) == 1

    def test_amplify_upgraded_2(self, combat):
        ctx = make_ctx(combat, "Amplify", upgraded=True)
        execute_effect("next_power_plays_twice", ctx)
        assert combat.player.statuses.get("Amplify", 0) == 2

    def test_loop_applies_power(self, combat):
        ctx = make_ctx(combat, "Loop")
        execute_effect("trigger_orb_passive_extra", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.loop_stacks == 1

    def test_loop_upgraded_2(self, combat):
        ctx = make_ctx(combat, "Loop", upgraded=True)
        execute_effect("trigger_orb_passive_extra", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.loop_stacks == 2

    def test_electrodynamics_sets_lightning_all(self, combat):
        ctx = make_ctx(combat, "Electrodynamics")
        execute_effect("lightning_hits_all", ctx)
        mgr = get_orb_manager(combat)
        assert mgr.lightning_hits_all is True


# =============================================================================
# 8. Card Manipulation
# =============================================================================

class TestCardManipulationBehavior:

    def test_all_for_one_returns_0_cost_cards(self, combat):
        combat.discard_pile = ["Zap", "Claw", "Defend_B", "Dualcast"]
        # Zap cost=1, Claw cost=0, Defend_B cost=1, Dualcast cost=1
        ctx = make_ctx(combat, "All For One")
        execute_effect("return_all_0_cost_from_discard", ctx)

        # Only Claw should come back (cost 0)
        assert "Claw" in combat.hand
        assert "Claw" not in combat.discard_pile

    def test_hologram_returns_card_from_discard(self, combat):
        combat.discard_pile = ["Strike_B", "Defend_B"]
        ctx = make_ctx(combat, "Hologram", extra={"hologram_choice": 0})
        execute_effect("return_card_from_discard", ctx)

        assert len(combat.hand) == 1
        assert len(combat.discard_pile) == 1

    def test_hologram_base_exhausts(self):
        card = get_card("Hologram")
        assert card.exhaust is True

    def test_hologram_upgraded_no_exhaust(self):
        """Java: Hologram+ removes exhaust."""
        card = get_card("Hologram", upgraded=True)
        assert card.exhaust is False

    def test_seek_searches_draw_pile(self, combat):
        combat.draw_pile = ["Glacier", "Defragment", "Claw"]
        ctx = make_ctx(combat, "Seek", extra={"seek_choices": [0]})
        execute_effect("search_draw_pile", ctx)

        assert len(combat.hand) == 1
        assert len(combat.draw_pile) == 2

    def test_seek_upgraded_searches_2(self, combat):
        combat.draw_pile = ["Glacier", "Defragment", "Claw"]
        ctx = make_ctx(combat, "Seek", upgraded=True, extra={"seek_choices": [0, 0]})
        execute_effect("search_draw_pile", ctx)

        assert len(combat.hand) == 2
        assert len(combat.draw_pile) == 1

    def test_reboot_shuffles_and_draws(self, combat):
        combat.hand = ["Strike_B", "Defend_B"]
        combat.discard_pile = ["Zap"]
        combat.draw_pile = ["Claw"]
        combat.shuffle_rng = Random(42)
        combat.card_random_rng = Random(43)

        ctx = make_ctx(combat, "Reboot")
        execute_effect("shuffle_hand_and_discard_draw", ctx)

        # All cards combined = 4 total, draw 4
        assert len(combat.hand) == 4

    def test_reboot_upgraded_draws_6(self, combat):
        # Fill with more cards
        combat.hand = ["Strike_B"] * 3
        combat.discard_pile = ["Defend_B"] * 3
        combat.draw_pile = ["Zap"] * 3
        combat.shuffle_rng = Random(42)
        combat.card_random_rng = Random(43)

        ctx = make_ctx(combat, "Reboot", upgraded=True)
        execute_effect("shuffle_hand_and_discard_draw", ctx)

        assert len(combat.hand) == 6

    def test_recycle_exhausts_and_gains_energy(self, combat):
        combat.hand = ["Defend_B"]
        combat.card_costs = {"Defend_B": 1}
        ctx = make_ctx(combat, "Recycle", extra={"recycle_choice": 0})
        initial_energy = combat.energy

        execute_effect("exhaust_card_gain_energy", ctx)

        assert len(combat.hand) == 0
        assert "Defend_B" in combat.exhaust_pile
        assert combat.energy == initial_energy + 1

    def test_rebound_applies_power(self, combat):
        ctx = make_ctx(combat, "Rebound")
        execute_effect("next_card_on_top_of_draw", ctx)
        assert combat.player.statuses.get("Rebound", 0) == 1

    def test_white_noise_adds_power_card(self, combat):
        combat.card_random_rng = Random(42)
        ctx = make_ctx(combat, "White Noise")
        execute_effect("add_random_power_to_hand_cost_0", ctx)

        assert len(combat.hand) == 1
        # Should be a power card set to cost 0
        added = combat.hand[0]
        assert combat.card_costs.get(added, -1) == 0


# =============================================================================
# 9. Conditional Cards
# =============================================================================

class TestConditionalCards:

    def test_go_for_the_eyes_weak_if_attacking(self, combat):
        combat.enemies[0].move_damage = 10
        ctx = make_ctx(combat, "Go for the Eyes")
        execute_effect("if_attacking_apply_weak", ctx)
        assert combat.enemies[0].statuses.get("Weak", 0) == 1

    def test_go_for_the_eyes_no_weak_if_not_attacking(self, combat):
        combat.enemies[0].move_damage = 0
        ctx = make_ctx(combat, "Go for the Eyes")
        execute_effect("if_attacking_apply_weak", ctx)
        assert combat.enemies[0].statuses.get("Weak", 0) == 0

    def test_go_for_the_eyes_upgraded_2_weak(self, combat):
        combat.enemies[0].move_damage = 10
        ctx = make_ctx(combat, "Go for the Eyes", upgraded=True)
        execute_effect("if_attacking_apply_weak", ctx)
        assert combat.enemies[0].statuses.get("Weak", 0) == 2

    def test_ftl_draws_if_under_threshold(self, combat):
        combat.cards_played_this_turn = 1
        ctx = make_ctx(combat, "FTL")  # threshold=3
        execute_effect("if_played_less_than_x_draw", ctx)
        assert len(ctx.cards_drawn) == 1

    def test_ftl_no_draw_if_at_threshold(self, combat):
        combat.cards_played_this_turn = 3
        ctx = make_ctx(combat, "FTL")  # threshold=3
        execute_effect("if_played_less_than_x_draw", ctx)
        assert len(ctx.cards_drawn) == 0

    def test_ftl_upgraded_threshold_4(self, combat):
        combat.cards_played_this_turn = 3
        ctx = make_ctx(combat, "FTL", upgraded=True)  # threshold=4
        execute_effect("if_played_less_than_x_draw", ctx)
        assert len(ctx.cards_drawn) == 1

    def test_sunder_gains_energy_on_kill(self, combat):
        combat.enemies[0].hp = 0  # Already dead
        initial_energy = combat.energy
        ctx = make_ctx(combat, "Sunder")
        execute_effect("if_fatal_gain_3_energy", ctx)
        assert combat.energy == initial_energy + 3

    def test_sunder_no_energy_if_alive(self, combat):
        combat.enemies[0].hp = 50
        initial_energy = combat.energy
        ctx = make_ctx(combat, "Sunder")
        execute_effect("if_fatal_gain_3_energy", ctx)
        assert combat.energy == initial_energy

    def test_auto_shields_blocks_if_no_block(self, combat):
        combat.player.block = 0
        ctx = make_ctx(combat, "Auto Shields")
        execute_effect("only_if_no_block", ctx)
        assert ctx.extra_data.get("auto_shields_blocked") is None

    def test_auto_shields_skips_if_has_block(self, combat):
        combat.player.block = 5
        ctx = make_ctx(combat, "Auto Shields")
        execute_effect("only_if_no_block", ctx)
        assert ctx.extra_data.get("auto_shields_blocked") is True

    def test_melter_removes_enemy_block(self, combat):
        combat.enemies[0].block = 15
        ctx = make_ctx(combat, "Melter")
        execute_effect("remove_enemy_block", ctx)
        assert combat.enemies[0].block == 0


# =============================================================================
# 10. Random Target Cards
# =============================================================================

class TestRandomTargetCards:

    def test_rip_and_tear_hits_twice(self, combat):
        combat.card_random_rng = Random(42)
        card = get_card("Rip and Tear")
        ctx = make_ctx(combat, "Rip and Tear")
        before_hp = combat.enemies[0].hp

        execute_effect("damage_random_enemy_twice", ctx)

        # 7 damage * 2 hits
        assert combat.enemies[0].hp == before_hp - 14

    def test_scrape_draws_and_discards_nonzero(self, combat):
        # Put mixed cost cards in draw pile
        combat.draw_pile = ["Claw", "Defend_B", "Claw", "Strike_B"]
        ctx = make_ctx(combat, "Scrape")

        execute_effect("draw_discard_non_zero_cost", ctx)

        # Draw 4, discard non-0-cost
        # Claw(0), Defend_B(1), Claw(0), Strike_B(1) from top
        # Defend_B and Strike_B should be discarded
        zero_cost_in_hand = [c for c in combat.hand if c == "Claw"]
        assert len(zero_cost_in_hand) == 2


# =============================================================================
# 11. Block Cards
# =============================================================================

class TestBlockCards:

    def test_stack_block_equals_discard_size(self, combat):
        combat.discard_pile = ["A", "B", "C", "D", "E"]
        ctx = make_ctx(combat, "Stack")
        execute_effect("block_equals_discard_size", ctx)
        assert ctx.block_gained == 5

    def test_stack_upgraded_adds_3(self, combat):
        combat.discard_pile = ["A", "B", "C"]
        ctx = make_ctx(combat, "Stack", upgraded=True)
        execute_effect("block_equals_discard_size", ctx)
        assert ctx.block_gained == 6  # 3 + 3

    def test_steam_barrier_tracks_loss(self, combat):
        card = get_card("Steam")
        ctx = make_ctx(combat, "Steam")
        execute_effect("lose_1_block_permanently", ctx)
        # Effect tracks the loss
        assert ctx.extra_data.get(f"steam_barrier_loss_{card.id}", 0) == 1

    def test_reinforced_body_blocks_x_times(self, combat):
        card = get_card("Reinforced Body")
        ctx = make_ctx(combat, "Reinforced Body", extra={"x_cost": 3})
        execute_effect("block_x_times", ctx)
        # 7 block * 3 = 21
        assert ctx.block_gained == 21

    def test_reinforced_body_upgraded_9_per(self, combat):
        ctx = make_ctx(combat, "Reinforced Body", upgraded=True, extra={"x_cost": 2})
        execute_effect("block_x_times", ctx)
        # 9 block * 2 = 18
        assert ctx.block_gained == 18

    def test_genetic_algorithm_tracks_increase(self, combat):
        card = get_card("Genetic Algorithm")
        ctx = make_ctx(combat, "Genetic Algorithm")
        execute_effect("block_increases_permanently", ctx)
        key = f"genetic_bonus_{card.id}"
        assert ctx.extra_data.get(key, 0) == 2

    def test_genetic_algorithm_upgraded_increases_3(self, combat):
        card = get_card("Genetic Algorithm", upgraded=True)
        ctx = make_ctx(combat, "Genetic Algorithm", upgraded=True)
        execute_effect("block_increases_permanently", ctx)
        key = f"genetic_bonus_{card.id}"
        assert ctx.extra_data.get(key, 0) == 3


# =============================================================================
# 12. Energy Cards
# =============================================================================

class TestEnergyCards:

    def test_charge_battery_next_turn_energy(self, combat):
        ctx = make_ctx(combat, "Conserve Battery")
        execute_effect("gain_1_energy_next_turn", ctx)
        assert combat.player.statuses.get("EnergyNextTurn", 0) == 1

    def test_turbo_gains_energy(self, combat):
        initial = combat.energy
        ctx = make_ctx(combat, "Turbo")
        execute_effect("gain_energy_magic", ctx)
        assert combat.energy == initial + 2

    def test_turbo_upgraded_gains_3(self, combat):
        initial = combat.energy
        ctx = make_ctx(combat, "Turbo", upgraded=True)
        execute_effect("gain_energy_magic", ctx)
        assert combat.energy == initial + 3

    def test_turbo_adds_void(self, combat):
        ctx = make_ctx(combat, "Turbo")
        execute_effect("add_void_to_discard", ctx)
        assert "Void" in combat.discard_pile

    def test_double_energy_doubles(self, combat):
        combat.energy = 3
        ctx = make_ctx(combat, "Double Energy")
        execute_effect("double_energy", ctx)
        assert combat.energy == 6

    def test_aggregate_energy_per_draw_cards(self, combat):
        combat.draw_pile = ["A"] * 12
        ctx = make_ctx(combat, "Aggregate")  # divisor=4
        initial = combat.energy
        execute_effect("gain_energy_per_x_cards_in_draw", ctx)
        # 12 // 4 = 3
        assert combat.energy == initial + 3

    def test_aggregate_upgraded_divisor_3(self, combat):
        combat.draw_pile = ["A"] * 12
        ctx = make_ctx(combat, "Aggregate", upgraded=True)  # divisor=3
        initial = combat.energy
        execute_effect("gain_energy_per_x_cards_in_draw", ctx)
        # 12 // 3 = 4
        assert combat.energy == initial + 4

    def test_overclock_adds_burn(self, combat):
        ctx = make_ctx(combat, "Steam Power")
        execute_effect("add_burn_to_discard", ctx)
        assert "Burn" in combat.discard_pile


# =============================================================================
# 13. Lock-On
# =============================================================================

class TestLockOn:

    def test_lockon_applies_debuff(self, combat):
        ctx = make_ctx(combat, "Lockon")
        execute_effect("apply_lockon", ctx)
        assert combat.enemies[0].statuses.get("Lock-On", 0) == 2

    def test_lockon_upgraded_3(self, combat):
        ctx = make_ctx(combat, "Lockon", upgraded=True)
        execute_effect("apply_lockon", ctx)
        assert combat.enemies[0].statuses.get("Lock-On", 0) == 3


# =============================================================================
# 14. Retain / Equilibrium
# =============================================================================

class TestRetain:

    def test_equilibrium_retains_hand(self, combat):
        ctx = make_ctx(combat, "Undo")
        execute_effect("retain_hand", ctx)
        assert combat.player.statuses.get("RetainHand", 0) == 1


# =============================================================================
# 15. Claw / Streamline / Force Field Mechanics
# =============================================================================

class TestSpecialCostMechanics:

    def test_claw_increases_damage(self, combat):
        ctx = make_ctx(combat, "Claw")
        execute_effect("increase_all_claw_damage", ctx)
        assert ctx.extra_data.get("claw_bonus", 0) == 2

    def test_streamline_reduces_cost(self, combat):
        card = get_card("Streamline")
        ctx = make_ctx(combat, "Streamline")
        execute_effect("reduce_cost_permanently", ctx)
        assert combat.card_costs.get("Streamline", 2) == 1

    def test_streamline_cost_floors_at_0(self, combat):
        card = get_card("Streamline")
        combat.card_costs["Streamline"] = 0
        ctx = make_ctx(combat, "Streamline")
        execute_effect("reduce_cost_permanently", ctx)
        assert combat.card_costs.get("Streamline") == 0

    def test_force_field_effect_registered(self, combat):
        """Force Field's cost_reduces_per_power_played is a passive tracker."""
        handler = get_effect_handler("cost_reduces_per_power_played")
        assert handler is not None


# =============================================================================
# 16. Card Data Parity (values verified against Java)
# =============================================================================

class TestCardDataParity:
    """Verify all Defect card data values match Java source."""

    @pytest.mark.parametrize("card_id,cost,damage,block,magic", [
        # Basic
        ("Strike_B", 1, 6, -1, -1),
        ("Defend_B", 1, -1, 5, -1),
        ("Zap", 1, -1, -1, 1),
        ("Dualcast", 1, -1, -1, -1),
        # Common Attacks
        ("Ball Lightning", 1, 7, -1, 1),
        ("Barrage", 1, 4, -1, -1),
        ("Beam Cell", 0, 3, -1, 1),
        ("Claw", 0, 3, -1, 2),
        ("Cold Snap", 1, 6, -1, 1),
        ("Compile Driver", 1, 7, -1, 1),
        ("Go for the Eyes", 0, 3, -1, 1),
        ("Rebound", 1, 9, -1, -1),
        ("Streamline", 2, 15, -1, 1),
        ("Sweeping Beam", 1, 6, -1, 1),
        # Common Skills
        ("Conserve Battery", 1, -1, 7, -1),
        ("Coolheaded", 1, -1, -1, 1),
        ("Hologram", 1, -1, 3, -1),
        ("Leap", 1, -1, 9, -1),
        ("Redo", 1, -1, -1, -1),
        ("Stack", 1, -1, 0, -1),
        ("Steam", 0, -1, 6, -1),
        ("Turbo", 0, -1, -1, 2),
        # Uncommon Attacks
        ("Blizzard", 1, 0, -1, 2),
        ("Doom and Gloom", 2, 10, -1, 1),
        ("FTL", 0, 5, -1, 3),
        ("Lockon", 1, 8, -1, 2),
        ("Melter", 1, 10, -1, -1),
        ("Rip and Tear", 1, 7, -1, 2),
        ("Scrape", 1, 7, -1, 4),
        ("Sunder", 3, 24, -1, -1),
        # Uncommon Skills
        ("Aggregate", 1, -1, -1, 4),
        ("Auto Shields", 1, -1, 11, -1),
        ("BootSequence", 0, -1, 10, -1),
        ("Chaos", 1, -1, -1, 1),
        ("Chill", 0, -1, -1, 1),
        ("Consume", 2, -1, -1, 2),
        ("Darkness", 1, -1, -1, 1),
        ("Double Energy", 1, -1, -1, -1),
        ("Undo", 2, -1, 13, 1),
        ("Force Field", 4, -1, 12, -1),
        ("Fusion", 2, -1, -1, 1),
        ("Genetic Algorithm", 1, -1, 1, 2),
        ("Glacier", 2, -1, 7, 2),
        ("Steam Power", 0, -1, -1, 2),
        ("Recycle", 1, -1, -1, -1),
        ("Reinforced Body", -1, -1, 7, -1),
        ("Reprogram", 1, -1, -1, 1),
        ("Skim", 1, -1, -1, 3),
        ("Tempest", -1, -1, -1, -1),
        ("White Noise", 1, -1, -1, -1),
        # "Impulse" removed: not in CardLibrary (dead code)
        # Uncommon Powers
        ("Capacitor", 1, -1, -1, 2),
        ("Defragment", 1, -1, -1, 1),
        ("Heatsinks", 1, -1, -1, 1),
        ("Hello World", 1, -1, -1, -1),
        ("Loop", 1, -1, -1, 1),
        ("Self Repair", 1, -1, -1, 7),
        ("Static Discharge", 1, -1, -1, 1),
        ("Storm", 1, -1, -1, 1),
        # Rare Attacks
        ("All For One", 2, 10, -1, -1),
        ("Core Surge", 1, 11, -1, 1),
        ("Hyperbeam", 2, 26, -1, 3),
        ("Meteor Strike", 5, 24, -1, 3),
        ("Thunder Strike", 3, 7, -1, -1),
        # Rare Skills
        ("Amplify", 1, -1, -1, 1),
        ("Fission", 0, -1, -1, 1),
        ("Multi-Cast", -1, -1, -1, -1),
        ("Rainbow", 2, -1, -1, -1),
        ("Reboot", 0, -1, -1, 4),
        ("Seek", 0, -1, -1, 1),
        # Rare Powers
        ("Biased Cognition", 1, -1, -1, 4),
        ("Buffer", 2, -1, -1, 1),
        ("Creative AI", 3, -1, -1, 1),
        ("Echo Form", 3, -1, -1, -1),
        ("Electrodynamics", 2, -1, -1, 2),
        ("Machine Learning", 1, -1, -1, 1),
    ])
    def test_card_base_values(self, card_id, cost, damage, block, magic):
        card = get_card(card_id)
        assert card.cost == cost, f"{card_id} cost: {card.cost} != {cost}"
        assert card.damage == damage, f"{card_id} damage: {card.damage} != {damage}"
        assert card.block == block, f"{card_id} block: {card.block} != {block}"
        assert card.magic_number == magic, f"{card_id} magic: {card.magic_number} != {magic}"

    @pytest.mark.parametrize("card_id,upgrade_damage,upgrade_block,upgrade_magic,upgrade_cost", [
        ("Strike_B", 3, 0, 0, None),
        ("Defend_B", 0, 3, 0, None),
        ("Zap", 0, 0, 0, 0),
        ("Dualcast", 0, 0, 0, 0),
        ("Ball Lightning", 3, 0, 0, None),
        ("Barrage", 2, 0, 0, None),
        ("Beam Cell", 1, 0, 1, None),
        ("Claw", 2, 0, 0, None),
        ("Cold Snap", 3, 0, 0, None),
        ("Compile Driver", 3, 0, 0, None),
        ("Go for the Eyes", 1, 0, 1, None),
        ("Rebound", 3, 0, 0, None),
        ("Streamline", 5, 0, 0, None),
        ("Sweeping Beam", 3, 0, 0, None),
        ("Conserve Battery", 0, 3, 0, None),
        ("Coolheaded", 0, 0, 1, None),
        ("Hologram", 0, 2, 0, None),
        ("Leap", 0, 3, 0, None),
        ("Redo", 0, 0, 0, 0),
        ("Stack", 0, 3, 0, None),
        ("Steam", 0, 2, 0, None),
        ("Turbo", 0, 0, 1, None),
        ("Blizzard", 0, 0, 1, None),
        ("Doom and Gloom", 4, 0, 0, None),
        ("FTL", 1, 0, 1, None),
        ("Lockon", 3, 0, 1, None),
        ("Melter", 4, 0, 0, None),
        ("Rip and Tear", 2, 0, 0, None),
        ("Scrape", 3, 0, 1, None),
        ("Sunder", 8, 0, 0, None),
        ("Aggregate", 0, 0, -1, None),
        ("Auto Shields", 0, 4, 0, None),
        ("BootSequence", 0, 3, 0, None),
        ("Chaos", 0, 0, 1, None),
        ("Consume", 0, 0, 1, None),
        ("Double Energy", 0, 0, 0, 0),
        ("Undo", 0, 3, 0, None),
        ("Force Field", 0, 4, 0, None),
        ("Fusion", 0, 0, 0, 1),
        ("Genetic Algorithm", 0, 0, 1, None),
        ("Glacier", 0, 3, 0, None),
        ("Steam Power", 0, 0, 1, None),
        ("Recycle", 0, 0, 0, 0),
        ("Reinforced Body", 0, 2, 0, None),
        ("Reprogram", 0, 0, 1, None),
        ("Skim", 0, 0, 1, None),
        ("White Noise", 0, 0, 0, 0),
        ("Capacitor", 0, 0, 1, None),
        ("Defragment", 0, 0, 1, None),
        ("Heatsinks", 0, 0, 1, None),
        ("Loop", 0, 0, 1, None),
        ("Self Repair", 0, 0, 3, None),
        ("Static Discharge", 0, 0, 1, None),
        ("All For One", 4, 0, 0, None),
        ("Core Surge", 4, 0, 0, None),
        ("Hyperbeam", 8, 0, 0, None),
        ("Meteor Strike", 6, 0, 0, None),
        ("Thunder Strike", 2, 0, 0, None),
        ("Amplify", 0, 0, 1, None),
        ("Reboot", 0, 0, 2, None),
        ("Seek", 0, 0, 1, None),
        ("Biased Cognition", 0, 0, 1, None),
        ("Buffer", 0, 0, 1, None),
        ("Creative AI", 0, 0, 0, 2),
        ("Electrodynamics", 0, 0, 1, None),
    ])
    def test_card_upgrade_values(self, card_id, upgrade_damage, upgrade_block,
                                  upgrade_magic, upgrade_cost):
        card = get_card(card_id)
        assert card.upgrade_damage == upgrade_damage, f"{card_id} upg_damage"
        assert card.upgrade_block == upgrade_block, f"{card_id} upg_block"
        assert card.upgrade_magic == upgrade_magic, f"{card_id} upg_magic"
        assert card.upgrade_cost == upgrade_cost, f"{card_id} upg_cost"


# =============================================================================
# 17. Upgrade Flag Changes
# =============================================================================

class TestUpgradeFlagChanges:
    """Verify cards that change exhaust/ethereal on upgrade."""

    def test_hologram_upgraded_removes_exhaust(self):
        card = get_card("Hologram", upgraded=True)
        assert card.exhaust is False

    def test_rainbow_upgraded_removes_exhaust(self):
        card = get_card("Rainbow", upgraded=True)
        assert card.exhaust is False

    def test_echo_form_upgraded_removes_ethereal(self):
        card = get_card("Echo Form", upgraded=True)
        assert card.ethereal is False

    def test_impulse_is_dead_code(self):
        """Impulse is dead code (not in CardLibrary) and should raise ValueError."""
        import pytest
        with pytest.raises(ValueError, match="dead code"):
            get_card("Impulse")

    def test_chill_always_exhausts(self):
        base = get_card("Chill")
        up = get_card("Chill", upgraded=True)
        assert base.exhaust is True
        assert up.exhaust is True

    def test_tempest_always_exhausts(self):
        base = get_card("Tempest")
        up = get_card("Tempest", upgraded=True)
        assert base.exhaust is True
        assert up.exhaust is True

    def test_seek_always_exhausts(self):
        base = get_card("Seek")
        up = get_card("Seek", upgraded=True)
        assert base.exhaust is True
        assert up.exhaust is True

    def test_double_energy_always_exhausts(self):
        base = get_card("Double Energy")
        up = get_card("Double Energy", upgraded=True)
        assert base.exhaust is True
        assert up.exhaust is True


# =============================================================================
# 18. Card Color & Type
# =============================================================================

class TestCardColorAndType:
    """All Defect cards must be BLUE."""

    def test_all_defect_cards_are_blue(self):
        for card_id, card in DEFECT_CARDS.items():
            assert card.color == CardColor.BLUE, f"{card_id} is not BLUE"

    @pytest.mark.parametrize("card_id,expected_type", [
        ("Zap", CardType.SKILL),
        ("Dualcast", CardType.SKILL),
        ("Ball Lightning", CardType.ATTACK),
        ("Coolheaded", CardType.SKILL),
        ("Defragment", CardType.POWER),
        ("Echo Form", CardType.POWER),
        ("Storm", CardType.POWER),
        ("Biased Cognition", CardType.POWER),
        ("Buffer", CardType.POWER),
        ("Amplify", CardType.SKILL),
        ("Seek", CardType.SKILL),
        ("All For One", CardType.ATTACK),
        ("Hyperbeam", CardType.ATTACK),
        ("Sunder", CardType.ATTACK),
        ("Aggregate", CardType.SKILL),
        # ("Impulse", CardType.SKILL),  # Removed: not in CardLibrary (dead code)
    ])
    def test_card_type(self, card_id, expected_type):
        card = get_card(card_id)
        assert card.card_type == expected_type, f"{card_id} type"


# =============================================================================
# 19. DEFECT_CARD_EFFECTS map completeness
# =============================================================================

class TestEffectsMapCompleteness:
    """The DEFECT_CARD_EFFECTS dict should have entries for all key cards."""

    def test_all_68_cards_have_effect_entries(self):
        expected = [
            "Strike_B", "Defend_B", "Zap", "Dualcast",
            "Ball Lightning", "Barrage", "Beam Cell", "Gash",  # Claw.java ID = "Gash"
            "Cold Snap", "Compile Driver", "Go for the Eyes",
            "Rebound", "Streamline", "Sweeping Beam",
            "Conserve Battery", "Coolheaded", "Hologram", "Leap",
            "Redo", "Stack", "Steam", "Turbo",
            "Blizzard", "Doom and Gloom", "FTL", "Lockon",
            "Melter", "Rip and Tear", "Scrape", "Sunder",
            "Aggregate", "Auto Shields", "BootSequence", "Chaos",
            "Chill", "Consume", "Darkness", "Double Energy",
            "Undo", "Force Field", "Fusion", "Genetic Algorithm",
            "Glacier", "Steam Power", "Recycle", "Reinforced Body",
            "Reprogram", "Skim", "Tempest", "White Noise",
            "Capacitor", "Defragment", "Heatsinks", "Hello World",
            "Loop", "Self Repair", "Static Discharge", "Storm",
            "All For One", "Core Surge", "Hyperbeam",
            "Meteor Strike", "Thunder Strike",
            "Amplify", "Fission", "Multi-Cast", "Rainbow",
            "Reboot", "Seek",
            "Biased Cognition", "Buffer", "Creative AI",
            "Echo Form", "Electrodynamics", "Machine Learning",
        ]
        missing = [c for c in expected if c not in DEFECT_CARD_EFFECTS]
        assert not missing, f"Missing from DEFECT_CARD_EFFECTS: {missing}"


# =============================================================================
# 20. trigger_orb_start_end effect (formerly Impulse card, now dead code)
# =============================================================================

class TestTriggerOrbStartEnd:
    """The trigger_orb_start_end effect handler still exists even though
    the Impulse card was removed (not in CardLibrary, dead code).
    Test the effect handler directly without referencing the dead card."""

    def test_trigger_orb_start_end_fires_frost_passive(self, combat):
        """trigger_orb_start_end triggers each orb's passive once."""
        channel_orb(combat, "Frost")
        before_block = combat.player.block

        ctx = make_ctx(combat)  # No card needed
        execute_effect("trigger_orb_start_end", ctx)

        # Frost passive fires once -> 2 block
        assert combat.player.block == before_block + 2
