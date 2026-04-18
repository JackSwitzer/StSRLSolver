"""
Audit tests for damage-triggered relics.

Verifies Python engine implementations against decompiled Java behavior.
"""

import pytest
import sys
sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.calc.damage import (
    calculate_damage, calculate_incoming_damage, apply_hp_loss,
    WEAK_MULT, VULN_MULT, WRATH_MULT,
)
from packages.engine.content.relics import (
    ALL_RELICS, TORII, TUNGSTEN_ROD, SELF_FORMING_CLAY, CENTENNIAL_PUZZLE,
    RED_SKULL, PAPER_CRANE, CHAMPIONS_BELT, HORN_CLEAT, TURNIP, THE_BOOT,
    FOSSILIZED_HELIX, RUNIC_CUBE, GINGER,
)


# =============================================================================
# Relic Data Definition Tests
# =============================================================================

class TestRelicDefinitions:
    """Verify relic data definitions match Java source."""

    def test_torii_is_rare(self):
        assert TORII.tier.value == "RARE"

    def test_torii_id(self):
        assert TORII.id == "Torii"

    def test_tungsten_rod_is_rare(self):
        assert TUNGSTEN_ROD.tier.value == "RARE"

    def test_tungsten_rod_id(self):
        assert TUNGSTEN_ROD.id == "TungstenRod"

    def test_self_forming_clay_is_uncommon(self):
        assert SELF_FORMING_CLAY.tier.value == "UNCOMMON"

    def test_centennial_puzzle_is_common(self):
        assert CENTENNIAL_PUZZLE.tier.value == "COMMON"

    def test_red_skull_is_common_ironclad(self):
        assert RED_SKULL.tier.value == "COMMON"
        assert RED_SKULL.player_class.value == "RED"

    def test_paper_crane_is_uncommon_silent(self):
        assert PAPER_CRANE.tier.value == "UNCOMMON"
        assert PAPER_CRANE.player_class.value == "GREEN"

    def test_champions_belt_is_rare_ironclad(self):
        assert CHAMPIONS_BELT.tier.value == "RARE"
        assert CHAMPIONS_BELT.player_class.value == "RED"

    def test_horn_cleat_is_uncommon(self):
        assert HORN_CLEAT.tier.value == "UNCOMMON"

    def test_horn_cleat_id(self):
        assert HORN_CLEAT.id == "HornCleat"

    def test_turnip_is_rare(self):
        assert TURNIP.tier.value == "RARE"

    def test_ginger_is_rare(self):
        assert GINGER.tier.value == "RARE"

    def test_boot_is_common(self):
        assert THE_BOOT.tier.value == "COMMON"
        assert THE_BOOT.id == "Boot"

    def test_fossilized_helix_is_rare(self):
        assert FOSSILIZED_HELIX.tier.value == "RARE"

    def test_runic_cube_is_boss(self):
        assert RUNIC_CUBE.tier.value == "BOSS"

    def test_all_audited_relics_in_registry(self):
        """All audited relics must be in ALL_RELICS."""
        expected_ids = [
            "Torii", "TungstenRod", "Self Forming Clay", "Centennial Puzzle",
            "Red Skull", "Paper Crane", "Champion Belt", "HornCleat",
            "Turnip", "Boot", "FossilizedHelix", "Runic Cube",
        ]
        for rid in expected_ids:
            assert rid in ALL_RELICS, f"Relic '{rid}' not in ALL_RELICS"


# =============================================================================
# Torii Tests (onAttacked: damage 2-5 reduced to 1)
# =============================================================================

class TestTorii:
    """Java: onAttacked -- if owner != null, type != HP_LOSS, type != THORNS,
    damageAmount > 1 && damageAmount <= 5, return 1."""

    def test_torii_reduces_2_to_1(self):
        hp, _ = calculate_incoming_damage(2, block=0, torii=True)
        assert hp == 1

    def test_torii_reduces_3_to_1(self):
        hp, _ = calculate_incoming_damage(3, block=0, torii=True)
        assert hp == 1

    def test_torii_reduces_5_to_1(self):
        hp, _ = calculate_incoming_damage(5, block=0, torii=True)
        assert hp == 1

    def test_torii_does_not_reduce_1(self):
        """Java: damageAmount > 1 required. 1 damage should stay 1."""
        hp, _ = calculate_incoming_damage(1, block=0, torii=True)
        assert hp == 1

    def test_torii_does_not_reduce_6(self):
        """6 damage is above threshold, stays 6."""
        hp, _ = calculate_incoming_damage(6, block=0, torii=True)
        assert hp == 6

    def test_torii_does_not_reduce_0(self):
        hp, _ = calculate_incoming_damage(0, block=0, torii=True)
        assert hp == 0

    def test_torii_with_block_absorbs(self):
        """4 damage, 5 block, Torii: block absorbs all 4, Torii doesn't trigger (post-block hp=0)."""
        hp, blk = calculate_incoming_damage(4, block=5, torii=True)
        assert hp == 0
        assert blk == 1  # 5 - 4 = 1 (Torii applies after block, not before)

    def test_torii_with_intangible(self):
        """Intangible caps to 1 first, then Torii range doesn't apply (1 is not > 1)."""
        hp, _ = calculate_incoming_damage(10, block=0, intangible=True, torii=True)
        assert hp == 1

    def test_torii_vuln_pushes_into_range(self):
        """3 damage + Vuln = 4 (in range). Torii reduces to 1."""
        hp, _ = calculate_incoming_damage(3, block=0, vuln=True, torii=True)
        # 3 * 1.5 = 4.5 -> 4, in range 2-5, Torii -> 1
        assert hp == 1

    def test_torii_vuln_pushes_out_of_range(self):
        """4 damage + Vuln = 6 (out of range). Torii does nothing."""
        hp, _ = calculate_incoming_damage(4, block=0, vuln=True, torii=True)
        # 4 * 1.5 = 6, out of range
        assert hp == 6


# =============================================================================
# Tungsten Rod Tests (onLoseHpLast: reduce HP loss by 1)
# =============================================================================

class TestTungstenRod:
    """Java: onLoseHpLast -- if damageAmount > 0, return damageAmount - 1."""

    def test_reduces_hp_loss_by_1(self):
        hp, _ = calculate_incoming_damage(10, block=5, tungsten_rod=True)
        # 10 - 5 = 5 hp loss, - 1 = 4
        assert hp == 4

    def test_does_not_reduce_zero_loss(self):
        hp, _ = calculate_incoming_damage(5, block=10, tungsten_rod=True)
        assert hp == 0

    def test_reduces_1_to_0(self):
        hp, _ = calculate_incoming_damage(6, block=5, tungsten_rod=True)
        # 6 - 5 = 1, - 1 = 0
        assert hp == 0

    def test_hp_loss_type(self):
        """Tungsten Rod works on HP_LOSS damage type too."""
        assert apply_hp_loss(5, tungsten_rod=True) == 4
        assert apply_hp_loss(1, tungsten_rod=True) == 0
        assert apply_hp_loss(0, tungsten_rod=True) == 0

    def test_with_intangible(self):
        """Intangible caps to 1, then Tungsten Rod reduces to 0."""
        assert apply_hp_loss(10, intangible=True, tungsten_rod=True) == 0

    def test_tungsten_rod_with_torii(self):
        """Torii reduces 4 to 1, block absorbs 0, Tungsten Rod reduces 1 to 0."""
        hp, _ = calculate_incoming_damage(4, block=0, torii=True, tungsten_rod=True)
        assert hp == 0


# =============================================================================
# Paper Crane Tests (Weak does 40% reduction instead of 25%)
# =============================================================================

class TestPaperCrane:
    """Java: WEAK_EFFECTIVENESS = 0.6f (attacker deals 60% = 40% reduction)."""

    def test_weak_with_paper_crane(self):
        """10 * 0.6 = 6 with Paper Crane."""
        result = calculate_damage(10, weak=True, weak_paper_crane=True)
        assert result == 6

    def test_weak_without_paper_crane(self):
        """10 * 0.75 = 7.5 -> 7 without Paper Crane."""
        result = calculate_damage(10, weak=True)
        assert result == 7

    def test_paper_crane_with_strength(self):
        """(6 + 3) * 0.6 = 5.4 -> 5."""
        result = calculate_damage(6, strength=3, weak=True, weak_paper_crane=True)
        assert result == 5


# =============================================================================
# The Boot Tests (onAttackToChangeDamage: min 5 damage on attacks)
# =============================================================================

class TestTheBoot:
    """Java: onAttackToChangeDamage -- if damage > 0 and < 5, return 5.
    This is for OUTGOING player attacks. The Boot data definition exists
    but the actual damage pipeline does not implement it."""

    def test_boot_data_definition_exists(self):
        assert THE_BOOT.id == "Boot"
        assert "onAttackToChangeDamage" in THE_BOOT.effects[0]

    def test_boot_effect_description_matches_java(self):
        """Boot should raise attacks dealing 1-4 to 5."""
        effect = THE_BOOT.effects[0]
        assert "5" in effect


# =============================================================================
# Horn Cleat Tests (atTurnStart turn 2: gain 14 Block)
# =============================================================================

class TestHornCleat:
    """Java: counter-based, triggers at counter==2 with 14 block, once per combat."""

    def test_horn_cleat_data(self):
        assert HORN_CLEAT.id == "HornCleat"

    def test_horn_cleat_block_value(self):
        """Java uses TURN_ACTIVATION=2 and block=14."""
        effect = HORN_CLEAT.effects[0]
        assert "14" in effect
        assert "turn 2" in effect.lower() or "Turn 2" in effect


# =============================================================================
# Self-Forming Clay Tests (wasHPLost: gain 3 Block next turn)
# =============================================================================

class TestSelfFormingClay:
    """Java: wasHPLost -- if in combat and damageAmount > 0,
    apply NextTurnBlockPower(3). NOT IMPLEMENTED in combat engine."""

    def test_data_definition_exists(self):
        assert SELF_FORMING_CLAY.id == "Self Forming Clay"

    def test_effect_description(self):
        assert "wasHPLost" in SELF_FORMING_CLAY.effects[0]
        assert "3" in SELF_FORMING_CLAY.effects[0]

    def test_is_ironclad_only(self):
        """Java: Self Forming Clay is Ironclad-only (canSpawn checks)."""
        assert SELF_FORMING_CLAY.player_class.value == "RED"


# =============================================================================
# Centennial Puzzle Tests (wasHPLost first time: draw 3)
# =============================================================================

class TestCentennialPuzzle:
    """Java: wasHPLost -- if damageAmount > 0 and in combat and !usedThisCombat,
    draw 3 and set usedThisCombat=true."""

    def test_data_definition(self):
        assert CENTENNIAL_PUZZLE.id == "Centennial Puzzle"

    def test_draws_3_cards(self):
        assert "3" in CENTENNIAL_PUZZLE.effects[0]

    def test_once_per_combat(self):
        assert "first time" in CENTENNIAL_PUZZLE.effects[0]


# =============================================================================
# Red Skull Tests (onBloodied: +3 Str, onNotBloodied: -3 Str)
# =============================================================================

class TestRedSkull:
    """Java: Dynamic strength toggle at 50% HP. NOT IMPLEMENTED in combat."""

    def test_data_definition(self):
        assert RED_SKULL.id == "Red Skull"

    def test_strength_value(self):
        """Java: STR_AMT = 3."""
        assert "3" in RED_SKULL.effects[0]

    def test_ironclad_only(self):
        assert RED_SKULL.player_class.value == "RED"


# =============================================================================
# Champion's Belt Tests (onTrigger: apply 1 Weak when Vulnerable applied)
# =============================================================================

class TestChampionsBelt:
    """Java: onTrigger(target) -- apply 1 Weak to target.
    Called when Vulnerable is applied. NOT IMPLEMENTED."""

    def test_data_definition(self):
        assert CHAMPIONS_BELT.id == "Champion Belt"

    def test_weak_amount(self):
        """Java: EFFECT = 1 Weak."""
        assert "1 Weak" in CHAMPIONS_BELT.effects[0]


# =============================================================================
# Turnip Tests (Frail immunity)
# =============================================================================

class TestTurnip:
    """Java: No hooks -- checked in ApplyPowerAction for Frail."""

    def test_data_definition(self):
        assert TURNIP.id == "Turnip"

    def test_effect(self):
        assert "Frail" in TURNIP.effects[0]


# =============================================================================
# Ginger Tests (Weak immunity)
# =============================================================================

class TestGinger:
    """Java: No hooks -- checked in ApplyPowerAction for Weak."""

    def test_data_definition(self):
        assert GINGER.id == "Ginger"

    def test_effect(self):
        assert "Weak" in GINGER.effects[0]


# =============================================================================
# Fossilized Helix Tests (prevent first damage per combat)
# =============================================================================

class TestFossilizedHelix:
    """Java: Uses Buffer-like mechanic. NOT IMPLEMENTED in combat."""

    def test_data_definition(self):
        assert FOSSILIZED_HELIX.id == "FossilizedHelix"

    def test_once_per_combat(self):
        assert "first time" in FOSSILIZED_HELIX.effects[0]


# =============================================================================
# Runic Cube Tests (wasHPLost: draw 1)
# =============================================================================

class TestRunicCube:
    """Java: wasHPLost -- draw 1 card. NOT IMPLEMENTED in combat."""

    def test_data_definition(self):
        assert RUNIC_CUBE.id == "Runic Cube"

    def test_is_boss_relic(self):
        assert RUNIC_CUBE.tier.value == "BOSS"

    def test_draw_on_hp_loss(self):
        assert "wasHPLost" in RUNIC_CUBE.effects[0]
        assert "1" in RUNIC_CUBE.effects[0]


# =============================================================================
# Integration: Torii + Tungsten Rod combo
# =============================================================================

class TestRelicCombos:
    """Test interactions between multiple damage relics."""

    def test_torii_then_tungsten_rod(self):
        """3 damage -> Torii reduces to 1 -> Tungsten Rod reduces to 0."""
        hp, _ = calculate_incoming_damage(3, block=0, torii=True, tungsten_rod=True)
        assert hp == 0

    def test_large_damage_only_tungsten(self):
        """10 damage, no Torii effect, Tungsten Rod -1 = 9."""
        hp, _ = calculate_incoming_damage(10, block=0, tungsten_rod=True)
        assert hp == 9

    def test_intangible_torii_tungsten(self):
        """50 damage -> Intangible caps to 1 -> Torii N/A (1 not in 2-5) -> Tungsten Rod -> 0."""
        hp, _ = calculate_incoming_damage(50, block=0, intangible=True, torii=True, tungsten_rod=True)
        assert hp == 0

    def test_vuln_wrath_torii(self):
        """2 damage in Wrath + Vuln: 2*2=4, 4*1.5=6 -> out of Torii range."""
        hp, _ = calculate_incoming_damage(2, block=0, is_wrath=True, vuln=True, torii=True)
        assert hp == 6

    def test_small_wrath_damage_torii(self):
        """1 damage in Wrath: 1*2=2, in Torii range -> 1."""
        hp, _ = calculate_incoming_damage(1, block=0, is_wrath=True, torii=True)
        assert hp == 1
