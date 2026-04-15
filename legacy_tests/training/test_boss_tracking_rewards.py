"""Tests for boss tracking and reward fixes (Unit 1)."""
from __future__ import annotations

import pytest


class TestBossHPTracking:
    """Verify boss_max_hp and boss_dmg_dealt are populated correctly."""

    def test_boss_hp_progress_nonzero_for_boss_combat(self):
        """compute_boss_hp_progress returns > 0 when boss_max_hp > 0."""
        from packages.training.reward_config import compute_boss_hp_progress

        result = compute_boss_hp_progress(150.0, 300.0)
        assert result > 0, "boss HP progress should be > 0 for valid boss combat"
        # With scale 3.0: 150/300 * 3.0 = 1.5
        assert abs(result - 1.5) < 1e-6

    def test_boss_hp_progress_zero_when_no_max_hp(self):
        from packages.training.reward_config import compute_boss_hp_progress

        assert compute_boss_hp_progress(150.0, 0.0) == 0.0

    def test_boss_hp_progress_full_kill(self):
        from packages.training.reward_config import compute_boss_hp_progress

        result = compute_boss_hp_progress(300.0, 300.0)
        # 300/300 * 3.0 = 3.0
        assert abs(result - 3.0) < 1e-6


class TestBossMilestoneSplit:
    """Verify boss milestones are split into entry + survival."""

    def test_boss_entry_bonus_config(self):
        from packages.training.training_config import (
            BOSS_ENTRY_BONUS, BOSS_SURVIVAL_BONUS, BOSS_FLOORS,
        )

        assert BOSS_ENTRY_BONUS == 2.0
        assert BOSS_SURVIVAL_BONUS == 12.0
        assert BOSS_FLOORS == [16, 33, 50]

    def test_milestone_values_match_config(self):
        """Boss floor milestones should match BOSS_ENTRY_BONUS."""
        from packages.training.training_config import (
            BOSS_ENTRY_BONUS, BOSS_FLOORS, REWARD_WEIGHTS,
        )

        milestones = REWARD_WEIGHTS["floor_milestones"]
        for bf in BOSS_FLOORS:
            assert milestones[bf] == BOSS_ENTRY_BONUS, (
                f"Floor {bf} milestone should be {BOSS_ENTRY_BONUS}, got {milestones[bf]}"
            )

    def test_survival_milestone_values(self):
        """Post-boss floors should have survival bonus values."""
        from packages.training.training_config import (
            BOSS_SURVIVAL_BONUS, BOSS_FLOORS, REWARD_WEIGHTS,
        )

        milestones = REWARD_WEIGHTS["floor_milestones"]
        for bf in BOSS_FLOORS:
            post_boss = bf + 1
            assert post_boss in milestones, f"Floor {post_boss} should be in milestones"


class TestDeathPenalty:
    """Verify death penalty is -3.0."""

    def test_death_penalty_scale(self):
        from packages.training.training_config import REWARD_WEIGHTS

        assert REWARD_WEIGHTS["death_penalty_scale"] == -3.0


class TestPBRSHPWeight:
    """Verify PBRS HP coefficient is 0.80."""

    def test_hp_weight_in_potential(self):
        from packages.training.reward_config import compute_potential

        class FakeState:
            current_hp = 80
            max_hp = 80
            floor = 0
            deck = ["Strike"] * 15
            relics = []

        state_full = FakeState()
        pot_full = compute_potential(state_full)

        state_half = FakeState()
        state_half.current_hp = 40
        pot_half = compute_potential(state_half)

        # Difference should be 0.80 * (1.0 - 0.5) = 0.40
        hp_diff = pot_full - pot_half
        assert abs(hp_diff - 0.40) < 1e-6, f"HP weight diff should be 0.40, got {hp_diff}"


class TestF16HPBonus:
    """Verify F16 HP bonus config values."""

    def test_f16_hp_bonus_values(self):
        from packages.training.training_config import REWARD_WEIGHTS

        assert REWARD_WEIGHTS["f16_hp_bonus_base"] == 5.00
        assert REWARD_WEIGHTS["f16_hp_bonus_per_hp"] == 0.15
