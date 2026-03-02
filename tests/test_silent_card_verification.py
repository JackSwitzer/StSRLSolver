"""
CRD-SI-001: Silent Card Behavior Verification Tests

Comprehensive behavioral tests for all 61 Silent cards.
Each test verifies that the effect handler produces the correct outcome
when executed against a CombatState, matching Java decompiled behavior.
"""

import pytest

from packages.engine.state.combat import (
    CombatState, EntityState, EnemyCombatState,
)
from packages.engine.content.cards import (
    Card, CardType, CardRarity, CardTarget, CardColor,
    get_card, ALL_CARDS,
)
from packages.engine.effects.registry import EffectContext, execute_effect
from packages.engine.effects.executor import EffectExecutor, create_executor


# =============================================================================
# Test Helpers
# =============================================================================

def make_enemy(hp=44, statuses=None, enemy_id="JawWorm"):
    """Create a standard test enemy."""
    return EnemyCombatState(
        hp=hp, max_hp=hp, block=0,
        statuses=statuses or {},
        id=enemy_id,
        move_id=0,
        move_damage=11,
        move_hits=1,
        move_block=0,
        move_effects={},
    )


def make_combat(
    hand=None,
    draw_pile=None,
    discard_pile=None,
    energy=3,
    enemies=None,
    player_hp=80,
    statuses=None,
    relics=None,
):
    """Create a CombatState for testing."""
    if enemies is None:
        enemies = [make_enemy()]
    return CombatState(
        player=EntityState(hp=player_hp, max_hp=80, block=0, statuses=statuses or {}),
        energy=energy,
        max_energy=3,
        stance="Neutral",
        hand=hand or [],
        draw_pile=draw_pile or [],
        discard_pile=discard_pile or [],
        exhaust_pile=[],
        enemies=enemies,
        potions=[],
        relics=relics or [],
        turn=1,
        cards_played_this_turn=0,
        attacks_played_this_turn=0,
        skills_played_this_turn=0,
        powers_played_this_turn=0,
        relic_counters={},
        card_costs={},
    )


def play_card_on_state(state, card, target_idx=0, free=False):
    """Play a card using the EffectExecutor."""
    executor = create_executor(state)
    return executor.play_card(card, target_idx=target_idx, free=free)


def make_ctx(state, card=None, target=None, target_idx=-1):
    """Create an EffectContext for direct effect testing."""
    return EffectContext(
        state=state,
        card=card,
        target=target,
        target_idx=target_idx,
        is_upgraded=card.upgraded if card else False,
        magic_number=card.magic_number if card and card.magic_number > 0 else 0,
    )


# =============================================================================
# POISON CARD BEHAVIOR TESTS
# =============================================================================


class TestPoisonBehavior:
    """Test poison-related card effects execute correctly."""

    def test_deadly_poison_applies_poison(self):
        """Deadly Poison: apply 5 Poison to target."""
        state = make_combat()
        card = get_card("Deadly Poison")
        ctx = make_ctx(state, card, state.enemies[0], 0)
        execute_effect("apply_poison", ctx)
        assert state.enemies[0].statuses.get("Poison", 0) == 5

    def test_deadly_poison_upgraded_applies_7(self):
        """Deadly Poison+: apply 7 Poison to target."""
        state = make_combat()
        card = get_card("Deadly Poison", upgraded=True)
        ctx = make_ctx(state, card, state.enemies[0], 0)
        execute_effect("apply_poison", ctx)
        assert state.enemies[0].statuses.get("Poison", 0) == 7

    def test_poisoned_stab_applies_poison(self):
        """Poisoned Stab: deal damage AND apply 3 Poison."""
        state = make_combat(hand=["Poisoned Stab"], energy=1)
        card = get_card("Poisoned Stab")
        result = play_card_on_state(state, card, target_idx=0)
        assert result.success
        assert state.enemies[0].statuses.get("Poison", 0) == 3

    def test_poisoned_stab_upgraded_applies_4(self):
        """Poisoned Stab+: apply 4 Poison."""
        state = make_combat(hand=["Poisoned Stab+"], energy=1)
        card = get_card("Poisoned Stab", upgraded=True)
        result = play_card_on_state(state, card, target_idx=0)
        assert state.enemies[0].statuses.get("Poison", 0) == 4

    def test_bane_double_damage_on_poisoned(self):
        """Bane: if target poisoned, deal damage twice (Java: BaneAction)."""
        state = make_combat(hand=["Bane"], energy=1)
        state.enemies[0].statuses["Poison"] = 5
        card = get_card("Bane")
        result = play_card_on_state(state, card, target_idx=0)
        assert result.success
        # Bane deals base damage + extra damage when poisoned
        assert result.damage_dealt > 0

    def test_bane_no_double_without_poison(self):
        """Bane: no extra damage if target not poisoned."""
        enemy = make_enemy(hp=100)
        state = make_combat(hand=["Bane"], energy=1, enemies=[enemy])
        card = get_card("Bane")
        result = play_card_on_state(state, card, target_idx=0)
        # Without poison, should only deal base 7 damage
        assert result.damage_dealt == 7

    def test_catalyst_doubles_poison(self):
        """Catalyst: double target's Poison (Java: DoublePoisonAction)."""
        state = make_combat()
        state.enemies[0].statuses["Poison"] = 5
        card = get_card("Catalyst")
        ctx = make_ctx(state, card, state.enemies[0], 0)
        execute_effect("double_poison", ctx)
        assert state.enemies[0].statuses["Poison"] == 10  # 5 + 5

    def test_catalyst_upgraded_triples_poison(self):
        """Catalyst+: triple target's Poison (Java: TriplePoisonAction)."""
        state = make_combat()
        state.enemies[0].statuses["Poison"] = 5
        card = get_card("Catalyst", upgraded=True)
        ctx = make_ctx(state, card, state.enemies[0], 0)
        execute_effect("double_poison", ctx)
        assert state.enemies[0].statuses["Poison"] == 15  # 5 + 10

    def test_noxious_fumes_applies_power(self):
        """Noxious Fumes: apply NoxiousFumes power to player (2 base)."""
        state = make_combat()
        card = get_card("Noxious Fumes")
        ctx = make_ctx(state, card)
        execute_effect("apply_poison_all_each_turn", ctx)
        assert state.player.statuses.get("NoxiousFumes", 0) == 2

    def test_noxious_fumes_upgraded_3(self):
        """Noxious Fumes+: NoxiousFumes power with amount 3."""
        state = make_combat()
        card = get_card("Noxious Fumes", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("apply_poison_all_each_turn", ctx)
        assert state.player.statuses.get("NoxiousFumes", 0) == 3

    def test_bouncing_flask_applies_poison_3_times(self):
        """Bouncing Flask: apply 3 Poison to random enemies 3 times."""
        enemies = [make_enemy(hp=30, enemy_id=f"Louse{i}") for i in range(2)]
        state = make_combat(enemies=enemies)
        state.card_rng_state = (42, 17)  # Deterministic seed
        card = get_card("Bouncing Flask")
        ctx = make_ctx(state, card, state.enemies[0], 0)
        execute_effect("apply_poison_random_3_times", ctx)
        total_poison = sum(e.statuses.get("Poison", 0) for e in state.enemies)
        assert total_poison == 9  # 3 poison * 3 bounces

    def test_crippling_poison_applies_to_all(self):
        """Crippling Poison: Poison + Weak to all enemies."""
        enemies = [make_enemy(hp=30, enemy_id=f"Louse{i}") for i in range(3)]
        state = make_combat(enemies=enemies)
        card = get_card("Crippling Poison")
        ctx = make_ctx(state, card)
        execute_effect("apply_poison_all", ctx)
        execute_effect("apply_weak_2_all", ctx)
        for e in state.enemies:
            assert e.statuses.get("Poison", 0) == 4
            assert e.statuses.get("Weak", 0) == 2

    def test_corpse_explosion_applies_poison_and_power(self):
        """Corpse Explosion: apply 6 Poison + CorpseExplosion to target."""
        state = make_combat()
        card = get_card("Corpse Explosion")
        ctx = make_ctx(state, card, state.enemies[0], 0)
        execute_effect("apply_poison", ctx)
        execute_effect("apply_corpse_explosion", ctx)
        assert state.enemies[0].statuses.get("Poison", 0) == 6
        assert state.enemies[0].statuses.get("CorpseExplosion", 0) == 1


# =============================================================================
# SHIV CARD BEHAVIOR TESTS
# =============================================================================


class TestShivBehavior:
    """Test shiv generation and related effects."""

    def test_blade_dance_adds_3_shivs(self):
        """Blade Dance: add 3 Shivs to hand."""
        state = make_combat(hand=[])
        card = get_card("Blade Dance")
        ctx = make_ctx(state, card)
        execute_effect("add_shivs_to_hand", ctx)
        assert state.hand.count("Shiv") == 3

    def test_blade_dance_upgraded_adds_4_shivs(self):
        """Blade Dance+: add 4 Shivs to hand."""
        state = make_combat(hand=[])
        card = get_card("Blade Dance", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("add_shivs_to_hand", ctx)
        assert state.hand.count("Shiv") == 4

    def test_cloak_and_dagger_adds_1_shiv(self):
        """Cloak and Dagger: 6 block + add 1 Shiv."""
        state = make_combat(hand=[])
        card = get_card("Cloak And Dagger")
        ctx = make_ctx(state, card)
        execute_effect("add_shivs_to_hand", ctx)
        assert state.hand.count("Shiv") == 1

    def test_cloak_and_dagger_upgraded_adds_2_shivs(self):
        """Cloak and Dagger+: add 2 Shivs."""
        state = make_combat(hand=[])
        card = get_card("Cloak And Dagger", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("add_shivs_to_hand", ctx)
        assert state.hand.count("Shiv") == 2

    def test_storm_of_steel_discards_hand_adds_shivs(self):
        """Storm of Steel: discard hand, add Shivs equal to discarded."""
        state = make_combat(hand=["Strike_G", "Defend_G", "Neutralize"])
        card = get_card("Storm of Steel")
        ctx = make_ctx(state, card)
        # Discard hand first
        execute_effect("discard_hand", ctx)
        assert len(state.hand) == 0
        assert ctx.extra_data["cards_discarded_count"] == 3
        # Then add shivs
        execute_effect("add_shivs_equal_to_discarded", ctx)
        assert state.hand.count("Shiv") == 3

    def test_storm_of_steel_upgraded_adds_upgraded_shivs(self):
        """Storm of Steel+: Shivs generated are upgraded."""
        state = make_combat(hand=["Strike_G", "Defend_G"])
        card = get_card("Storm of Steel", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("discard_hand", ctx)
        execute_effect("add_shivs_equal_to_discarded", ctx)
        assert state.hand.count("Shiv+") == 2

    def test_accuracy_sets_power(self):
        """Accuracy: apply Accuracy power with amount 4."""
        state = make_combat()
        card = get_card("Accuracy")
        ctx = make_ctx(state, card)
        execute_effect("shivs_deal_more_damage", ctx)
        assert state.player.statuses.get("Accuracy", 0) == 4

    def test_accuracy_upgraded_6(self):
        """Accuracy+: Accuracy power with amount 6."""
        state = make_combat()
        card = get_card("Accuracy", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("shivs_deal_more_damage", ctx)
        assert state.player.statuses.get("Accuracy", 0) == 6

    def test_infinite_blades_sets_power(self):
        """Infinite Blades: apply InfiniteBlades power."""
        state = make_combat()
        card = get_card("Infinite Blades")
        ctx = make_ctx(state, card)
        execute_effect("add_shiv_each_turn", ctx)
        assert state.player.statuses.get("InfiniteBlades", 0) == 1


# =============================================================================
# DISCARD CARD BEHAVIOR TESTS
# =============================================================================


class TestDiscardBehavior:
    """Test discard-related effects."""

    def test_acrobatics_draws_3_discards_1(self):
        """Acrobatics: draw 3 cards, then discard 1."""
        state = make_combat(
            hand=["Acrobatics"],
            draw_pile=["Strike_G", "Defend_G", "Neutralize", "Slice"],
        )
        card = get_card("Acrobatics")
        ctx = make_ctx(state, card)
        execute_effect("draw_x", ctx)
        assert len(state.hand) == 4  # Acrobatics + 3 drawn
        execute_effect("discard_1", ctx)
        assert len(state.hand) == 3  # After discarding 1
        assert len(state.discard_pile) == 1

    def test_acrobatics_upgraded_draws_4(self):
        """Acrobatics+: draw 4 cards, discard 1."""
        state = make_combat(
            hand=[],
            draw_pile=["Strike_G", "Defend_G", "Neutralize", "Slice", "Bane"],
        )
        card = get_card("Acrobatics", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("draw_x", ctx)
        assert len(state.hand) == 4

    def test_prepared_draws_and_discards_1(self):
        """Prepared: draw 1, discard 1."""
        state = make_combat(
            hand=["Prepared"],
            draw_pile=["Strike_G"],
        )
        card = get_card("Prepared")
        ctx = make_ctx(state, card)
        execute_effect("draw_x", ctx)
        assert len(state.hand) == 2  # Prepared + Strike_G
        execute_effect("discard_x", ctx)
        assert len(state.hand) == 1
        assert len(state.discard_pile) == 1

    def test_prepared_upgraded_draws_and_discards_2(self):
        """Prepared+: draw 2, discard 2."""
        state = make_combat(
            hand=[],
            draw_pile=["Strike_G", "Defend_G", "Neutralize"],
        )
        card = get_card("Prepared", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("draw_x", ctx)
        assert len(state.hand) == 2
        execute_effect("discard_x", ctx)
        assert len(state.hand) == 0
        assert len(state.discard_pile) == 2

    def test_calculated_gamble_discards_hand_draws_same(self):
        """Calculated Gamble: discard hand, draw equal number."""
        state = make_combat(
            hand=["Strike_G", "Defend_G", "Neutralize"],
            draw_pile=["Bane", "Slice", "Footwork", "Backflip"],
        )
        card = get_card("Calculated Gamble")
        ctx = make_ctx(state, card)
        execute_effect("discard_hand_draw_same", ctx)
        assert len(state.hand) == 3  # Drew same number as discarded
        assert len(state.discard_pile) == 3

    def test_calculated_gamble_base_exhausts(self):
        """Calculated Gamble base: exhaust=True."""
        card = get_card("Calculated Gamble")
        assert card.exhaust is True

    def test_calculated_gamble_upgraded_no_exhaust(self):
        """Calculated Gamble+: exhaust=False (Java: this.exhaust = false on upgrade)."""
        card = get_card("Calculated Gamble", upgraded=True)
        assert card.exhaust is False

    def test_all_out_attack_discards_random(self):
        """All-Out Attack: discard 1 random card after dealing damage."""
        state = make_combat(hand=["All Out Attack", "Strike_G", "Defend_G"])
        card = get_card("All Out Attack")
        ctx = make_ctx(state, card)
        initial_hand_size = len(state.hand)
        execute_effect("discard_random_1", ctx)
        # One card should be discarded
        assert len(state.hand) == initial_hand_size - 1
        assert len(state.discard_pile) == 1

    def test_concentrate_discards_3_gains_2_energy(self):
        """Concentrate: discard 3 cards, gain 2 energy."""
        state = make_combat(
            hand=["Strike_G", "Defend_G", "Neutralize", "Slice"],
            energy=0,
        )
        card = get_card("Concentrate")
        ctx = make_ctx(state, card)
        execute_effect("discard_x", ctx)
        assert len(state.discard_pile) == 3
        execute_effect("gain_energy_2", ctx)
        assert state.energy == 2

    def test_concentrate_upgraded_discards_2(self):
        """Concentrate+: discard 2 (reduced), gain 2 energy."""
        state = make_combat(
            hand=["Strike_G", "Defend_G", "Neutralize"],
            energy=0,
        )
        card = get_card("Concentrate", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("discard_x", ctx)
        assert len(state.discard_pile) == 2  # Upgraded: magic_number = 2

    def test_dagger_throw_draws_1_discards_1(self):
        """Dagger Throw: deal damage, draw 1, discard 1."""
        state = make_combat(
            hand=["Dagger Throw"],
            draw_pile=["Bane"],
        )
        card = get_card("Dagger Throw")
        ctx = make_ctx(state, card, state.enemies[0], 0)
        execute_effect("draw_1", ctx)
        assert "Bane" in state.hand
        execute_effect("discard_1", ctx)
        assert len(state.discard_pile) == 1

    def test_survivor_block_and_discard(self):
        """Survivor: gain 8 block, discard 1."""
        state = make_combat(hand=["Survivor", "Strike_G"], energy=1)
        card = get_card("Survivor")
        result = play_card_on_state(state, card, target_idx=-1)
        assert result.success
        assert state.player.block == 8
        assert len(state.discard_pile) >= 1


# =============================================================================
# DISCARD TRIGGER BEHAVIOR TESTS
# =============================================================================


class TestDiscardTriggerBehavior:
    """Test cards that trigger when discarded."""

    def test_reflex_is_unplayable(self):
        """Reflex: cost -2 means unplayable."""
        card = get_card("Reflex")
        assert card.cost == -2
        assert "unplayable" in card.effects

    def test_reflex_draw_on_discard(self):
        """Reflex: when manually discarded, draw 2 cards."""
        state = make_combat(
            hand=["Reflex", "Strike_G"],
            draw_pile=["Bane", "Slice", "Defend_G"],
        )
        ctx = make_ctx(state, get_card("Reflex"))
        initial_hand = len(state.hand)
        # Manually discard Reflex
        ctx.discard_card("Reflex")
        # Reflex trigger should have drawn 2 cards
        assert len(state.hand) == initial_hand - 1 + 2  # -1 discard, +2 draw

    def test_reflex_upgraded_draws_3(self):
        """Reflex+: draw 3 when discarded."""
        state = make_combat(
            hand=["Reflex+", "Strike_G"],
            draw_pile=["Bane", "Slice", "Defend_G", "Neutralize"],
        )
        ctx = make_ctx(state, get_card("Reflex", upgraded=True))
        initial_hand = len(state.hand)
        ctx.discard_card("Reflex+")
        assert len(state.hand) == initial_hand - 1 + 3

    def test_tactician_is_unplayable(self):
        """Tactician: cost -2 means unplayable."""
        card = get_card("Tactician")
        assert card.cost == -2
        assert "unplayable" in card.effects

    def test_tactician_energy_on_discard(self):
        """Tactician: when manually discarded, gain 1 energy."""
        state = make_combat(hand=["Tactician", "Strike_G"], energy=0)
        ctx = make_ctx(state, get_card("Tactician"))
        ctx.discard_card("Tactician")
        assert state.energy == 1

    def test_tactician_upgraded_energy_2(self):
        """Tactician+: gain 2 energy when discarded."""
        state = make_combat(hand=["Tactician+", "Strike_G"], energy=0)
        ctx = make_ctx(state, get_card("Tactician", upgraded=True))
        ctx.discard_card("Tactician+")
        assert state.energy == 2

    def test_eviscerate_cost_reduces_per_discard(self):
        """Eviscerate: cost starts at 3, tracked as passive (Java: didDiscard)."""
        card = get_card("Eviscerate")
        assert card.cost == 3
        assert "cost_reduces_per_discard" in card.effects

    def test_sneaky_strike_refunds_if_discarded(self):
        """Sneaky Strike: refund 2 energy if a card was discarded this turn."""
        state = make_combat(hand=["Underhanded Strike"], energy=2)
        state.discarded_this_turn = 1  # Something was discarded
        card = get_card("Underhanded Strike")
        ctx = make_ctx(state, card, state.enemies[0], 0)
        execute_effect("refund_2_energy_if_discarded_this_turn", ctx)
        assert state.energy == 4  # 2 + 2 refund

    def test_sneaky_strike_no_refund_without_discard(self):
        """Sneaky Strike: no refund if nothing discarded."""
        state = make_combat(hand=["Underhanded Strike"], energy=2)
        state.discarded_this_turn = 0
        card = get_card("Underhanded Strike")
        ctx = make_ctx(state, card, state.enemies[0], 0)
        execute_effect("refund_2_energy_if_discarded_this_turn", ctx)
        assert state.energy == 2  # No refund


# =============================================================================
# X-COST CARD BEHAVIOR TESTS
# =============================================================================


class TestXCostBehavior:
    """Test X-cost card effects."""

    def test_skewer_deals_damage_x_times(self):
        """Skewer: deal 7 damage X times based on energy spent."""
        enemy = make_enemy(hp=100)
        state = make_combat(enemies=[enemy], energy=3)
        card = get_card("Skewer")
        ctx = make_ctx(state, card, state.enemies[0], 0)
        ctx.energy_spent = 3
        execute_effect("damage_x_times_energy", ctx)
        assert ctx.damage_dealt == 21  # 7 * 3

    def test_skewer_upgraded_10_damage(self):
        """Skewer+: deal 10 damage X times."""
        enemy = make_enemy(hp=100)
        state = make_combat(enemies=[enemy], energy=2)
        card = get_card("Skewer", upgraded=True)
        ctx = make_ctx(state, card, state.enemies[0], 0)
        ctx.energy_spent = 2
        execute_effect("damage_x_times_energy", ctx)
        assert ctx.damage_dealt == 20  # 10 * 2

    def test_malaise_applies_weak_x(self):
        """Malaise: apply X Weak to target."""
        state = make_combat(energy=3)
        card = get_card("Malaise")
        ctx = make_ctx(state, card, state.enemies[0], 0)
        ctx.energy_spent = 3
        execute_effect("apply_weak_x", ctx)
        assert state.enemies[0].statuses.get("Weak", 0) == 3

    def test_malaise_applies_strength_down_x(self):
        """Malaise: apply X Strength down to target."""
        state = make_combat(energy=3)
        card = get_card("Malaise")
        ctx = make_ctx(state, card, state.enemies[0], 0)
        ctx.energy_spent = 3
        execute_effect("apply_strength_down_x", ctx)
        assert state.enemies[0].statuses.get("Strength", 0) == -3

    def test_malaise_upgraded_x_plus_1(self):
        """Malaise+: apply X+1 Weak and Strength down."""
        state = make_combat(energy=3)
        card = get_card("Malaise", upgraded=True)
        ctx = make_ctx(state, card, state.enemies[0], 0)
        ctx.energy_spent = 3
        execute_effect("apply_weak_x", ctx)
        assert state.enemies[0].statuses.get("Weak", 0) == 4  # X+1

    def test_doppelganger_draws_and_energy_next_turn(self):
        """Doppelganger: draw X and gain X energy next turn."""
        state = make_combat(energy=3)
        card = get_card("Doppelganger")
        ctx = make_ctx(state, card)
        ctx.energy_spent = 3
        execute_effect("draw_x_next_turn", ctx)
        execute_effect("gain_x_energy_next_turn", ctx)
        assert state.player.statuses.get("NextTurnDraw", 0) == 3
        assert state.player.statuses.get("NextTurnEnergy", 0) == 3

    def test_doppelganger_upgraded_x_plus_1(self):
        """Doppelganger+: draw X+1 and gain X+1 energy next turn."""
        state = make_combat(energy=3)
        card = get_card("Doppelganger", upgraded=True)
        ctx = make_ctx(state, card)
        ctx.energy_spent = 3
        execute_effect("draw_x_next_turn", ctx)
        execute_effect("gain_x_energy_next_turn", ctx)
        assert state.player.statuses.get("NextTurnDraw", 0) == 4
        assert state.player.statuses.get("NextTurnEnergy", 0) == 4


# =============================================================================
# BLOCK / DEFENSE CARD BEHAVIOR TESTS
# =============================================================================


class TestBlockBehavior:
    """Test block-related card effects."""

    def test_blur_retains_block(self):
        """Blur: gain block + set Blur status (block not removed next turn)."""
        state = make_combat(hand=["Blur"], energy=1)
        card = get_card("Blur")
        result = play_card_on_state(state, card, target_idx=-1)
        assert result.success
        assert state.player.block == 5
        assert state.player.statuses.get("Blur", 0) >= 1

    def test_blur_upgraded_8_block(self):
        """Blur+: 8 block."""
        state = make_combat(hand=["Blur+"], energy=1)
        card = get_card("Blur", upgraded=True)
        result = play_card_on_state(state, card, target_idx=-1)
        assert state.player.block == 8

    def test_dodge_and_roll_block_now_and_next_turn(self):
        """Dodge and Roll: gain 4 block + NextTurnBlock power (Java: NextTurnBlockPower)."""
        state = make_combat(hand=["Dodge and Roll"], energy=1)
        card = get_card("Dodge and Roll")
        result = play_card_on_state(state, card, target_idx=-1)
        assert result.success
        assert state.player.block == 4
        assert state.player.statuses.get("NextTurnBlock", 0) == 4

    def test_after_image_sets_power(self):
        """After Image: apply AfterImage power (1 block per card played)."""
        state = make_combat()
        card = get_card("After Image")
        ctx = make_ctx(state, card)
        execute_effect("gain_1_block_per_card_played", ctx)
        assert state.player.statuses.get("AfterImage", 0) == 1

    def test_escape_plan_draws_1(self):
        """Escape Plan: draw 1, if drawn card is a Skill gain block."""
        state = make_combat(
            hand=[],
            draw_pile=["Defend_G"],  # A Skill card
        )
        card = get_card("Escape Plan")
        ctx = make_ctx(state, card)
        execute_effect("draw_1", ctx)
        assert "Defend_G" in state.hand
        # Now check if skill was drawn for bonus block
        execute_effect("if_skill_drawn_gain_block", ctx)
        assert ctx.block_gained == 3  # Base block of Escape Plan


# =============================================================================
# ENERGY / DRAW CARD BEHAVIOR TESTS
# =============================================================================


class TestEnergyDrawBehavior:
    """Test energy and draw card effects."""

    def test_adrenaline_gains_1_energy(self):
        """Adrenaline: gain 1 energy, draw 2, exhaust."""
        state = make_combat(
            hand=["Adrenaline"],
            draw_pile=["Strike_G", "Defend_G", "Bane"],
            energy=0,
        )
        card = get_card("Adrenaline")
        ctx = make_ctx(state, card)
        execute_effect("gain_energy_magic", ctx)
        assert state.energy == 1
        execute_effect("draw_2", ctx)
        assert len(state.hand) == 3  # Adrenaline + 2 drawn

    def test_adrenaline_upgraded_gains_2_energy(self):
        """Adrenaline+: gain 2 energy."""
        state = make_combat(energy=0)
        card = get_card("Adrenaline", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("gain_energy_magic", ctx)
        assert state.energy == 2

    def test_outmaneuver_energy_next_turn(self):
        """Outmaneuver: gain 2 energy next turn."""
        state = make_combat()
        card = get_card("Outmaneuver")
        ctx = make_ctx(state, card)
        execute_effect("gain_energy_next_turn", ctx)
        assert state.player.statuses.get("NextTurnEnergy", 0) == 2

    def test_outmaneuver_upgraded_3_energy(self):
        """Outmaneuver+: gain 3 energy next turn."""
        state = make_combat()
        card = get_card("Outmaneuver", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("gain_energy_next_turn", ctx)
        assert state.player.statuses.get("NextTurnEnergy", 0) == 3

    def test_flying_knee_1_energy_next_turn(self):
        """Flying Knee: 1 energy next turn."""
        state = make_combat()
        card = get_card("Flying Knee")
        ctx = make_ctx(state, card)
        execute_effect("gain_energy_next_turn_1", ctx)
        assert state.player.statuses.get("NextTurnEnergy", 0) == 1

    def test_expertise_draws_to_6(self):
        """Expertise: draw until you have 6 cards."""
        state = make_combat(
            hand=["Expertise", "Strike_G"],
            draw_pile=["Bane", "Slice", "Defend_G", "Neutralize", "Backflip"],
        )
        card = get_card("Expertise")
        ctx = make_ctx(state, card)
        execute_effect("draw_to_x_cards", ctx)
        assert len(state.hand) == 6

    def test_expertise_upgraded_draws_to_7(self):
        """Expertise+: draw until 7 cards."""
        state = make_combat(
            hand=["Expertise"],
            draw_pile=["Bane", "Slice", "Defend_G", "Neutralize", "Backflip", "Choke", "Footwork"],
        )
        card = get_card("Expertise", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("draw_to_x_cards", ctx)
        assert len(state.hand) == 7

    def test_predator_draw_2_next_turn(self):
        """Predator: apply DrawCardNextTurnPower with 2."""
        state = make_combat()
        card = get_card("Predator")
        ctx = make_ctx(state, card)
        execute_effect("draw_2_next_turn", ctx)
        assert state.player.statuses.get("NextTurnDraw", 0) == 2


# =============================================================================
# DAMAGE CARD BEHAVIOR TESTS
# =============================================================================


class TestDamageBehavior:
    """Test damage-dealing card effects."""

    def test_dagger_spray_hits_all_twice(self):
        """Dagger Spray: 4 damage to all enemies, 2 times."""
        enemies = [make_enemy(hp=30, enemy_id=f"Louse{i}") for i in range(3)]
        state = make_combat(enemies=enemies)
        card = get_card("Dagger Spray")
        ctx = make_ctx(state, card)
        execute_effect("damage_all_x_times", ctx)
        # Each enemy should take 4 * 2 = 8 damage
        for e in state.enemies:
            assert e.hp == 22

    def test_finisher_damage_per_attack(self):
        """Finisher: deal 6 damage per attack played this turn."""
        enemy = make_enemy(hp=100)
        state = make_combat(enemies=[enemy])
        state.attacks_played_this_turn = 3
        card = get_card("Finisher")
        ctx = make_ctx(state, card, state.enemies[0], 0)
        execute_effect("damage_per_attack_this_turn", ctx)
        assert ctx.damage_dealt == 18  # 6 * 3

    def test_flechettes_damage_per_skill(self):
        """Flechettes: deal 4 damage per skill in hand."""
        state = make_combat(
            hand=["Backflip", "Defend_G", "Blur"],  # 3 skills
        )
        card = get_card("Flechettes")
        ctx = make_ctx(state, card, state.enemies[0], 0)
        execute_effect("damage_per_skill_in_hand", ctx)
        assert ctx.damage_dealt == 12  # 4 * 3

    def test_glass_knife_deals_damage_twice(self):
        """Glass Knife: deal 8 damage x2."""
        card = get_card("Glass Knife")
        assert card.damage == 8
        assert card.magic_number == 2
        assert "damage_x_times" in card.effects

    def test_glass_knife_reduces_damage_after_play(self):
        """Glass Knife: reduce_damage_by_2 marks damage reduction."""
        state = make_combat()
        card = get_card("Glass Knife")
        ctx = make_ctx(state, card)
        execute_effect("reduce_damage_by_2", ctx)
        assert ctx.extra_data.get("reduce_damage_this_combat") == 2

    def test_grand_finale_50_damage_all(self):
        """Grand Finale: 50 damage to all (0 cost, only if draw empty)."""
        card = get_card("Grand Finale")
        assert card.cost == 0
        assert card.damage == 50
        assert card.target == CardTarget.ALL_ENEMY

    def test_grand_finale_upgraded_60(self):
        """Grand Finale+: 60 damage."""
        card = get_card("Grand Finale", upgraded=True)
        assert card.damage == 60

    def test_choke_applies_choked_status(self):
        """Choke: deal 12 damage + apply 3 Choked to target."""
        state = make_combat()
        card = get_card("Choke")
        ctx = make_ctx(state, card, state.enemies[0], 0)
        execute_effect("apply_choke", ctx)
        assert state.enemies[0].statuses.get("Choked", 0) == 3

    def test_choke_upgraded_5(self):
        """Choke+: apply 5 Choked."""
        state = make_combat()
        card = get_card("Choke", upgraded=True)
        ctx = make_ctx(state, card, state.enemies[0], 0)
        execute_effect("apply_choke", ctx)
        assert state.enemies[0].statuses.get("Choked", 0) == 5

    def test_a_thousand_cuts_sets_power(self):
        """A Thousand Cuts: apply ThousandCuts power with amount 1."""
        state = make_combat()
        card = get_card("A Thousand Cuts")
        ctx = make_ctx(state, card)
        execute_effect("deal_damage_per_card_played", ctx)
        assert state.player.statuses.get("ThousandCuts", 0) == 1

    def test_a_thousand_cuts_upgraded_2(self):
        """A Thousand Cuts+: ThousandCuts power with amount 2."""
        state = make_combat()
        card = get_card("A Thousand Cuts", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("deal_damage_per_card_played", ctx)
        assert state.player.statuses.get("ThousandCuts", 0) == 2

    def test_unload_discards_non_attacks(self):
        """Unload: discard all non-Attack cards from hand."""
        state = make_combat(
            hand=["Unload", "Strike_G", "Defend_G", "Backflip", "Bane"],
        )
        card = get_card("Unload")
        ctx = make_ctx(state, card)
        execute_effect("discard_non_attacks", ctx)
        # Defend_G and Backflip are skills, should be discarded
        # Strike_G and Bane are attacks, should stay
        # Unload itself should also stay (it's an attack)
        remaining_attacks = {"Unload", "Strike_G", "Bane"}
        for c in state.hand:
            base = c.rstrip("+")
            assert base in remaining_attacks or base in {"Unload"}, f"Non-attack {c} remained in hand"


# =============================================================================
# CONDITIONAL CARD BEHAVIOR TESTS
# =============================================================================


class TestConditionalBehavior:
    """Test conditional card effects."""

    def test_heel_hook_gains_energy_if_weak(self):
        """Heel Hook: if target weak, gain 1 energy and draw 1."""
        state = make_combat(
            draw_pile=["Strike_G"],
            energy=0,
        )
        state.enemies[0].statuses["Weak"] = 2
        card = get_card("Heel Hook")
        ctx = make_ctx(state, card, state.enemies[0], 0)
        execute_effect("if_target_weak_gain_energy_draw", ctx)
        assert state.energy == 1
        assert len(state.hand) == 1  # Drew 1

    def test_heel_hook_no_bonus_without_weak(self):
        """Heel Hook: no energy/draw if target not weak."""
        state = make_combat(energy=0)
        card = get_card("Heel Hook")
        ctx = make_ctx(state, card, state.enemies[0], 0)
        execute_effect("if_target_weak_gain_energy_draw", ctx)
        assert state.energy == 0
        assert len(state.hand) == 0


# =============================================================================
# POWER CARD BEHAVIOR TESTS
# =============================================================================


class TestPowerBehavior:
    """Test power card effects."""

    def test_footwork_gains_dexterity(self):
        """Footwork: gain 2 Dexterity."""
        state = make_combat()
        card = get_card("Footwork")
        ctx = make_ctx(state, card)
        execute_effect("gain_dexterity", ctx)
        assert state.player.statuses.get("Dexterity", 0) == 2

    def test_footwork_upgraded_3(self):
        """Footwork+: gain 3 Dexterity."""
        state = make_combat()
        card = get_card("Footwork", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("gain_dexterity", ctx)
        assert state.player.statuses.get("Dexterity", 0) == 3

    def test_caltrops_gains_thorns(self):
        """Caltrops: gain 3 Thorns."""
        state = make_combat()
        card = get_card("Caltrops")
        ctx = make_ctx(state, card)
        execute_effect("gain_thorns", ctx)
        assert state.player.statuses.get("Thorns", 0) == 3

    def test_caltrops_upgraded_5(self):
        """Caltrops+: gain 5 Thorns."""
        state = make_combat()
        card = get_card("Caltrops", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("gain_thorns", ctx)
        assert state.player.statuses.get("Thorns", 0) == 5

    def test_envenom_applies_power(self):
        """Envenom: attacks apply Poison (via power)."""
        state = make_combat()
        card = get_card("Envenom")
        ctx = make_ctx(state, card)
        execute_effect("attacks_apply_poison", ctx)
        assert state.player.statuses.get("Envenom", 0) == 1

    def test_well_laid_plans_applies_power(self):
        """Well-Laid Plans: retain cards at end of turn."""
        state = make_combat()
        card = get_card("Well Laid Plans")
        ctx = make_ctx(state, card)
        execute_effect("retain_cards_each_turn", ctx)
        assert state.player.statuses.get("WellLaidPlans", 0) == 1

    def test_well_laid_plans_upgraded_2(self):
        """Well-Laid Plans+: retain 2 cards."""
        state = make_combat()
        card = get_card("Well Laid Plans", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("retain_cards_each_turn", ctx)
        assert state.player.statuses.get("WellLaidPlans", 0) == 2

    def test_tools_of_the_trade_applies_power(self):
        """Tools of the Trade: apply ToolsOfTheTrade power."""
        state = make_combat()
        card = get_card("Tools of the Trade")
        ctx = make_ctx(state, card)
        execute_effect("draw_1_discard_1_each_turn", ctx)
        assert state.player.statuses.get("ToolsOfTheTrade", 0) == 1

    def test_wraith_form_intangible_and_dex_loss(self):
        """Wraith Form: gain 2 Intangible + WraithFormPower (lose dex each turn)."""
        state = make_combat()
        card = get_card("Wraith Form v2")
        ctx = make_ctx(state, card)
        execute_effect("gain_intangible", ctx)
        execute_effect("lose_1_dexterity_each_turn", ctx)
        assert state.player.statuses.get("Intangible", 0) == 2
        assert state.player.statuses.get("WraithFormPower", 0) == 1

    def test_wraith_form_upgraded_3_intangible(self):
        """Wraith Form+: gain 3 Intangible."""
        state = make_combat()
        card = get_card("Wraith Form v2", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("gain_intangible", ctx)
        assert state.player.statuses.get("Intangible", 0) == 3

    def test_phantasmal_killer_applies_power(self):
        """Phantasmal Killer: apply PhantasmalKiller power (double damage next turn)."""
        state = make_combat()
        card = get_card("Phantasmal Killer")
        ctx = make_ctx(state, card)
        execute_effect("double_damage_next_turn", ctx)
        assert state.player.statuses.get("PhantasmalKiller", 0) == 1


# =============================================================================
# STRENGTH REDUCTION BEHAVIOR TESTS
# =============================================================================


class TestStrengthReductionBehavior:
    """Test Piercing Wail and similar strength-reducing effects."""

    def test_piercing_wail_reduces_strength(self):
        """Piercing Wail: reduce Strength of all enemies by 6 (temp).

        Java: applies StrengthPower(-6) + GainStrengthPower(6).
        """
        enemies = [make_enemy(hp=30, enemy_id=f"Louse{i}") for i in range(2)]
        state = make_combat(enemies=enemies)
        card = get_card("PiercingWail")
        ctx = make_ctx(state, card)
        execute_effect("reduce_strength_all_enemies", ctx)
        for e in state.enemies:
            assert e.statuses.get("Strength", 0) == -6
            # Temp loss tracked for end-of-turn restoration
            assert e.statuses.get("TempStrengthLoss", 0) == 6

    def test_piercing_wail_upgraded_8(self):
        """Piercing Wail+: reduce Strength by 8."""
        state = make_combat()
        card = get_card("PiercingWail", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("reduce_strength_all_enemies", ctx)
        assert state.enemies[0].statuses.get("Strength", 0) == -8


# =============================================================================
# SPECIAL EFFECT BEHAVIOR TESTS
# =============================================================================


class TestSpecialEffectBehavior:
    """Test special card effects."""

    def test_burst_applies_power(self):
        """Burst: next skill played twice (Burst power with 1)."""
        state = make_combat()
        card = get_card("Burst")
        ctx = make_ctx(state, card)
        execute_effect("double_next_skills", ctx)
        assert state.player.statuses.get("Burst", 0) == 1

    def test_burst_upgraded_2_skills(self):
        """Burst+: next 2 skills played twice."""
        state = make_combat()
        card = get_card("Burst", upgraded=True)
        ctx = make_ctx(state, card)
        execute_effect("double_next_skills", ctx)
        assert state.player.statuses.get("Burst", 0) == 2

    def test_bullet_time_no_draw_cards_cost_0(self):
        """Bullet Time: no draw + cards cost 0 this turn."""
        state = make_combat()
        card = get_card("Bullet Time")
        ctx = make_ctx(state, card)
        execute_effect("no_draw_this_turn", ctx)
        execute_effect("cards_cost_0_this_turn", ctx)
        assert state.player.statuses.get("NoDraw", 0) == 1
        assert state.player.statuses.get("ZeroCostCards", 0) == 1

    def test_endless_agony_copy_marker(self):
        """Endless Agony: marks that copy should be added when drawn (passive)."""
        card = get_card("Endless Agony")
        assert "copy_to_hand_when_drawn" in card.effects
        assert card.exhaust is True

    def test_nightmare_selection_needed(self):
        """Nightmare: marks that card selection is needed (3 copies)."""
        state = make_combat()
        card = get_card("Night Terror")
        ctx = make_ctx(state, card)
        execute_effect("copy_card_to_hand_next_turn", ctx)
        assert ctx.extra_data.get("nightmare_copies") == 3
        assert ctx.extra_data.get("nightmare_selection_needed") is True

    def test_setup_selection_needed(self):
        """Setup: marks that card selection is needed."""
        state = make_combat()
        card = get_card("Setup")
        ctx = make_ctx(state, card)
        execute_effect("put_card_on_draw_pile_cost_0", ctx)
        assert ctx.extra_data.get("setup_selection_needed") is True

    def test_distraction_adds_random_skill(self):
        """Distraction: add a random skill to hand at cost 0."""
        state = make_combat(hand=[])
        card = get_card("Distraction")
        ctx = make_ctx(state, card)
        execute_effect("add_random_skill_cost_0", ctx)
        assert len(state.hand) == 1
        # Verify it's a skill
        added_card_id = state.hand[0]
        base_id = added_card_id.rstrip("+")
        if base_id in ALL_CARDS:
            assert ALL_CARDS[base_id].card_type == CardType.SKILL

    def test_alchemize_marks_potion_obtain(self):
        """Alchemize: marks that a random potion should be obtained."""
        state = make_combat()
        card = get_card("Venomology")
        ctx = make_ctx(state, card)
        execute_effect("obtain_random_potion", ctx)
        assert ctx.extra_data.get("obtain_potion") is True

    def test_masterful_stab_passive_tracker(self):
        """Masterful Stab: cost increases when damaged (passive tracking)."""
        card = get_card("Masterful Stab")
        assert card.cost == 0
        assert card.damage == 12
        assert "cost_increases_when_damaged" in card.effects

    def test_grand_finale_only_playable_draw_empty(self):
        """Grand Finale: only_playable_if_draw_pile_empty is a marker."""
        card = get_card("Grand Finale")
        assert "only_playable_if_draw_pile_empty" in card.effects


# =============================================================================
# FULL PLAY CARD INTEGRATION TESTS
# =============================================================================


class TestFullPlayIntegration:
    """Test full card play through EffectExecutor."""

    def test_play_neutralize_deals_damage_applies_weak(self):
        """Neutralize: 0 cost, 3 damage, apply 1 Weak."""
        enemy = make_enemy(hp=50)
        state = make_combat(hand=["Neutralize"], energy=3, enemies=[enemy])
        card = get_card("Neutralize")
        result = play_card_on_state(state, card, target_idx=0)
        assert result.success
        assert result.energy_spent == 0  # 0 cost
        assert enemy.hp < 50  # Dealt damage
        assert enemy.statuses.get("Weak", 0) == 1

    def test_play_bane_on_poisoned_target(self):
        """Bane: full play with poisoned target deals double damage."""
        enemy = make_enemy(hp=100)
        enemy.statuses["Poison"] = 5
        state = make_combat(hand=["Bane"], energy=1, enemies=[enemy])
        card = get_card("Bane")
        result = play_card_on_state(state, card, target_idx=0)
        assert result.success
        # Base 7 + extra 7 from poison = 14 total
        assert result.damage_dealt == 14

    def test_play_blade_dance(self):
        """Blade Dance: play adds 3 shivs to hand."""
        state = make_combat(hand=["Blade Dance"], energy=1)
        card = get_card("Blade Dance")
        result = play_card_on_state(state, card, target_idx=-1)
        assert result.success
        shiv_count = state.hand.count("Shiv")
        assert shiv_count == 3

    def test_play_cloak_and_dagger(self):
        """Cloak and Dagger: 6 block + 1 Shiv."""
        state = make_combat(hand=["Cloak And Dagger"], energy=1)
        card = get_card("Cloak And Dagger")
        result = play_card_on_state(state, card, target_idx=-1)
        assert result.success
        assert state.player.block == 6
        assert state.hand.count("Shiv") == 1

    def test_play_backflip(self):
        """Backflip: 5 block + draw 2."""
        state = make_combat(
            hand=["Backflip"],
            draw_pile=["Strike_G", "Defend_G", "Bane"],
            energy=1,
        )
        card = get_card("Backflip")
        result = play_card_on_state(state, card, target_idx=-1)
        assert result.success
        assert state.player.block == 5
        # Executor draws 2 cards; original card remains in hand (removal handled by combat engine)
        assert len(state.hand) == 3  # Backflip + 2 drawn

    def test_play_footwork(self):
        """Footwork: play applies Dexterity power."""
        state = make_combat(hand=["Footwork"], energy=1)
        card = get_card("Footwork")
        result = play_card_on_state(state, card, target_idx=-1)
        assert result.success
        assert state.player.statuses.get("Dexterity", 0) == 2

    def test_play_noxious_fumes(self):
        """Noxious Fumes: play applies NoxiousFumes power."""
        state = make_combat(hand=["Noxious Fumes"], energy=1)
        card = get_card("Noxious Fumes")
        result = play_card_on_state(state, card, target_idx=-1)
        assert result.success
        assert state.player.statuses.get("NoxiousFumes", 0) == 2

    def test_play_caltrops(self):
        """Caltrops: play applies Thorns."""
        state = make_combat(hand=["Caltrops"], energy=1)
        card = get_card("Caltrops")
        result = play_card_on_state(state, card, target_idx=-1)
        assert result.success
        assert state.player.statuses.get("Thorns", 0) == 3

    def test_play_deadly_poison(self):
        """Deadly Poison: apply 5 Poison to target."""
        state = make_combat(hand=["Deadly Poison"], energy=1)
        card = get_card("Deadly Poison")
        result = play_card_on_state(state, card, target_idx=0)
        assert result.success
        assert state.enemies[0].statuses.get("Poison", 0) == 5


# =============================================================================
# CARD DATA PARITY TESTS (Java-verified values)
# =============================================================================


class TestCardDataParity:
    """Verify card data matches Java decompiled source exactly."""

    def test_all_silent_cards_have_effects_registered(self):
        """Every Silent card in SILENT_CARD_EFFECTS has matching effects on Card object."""
        from packages.engine.effects.cards import SILENT_CARD_EFFECTS
        from packages.engine.content.cards import SILENT_CARDS

        for card_id, expected_effects in SILENT_CARD_EFFECTS.items():
            assert card_id in SILENT_CARDS, f"Card {card_id} in effects but not in SILENT_CARDS"
            card = SILENT_CARDS[card_id]
            for eff in expected_effects:
                assert eff in card.effects, f"Card {card_id} missing effect {eff}"

    def test_choke_java_parity(self):
        """Choke: Java baseDamage=12, magicNumber=3, upgrade +2 magic."""
        card = get_card("Choke")
        assert card.damage == 12
        assert card.magic_number == 3
        card_up = get_card("Choke", upgraded=True)
        assert card_up.magic_number == 5

    def test_eviscerate_java_parity(self):
        """Eviscerate: Java cost=3, baseDamage=7, hits 3 times."""
        card = get_card("Eviscerate")
        assert card.cost == 3
        assert card.damage == 7
        assert card.magic_number == 3  # 3 hits

    def test_terror_java_parity(self):
        """Terror: Java applies 99 Vulnerable, cost 1, upgrade cost 0."""
        card = get_card("Terror")
        assert card.cost == 1
        assert card.magic_number == 99
        assert card.exhaust is True
        card_up = get_card("Terror", upgraded=True)
        assert card_up.current_cost == 0

    def test_predator_java_parity(self):
        """Predator: Java baseDamage=15, draw 2 next turn."""
        card = get_card("Predator")
        assert card.damage == 15
        assert "draw_2_next_turn" in card.effects
        card_up = get_card("Predator", upgraded=True)
        assert card_up.damage == 20

    def test_nightmare_java_parity(self):
        """Nightmare: Java ID='Night Terror', cost=3, upgrade cost=2, 3 copies."""
        card = get_card("Night Terror")
        assert card.cost == 3
        assert card.magic_number == 3
        assert card.exhaust is True
        card_up = get_card("Night Terror", upgraded=True)
        assert card_up.current_cost == 2

    def test_alchemize_java_parity(self):
        """Alchemize: Java ID='Venomology', cost=1, upgrade cost=0."""
        card = get_card("Venomology")
        assert card.cost == 1
        assert card.exhaust is True
        card_up = get_card("Venomology", upgraded=True)
        assert card_up.current_cost == 0

    def test_sneaky_strike_java_parity(self):
        """SneakyStrike: Java ID='Underhanded Strike', cost=2, baseDamage=12."""
        card = get_card("Underhanded Strike")
        assert card.cost == 2
        assert card.damage == 12
        card_up = get_card("Underhanded Strike", upgraded=True)
        assert card_up.damage == 16

    def test_wraith_form_java_parity(self):
        """WraithForm: Java ID='Wraith Form v2', cost=3, magic=2/3."""
        card = get_card("Wraith Form v2")
        assert card.cost == 3
        assert card.magic_number == 2
        card_up = get_card("Wraith Form v2", upgraded=True)
        assert card_up.magic_number == 3

    def test_phantasmal_killer_java_parity(self):
        """PhantasmalKiller: Java cost=1, upgrade cost=0."""
        card = get_card("Phantasmal Killer")
        assert card.cost == 1
        card_up = get_card("Phantasmal Killer", upgraded=True)
        assert card_up.current_cost == 0

    def test_bullet_time_java_parity(self):
        """BulletTime: Java cost=3, upgrade cost=2."""
        card = get_card("Bullet Time")
        assert card.cost == 3
        card_up = get_card("Bullet Time", upgraded=True)
        assert card_up.current_cost == 2

    def test_masterful_stab_java_parity(self):
        """MasterfulStab: Java cost=0, baseDamage=12, upgrade +4."""
        card = get_card("Masterful Stab")
        assert card.cost == 0
        assert card.damage == 12
        card_up = get_card("Masterful Stab", upgraded=True)
        assert card_up.damage == 16

    def test_glass_knife_java_parity(self):
        """GlassKnife: Java baseDamage=8, upgrade +4, hits twice, -2 per use."""
        card = get_card("Glass Knife")
        assert card.damage == 8
        assert card.magic_number == 2
        card_up = get_card("Glass Knife", upgraded=True)
        assert card_up.damage == 12

    def test_envenom_java_parity(self):
        """Envenom: Java cost=2, upgrade cost=1."""
        card = get_card("Envenom")
        assert card.cost == 2
        card_up = get_card("Envenom", upgraded=True)
        assert card_up.current_cost == 1

    def test_tools_of_the_trade_java_parity(self):
        """ToolsOfTheTrade: Java cost=1, upgrade cost=0."""
        card = get_card("Tools of the Trade")
        assert card.cost == 1
        card_up = get_card("Tools of the Trade", upgraded=True)
        assert card_up.current_cost == 0

    def test_distraction_java_parity(self):
        """Distraction: Java cost=1, upgrade cost=0, exhaust."""
        card = get_card("Distraction")
        assert card.cost == 1
        assert card.exhaust is True
        card_up = get_card("Distraction", upgraded=True)
        assert card_up.current_cost == 0

    def test_bouncing_flask_java_parity(self):
        """BouncingFlask: Java cost=2, magic=3, upgrade +1 magic (3 bounces always)."""
        card = get_card("Bouncing Flask")
        assert card.cost == 2
        assert card.magic_number == 3
        card_up = get_card("Bouncing Flask", upgraded=True)
        assert card_up.magic_number == 4

    def test_crippling_poison_java_parity(self):
        """CripplingPoison: Java cost=2, magic=4, upgrade +3, exhaust."""
        card = get_card("Crippling Poison")
        assert card.cost == 2
        assert card.magic_number == 4
        assert card.exhaust is True
        card_up = get_card("Crippling Poison", upgraded=True)
        assert card_up.magic_number == 7

    def test_dodge_and_roll_java_parity(self):
        """DodgeAndRoll: Java baseBlock=4, upgrade +2."""
        card = get_card("Dodge and Roll")
        assert card.block == 4
        card_up = get_card("Dodge and Roll", upgraded=True)
        assert card_up.block == 6

    def test_catalyst_java_parity(self):
        """Catalyst: Java cost=1, exhaust, base=double, upgrade=triple."""
        card = get_card("Catalyst")
        assert card.cost == 1
        assert card.exhaust is True

    def test_escape_plan_java_parity(self):
        """EscapePlan: Java cost=0, baseBlock=3, upgrade +2."""
        card = get_card("Escape Plan")
        assert card.cost == 0
        assert card.block == 3
        card_up = get_card("Escape Plan", upgraded=True)
        assert card_up.block == 5
