"""
Tests for Watcher Card Effect Implementations.

Tests the actual effect execution using EffectContext, not just card data.
Covers:
- Stance changes and energy generation
- Mantra accumulation and Divinity trigger
- Scry mechanics and triggered effects
- Draw and card manipulation
- Status applications
- Block generation
- Damage dealing (including Wallop, Spirit Shield, etc.)
- Turn control (Conclude, Vault)
- Card generation (Insight, Smite, Safety, Through Violence)
"""

import pytest

from packages.engine.state.combat import CombatState, EntityState, EnemyCombatState
from packages.engine.effects.registry import EffectContext, execute_effect
from packages.engine.effects import cards as card_effects  # noqa: F401 - imports to register effects
from packages.engine.content.cards import get_card, CardType


# =============================================================================
# Test Fixtures
# =============================================================================

@pytest.fixture
def basic_combat_state():
    """Create a basic combat state for testing."""
    player = EntityState(hp=72, max_hp=72)
    enemy = EnemyCombatState(
        hp=50, max_hp=50, id="test_enemy", name="Test Enemy",
        move_damage=10, move_hits=1
    )
    state = CombatState(
        player=player,
        energy=3,
        max_energy=3,
        hand=["Strike_P", "Defend_P", "Eruption"],
        draw_pile=["Strike_P", "Strike_P", "Defend_P", "Defend_P", "Vigilance"],
        discard_pile=[],
        enemies=[enemy],
        stance="Neutral",
    )
    return state


@pytest.fixture
def multi_enemy_state():
    """Combat state with multiple enemies."""
    player = EntityState(hp=72, max_hp=72)
    enemies = [
        EnemyCombatState(hp=40, max_hp=40, id="e1", name="Enemy 1", move_damage=8, move_hits=1),
        EnemyCombatState(hp=30, max_hp=30, id="e2", name="Enemy 2", move_damage=6, move_hits=2),
        EnemyCombatState(hp=50, max_hp=50, id="e3", name="Enemy 3", move_damage=12, move_hits=1),
    ]
    state = CombatState(
        player=player,
        energy=3,
        max_energy=3,
        hand=["Consecrate", "BowlingBash", "Ragnarok"],
        draw_pile=["Strike_P"] * 5,
        discard_pile=[],
        enemies=enemies,
        stance="Neutral",
    )
    return state


@pytest.fixture
def ctx_basic(basic_combat_state):
    """Create effect context from basic state."""
    return EffectContext(
        state=basic_combat_state,
        target=basic_combat_state.enemies[0],
        target_idx=0,
    )


@pytest.fixture
def ctx_multi(multi_enemy_state):
    """Create effect context with multiple enemies."""
    return EffectContext(
        state=multi_enemy_state,
        target=multi_enemy_state.enemies[0],
        target_idx=0,
    )


# =============================================================================
# Stance Change Tests
# =============================================================================

class TestStanceEffects:
    """Test stance change effects."""

    def test_enter_wrath(self, ctx_basic):
        """Test entering Wrath stance."""
        assert ctx_basic.stance == "Neutral"
        execute_effect("enter_wrath", ctx_basic)
        assert ctx_basic.stance == "Wrath"

    def test_enter_calm(self, ctx_basic):
        """Test entering Calm stance."""
        execute_effect("enter_calm", ctx_basic)
        assert ctx_basic.stance == "Calm"

    def test_exit_calm_gains_energy(self, ctx_basic):
        """Exiting Calm grants 2 energy."""
        ctx_basic.state.stance = "Calm"
        initial_energy = ctx_basic.energy
        result = ctx_basic.change_stance("Neutral")
        assert ctx_basic.energy == initial_energy + 2
        assert result["energy_gained"] == 2

    def test_exit_calm_to_wrath_gains_energy(self, ctx_basic):
        """Exiting Calm to Wrath grants 2 energy."""
        ctx_basic.state.stance = "Calm"
        initial_energy = ctx_basic.energy
        result = ctx_basic.change_stance("Wrath")
        assert ctx_basic.energy == initial_energy + 2
        assert ctx_basic.stance == "Wrath"

    def test_enter_divinity(self, ctx_basic):
        """Entering Divinity grants 3 energy."""
        initial_energy = ctx_basic.energy
        execute_effect("enter_divinity", ctx_basic)
        assert ctx_basic.stance == "Divinity"
        assert ctx_basic.energy == initial_energy + 3

    def test_exit_stance(self, ctx_basic):
        """Test exit_stance effect."""
        ctx_basic.state.stance = "Wrath"
        execute_effect("exit_stance", ctx_basic)
        assert ctx_basic.stance == "Neutral"

    def test_calm_to_divinity_combo(self, ctx_basic):
        """Going from Calm to Divinity grants 2 + 3 = 5 energy."""
        ctx_basic.state.stance = "Calm"
        initial_energy = ctx_basic.energy
        result = ctx_basic.change_stance("Divinity")
        assert ctx_basic.energy == initial_energy + 5  # 2 from Calm exit + 3 from Divinity
        assert ctx_basic.stance == "Divinity"


# =============================================================================
# Mantra and Divinity Tests
# =============================================================================

class TestMantraEffects:
    """Test mantra accumulation and Divinity trigger."""

    def test_gain_mantra_basic(self, ctx_basic):
        """Test basic mantra gain."""
        ctx_basic.magic_number = 3
        execute_effect("gain_mantra", ctx_basic)
        assert ctx_basic.get_player_status("Mantra") == 3

    def test_mantra_triggers_divinity_at_10(self, ctx_basic):
        """Reaching 10 mantra triggers Divinity."""
        ctx_basic.state.player.statuses["Mantra"] = 7
        initial_energy = ctx_basic.energy
        result = ctx_basic.gain_mantra(3)
        assert result["divinity_triggered"] == True
        assert ctx_basic.stance == "Divinity"
        # 3 energy from Divinity entry
        assert ctx_basic.energy >= initial_energy + 3

    def test_mantra_excess_carries_over(self, ctx_basic):
        """Excess mantra above 10 carries over."""
        result = ctx_basic.gain_mantra(13)
        assert result["divinity_triggered"] == True
        # Mantra should be 13 - 10 = 3
        assert ctx_basic.get_player_status("Mantra") == 3

    def test_prostrate_gains_mantra(self, ctx_basic):
        """Prostrate: gain 2 mantra (3 upgraded)."""
        ctx_basic.magic_number = 2
        execute_effect("gain_mantra", ctx_basic)
        assert ctx_basic.get_player_status("Mantra") == 2


# =============================================================================
# Conditional Stance Effects
# =============================================================================

class TestConditionalStanceEffects:
    """Test cards with conditional stance behavior."""

    def test_inner_peace_in_calm_draws(self, ctx_basic):
        """Inner Peace in Calm draws 3 cards."""
        ctx_basic.state.stance = "Calm"
        ctx_basic.is_upgraded = False
        initial_hand = len(ctx_basic.hand)
        execute_effect("if_calm_draw_3_else_calm", ctx_basic)
        assert len(ctx_basic.hand) == initial_hand + 3

    def test_inner_peace_not_in_calm_enters_calm(self, ctx_basic):
        """Inner Peace outside Calm enters Calm."""
        ctx_basic.state.stance = "Neutral"
        execute_effect("if_calm_draw_3_else_calm", ctx_basic)
        assert ctx_basic.stance == "Calm"

    def test_inner_peace_canonical_effect_in_calm(self, ctx_basic):
        """Inner Peace canonical effect (if_calm_draw_else_calm) draws 3 in Calm."""
        ctx_basic.state.stance = "Calm"
        ctx_basic.is_upgraded = False
        initial_hand = len(ctx_basic.hand)
        execute_effect("if_calm_draw_else_calm", ctx_basic)
        assert len(ctx_basic.hand) == initial_hand + 3

    def test_inner_peace_canonical_effect_not_in_calm(self, ctx_basic):
        """Inner Peace canonical effect enters Calm from Neutral."""
        ctx_basic.state.stance = "Neutral"
        execute_effect("if_calm_draw_else_calm", ctx_basic)
        assert ctx_basic.stance == "Calm"

    def test_inner_peace_upgraded_draws_4(self, ctx_basic):
        """Inner Peace upgraded draws 4 cards in Calm."""
        ctx_basic.state.stance = "Calm"
        ctx_basic.is_upgraded = True
        initial_hand = len(ctx_basic.hand)
        execute_effect("if_calm_draw_else_calm", ctx_basic)
        assert len(ctx_basic.hand) == initial_hand + 4

    def test_inner_peace_from_wrath_enters_calm(self, ctx_basic):
        """Inner Peace from Wrath stance enters Calm."""
        ctx_basic.state.stance = "Wrath"
        execute_effect("if_calm_draw_else_calm", ctx_basic)
        assert ctx_basic.stance == "Calm"

    def test_indignation_in_wrath_gains_mantra(self, ctx_basic):
        """Indignation in Wrath gains 3 mantra."""
        ctx_basic.state.stance = "Wrath"
        ctx_basic.is_upgraded = False
        execute_effect("if_wrath_gain_mantra_else_wrath", ctx_basic)
        assert ctx_basic.get_player_status("Mantra") == 3

    def test_indignation_not_in_wrath_enters_wrath(self, ctx_basic):
        """Indignation outside Wrath enters Wrath."""
        ctx_basic.state.stance = "Neutral"
        execute_effect("if_wrath_gain_mantra_else_wrath", ctx_basic)
        assert ctx_basic.stance == "Wrath"

    def test_halt_wrath_bonus(self, ctx_basic):
        """Halt gains extra block in Wrath."""
        ctx_basic.state.stance = "Wrath"
        ctx_basic.is_upgraded = False
        initial_block = ctx_basic.player.block
        execute_effect("if_in_wrath_extra_block_6", ctx_basic)
        assert ctx_basic.player.block == initial_block + 6

    def test_halt_no_bonus_outside_wrath(self, ctx_basic):
        """Halt gains no extra block outside Wrath."""
        ctx_basic.state.stance = "Calm"
        initial_block = ctx_basic.player.block
        execute_effect("if_in_wrath_extra_block_6", ctx_basic)
        assert ctx_basic.player.block == initial_block  # No change

    def test_fear_no_evil_enemy_attacking(self, ctx_basic):
        """Fear No Evil enters Calm if enemy is attacking."""
        ctx_basic.target.move_damage = 10  # Enemy is attacking
        ctx_basic.state.stance = "Neutral"
        execute_effect("if_enemy_attacking_enter_calm", ctx_basic)
        assert ctx_basic.stance == "Calm"

    def test_fear_no_evil_enemy_not_attacking(self, ctx_basic):
        """Fear No Evil does not enter Calm if enemy is not attacking."""
        ctx_basic.target.move_damage = 0  # Enemy is not attacking
        ctx_basic.state.stance = "Neutral"
        execute_effect("if_enemy_attacking_enter_calm", ctx_basic)
        assert ctx_basic.stance == "Neutral"  # No change


# =============================================================================
# Scry Tests
# =============================================================================

class TestScryEffects:
    """Test scry mechanics and triggered effects."""

    def test_scry_basic(self, ctx_basic):
        """Test basic scry functionality."""
        ctx_basic.magic_number = 3
        initial_draw = len(ctx_basic.draw_pile)
        scried = ctx_basic.scry(3)
        # Cards should be returned to draw pile
        assert len(ctx_basic.draw_pile) == initial_draw

    def test_nirvana_block_on_scry(self, ctx_basic):
        """Nirvana grants block when scrying."""
        ctx_basic.state.player.statuses["Nirvana"] = 3
        initial_block = ctx_basic.player.block
        ctx_basic.scry(2)
        # Nirvana grants block per card scried
        assert ctx_basic.player.block == initial_block + (3 * 2)

    def test_weave_moves_to_hand_on_scry(self, ctx_basic):
        """Weave moves from discard to hand on scry."""
        ctx_basic.state.discard_pile.append("Weave")
        ctx_basic.scry(1)
        assert "Weave" in ctx_basic.hand
        assert "Weave" not in ctx_basic.discard_pile


# =============================================================================
# Draw and Card Manipulation Tests
# =============================================================================

class TestDrawEffects:
    """Test draw and card manipulation effects."""

    def test_draw_1(self, ctx_basic):
        """Draw 1 card."""
        initial_hand = len(ctx_basic.hand)
        execute_effect("draw_1", ctx_basic)
        assert len(ctx_basic.hand) == initial_hand + 1

    def test_draw_2(self, ctx_basic):
        """Draw 2 cards."""
        initial_hand = len(ctx_basic.hand)
        execute_effect("draw_2", ctx_basic)
        assert len(ctx_basic.hand) == initial_hand + 2

    def test_draw_cards_magic_number(self, ctx_basic):
        """Draw cards based on magic number."""
        ctx_basic.magic_number = 3
        initial_hand = len(ctx_basic.hand)
        execute_effect("draw_cards", ctx_basic)
        assert len(ctx_basic.hand) == initial_hand + 3

    def test_draw_shuffles_discard_when_empty(self, ctx_basic):
        """Draw shuffles discard into draw when draw is empty."""
        ctx_basic.state.draw_pile.clear()
        ctx_basic.state.discard_pile = ["Strike_P", "Defend_P", "Eruption"]
        initial_hand = len(ctx_basic.hand)
        ctx_basic.draw_cards(2)
        assert len(ctx_basic.hand) == initial_hand + 2
        assert len(ctx_basic.draw_pile) == 1  # One card left after drawing 2 of 3

    def test_scrawl_draw_until_full(self, ctx_basic):
        """Scrawl draws until hand has 10 cards."""
        ctx_basic.state.hand = ["Strike_P", "Defend_P"]  # 2 cards in hand
        ctx_basic.state.draw_pile = ["Strike_P"] * 10
        execute_effect("draw_until_hand_full", ctx_basic)
        assert len(ctx_basic.hand) == 10  # Should be full

    def test_scrawl_no_draw_if_hand_full(self, ctx_basic):
        """Scrawl does not draw if hand already has 10 cards."""
        ctx_basic.state.hand = ["Strike_P"] * 10  # Already full
        ctx_basic.state.draw_pile = ["Defend_P"] * 5
        initial_draw = len(ctx_basic.draw_pile)
        execute_effect("draw_until_hand_full", ctx_basic)
        assert len(ctx_basic.draw_pile) == initial_draw  # No draw happened


# =============================================================================
# Card Generation Tests
# =============================================================================

class TestCardGenerationEffects:
    """Test effects that generate cards."""

    def test_add_insight_to_draw(self, ctx_basic):
        """Evaluate adds Insight to top of draw pile."""
        ctx_basic.is_upgraded = False
        execute_effect("add_insight_to_draw", ctx_basic)
        assert ctx_basic.draw_pile[-1] == "Insight"

    def test_add_insight_upgraded(self, ctx_basic):
        """Upgraded Evaluate adds Insight+ to draw pile."""
        ctx_basic.is_upgraded = True
        execute_effect("add_insight_to_draw", ctx_basic)
        assert ctx_basic.draw_pile[-1] == "Insight+"

    def test_add_smite_to_hand(self, ctx_basic):
        """Carve Reality adds Smite to hand."""
        ctx_basic.is_upgraded = False
        execute_effect("add_smite_to_hand", ctx_basic)
        assert "Smite" in ctx_basic.hand

    def test_add_safety_to_hand(self, ctx_basic):
        """Deceive Reality adds Safety to hand."""
        ctx_basic.is_upgraded = False
        execute_effect("add_safety_to_hand", ctx_basic)
        assert "Safety" in ctx_basic.hand

    def test_add_through_violence_to_draw(self, ctx_basic):
        """Reach Heaven adds Through Violence to draw pile."""
        ctx_basic.is_upgraded = False
        execute_effect("add_through_violence_to_draw", ctx_basic)
        assert ctx_basic.draw_pile[-1] == "ThroughViolence"

    def test_shuffle_beta_into_draw(self, ctx_basic):
        """Alpha shuffles Beta into draw pile."""
        ctx_basic.is_upgraded = False
        initial_draw = len(ctx_basic.draw_pile)
        execute_effect("shuffle_beta_into_draw", ctx_basic)
        assert len(ctx_basic.draw_pile) == initial_draw + 1
        assert "Beta" in ctx_basic.draw_pile

    def test_shuffle_omega_into_draw(self, ctx_basic):
        """Beta shuffles Omega into draw pile."""
        initial_draw = len(ctx_basic.draw_pile)
        execute_effect("shuffle_omega_into_draw", ctx_basic)
        assert len(ctx_basic.draw_pile) == initial_draw + 1
        assert "Omega" in ctx_basic.draw_pile


# =============================================================================
# Block and Damage Tests
# =============================================================================

class TestBlockAndDamageEffects:
    """Test block and damage related effects."""

    def test_gain_block(self, ctx_basic):
        """Test basic block gain."""
        initial_block = ctx_basic.player.block
        ctx_basic.gain_block(10)
        assert ctx_basic.player.block == initial_block + 10

    def test_deal_damage_to_target(self, ctx_basic):
        """Test dealing damage to target."""
        initial_hp = ctx_basic.target.hp
        damage = ctx_basic.deal_damage_to_target(15)
        assert ctx_basic.target.hp == initial_hp - 15
        assert damage == 15

    def test_deal_damage_blocked(self, ctx_basic):
        """Test damage blocked by enemy block."""
        ctx_basic.target.block = 10
        initial_hp = ctx_basic.target.hp
        damage = ctx_basic.deal_damage_to_target(15)
        # 10 blocked, 5 HP damage
        assert ctx_basic.target.block == 0
        assert ctx_basic.target.hp == initial_hp - 5
        assert damage == 5

    def test_spirit_shield_block(self, ctx_basic):
        """Spirit Shield gains block per card in hand."""
        ctx_basic.state.hand = ["Strike_P"] * 5
        ctx_basic.magic_number = 3  # 3 block per card
        initial_block = ctx_basic.player.block
        execute_effect("gain_block_per_card_in_hand", ctx_basic)
        assert ctx_basic.player.block == initial_block + 15  # 5 cards * 3

    def test_wallop_block_from_damage(self, ctx_basic):
        """Wallop: gain block equal to unblocked damage dealt."""
        ctx_basic.target.block = 0
        ctx_basic.target.hp = 50
        # Deal 15 damage
        ctx_basic.deal_damage_to_target(15)
        ctx_basic.damage_dealt = 15
        initial_block = ctx_basic.player.block
        execute_effect("gain_block_equal_unblocked_damage", ctx_basic)
        assert ctx_basic.player.block == initial_block + 15


# =============================================================================
# Status Application Tests
# =============================================================================

class TestStatusEffects:
    """Test status application effects."""

    def test_apply_weak_to_target(self, ctx_basic):
        """Apply Weak to target."""
        ctx_basic.apply_status_to_target("Weak", 2)
        assert ctx_basic.target.statuses.get("Weak") == 2

    def test_apply_vulnerable_to_target(self, ctx_basic):
        """Apply Vulnerable to target."""
        ctx_basic.apply_status_to_target("Vulnerable", 1)
        assert ctx_basic.target.statuses.get("Vulnerable") == 1

    def test_apply_mark_to_target(self, ctx_basic):
        """Pressure Points applies Mark."""
        ctx_basic.magic_number = 8
        execute_effect("apply_mark", ctx_basic)
        assert ctx_basic.target.statuses.get("Mark") == 8

    def test_trigger_all_marks(self, ctx_multi):
        """Trigger all Marks deals damage to all enemies."""
        ctx_multi.enemies[0].statuses["Mark"] = 5
        ctx_multi.enemies[1].statuses["Mark"] = 10
        ctx_multi.enemies[2].statuses["Mark"] = 0

        initial_hp_0 = ctx_multi.enemies[0].hp
        initial_hp_1 = ctx_multi.enemies[1].hp
        initial_hp_2 = ctx_multi.enemies[2].hp

        execute_effect("trigger_all_marks", ctx_multi)

        assert ctx_multi.enemies[0].hp == initial_hp_0 - 5
        assert ctx_multi.enemies[1].hp == initial_hp_1 - 10
        assert ctx_multi.enemies[2].hp == initial_hp_2  # No mark, no damage

    def test_artifact_blocks_debuff(self, ctx_basic):
        """Artifact blocks debuff application."""
        ctx_basic.target.statuses["Artifact"] = 1
        result = ctx_basic.apply_status_to_target("Weak", 2)
        assert result == False
        assert ctx_basic.target.statuses.get("Weak", 0) == 0
        assert ctx_basic.target.statuses.get("Artifact", 0) == 0


# =============================================================================
# Conditional Last Card Effects
# =============================================================================

class TestConditionalLastCardEffects:
    """Test effects that depend on last card played."""

    def test_follow_up_energy_after_attack(self, ctx_basic):
        """Follow-Up gains energy if last card was Attack."""
        ctx_basic.extra_data["last_card_type"] = "ATTACK"
        initial_energy = ctx_basic.energy
        execute_effect("if_last_card_attack_gain_energy", ctx_basic)
        assert ctx_basic.energy == initial_energy + 1

    def test_follow_up_no_energy_after_skill(self, ctx_basic):
        """Follow-Up doesn't gain energy if last card was Skill."""
        ctx_basic.extra_data["last_card_type"] = "SKILL"
        initial_energy = ctx_basic.energy
        execute_effect("if_last_card_attack_gain_energy", ctx_basic)
        assert ctx_basic.energy == initial_energy

    def test_sash_whip_weak_after_attack(self, ctx_basic):
        """Sash Whip applies Weak if last card was Attack."""
        ctx_basic.extra_data["last_card_type"] = "ATTACK"
        ctx_basic.is_upgraded = False
        execute_effect("if_last_card_attack_weak", ctx_basic)
        assert ctx_basic.target.statuses.get("Weak") == 1

    def test_crush_joints_vulnerable_after_skill(self, ctx_basic):
        """Crush Joints applies Vulnerable if last card was Skill."""
        ctx_basic.extra_data["last_card_type"] = "SKILL"
        ctx_basic.is_upgraded = False
        execute_effect("if_last_card_skill_vulnerable", ctx_basic)
        assert ctx_basic.target.statuses.get("Vulnerable") == 1


# =============================================================================
# Turn Control Tests
# =============================================================================

class TestTurnControlEffects:
    """Test turn control effects."""

    def test_end_turn(self, ctx_basic):
        """Conclude ends the turn."""
        execute_effect("end_turn", ctx_basic)
        assert ctx_basic.should_end_turn() == True

    def test_vault_extra_turn(self, ctx_basic):
        """Vault grants an extra turn."""
        execute_effect("take_extra_turn", ctx_basic)
        assert ctx_basic.extra_data.get("extra_turn") == True


# =============================================================================
# Special Card Effects
# =============================================================================

class TestSpecialCardEffects:
    """Test special card effects."""

    def test_miracle_energy_gain(self, ctx_basic):
        """Miracle gains 1 energy (2 upgraded)."""
        ctx_basic.is_upgraded = False
        initial_energy = ctx_basic.energy
        execute_effect("gain_1_energy", ctx_basic)
        assert ctx_basic.energy == initial_energy + 1

    def test_miracle_upgraded_energy(self, ctx_basic):
        """Miracle+ gains 2 energy."""
        ctx_basic.is_upgraded = True
        initial_energy = ctx_basic.energy
        execute_effect("gain_1_energy", ctx_basic)
        assert ctx_basic.energy == initial_energy + 2

    def test_judgement_kill(self, ctx_basic):
        """Judgement kills enemy if HP <= 30."""
        ctx_basic.target.hp = 25
        ctx_basic.is_upgraded = False
        execute_effect("if_enemy_hp_below_kill", ctx_basic)
        assert ctx_basic.target.hp == 0

    def test_judgement_no_kill_above_threshold(self, ctx_basic):
        """Judgement doesn't kill if HP > 30."""
        ctx_basic.target.hp = 35
        ctx_basic.is_upgraded = False
        execute_effect("if_enemy_hp_below_kill", ctx_basic)
        assert ctx_basic.target.hp == 35

    def test_judgement_upgraded_threshold(self, ctx_basic):
        """Judgement+ kills if HP <= 40."""
        ctx_basic.target.hp = 35
        ctx_basic.is_upgraded = True
        execute_effect("if_enemy_hp_below_kill", ctx_basic)
        assert ctx_basic.target.hp == 0


# =============================================================================
# Power Application Tests
# =============================================================================

class TestPowerEffects:
    """Test power status effects."""

    def test_mental_fortress_power(self, ctx_basic):
        """Mental Fortress applies status."""
        ctx_basic.magic_number = 4
        execute_effect("on_stance_change_gain_block", ctx_basic)
        assert ctx_basic.get_player_status("MentalFortress") == 4

    def test_nirvana_power(self, ctx_basic):
        """Nirvana applies status."""
        ctx_basic.magic_number = 3
        execute_effect("on_scry_gain_block", ctx_basic)
        assert ctx_basic.get_player_status("Nirvana") == 3

    def test_rushdown_power(self, ctx_basic):
        """Rushdown applies status."""
        ctx_basic.magic_number = 2
        execute_effect("on_wrath_draw", ctx_basic)
        assert ctx_basic.get_player_status("Rushdown") == 2

    def test_like_water_power(self, ctx_basic):
        """Like Water applies status."""
        ctx_basic.magic_number = 5
        execute_effect("if_calm_end_turn_gain_block", ctx_basic)
        assert ctx_basic.get_player_status("LikeWater") == 5


# =============================================================================
# Flurry of Blows and Weave Triggers
# =============================================================================

class TestAutoPlayTriggers:
    """Test cards that auto-play from discard."""

    def test_flurry_moves_on_stance_change(self, ctx_basic):
        """Flurry of Blows moves to hand on stance change."""
        ctx_basic.state.discard_pile.append("FlurryOfBlows")
        ctx_basic.change_stance("Wrath")
        assert "FlurryOfBlows" in ctx_basic.hand
        assert "FlurryOfBlows" not in ctx_basic.discard_pile

    def test_multiple_flurries_move(self, ctx_basic):
        """Multiple Flurry of Blows cards move on stance change."""
        ctx_basic.state.discard_pile.extend(["FlurryOfBlows", "FlurryOfBlows"])
        ctx_basic.change_stance("Wrath")
        flurry_count = ctx_basic.hand.count("FlurryOfBlows")
        assert flurry_count >= 2


# =============================================================================
# Wish Effect Tests
# =============================================================================

class TestWishEffect:
    """Test Wish choices."""

    def test_wish_strength(self, ctx_basic):
        """Wish: choose Strength (default)."""
        ctx_basic.extra_data["wish_choice"] = 1
        ctx_basic.is_upgraded = False
        execute_effect("choose_plated_armor_or_strength_or_gold", ctx_basic)
        assert ctx_basic.get_player_status("Strength") == 3

    def test_wish_plated_armor(self, ctx_basic):
        """Wish: choose Plated Armor."""
        ctx_basic.extra_data["wish_choice"] = 0
        ctx_basic.is_upgraded = False
        execute_effect("choose_plated_armor_or_strength_or_gold", ctx_basic)
        assert ctx_basic.get_player_status("PlatedArmor") == 3

    def test_wish_gold(self, ctx_basic):
        """Wish: choose Gold."""
        ctx_basic.extra_data["wish_choice"] = 2
        ctx_basic.is_upgraded = False
        execute_effect("choose_plated_armor_or_strength_or_gold", ctx_basic)
        assert ctx_basic.extra_data.get("gold_gained") == 50

    def test_wish_upgraded_values(self, ctx_basic):
        """Wish+: upgraded values."""
        ctx_basic.extra_data["wish_choice"] = 1
        ctx_basic.is_upgraded = True
        execute_effect("choose_plated_armor_or_strength_or_gold", ctx_basic)
        assert ctx_basic.get_player_status("Strength") == 4


# =============================================================================
# Blasphemy Tests
# =============================================================================

class TestBlasphemyEffect:
    """Test Blasphemy mechanics."""

    def test_blasphemy_enters_divinity(self, ctx_basic):
        """Blasphemy enters Divinity."""
        execute_effect("enter_divinity", ctx_basic)
        assert ctx_basic.stance == "Divinity"

    def test_blasphemy_sets_death_timer(self, ctx_basic):
        """Blasphemy sets death at end of next turn."""
        execute_effect("die_next_turn", ctx_basic)
        assert ctx_basic.get_player_status("Blasphemy") == 1


# =============================================================================
# Heal Effects
# =============================================================================

class TestHealEffects:
    """Test heal effects."""

    def test_heal_player(self, ctx_basic):
        """Test basic healing."""
        ctx_basic.player.hp = 50
        ctx_basic.heal_player(10)
        assert ctx_basic.player.hp == 60

    def test_heal_capped_at_max(self, ctx_basic):
        """Healing capped at max HP."""
        ctx_basic.player.hp = 70
        ctx_basic.player.max_hp = 72
        ctx_basic.heal_player(10)
        assert ctx_basic.player.hp == 72


# =============================================================================
# Integration Tests
# =============================================================================

class TestEffectIntegration:
    """Integration tests for multiple effects working together."""

    def test_calm_to_wrath_with_mental_fortress(self, ctx_basic):
        """Mental Fortress triggers on Calm to Wrath transition."""
        ctx_basic.state.stance = "Calm"
        ctx_basic.state.player.statuses["MentalFortress"] = 4
        initial_block = ctx_basic.player.block
        initial_energy = ctx_basic.energy

        result = ctx_basic.change_stance("Wrath")

        # Should gain 4 block from Mental Fortress
        assert ctx_basic.player.block >= initial_block + 4
        # Should gain 2 energy from exiting Calm
        assert ctx_basic.energy == initial_energy + 2
        assert ctx_basic.stance == "Wrath"

    def test_rushdown_triggers_on_wrath_entry(self, ctx_basic):
        """Rushdown draws cards when entering Wrath."""
        ctx_basic.state.player.statuses["Rushdown"] = 2
        initial_hand = len(ctx_basic.hand)

        ctx_basic.change_stance("Wrath")

        # Should draw 2 cards
        assert len(ctx_basic.hand) == initial_hand + 2


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
