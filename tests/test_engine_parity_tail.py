"""
Tests for engine parity tail — missing power triggers, relic handlers, and
edge cases identified in granular-powers.md, granular-relics.md, and
granular-events.md work units.

Covers:
- Electro (Electrodynamics) power trigger: lightning hits all
- Draw power: draw count integration
- Draw Reduction: lifecycle and draw count integration
- Violet Lotus relic: stance exit energy (alias coverage)
- Pen Nib relic: counter increment + damage double
- Incense Burner relic: 6-turn Intangible cycle
- Ice Cream relic: energy conservation
- Snecko Eye / Confusion: cost randomization on draw
- Toy Ornithopter relic: heal on potion use
"""

import pytest
from packages.engine.state.combat import (
    CombatState, EntityState, EnemyCombatState, create_combat, create_enemy,
)
from packages.engine.combat_engine import CombatEngine, CombatPhase
from packages.engine.content.cards import get_card, CardType
from packages.engine.registry import (
    execute_power_triggers, execute_relic_triggers,
    PowerContext, RelicContext, POWER_REGISTRY, RELIC_REGISTRY,
)
# Import modules to register handlers
from packages.engine.registry import powers as _powers  # noqa: F401
from packages.engine.registry import relics as _relics  # noqa: F401


# =============================================================================
# Helpers
# =============================================================================

def _make_state(
    player_hp=50, player_max_hp=50, enemies=None, deck=None,
    energy=3, relics=None, gold=100,
):
    if enemies is None:
        enemies = [EnemyCombatState(hp=30, max_hp=30, id="test_enemy")]
    if deck is None:
        deck = ["Strike_R"]
    state = create_combat(
        player_hp=player_hp, player_max_hp=player_max_hp,
        enemies=enemies, deck=deck, energy=energy,
        relics=relics or [],
    )
    state.gold = gold
    return state


# =============================================================================
# Electro (Electrodynamics) Power
# =============================================================================

class TestElectroPower:
    """Electrodynamics: Lightning orbs hit ALL enemies."""

    def test_electro_registered(self):
        """Electrodynamics should be registered for atBattleStart."""
        assert POWER_REGISTRY.has_handler("atBattleStart", "Electrodynamics")

    def test_electro_alias_registered(self):
        """Electro alias should be registered for atBattleStart."""
        assert POWER_REGISTRY.has_handler("atBattleStart", "Electro")

    def test_electro_sets_lightning_hits_all(self):
        """Electrodynamics trigger should set lightning_hits_all on OrbManager."""
        from packages.engine.effects.orbs import get_orb_manager
        state = _make_state()
        state.player.statuses["Electrodynamics"] = 1
        execute_power_triggers("atBattleStart", state, state.player)
        manager = get_orb_manager(state)
        assert manager.lightning_hits_all is True

    def test_electro_alias_sets_lightning_hits_all(self):
        """Electro alias trigger should also set lightning_hits_all."""
        from packages.engine.effects.orbs import get_orb_manager
        state = _make_state()
        state.player.statuses["Electro"] = 1
        execute_power_triggers("atBattleStart", state, state.player)
        manager = get_orb_manager(state)
        assert manager.lightning_hits_all is True

    def test_lightning_hits_all_enemies_with_electro(self):
        """Lightning orb evoke should hit all enemies when Electro active."""
        from packages.engine.effects.orbs import get_orb_manager, channel_orb
        state = _make_state(
            enemies=[
                EnemyCombatState(hp=30, max_hp=30, id="enemy_0"),
                EnemyCombatState(hp=30, max_hp=30, id="enemy_1"),
            ]
        )
        state.player.statuses["Electrodynamics"] = 1
        # Reduce orb slots to 1 via OrbSlots status (base=3, need -2)
        state.player.statuses["OrbSlots"] = -2
        manager = get_orb_manager(state)
        manager.lightning_hits_all = True
        assert manager.max_slots == 1
        # First channel fills the slot
        channel_orb(state, "Lightning")
        assert len(manager.orbs) == 1
        # Second channel evokes the first (8 damage to all enemies)
        channel_orb(state, "Lightning")
        # Both enemies should have taken damage (8 base evoke, 0 focus)
        assert state.enemies[0].hp < 30, f"enemy_0 hp={state.enemies[0].hp}"
        assert state.enemies[1].hp < 30, f"enemy_1 hp={state.enemies[1].hp}"


# =============================================================================
# Draw Power Integration
# =============================================================================

class TestDrawPowerIntegration:
    """Draw power: increases cards drawn per turn."""

    def test_draw_status_increases_draw_count(self):
        """Having Draw status should increase hand size after turn start."""
        state = _make_state(
            deck=["Strike_R"] * 20,
            energy=3,
        )
        state.player.statuses["Draw"] = 2
        engine = CombatEngine(state)
        engine.start_combat()
        # Should draw 5 (base) + 2 (Draw power) = 7 cards
        assert len(state.hand) == 7

    def test_draw_reduction_decreases_draw_count(self):
        """Draw Reduction status should reduce hand size."""
        state = _make_state(
            deck=["Strike_R"] * 20,
            energy=3,
        )
        state.player.statuses["Draw Reduction"] = 1
        engine = CombatEngine(state)
        engine.start_combat()
        # Should draw 5 (base) - 1 (Draw Reduction) = 4 cards
        assert len(state.hand) == 4

    def test_draw_and_reduction_combine(self):
        """Draw and Draw Reduction should combine correctly."""
        state = _make_state(
            deck=["Strike_R"] * 20,
            energy=3,
        )
        state.player.statuses["Draw"] = 3
        state.player.statuses["Draw Reduction"] = 1
        engine = CombatEngine(state)
        engine.start_combat()
        # Should draw 5 + 3 - 1 = 7 cards
        assert len(state.hand) == 7


# =============================================================================
# Violet Lotus Relic (alias coverage)
# =============================================================================

class TestVioletLotusRelic:
    """Violet Lotus: +1 energy when exiting Calm."""

    def test_violet_lotus_registered_both_aliases(self):
        """Both VioletLotus and 'Violet Lotus' should be registered."""
        assert RELIC_REGISTRY.has_handler("onChangeStance", "VioletLotus")
        assert RELIC_REGISTRY.has_handler("onChangeStance", "Violet Lotus")

    def test_violet_lotus_grants_energy_on_calm_exit(self):
        """Exiting Calm with Violet Lotus should grant +1 energy (total 3 from Calm)."""
        state = _make_state(relics=["Violet Lotus"], energy=0)
        trigger_data = {"old_stance": "Calm", "new_stance": "Wrath"}
        execute_relic_triggers("onChangeStance", state, trigger_data)
        # Violet Lotus grants 1 additional energy (Calm base 2 is handled elsewhere)
        assert state.energy == 1

    def test_violet_lotus_no_energy_from_wrath_exit(self):
        """Exiting Wrath should NOT grant energy from Violet Lotus."""
        state = _make_state(relics=["Violet Lotus"], energy=0)
        trigger_data = {"old_stance": "Wrath", "new_stance": "Neutral"}
        execute_relic_triggers("onChangeStance", state, trigger_data)
        assert state.energy == 0


# =============================================================================
# Pen Nib Relic
# =============================================================================

class TestPenNibRelic:
    """Pen Nib: double damage every 10th attack."""

    def test_pen_nib_counter_increment(self):
        """Pen Nib counter should increment on attack play."""
        state = _make_state(relics=["Pen Nib"])
        state.relic_counters["Pen Nib"] = 0
        card = get_card("Strike_R")
        trigger_data = {"card": card}
        execute_relic_triggers("onPlayCard", state, trigger_data)
        assert state.relic_counters["Pen Nib"] == 1

    def test_pen_nib_no_increment_on_skill(self):
        """Pen Nib counter should NOT increment on skill play."""
        state = _make_state(relics=["Pen Nib"])
        state.relic_counters["Pen Nib"] = 0
        card = get_card("Defend_R")
        trigger_data = {"card": card}
        execute_relic_triggers("onPlayCard", state, trigger_data)
        assert state.relic_counters["Pen Nib"] == 0

    def test_pen_nib_doubles_at_10(self):
        """Pen Nib should double damage on 10th attack."""
        state = _make_state(relics=["Pen Nib"])
        state.relic_counters["Pen Nib"] = 9
        trigger_data = {"value": 6}
        result = execute_relic_triggers("atDamageGive", state, trigger_data)
        # Should return doubled damage
        assert result == 12


# =============================================================================
# Incense Burner Relic
# =============================================================================

class TestIncenseBurnerRelic:
    """Incense Burner: Intangible every 6 turns."""

    def test_incense_burner_counter_increments(self):
        """Counter should increment each turn."""
        state = _make_state(relics=["Incense Burner"])
        state.relic_counters["Incense Burner"] = 0
        execute_relic_triggers("atTurnStart", state)
        assert state.relic_counters["Incense Burner"] == 1

    def test_incense_burner_intangible_at_6(self):
        """Should grant Intangible at turn 6 and reset counter."""
        state = _make_state(relics=["Incense Burner"])
        state.relic_counters["Incense Burner"] = 5
        execute_relic_triggers("atTurnStart", state)
        assert state.player.statuses.get("Intangible", 0) == 1
        assert state.relic_counters["Incense Burner"] == 0

    def test_incense_burner_full_cycle(self):
        """Full 6-turn cycle should grant exactly one Intangible."""
        state = _make_state(relics=["Incense Burner"])
        state.relic_counters["Incense Burner"] = 0
        for i in range(6):
            execute_relic_triggers("atTurnStart", state)
        assert state.player.statuses.get("Intangible", 0) == 1
        assert state.relic_counters["Incense Burner"] == 0


# =============================================================================
# Ice Cream Relic
# =============================================================================

class TestIceCreamRelic:
    """Ice Cream: unused energy carries over between turns."""

    def test_ice_cream_preserves_energy(self):
        """Energy should carry over to next turn with Ice Cream."""
        state = _make_state(
            relics=["Ice Cream"],
            deck=["Strike_R"] * 20,
            energy=3,
        )
        state.max_energy = 3
        engine = CombatEngine(state)
        engine.start_combat()
        # After start_combat, turn 1 begins: energy = max_energy (with Ice Cream: += max)
        turn1_energy = state.energy
        # End turn without playing anything
        from packages.engine.state.combat import EndTurn
        engine.execute_action(EndTurn())
        # After turn 2 start: leftover + max_energy
        assert state.energy == turn1_energy + 3

    def test_without_ice_cream_energy_resets(self):
        """Without Ice Cream, energy should reset each turn."""
        state = _make_state(
            deck=["Strike_R"] * 20,
            energy=3,
        )
        state.max_energy = 3
        engine = CombatEngine(state)
        engine.start_combat()
        from packages.engine.state.combat import EndTurn
        engine.execute_action(EndTurn())
        assert state.energy == 3


# =============================================================================
# Snecko Eye / Confusion Power
# =============================================================================

class TestSneckoEyeConfusion:
    """Snecko Eye: Confused power randomizes card costs on draw."""

    def test_snecko_eye_applies_confused(self):
        """Snecko Eye should apply Confused at combat start."""
        state = _make_state(relics=["Snecko Eye"], deck=["Strike_R"] * 10)
        execute_relic_triggers("atBattleStart", state)
        assert state.player.statuses.get("Confused", 0) > 0

    def test_confusion_randomizes_cost(self):
        """Confusion power should set card cost to 0-3 on draw."""
        state = _make_state(deck=["Strike_R"] * 10)
        state.player.statuses["Confusion"] = 1
        # Provide RNG
        from packages.engine.state.rng import Random
        state.card_random_rng = Random(42)
        trigger_data = {"card_id": "Strike_R"}
        execute_power_triggers("onCardDraw", state, state.player, trigger_data)
        assert "Strike_R" in state.card_costs
        assert 0 <= state.card_costs["Strike_R"] <= 3


# =============================================================================
# Toy Ornithopter Relic
# =============================================================================

class TestToyOrnithopterRelic:
    """Toy Ornithopter: Heal 5 HP when using a potion."""

    def test_toy_ornithopter_registered(self):
        """Toy Ornithopter should be registered for onUsePotion."""
        assert RELIC_REGISTRY.has_handler("onUsePotion", "Toy Ornithopter")

    def test_toy_ornithopter_heals_on_potion(self):
        """Toy Ornithopter should heal 5 HP on potion use."""
        state = _make_state(
            relics=["Toy Ornithopter"],
            player_hp=40, player_max_hp=50,
        )
        execute_relic_triggers("onUsePotion", state)
        assert state.player.hp == 45

    def test_toy_ornithopter_no_overheal(self):
        """Toy Ornithopter heal should not exceed max HP."""
        state = _make_state(
            relics=["Toy Ornithopter"],
            player_hp=48, player_max_hp=50,
        )
        execute_relic_triggers("onUsePotion", state)
        assert state.player.hp == 50


# =============================================================================
# Draw Reduction Lifecycle
# =============================================================================

class TestDrawReductionLifecycle:
    """Draw Reduction: decrement per round with skip-first-round."""

    def test_draw_reduction_registered(self):
        assert POWER_REGISTRY.has_handler("atEndOfRound", "Draw Reduction")

    def test_draw_reduction_removes_at_zero(self):
        """Draw Reduction should be removed when it reaches 0."""
        state = _make_state()
        state.player.statuses["Draw Reduction"] = 1
        # First round: skip
        execute_power_triggers("atEndOfRound", state, state.player)
        assert state.player.statuses.get("Draw Reduction", 0) == 1
        # Second round: decrement to 0 -> remove
        execute_power_triggers("atEndOfRound", state, state.player)
        assert "Draw Reduction" not in state.player.statuses


# =============================================================================
# MasterReality Interaction (Java: MakeTempCardInHandAction auto-upgrades)
# =============================================================================

class TestMasterRealityAutoUpgrade:
    """MasterReality: created cards are auto-upgraded.

    Java: MakeTempCardInHandAction checks hasPower('MasterRealityPower')
    and upgrades non-Curse/non-Status cards. Verified against Java source.
    """

    def test_battle_hymn_upgrades_smite_with_master_reality(self):
        """BattleHymn should add Smite+ when MasterReality is active."""
        state = _make_state()
        state.player.statuses["BattleHymn"] = 1
        state.player.statuses["MasterReality"] = 1
        execute_power_triggers("atStartOfTurn", state, state.player)
        assert "Smite+" in state.hand

    def test_battle_hymn_normal_smite_without_master_reality(self):
        """BattleHymn should add Smite (not +) without MasterReality."""
        state = _make_state()
        state.player.statuses["BattleHymn"] = 1
        execute_power_triggers("atStartOfTurn", state, state.player)
        assert "Smite" in state.hand
        assert "Smite+" not in state.hand

    def test_infinite_blades_upgrades_shiv_with_master_reality(self):
        """InfiniteBlades should add Shiv+ when MasterReality is active."""
        state = _make_state()
        state.player.statuses["InfiniteBlades"] = 1
        state.player.statuses["MasterReality"] = 1
        execute_power_triggers("atStartOfTurn", state, state.player)
        assert "Shiv+" in state.hand

    def test_infinite_blades_normal_shiv_without_master_reality(self):
        """InfiniteBlades should add Shiv (not +) without MasterReality."""
        state = _make_state()
        state.player.statuses["InfiniteBlades"] = 1
        execute_power_triggers("atStartOfTurn", state, state.player)
        assert "Shiv" in state.hand
        assert "Shiv+" not in state.hand

    def test_pure_water_upgrades_miracle_with_master_reality(self):
        """PureWater relic should add Miracle+ when MasterReality is active."""
        state = _make_state(relics=["PureWater"])
        state.player.statuses["MasterReality"] = 1
        execute_relic_triggers("atBattleStartPreDraw", state)
        assert "Miracle+" in state.hand

    def test_pure_water_normal_miracle_without_master_reality(self):
        """PureWater relic should add Miracle (not +) without MasterReality."""
        state = _make_state(relics=["PureWater"])
        execute_relic_triggers("atBattleStartPreDraw", state)
        assert "Miracle" in state.hand
        assert "Miracle+" not in state.hand

    def test_holy_water_upgrades_miracles_with_master_reality(self):
        """HolyWater relic should add 3 Miracle+ when MasterReality is active."""
        state = _make_state(relics=["HolyWater"])
        state.player.statuses["MasterReality"] = 1
        execute_relic_triggers("atBattleStartPreDraw", state)
        upgraded_count = sum(1 for c in state.hand if c == "Miracle+")
        assert upgraded_count == 3

    def test_ninja_scroll_upgrades_shivs_with_master_reality(self):
        """NinjaScroll relic should add 3 Shiv+ when MasterReality is active."""
        state = _make_state(relics=["Ninja Scroll"])
        state.player.statuses["MasterReality"] = 1
        execute_relic_triggers("atBattleStartPreDraw", state)
        upgraded_count = sum(1 for c in state.hand if c == "Shiv+")
        assert upgraded_count == 3


# =============================================================================
# CarveReality / DeceiveReality MasterReality interaction
# =============================================================================

class TestCarveRealityMasterReality:
    """Carve Reality: upgrade only increases damage, never upgrades generated Smite.
    MasterReality is the only thing that upgrades the generated Smite."""

    def test_carve_reality_base_produces_base_smite(self):
        """Base Carve Reality should produce base Smite."""
        from packages.engine.effects.registry import EffectContext, execute_effect
        state = _make_state()
        ctx = EffectContext(state=state, is_upgraded=False)
        execute_effect("add_smite_to_hand", ctx)
        assert "Smite" in state.hand
        assert "Smite+" not in state.hand

    def test_carve_reality_upgraded_still_produces_base_smite(self):
        """Upgraded Carve Reality should still produce base Smite (Java: upgrade only increases damage)."""
        from packages.engine.effects.registry import EffectContext, execute_effect
        state = _make_state()
        ctx = EffectContext(state=state, is_upgraded=True)
        execute_effect("add_smite_to_hand", ctx)
        assert "Smite" in state.hand
        assert "Smite+" not in state.hand

    def test_carve_reality_master_reality_upgrades_smite(self):
        """MasterReality should upgrade the generated Smite to Smite+."""
        from packages.engine.effects.registry import EffectContext, execute_effect
        state = _make_state()
        state.player.statuses["MasterReality"] = 1
        ctx = EffectContext(state=state, is_upgraded=False)
        execute_effect("add_smite_to_hand", ctx)
        assert "Smite+" in state.hand


class TestDeceiveRealityMasterReality:
    """Deceive Reality: upgrade only increases block, never upgrades generated Safety.
    MasterReality is the only thing that upgrades the generated Safety."""

    def test_deceive_reality_base_produces_base_safety(self):
        """Base Deceive Reality should produce base Safety."""
        from packages.engine.effects.registry import EffectContext, execute_effect
        state = _make_state()
        ctx = EffectContext(state=state, is_upgraded=False)
        execute_effect("add_safety_to_hand", ctx)
        assert "Safety" in state.hand
        assert "Safety+" not in state.hand

    def test_deceive_reality_upgraded_still_produces_base_safety(self):
        """Upgraded Deceive Reality should still produce base Safety (Java: upgrade only increases block)."""
        from packages.engine.effects.registry import EffectContext, execute_effect
        state = _make_state()
        ctx = EffectContext(state=state, is_upgraded=True)
        execute_effect("add_safety_to_hand", ctx)
        assert "Safety" in state.hand
        assert "Safety+" not in state.hand

    def test_deceive_reality_master_reality_upgrades_safety(self):
        """MasterReality should upgrade the generated Safety to Safety+."""
        from packages.engine.effects.registry import EffectContext, execute_effect
        state = _make_state()
        state.player.statuses["MasterReality"] = 1
        ctx = EffectContext(state=state, is_upgraded=False)
        execute_effect("add_safety_to_hand", ctx)
        assert "Safety+" in state.hand


# =============================================================================
# BottledMiracle MasterReality interaction
# =============================================================================

class TestBottledMiracleMasterReality:
    """BottledMiracle potion: Java uses MakeTempCardInHandAction, so MasterReality applies."""

    def test_bottled_miracle_base_produces_base_miracles(self):
        """BottledMiracle without MasterReality produces base Miracles."""
        from packages.engine.registry import PotionContext
        from packages.engine.registry.potions import bottled_miracle
        state = _make_state()
        ctx = PotionContext(state=state, potion_id="BottledMiracle", potency=2)
        bottled_miracle(ctx)
        miracle_count = sum(1 for c in state.hand if c == "Miracle")
        assert miracle_count == 2
        assert "Miracle+" not in state.hand

    def test_bottled_miracle_master_reality_upgrades(self):
        """BottledMiracle with MasterReality produces Miracle+."""
        from packages.engine.registry import PotionContext
        from packages.engine.registry.potions import bottled_miracle
        state = _make_state()
        state.player.statuses["MasterReality"] = 1
        ctx = PotionContext(state=state, potion_id="BottledMiracle", potency=2)
        bottled_miracle(ctx)
        upgraded_count = sum(1 for c in state.hand if c == "Miracle+")
        assert upgraded_count == 2


# =============================================================================
# Generated card upgrade parity (Java: upgrade only affects source card stats)
# =============================================================================

class TestGeneratedCardUpgradeParity:
    """Java: upgrading a card that generates temp cards only affects the source
    card's stats (damage, block, cost), never the generated card. MasterReality
    is the only mechanism that upgrades generated cards via MakeTempCardInHandAction."""

    def test_evaluate_upgraded_still_produces_base_insight(self):
        """Upgraded Evaluate produces base Insight (upgrade only increases block)."""
        from packages.engine.effects.registry import EffectContext, execute_effect
        state = _make_state()
        ctx = EffectContext(state=state, is_upgraded=True)
        execute_effect("add_insight_to_draw", ctx)
        assert "Insight" in state.draw_pile
        assert "Insight+" not in state.draw_pile

    def test_reach_heaven_upgraded_still_produces_base_through_violence(self):
        """Upgraded Reach Heaven produces base ThroughViolence (upgrade only increases damage)."""
        from packages.engine.effects.registry import EffectContext, execute_effect
        state = _make_state()
        ctx = EffectContext(state=state, is_upgraded=True)
        execute_effect("add_through_violence_to_draw", ctx)
        assert "ThroughViolence" in state.draw_pile
        assert "ThroughViolence+" not in state.draw_pile

    def test_alpha_upgraded_still_produces_base_beta(self):
        """Upgraded Alpha produces base Beta (upgrade only makes it innate)."""
        from packages.engine.effects.registry import EffectContext, execute_effect
        state = _make_state()
        ctx = EffectContext(state=state, is_upgraded=True)
        execute_effect("shuffle_beta_into_draw", ctx)
        assert "Beta" in state.draw_pile
        assert "Beta+" not in state.draw_pile

    def test_dead_branch_master_reality_upgrades(self):
        """Dead Branch with MasterReality should produce upgraded cards."""
        state = _make_state(relics=["Dead Branch"])
        state.player.statuses["MasterReality"] = 1
        execute_relic_triggers("onExhaust", state)
        # All cards added should be upgraded (end with +)
        new_cards = [c for c in state.hand if c != "Strike_R"]
        assert len(new_cards) > 0
        for card in new_cards:
            assert card.endswith("+"), f"Expected upgraded card, got {card}"

    def test_dead_branch_no_master_reality_base_cards(self):
        """Dead Branch without MasterReality should produce base cards."""
        state = _make_state(relics=["Dead Branch"])
        execute_relic_triggers("onExhaust", state)
        new_cards = [c for c in state.hand if c != "Strike_R"]
        assert len(new_cards) > 0
        for card in new_cards:
            assert not card.endswith("+"), f"Expected base card, got {card}"
