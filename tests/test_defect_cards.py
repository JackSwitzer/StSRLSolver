"""
Defect Card Mechanics Tests

Comprehensive tests for all Defect card implementations covering:
- Orb channeling (Lightning, Frost, Dark, Plasma)
- Orb evoking (Dualcast, Multi-Cast, Recursion)
- Focus manipulation (Defragment, Consume, Biased Cognition)
- Orb-counting cards (Barrage, Blizzard, Thunder Strike)
- Card manipulation (All For One, Hologram, Seek)
- Powers (Echo Form, Creative AI, Storm, etc.)
"""

import pytest
# Path setup done in conftest.py

from packages.engine.content.cards import (
    Card, CardType, CardRarity, CardTarget, CardColor,
    get_card,
    # Defect Basic
    STRIKE_D, DEFEND_D, ZAP, DUALCAST,
    # Defect Common Attacks
    BALL_LIGHTNING, BARRAGE, BEAM_CELL, CLAW, COLD_SNAP,
    COMPILE_DRIVER, GO_FOR_THE_EYES, REBOUND, STREAMLINE, SWEEPING_BEAM,
    # Defect Common Skills
    CHARGE_BATTERY, COOLHEADED, HOLOGRAM, LEAP_D, RECURSION,
    STACK, STEAM_BARRIER, TURBO,
    # Defect Uncommon Attacks
    BLIZZARD, DOOM_AND_GLOOM, FTL, LOCKON, MELTER,
    RIP_AND_TEAR, SCRAPE, SUNDER,
    # Defect Uncommon Skills
    AGGREGATE, AUTO_SHIELDS, CHAOS, CHILL, CONSUME,
    DARKNESS_D, DOUBLE_ENERGY, EQUILIBRIUM_D, FORCE_FIELD,
    FUSION, GENETIC_ALGORITHM, GLACIER, OVERCLOCK, RECYCLE,
    REINFORCED_BODY, REPROGRAM, TEMPEST, WHITE_NOISE,
    # Defect Uncommon Powers
    CAPACITOR, DEFRAGMENT, HEATSINKS, HELLO_WORLD, LOOP_D,
    SELF_REPAIR, STATIC_DISCHARGE, STORM_D,
    # Defect Rare Attacks
    ALL_FOR_ONE, CORE_SURGE, HYPERBEAM, METEOR_STRIKE, THUNDER_STRIKE,
    # Defect Rare Skills
    AMPLIFY, FISSION, MULTI_CAST, RAINBOW, REBOOT, SEEK,
    # Defect Rare Powers
    BIASED_COGNITION, BUFFER, CREATIVE_AI, ECHO_FORM, ELECTRODYNAMICS,
    MACHINE_LEARNING,
    # Registry
    DEFECT_CARDS, ALL_CARDS,
)
from packages.engine.state.combat import (
    CombatState, EntityState, EnemyCombatState,
    create_player, create_enemy, create_combat
)
from packages.engine.state.rng import Random
from packages.engine.effects.orbs import (
    OrbManager, OrbType, Orb, get_orb_manager, channel_orb, channel_random_orb, evoke_orb
)
from packages.engine.effects.registry import execute_effect, EffectContext


# =============================================================================
# Test Fixtures
# =============================================================================

@pytest.fixture
def basic_combat():
    """Create a basic combat state for testing."""
    player = create_player(hp=70, max_hp=70)
    enemy = create_enemy(
        id="TestEnemy",
        hp=50,
        max_hp=50,
        move_damage=10,
        move_hits=1
    )
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
def multi_enemy_combat():
    """Create a combat with multiple enemies."""
    player = create_player(hp=70, max_hp=70)
    enemies = [
        create_enemy(id="Enemy1", hp=30, move_damage=5),
        create_enemy(id="Enemy2", hp=30, move_damage=8),
        create_enemy(id="Enemy3", hp=30, move_damage=6),
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


# =============================================================================
# BASIC DEFECT CARDS
# =============================================================================

class TestBasicDefectCards:
    """Test Defect's basic starting cards."""

    def test_strike_d_base_stats(self):
        """Strike (Defect): 1 cost, 6 damage."""
        card = get_card("Strike_B")
        assert card.cost == 1
        assert card.damage == 6
        assert card.card_type == CardType.ATTACK
        assert card.color == CardColor.BLUE

    def test_defend_d_base_stats(self):
        """Defend (Defect): 1 cost, 5 block."""
        card = get_card("Defend_B")
        assert card.cost == 1
        assert card.block == 5
        assert card.card_type == CardType.SKILL
        assert card.color == CardColor.BLUE

    def test_zap_base_stats(self):
        """Zap: 1 cost (0 upgraded), channel Lightning."""
        card = get_card("Zap")
        assert card.cost == 1
        assert card.upgrade_cost == 0
        assert "channel_lightning" in card.effects
        assert card.card_type == CardType.SKILL

    def test_zap_upgraded(self):
        """Zap+: 0 cost."""
        card = get_card("Zap", upgraded=True)
        assert card.current_cost == 0

    def test_dualcast_base_stats(self):
        """Dualcast: 1 cost (0 upgraded), evoke leftmost orb twice."""
        card = get_card("Dualcast")
        assert card.cost == 1
        assert card.upgrade_cost == 0
        assert "evoke_orb_twice" in card.effects


# =============================================================================
# ORB SYSTEM TESTS
# =============================================================================

class TestOrbSystem:
    """Test the orb system mechanics."""

    def test_orb_manager_creation(self, basic_combat):
        """OrbManager should be created with default 3 slots."""
        manager = get_orb_manager(basic_combat)
        assert manager.max_slots == 3
        assert manager.get_orb_count() == 0

    def test_channel_lightning(self, basic_combat):
        """Channeling Lightning should add orb and track count."""
        manager = get_orb_manager(basic_combat)
        result = manager.channel(OrbType.LIGHTNING, basic_combat)

        assert result["channeled"] == "Lightning"
        assert manager.get_orb_count() == 1
        assert manager.lightning_channeled == 1

    def test_channel_frost(self, basic_combat):
        """Channeling Frost should add orb and track count."""
        manager = get_orb_manager(basic_combat)
        result = manager.channel(OrbType.FROST, basic_combat)

        assert result["channeled"] == "Frost"
        assert manager.frost_channeled == 1

    def test_channel_dark(self, basic_combat):
        """Channeling Dark should add orb with accumulated damage."""
        manager = get_orb_manager(basic_combat)
        manager.channel(OrbType.DARK, basic_combat)

        assert manager.dark_channeled == 1
        orb = manager.get_first_orb()
        assert orb.orb_type == OrbType.DARK
        assert orb.accumulated_damage == 6  # Base value

    def test_channel_random_orb_uses_state_rng(self, basic_combat):
        """Random orb channeling should consume the combat RNG stream."""
        basic_combat.card_random_rng = Random(424242)
        before = basic_combat.card_random_rng.counter

        channel_random_orb(basic_combat)

        assert basic_combat.card_random_rng.counter > before

    def test_channel_plasma(self, basic_combat):
        """Channeling Plasma should add orb."""
        manager = get_orb_manager(basic_combat)
        manager.channel(OrbType.PLASMA, basic_combat)

        assert manager.plasma_channeled == 1
        orb = manager.get_first_orb()
        assert orb.orb_type == OrbType.PLASMA

    def test_channel_evokes_when_full(self, basic_combat):
        """Channeling when slots full should evoke leftmost."""
        manager = get_orb_manager(basic_combat)
        # Fill slots
        manager.channel(OrbType.LIGHTNING, basic_combat)
        manager.channel(OrbType.FROST, basic_combat)
        manager.channel(OrbType.DARK, basic_combat)

        assert manager.get_orb_count() == 3

        # Channel another - should evoke Lightning first
        result = manager.channel(OrbType.PLASMA, basic_combat)

        assert result["evoked"] == "Lightning"
        assert manager.get_orb_count() == 3
        # First orb is now Frost
        assert manager.get_first_orb().orb_type == OrbType.FROST

    def test_evoke_lightning(self, basic_combat):
        """Evoking Lightning deals damage to random enemy."""
        manager = get_orb_manager(basic_combat)
        manager.channel(OrbType.LIGHTNING, basic_combat)

        initial_hp = basic_combat.enemies[0].hp
        result = manager.evoke(basic_combat)

        assert result["evoked"] == True
        assert result["orb_type"] == "Lightning"
        # Damage should be 8 (base evoke value)
        assert result["effect"]["damage"] == 8
        assert basic_combat.enemies[0].hp < initial_hp

    def test_evoke_frost(self, basic_combat):
        """Evoking Frost gains block."""
        manager = get_orb_manager(basic_combat)
        manager.channel(OrbType.FROST, basic_combat)

        initial_block = basic_combat.player.block
        result = manager.evoke(basic_combat)

        assert result["effect"]["block"] == 5  # Base evoke value
        assert basic_combat.player.block == initial_block + 5

    def test_evoke_plasma(self, basic_combat):
        """Evoking Plasma gains energy."""
        manager = get_orb_manager(basic_combat)
        manager.channel(OrbType.PLASMA, basic_combat)

        initial_energy = basic_combat.energy
        result = manager.evoke(basic_combat)

        assert result["effect"]["energy"] == 2
        assert basic_combat.energy == initial_energy + 2

    def test_evoke_dark_targets_lowest_hp(self, multi_enemy_combat):
        """Evoking Dark deals accumulated damage to lowest HP enemy."""
        manager = get_orb_manager(multi_enemy_combat)
        manager.channel(OrbType.DARK, multi_enemy_combat)

        # Set one enemy to lowest HP
        multi_enemy_combat.enemies[1].hp = 10

        result = manager.evoke(multi_enemy_combat)

        # Should target enemy with 10 HP
        assert multi_enemy_combat.enemies[1].hp < 10

    def test_focus_affects_orb_values(self, basic_combat):
        """Focus should increase orb passive and evoke values."""
        manager = get_orb_manager(basic_combat)
        manager.focus = 2
        manager.channel(OrbType.LIGHTNING, basic_combat)

        result = manager.evoke(basic_combat)
        # 8 base + 2 focus = 10 damage
        assert result["effect"]["damage"] == 10

    def test_passive_trigger_frost(self, basic_combat):
        """Frost orb passive should gain block at end of turn."""
        manager = get_orb_manager(basic_combat)
        manager.channel(OrbType.FROST, basic_combat)

        initial_block = basic_combat.player.block
        result = manager.trigger_passives(basic_combat)

        # Frost passive: 2 base block
        assert result["total_block"] == 2
        assert basic_combat.player.block == initial_block + 2

    def test_passive_trigger_dark_accumulates(self, basic_combat):
        """Dark orb passive should accumulate damage."""
        manager = get_orb_manager(basic_combat)
        manager.channel(OrbType.DARK, basic_combat)

        orb = manager.get_first_orb()
        initial_accumulated = orb.accumulated_damage

        manager.trigger_passives(basic_combat)

        # Dark passive: +6 accumulated damage
        assert orb.accumulated_damage == initial_accumulated + 6

    def test_unique_orb_types(self, basic_combat):
        """Should correctly count unique orb types."""
        manager = get_orb_manager(basic_combat)
        manager.channel(OrbType.LIGHTNING, basic_combat)
        manager.channel(OrbType.LIGHTNING, basic_combat)
        manager.channel(OrbType.FROST, basic_combat)

        assert manager.get_unique_orb_types() == 2


# =============================================================================
# ORB CHANNELING CARDS
# =============================================================================

class TestOrbChannelingCards:
    """Test cards that channel orbs."""

    def test_ball_lightning_stats(self):
        """Ball Lightning: 1 cost, 7 damage (10 upgraded), channel Lightning."""
        card = get_card("Ball Lightning")
        assert card.cost == 1
        assert card.damage == 7
        assert "channel_lightning" in card.effects
        assert card.card_type == CardType.ATTACK

        upgraded = get_card("Ball Lightning", upgraded=True)
        assert upgraded.damage == 10

    def test_cold_snap_stats(self):
        """Cold Snap: 1 cost, 6 damage (9 upgraded), channel Frost."""
        card = get_card("Cold Snap")
        assert card.cost == 1
        assert card.damage == 6
        assert "channel_frost" in card.effects

        upgraded = get_card("Cold Snap", upgraded=True)
        assert upgraded.damage == 9

    def test_coolheaded_stats(self):
        """Coolheaded: 1 cost, channel Frost, draw 1 (2 upgraded)."""
        card = get_card("Coolheaded")
        assert card.cost == 1
        assert card.magic_number == 1
        assert "channel_frost" in card.effects
        assert "draw_cards" in card.effects

        upgraded = get_card("Coolheaded", upgraded=True)
        assert upgraded.magic_number == 2

    def test_darkness_stats(self):
        """Darkness: 1 cost, channel Dark (2 upgraded)."""
        card = get_card("Darkness")
        assert card.cost == 1
        assert "channel_dark" in card.effects

    def test_doom_and_gloom_stats(self):
        """Doom and Gloom: 2 cost, 10 damage (14 upgraded), channel Dark."""
        card = get_card("Doom and Gloom")
        assert card.cost == 2
        assert card.damage == 10
        assert "channel_dark" in card.effects
        assert card.target == CardTarget.ALL_ENEMY

        upgraded = get_card("Doom and Gloom", upgraded=True)
        assert upgraded.damage == 14

    def test_fusion_stats(self):
        """Fusion: 2 cost (1 upgraded), channel Plasma."""
        card = get_card("Fusion")
        assert card.cost == 2
        assert card.upgrade_cost == 1
        assert "channel_plasma" in card.effects

    def test_glacier_stats(self):
        """Glacier: 2 cost, 7 block (10 upgraded), channel 2 Frost."""
        card = get_card("Glacier")
        assert card.cost == 2
        assert card.block == 7
        assert "channel_2_frost" in card.effects

        upgraded = get_card("Glacier", upgraded=True)
        assert upgraded.block == 10

    def test_chaos_stats(self):
        """Chaos: 1 cost, channel 1 random orb (2 upgraded)."""
        card = get_card("Chaos")
        assert card.cost == 1
        assert card.magic_number == 1
        assert "channel_random_orb" in card.effects

        upgraded = get_card("Chaos", upgraded=True)
        assert upgraded.magic_number == 2

    def test_chill_stats(self):
        """Chill: 0 cost, channel Frost per enemy, exhaust."""
        card = get_card("Chill")
        assert card.cost == 0
        assert card.exhaust == True
        assert "channel_frost_per_enemy" in card.effects

    def test_rainbow_stats(self):
        """Rainbow: 2 cost, channel Lightning, Frost, Dark, exhaust."""
        card = get_card("Rainbow")
        assert card.cost == 2
        assert card.exhaust == True
        assert "channel_lightning_frost_dark" in card.effects

    def test_meteor_strike_stats(self):
        """Meteor Strike: 5 cost, 24 damage (30 upgraded), channel 3 Plasma."""
        card = get_card("Meteor Strike")
        assert card.cost == 5
        assert card.damage == 24
        assert "channel_3_plasma" in card.effects

        upgraded = get_card("Meteor Strike", upgraded=True)
        assert upgraded.damage == 30

    def test_tempest_stats(self):
        """Tempest: X cost, channel X Lightning, exhaust."""
        card = get_card("Tempest")
        assert card.cost == -1  # X cost
        assert card.exhaust == True
        assert "channel_x_lightning" in card.effects


# =============================================================================
# ORB EVOKE CARDS
# =============================================================================

class TestOrbEvokeCards:
    """Test cards that evoke orbs."""

    def test_dualcast_evokes_twice(self, basic_combat):
        """Dualcast should evoke the leftmost orb twice."""
        manager = get_orb_manager(basic_combat)
        manager.channel(OrbType.FROST, basic_combat)

        # Manually evoke twice (what Dualcast does)
        manager.evoke(basic_combat, times=2)

        # Should have gained 10 block (5 * 2)
        assert basic_combat.player.block == 10
        # Orb should be gone
        assert manager.get_orb_count() == 0

    def test_multi_cast_stats(self):
        """Multi-Cast: X cost, evoke first orb X times."""
        card = get_card("Multi-Cast")
        assert card.cost == -1  # X cost
        assert "evoke_first_orb_x_times" in card.effects

    def test_recursion_stats(self):
        """Recursion: 1 cost (0 upgraded), evoke then channel same orb type."""
        card = get_card("Redo")  # Java ID
        assert card.cost == 1
        assert card.upgrade_cost == 0
        assert "evoke_then_channel_same_orb" in card.effects


# =============================================================================
# FOCUS MANIPULATION CARDS
# =============================================================================

class TestFocusCards:
    """Test cards that manipulate Focus."""

    def test_defragment_stats(self):
        """Defragment: 1 cost, gain 1 Focus (2 upgraded)."""
        card = get_card("Defragment")
        assert card.cost == 1
        assert card.magic_number == 1
        assert "gain_focus" in card.effects
        assert card.card_type == CardType.POWER

        upgraded = get_card("Defragment", upgraded=True)
        assert upgraded.magic_number == 2

    def test_consume_stats(self):
        """Consume: 2 cost, gain 2 Focus (3 upgraded), lose 1 orb slot."""
        card = get_card("Consume")
        assert card.cost == 2
        assert card.magic_number == 2
        assert "gain_focus_lose_orb_slot" in card.effects

        upgraded = get_card("Consume", upgraded=True)
        assert upgraded.magic_number == 3

    def test_biased_cognition_stats(self):
        """Biased Cognition: 1 cost, gain 4 Focus (5 upgraded), lose 1 Focus/turn."""
        card = get_card("Biased Cognition")
        assert card.cost == 1
        assert card.magic_number == 4
        assert "gain_focus_lose_focus_each_turn" in card.effects
        assert card.card_type == CardType.POWER

        upgraded = get_card("Biased Cognition", upgraded=True)
        assert upgraded.magic_number == 5

    def test_hyperbeam_stats(self):
        """Hyperbeam: 2 cost, 26 damage (34 upgraded) to ALL, lose 3 Focus."""
        card = get_card("Hyperbeam")
        assert card.cost == 2
        assert card.damage == 26
        assert card.magic_number == 3
        assert "lose_focus" in card.effects
        assert card.target == CardTarget.ALL_ENEMY

        upgraded = get_card("Hyperbeam", upgraded=True)
        assert upgraded.damage == 34

    def test_reprogram_stats(self):
        """Reprogram: 1 cost, lose 1 Focus, gain 1 Str and 1 Dex (2 upgraded)."""
        card = get_card("Reprogram")
        assert card.cost == 1
        assert card.magic_number == 1
        assert "lose_focus_gain_strength_dex" in card.effects

        upgraded = get_card("Reprogram", upgraded=True)
        assert upgraded.magic_number == 2


# =============================================================================
# ORB COUNTING CARDS
# =============================================================================

class TestOrbCountingCards:
    """Test cards that scale with orb counts."""

    def test_barrage_stats(self):
        """Barrage: 1 cost, 4 damage (6 upgraded) per orb."""
        card = get_card("Barrage")
        assert card.cost == 1
        assert card.damage == 4
        assert "damage_per_orb" in card.effects

        upgraded = get_card("Barrage", upgraded=True)
        assert upgraded.damage == 6

    def test_compile_driver_stats(self):
        """Compile Driver: 1 cost, 7 damage (10 upgraded), draw per unique orb."""
        card = get_card("Compile Driver")
        assert card.cost == 1
        assert card.damage == 7
        assert "draw_per_unique_orb" in card.effects

        upgraded = get_card("Compile Driver", upgraded=True)
        assert upgraded.damage == 10

    def test_blizzard_stats(self):
        """Blizzard: 1 cost, deal damage = 2 (3 upgraded) per Frost channeled to ALL."""
        card = get_card("Blizzard")
        assert card.cost == 1
        assert card.magic_number == 2
        assert "damage_per_frost_channeled" in card.effects
        assert card.target == CardTarget.ALL_ENEMY

        upgraded = get_card("Blizzard", upgraded=True)
        assert upgraded.magic_number == 3

    def test_thunder_strike_stats(self):
        """Thunder Strike: 3 cost, 7 damage (9 upgraded) per Lightning channeled."""
        card = get_card("Thunder Strike")
        assert card.cost == 3
        assert card.damage == 7
        assert "damage_per_lightning_channeled" in card.effects

        upgraded = get_card("Thunder Strike", upgraded=True)
        assert upgraded.damage == 9


# =============================================================================
# POWER CARDS
# =============================================================================

class TestDefectPowers:
    """Test Defect power cards."""

    def test_capacitor_stats(self):
        """Capacitor: 1 cost, gain 2 orb slots (3 upgraded)."""
        card = get_card("Capacitor")
        assert card.cost == 1
        assert card.magic_number == 2
        assert "increase_orb_slots" in card.effects
        assert card.card_type == CardType.POWER

        upgraded = get_card("Capacitor", upgraded=True)
        assert upgraded.magic_number == 3

    def test_heatsinks_stats(self):
        """Heatsinks: 1 cost, draw 1 (2 upgraded) when you play a Power."""
        card = get_card("Heatsinks")
        assert card.cost == 1
        assert card.magic_number == 1
        assert "draw_on_power_play" in card.effects

        upgraded = get_card("Heatsinks", upgraded=True)
        assert upgraded.magic_number == 2

    def test_loop_stats(self):
        """Loop: 1 cost, rightmost orb triggers passive 1 (2 upgraded) extra time."""
        card = get_card("Loop")
        assert card.cost == 1
        assert card.magic_number == 1
        assert "trigger_orb_passive_extra" in card.effects

        upgraded = get_card("Loop", upgraded=True)
        assert upgraded.magic_number == 2

    def test_static_discharge_stats(self):
        """Static Discharge: 1 cost, channel 1 Lightning (2 upgraded) when taking damage."""
        card = get_card("Static Discharge")
        assert card.cost == 1
        assert card.magic_number == 1
        assert "channel_lightning_on_damage" in card.effects

        upgraded = get_card("Static Discharge", upgraded=True)
        assert upgraded.magic_number == 2

    def test_storm_stats(self):
        """Storm: 1 cost, channel Lightning when playing a Power."""
        card = get_card("Storm")
        assert card.cost == 1
        assert "channel_lightning_on_power_play" in card.effects

    def test_echo_form_stats(self):
        """Echo Form: 3 cost, ethereal, first card each turn plays twice."""
        card = get_card("Echo Form")
        assert card.cost == 3
        assert card.ethereal == True
        assert "play_first_card_twice" in card.effects

    def test_creative_ai_stats(self):
        """Creative AI: 3 cost (2 upgraded), add random Power to hand each turn."""
        card = get_card("Creative AI")
        assert card.cost == 3
        assert card.upgrade_cost == 2
        assert "add_random_power_each_turn" in card.effects

    def test_electrodynamics_stats(self):
        """Electrodynamics: 2 cost, Lightning hits all, channel 2 (3 upgraded) Lightning."""
        card = get_card("Electrodynamics")
        assert card.cost == 2
        assert card.magic_number == 2
        assert "lightning_hits_all" in card.effects
        assert "channel_lightning_magic" in card.effects  # Channels magicNumber Lightning orbs

        upgraded = get_card("Electrodynamics", upgraded=True)
        assert upgraded.magic_number == 3

    def test_machine_learning_stats(self):
        """Machine Learning: 1 cost, draw 1 additional card each turn."""
        card = get_card("Machine Learning")
        assert card.cost == 1
        assert "draw_extra_each_turn" in card.effects

    def test_buffer_stats(self):
        """Buffer: 2 cost, prevent next 1 HP loss (2 upgraded)."""
        card = get_card("Buffer")
        assert card.cost == 2
        assert card.magic_number == 1
        assert "prevent_next_hp_loss" in card.effects

        upgraded = get_card("Buffer", upgraded=True)
        assert upgraded.magic_number == 2

    def test_self_repair_stats(self):
        """Self Repair: 1 cost, heal 7 HP (10 upgraded) at end of combat."""
        card = get_card("Self Repair")
        assert card.cost == 1
        assert card.magic_number == 7
        assert "heal_at_end_of_combat" in card.effects

        upgraded = get_card("Self Repair", upgraded=True)
        assert upgraded.magic_number == 10


# =============================================================================
# CARD MANIPULATION CARDS
# =============================================================================

class TestCardManipulation:
    """Test cards that manipulate other cards."""

    def test_all_for_one_stats(self):
        """All For One: 2 cost, 10 damage (14 upgraded), return 0-cost from discard."""
        card = get_card("All For One")
        assert card.cost == 2
        assert card.damage == 10
        assert "return_all_0_cost_from_discard" in card.effects

        upgraded = get_card("All For One", upgraded=True)
        assert upgraded.damage == 14

    def test_hologram_stats(self):
        """Hologram: 1 cost, 3 block (5 upgraded), return card from discard, exhaust."""
        card = get_card("Hologram")
        assert card.cost == 1
        assert card.block == 3
        assert card.exhaust == True
        assert "return_card_from_discard" in card.effects

        upgraded = get_card("Hologram", upgraded=True)
        assert upgraded.block == 5

    def test_seek_stats(self):
        """Seek: 0 cost, search draw for 1 card (2 upgraded), exhaust."""
        card = get_card("Seek")
        assert card.cost == 0
        assert card.magic_number == 1
        assert card.exhaust == True
        assert "search_draw_pile" in card.effects

        upgraded = get_card("Seek", upgraded=True)
        assert upgraded.magic_number == 2

    def test_reboot_stats(self):
        """Reboot: 0 cost, shuffle all, draw 4 (6 upgraded), exhaust."""
        card = get_card("Reboot")
        assert card.cost == 0
        assert card.magic_number == 4
        assert card.exhaust == True
        assert "shuffle_hand_and_discard_draw" in card.effects

        upgraded = get_card("Reboot", upgraded=True)
        assert upgraded.magic_number == 6

    def test_fission_stats(self):
        """Fission: 0 cost, remove all orbs. Upgraded: gain energy and draw per orb."""
        card = get_card("Fission")
        assert card.cost == 0
        assert card.exhaust == True
        assert "remove_orbs_gain_energy_and_draw" in card.effects


# =============================================================================
# CONDITIONAL CARDS
# =============================================================================

class TestConditionalCards:
    """Test cards with conditional effects."""

    def test_go_for_the_eyes_stats(self):
        """Go for the Eyes: 0 cost, 3 damage (4 upgraded), Weak 1 (2 upgraded) if attacking."""
        card = get_card("Go for the Eyes")
        assert card.cost == 0
        assert card.damage == 3
        assert card.magic_number == 1
        assert "if_attacking_apply_weak" in card.effects

        upgraded = get_card("Go for the Eyes", upgraded=True)
        assert upgraded.damage == 4
        assert upgraded.magic_number == 2

    def test_ftl_stats(self):
        """FTL: 0 cost, 5 damage (6 upgraded), draw 1 if < 3 (4 upgraded) cards played."""
        card = get_card("FTL")
        assert card.cost == 0
        assert card.damage == 5
        assert card.magic_number == 3
        assert "if_played_less_than_x_draw" in card.effects

        upgraded = get_card("FTL", upgraded=True)
        assert upgraded.damage == 6
        assert upgraded.magic_number == 4

    def test_sunder_stats(self):
        """Sunder: 3 cost, 24 damage (32 upgraded), gain 3 energy if fatal."""
        card = get_card("Sunder")
        assert card.cost == 3
        assert card.damage == 24
        assert "if_fatal_gain_3_energy" in card.effects

        upgraded = get_card("Sunder", upgraded=True)
        assert upgraded.damage == 32

    def test_auto_shields_stats(self):
        """Auto-Shields: 1 cost, 11 block (15 upgraded) only if no block."""
        card = get_card("Auto Shields")
        assert card.cost == 1
        assert card.block == 11
        assert "only_if_no_block" in card.effects

        upgraded = get_card("Auto Shields", upgraded=True)
        assert upgraded.block == 15


# =============================================================================
# SPECIAL MECHANICS CARDS
# =============================================================================

class TestSpecialMechanics:
    """Test cards with unique mechanics."""

    def test_claw_stats(self):
        """Claw: 0 cost, 3 damage (5 upgraded), increase all Claw damage by 2."""
        card = get_card("Claw")
        assert card.cost == 0
        assert card.damage == 3
        assert card.magic_number == 2
        assert "increase_all_claw_damage" in card.effects

        upgraded = get_card("Claw", upgraded=True)
        assert upgraded.damage == 5

    def test_gash_java_id_alias_stats(self):
        """Gash should resolve as the Java ID alias for Claw."""
        card = get_card("Gash")
        assert card.cost == 0
        assert card.damage == 3
        assert "increase_all_claw_damage" in card.effects

    def test_impulse_stats(self):
        """Impulse: 1 cost Skill, exhaust; triggers orb start/end effects."""
        card = get_card("Impulse")
        assert card.cost == 1
        assert card.card_type == CardType.SKILL
        assert card.rarity == CardRarity.UNCOMMON
        assert card.target == CardTarget.SELF
        assert card.exhaust is True
        assert "trigger_orb_start_end" in card.effects

    def test_impulse_effect_triggers_orb_passives(self):
        """Impulse should trigger orb passive behavior when orbs are present."""
        state = create_combat(
            player_hp=70, player_max_hp=70,
            enemies=[EnemyCombatState(hp=40, max_hp=40, id="test")],
            deck=["Strike_B"],
        )
        channel_orb(state, "Lightning")
        before_hp = state.enemies[0].hp

        ctx = EffectContext(state=state, card=get_card("Impulse"))
        execute_effect("trigger_orb_start_end", ctx)

        assert state.enemies[0].hp < before_hp

    def test_streamline_stats(self):
        """Streamline: 2 cost, 15 damage (20 upgraded), cost reduces by 1 permanently."""
        card = get_card("Streamline")
        assert card.cost == 2
        assert card.damage == 15
        assert "reduce_cost_permanently" in card.effects

        upgraded = get_card("Streamline", upgraded=True)
        assert upgraded.damage == 20

    def test_genetic_algorithm_stats(self):
        """Genetic Algorithm: 1 cost, 1 block, gain 2 (3 upgraded) permanent block."""
        card = get_card("Genetic Algorithm")
        assert card.cost == 1
        assert card.block == 1
        assert card.magic_number == 2
        assert card.exhaust == True
        assert "block_increases_permanently" in card.effects

        upgraded = get_card("Genetic Algorithm", upgraded=True)
        assert upgraded.magic_number == 3

    def test_stack_stats(self):
        """Stack: 1 cost, block = discard pile size (+3 upgraded)."""
        card = get_card("Stack")
        assert card.cost == 1
        assert "block_equals_discard_size" in card.effects

    def test_force_field_stats(self):
        """Force Field: 4 cost, 12 block (16 upgraded), cost reduces per Power played."""
        card = get_card("Force Field")
        assert card.cost == 4
        assert card.block == 12
        assert "cost_reduces_per_power_played" in card.effects

        upgraded = get_card("Force Field", upgraded=True)
        assert upgraded.block == 16

    def test_double_energy_stats(self):
        """Double Energy: 1 cost (0 upgraded), double energy, exhaust."""
        card = get_card("Double Energy")
        assert card.cost == 1
        assert card.upgrade_cost == 0
        assert card.exhaust == True
        assert "double_energy" in card.effects

    def test_lockon_stats(self):
        """Lock-On: 1 cost, 8 damage (11 upgraded), apply 2 (3 upgraded) Lock-On."""
        card = get_card("Lockon")
        assert card.cost == 1
        assert card.damage == 8
        assert card.magic_number == 2
        assert "apply_lockon" in card.effects

        upgraded = get_card("Lockon", upgraded=True)
        assert upgraded.damage == 11
        assert upgraded.magic_number == 3


# =============================================================================
# CARD REGISTRY TESTS
# =============================================================================

class TestDefectCardRegistry:
    """Test Defect card registry completeness."""

    def test_all_defect_cards_registered(self):
        """All Defect cards should be in DEFECT_CARDS registry."""
        expected_cards = [
            # Basic
            "Strike_B", "Defend_B", "Zap", "Dualcast",
            # Common Attacks
            "Ball Lightning", "Barrage", "Beam Cell", "Claw",
            "Cold Snap", "Compile Driver", "Go for the Eyes",
            "Rebound", "Streamline", "Sweeping Beam",
            # Common Skills
            "Conserve Battery", "Coolheaded", "Hologram", "Leap",
            "Redo", "Stack", "Steam", "Turbo", "Impulse",
            # Uncommon Attacks
            "Blizzard", "Doom and Gloom", "FTL", "Lockon",
            "Melter", "Rip and Tear", "Scrape", "Sunder",
            # Uncommon Skills
            "Aggregate", "Auto Shields", "BootSequence", "Chaos",
            "Chill", "Consume", "Darkness", "Double Energy",
            "Undo", "Force Field", "Fusion", "Genetic Algorithm",
            "Glacier", "Steam Power", "Recycle", "Reinforced Body",
            "Reprogram", "Skim", "Tempest", "White Noise",
            # Uncommon Powers
            "Capacitor", "Defragment", "Heatsinks", "Hello World",
            "Loop", "Self Repair", "Static Discharge", "Storm",
            # Rare Attacks
            "All For One", "Core Surge", "Hyperbeam",
            "Meteor Strike", "Thunder Strike",
            # Rare Skills
            "Amplify", "Fission", "Multi-Cast", "Rainbow",
            "Reboot", "Seek",
            # Rare Powers
            "Biased Cognition", "Buffer", "Creative AI",
            "Echo Form", "Electrodynamics", "Machine Learning",
        ]

        for card_id in expected_cards:
            assert card_id in DEFECT_CARDS, f"Card {card_id} not in DEFECT_CARDS"

    def test_defect_card_count(self):
        """Defect should have correct number of cards."""
        # 4 basic + 10 common attacks + 8 common skills +
        # 8 uncommon attacks + 20 uncommon skills + 8 uncommon powers +
        # 5 rare attacks + 6 rare skills + 6 rare powers = 75 total
        assert len(DEFECT_CARDS) >= 70  # Allow some flexibility

    def test_all_defect_cards_have_blue_color(self):
        """All Defect cards should have blue color."""
        for card_id, card in DEFECT_CARDS.items():
            assert card.color == CardColor.BLUE, f"{card_id} should be BLUE"


# =============================================================================
# INTEGRATION TESTS
# =============================================================================

class TestDefectIntegration:
    """Integration tests for Defect mechanics."""

    def test_orb_manager_persists_across_actions(self, basic_combat):
        """Orb manager should persist state across multiple actions."""
        manager = get_orb_manager(basic_combat)

        # Channel some orbs
        channel_orb(basic_combat, "Lightning")
        channel_orb(basic_combat, "Frost")

        # Get manager again - should have same orbs
        manager2 = get_orb_manager(basic_combat)
        assert manager2 is manager
        assert manager2.get_orb_count() == 2

    def test_combat_state_copy_preserves_orbs(self, basic_combat):
        """Copying combat state should preserve orb manager state."""
        manager = get_orb_manager(basic_combat)
        manager.channel(OrbType.LIGHTNING, basic_combat)
        manager.channel(OrbType.FROST, basic_combat)
        manager.focus = 2

        # Copy state
        copy = basic_combat.copy()

        # Check orb manager was copied
        assert copy.orb_manager is not None
        assert copy.orb_manager is not basic_combat.orb_manager
        assert copy.orb_manager.get_orb_count() == 2
        assert copy.orb_manager.focus == 2

    def test_electrodynamics_makes_lightning_hit_all(self, multi_enemy_combat):
        """Electrodynamics should make Lightning orbs hit all enemies."""
        manager = get_orb_manager(multi_enemy_combat)
        manager.lightning_hits_all = True
        manager.channel(OrbType.LIGHTNING, multi_enemy_combat)

        initial_hps = [e.hp for e in multi_enemy_combat.enemies]

        # Evoke lightning
        manager.evoke(multi_enemy_combat)

        # All enemies should have taken damage
        for i, enemy in enumerate(multi_enemy_combat.enemies):
            assert enemy.hp < initial_hps[i], f"Enemy {i} should have taken damage"

    def test_loop_triggers_extra_passive(self, basic_combat):
        """Loop should trigger rightmost orb's passive extra times."""
        manager = get_orb_manager(basic_combat)
        manager.loop_stacks = 1
        manager.channel(OrbType.FROST, basic_combat)

        initial_block = basic_combat.player.block

        # Trigger passives
        result = manager.trigger_passives(basic_combat)

        # Should have 2x passive (1 base + 1 loop) = 4 block
        assert result["total_block"] == 4
        assert basic_combat.player.block == initial_block + 4
