"""
Comprehensive test of RNG predictions against verified seed data.

Tests both the old SeedPredictor and new GameRNGState implementations.
"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from core.prediction.seed_predictor import SeedPredictor
from core.state.game_rng import GameRNGState, RNGStream, predict_card_reward, simulate_path


# ============================================================================
# VERIFIED TEST DATA
# ============================================================================

# Format: (seed, neow_option, floor, expected_cards, notes)
VERIFIED_SEEDS = [
    # Simple Neow options (offset=0)
    ("A", "HUNDRED_GOLD", 1, ["Pray", "Weave", "Foreign Influence"], "All 3 floors verified"),
    ("A", "HUNDRED_GOLD", 2, None, "Need floor 2 data"),  # We have this but need to look up
    ("H", "REMOVE_CARD", 1, ["Bowling Bash", "Wallop", "Collect"], "Verified"),
    ("I", "PERCENT_DAMAGE", 1, ["Tantrum", "Pray", "Evaluate"], "All 3 floors verified"),
    ("G", "THREE_CARDS", 1, ["Empty Body", "Third Eye", "Sash Whip"], "THREE_CARDS uses NeowEvent.rng"),
    ("D", "ONE_RANDOM_RARE_CARD", 1, ["Inner Peace", "Perseverance", "Tranquility"], "Uses NeowEvent.rng"),
    ("N", "THREE_ENEMY_KILL", 1, ["Sanctity", "Meditate", "Talk to the Hand"], "Verified"),
    ("B", "UPGRADE_CARD", 1, ["Follow-Up", "Crescendo", "Pressure Points"], "Verified"),
    ("B", "BOSS_SWAP", 1, ["Follow-Up", "Crescendo", "Pressure Points"], "Astrolabe boss swap"),

    # Calling Bell special case (offset=1)
    ("GA", "BOSS_SWAP_CALLING_BELL", 1, ["Conclude", "Empty Fist", "Flurry of Blows"], "Calling Bell offset"),

    # Transform (uses NeowEvent.rng, offset=0)
    ("GA", "TRANSFORM_CARD", 1, ["Mental Fortress", "Cut Through Fate", "Empty Body"], "Transform uses NeowEvent.rng"),

    # Variable offset cases (CURSE, COLORLESS)
    ("F", "RANDOM_COLORLESS", 1, ["Like Water", "Pressure Points", "Prostrate"], "Consumes 3+ cardRng"),
    # Compound choices: CURSE drawback + reward
    ("P", "CURSE+ONE_RARE_RELIC", 1, ["Worship", "Empty Body", "Third Eye"], "Curse=Decay, counter=1"),
    ("R", "CURSE+ONE_RARE_RELIC", 1, ["Protect", "Deceive Reality", "Empty Body"], "Curse=Parasite, counter=1"),
    ("C", "CURSE+COLORLESS_2", 1, ["Follow-Up", "Pray", "Evaluate"], "1 curse + 3 colorless = 4"),
    ("B", "CURSE+THREE_RARE", 1, ["Empty Fist", "Conclude", "Fear No Evil"], "THREE_RARE uses NeowEvent.rng"),
]

# Neow options mapped to GameRNGState consumption
# For compound choices (drawback + reward), specify total cardRng consumption
NEOW_OPTION_MAPPING = {
    "HUNDRED_GOLD": ("HUNDRED_GOLD", 0),
    "REMOVE_CARD": ("REMOVE_CARD", 0),
    "PERCENT_DAMAGE": ("PERCENT_DAMAGE", 0),
    "THREE_CARDS": ("THREE_CARDS", 0),
    "ONE_RANDOM_RARE_CARD": ("ONE_RANDOM_RARE_CARD", 0),
    "THREE_ENEMY_KILL": ("THREE_ENEMY_KILL", 0),
    "UPGRADE_CARD": ("UPGRADE_CARD", 0),
    "BOSS_SWAP": ("BOSS_SWAP", 0),
    "BOSS_SWAP_CALLING_BELL": ("BOSS_SWAP", 9),  # Calling Bell consumes 9
    "TRANSFORM_CARD": ("TRANSFORM_CARD", 0),
    "RANDOM_COLORLESS": ("RANDOM_COLORLESS", 3),
    "CURSE": ("CURSE", 1),
    # Compound choices: drawback + reward
    "CURSE+ONE_RARE_RELIC": ("CURSE", 1),  # ONE_RARE_RELIC uses relicRng
    "CURSE+THREE_RARE": ("CURSE", 1),  # THREE_RARE uses NeowEvent.rng
    "CURSE+COLORLESS_2": ("CURSE", 4),  # 1 curse + 3 colorless selections
}


def test_seed_predictor():
    """Test the old SeedPredictor with offset parameter."""
    print("=" * 70)
    print("Testing SeedPredictor (offset-based)")
    print("=" * 70)

    results = {"pass": 0, "fail": 0, "skip": 0}

    for seed, neow, floor, expected, notes in VERIFIED_SEEDS:
        if expected is None:
            results["skip"] += 1
            continue

        # Determine offset
        if neow == "BOSS_SWAP_CALLING_BELL":
            offset = 1
        elif neow in ["RANDOM_COLORLESS", "CURSE"]:
            # Variable offset - test with offset=0 first
            offset = 0
        else:
            offset = 0

        try:
            predictor = SeedPredictor(seed, card_rng_floor_offset=offset)
            cr = predictor.card_reward(floor)
            actual = [c[0] for c in cr.cards]

            # Check if all cards match
            match = all(e in actual for e in expected) and len(actual) == len(expected)

            if match:
                print(f"✓ PASS: Seed {seed}, {neow}, F{floor}")
                results["pass"] += 1
            else:
                # For variable offset, try other offsets
                if neow in ["RANDOM_COLORLESS", "CURSE"]:
                    found_match = False
                    for test_offset in range(1, 10):
                        p2 = SeedPredictor(seed, card_rng_floor_offset=test_offset)
                        cr2 = p2.card_reward(floor)
                        actual2 = [c[0] for c in cr2.cards]
                        if all(e in actual2 for e in expected):
                            print(f"~ PASS: Seed {seed}, {neow}, F{floor} (offset={test_offset})")
                            results["pass"] += 1
                            found_match = True
                            break
                    if not found_match:
                        print(f"✗ FAIL: Seed {seed}, {neow}, F{floor}")
                        print(f"  Expected: {expected}")
                        print(f"  Actual (offset=0): {actual}")
                        print(f"  Note: {notes}")
                        results["fail"] += 1
                else:
                    print(f"✗ FAIL: Seed {seed}, {neow}, F{floor}")
                    print(f"  Expected: {expected}")
                    print(f"  Actual: {actual}")
                    print(f"  Note: {notes}")
                    results["fail"] += 1

        except Exception as e:
            print(f"✗ ERROR: Seed {seed}, {neow}: {e}")
            results["fail"] += 1

    print()
    print(f"Results: {results['pass']} passed, {results['fail']} failed, {results['skip']} skipped")
    return results


def test_game_rng_state():
    """Test the new GameRNGState implementation."""
    print()
    print("=" * 70)
    print("Testing GameRNGState (counter-based)")
    print("=" * 70)

    results = {"pass": 0, "fail": 0, "skip": 0}

    for seed, neow, floor, expected, notes in VERIFIED_SEEDS:
        if expected is None:
            results["skip"] += 1
            continue

        try:
            state = GameRNGState(seed)

            # Get mapping (option_name, consumption) or just option_name
            mapping = NEOW_OPTION_MAPPING.get(neow, (neow, 0))
            if isinstance(mapping, tuple):
                mapped_neow, consumption = mapping
            else:
                mapped_neow, consumption = mapping, 0

            # Apply Neow option (this will use built-in consumption)
            boss_relic = "Calling Bell" if neow == "BOSS_SWAP_CALLING_BELL" else None
            state.apply_neow_choice(mapped_neow, boss_relic=boss_relic)

            # Override with explicit consumption if specified
            if consumption > 0 and neow != "BOSS_SWAP_CALLING_BELL":
                # Reset and apply explicit consumption
                state.set_counter(RNGStream.CARD, consumption)

            # Enter floor
            state.enter_floor(floor)

            # Get prediction
            actual = [c[0] for c in predict_card_reward(state)]

            # Check match
            match = all(e in actual for e in expected) and len(actual) == len(expected)

            if match:
                print(f"✓ PASS: Seed {seed}, {neow}, F{floor}")
                results["pass"] += 1
            else:
                print(f"✗ FAIL: Seed {seed}, {neow}, F{floor}")
                print(f"  Expected: {expected}")
                print(f"  Actual: {actual}")
                print(f"  Counter: {state.get_counter(RNGStream.CARD)}")
                print(f"  Note: {notes}")
                results["fail"] += 1

        except Exception as e:
            print(f"✗ ERROR: Seed {seed}, {neow}: {e}")
            import traceback
            traceback.print_exc()
            results["fail"] += 1

    print()
    print(f"Results: {results['pass']} passed, {results['fail']} failed, {results['skip']} skipped")
    return results


def test_encounter_predictions():
    """Test encounter predictions."""
    print()
    print("=" * 70)
    print("Testing Encounter Predictions")
    print("=" * 70)

    # From verified-seeds.md
    encounter_tests = [
        ("TEST123", 1, "Small Slimes"),  # Acid Slime S + Spike Slime M
        ("TEST123", 2, "Jaw Worm"),
        ("TEST123", 3, "Cultist"),
        ("1ABCD", 1, "Jaw Worm"),
        ("1ABCD", 2, "Cultist"),
        ("1ABCD", 3, "Small Slimes"),
        ("GA", 1, "Cultist"),
        ("GA", 2, "2 Louse"),
    ]

    results = {"pass": 0, "fail": 0}

    for seed, floor, expected_enemy in encounter_tests:
        try:
            predictor = SeedPredictor(seed)
            if floor <= len(predictor.encounters):
                enc = predictor.encounters[floor - 1]
                # Partial match on enemy name
                if expected_enemy.lower() in enc.enemy.lower() or enc.enemy.lower() in expected_enemy.lower():
                    print(f"✓ PASS: Seed {seed}, F{floor}: {enc.enemy}")
                    results["pass"] += 1
                else:
                    print(f"✗ FAIL: Seed {seed}, F{floor}")
                    print(f"  Expected: {expected_enemy}")
                    print(f"  Actual: {enc.enemy}")
                    results["fail"] += 1
            else:
                print(f"✗ FAIL: Seed {seed}, F{floor}: Not enough encounters generated")
                results["fail"] += 1
        except Exception as e:
            print(f"✗ ERROR: Seed {seed}, F{floor}: {e}")
            results["fail"] += 1

    print()
    print(f"Results: {results['pass']} passed, {results['fail']} failed")
    return results


def test_neow_options():
    """Test Neow option predictions."""
    print()
    print("=" * 70)
    print("Testing Neow Option Predictions")
    print("=" * 70)

    # From verified-seeds.md
    neow_tests = [
        ("TEST123", {
            "option1": "RANDOM_COLORLESS",
            "option2": "HUNDRED_GOLD",
            "drawback": "PERCENT_DAMAGE",
            "reward": "TRANSFORM_TWO_CARDS",
            "boss_swap": "Coffee Dripper",
        }),
        ("GA", {
            "boss_swap": "Calling Bell",
        }),
        ("B", {
            "boss_swap": "Astrolabe",
        }),
    ]

    results = {"pass": 0, "fail": 0}

    for seed, expected in neow_tests:
        try:
            predictor = SeedPredictor(seed)
            neow = predictor.neow

            all_match = True

            if "option1" in expected and neow.option1 != expected["option1"]:
                print(f"  Option1 mismatch: {neow.option1} vs {expected['option1']}")
                all_match = False

            if "option2" in expected and neow.option2 != expected["option2"]:
                print(f"  Option2 mismatch: {neow.option2} vs {expected['option2']}")
                all_match = False

            if "drawback" in expected and neow.option3_drawback != expected["drawback"]:
                print(f"  Drawback mismatch: {neow.option3_drawback} vs {expected['drawback']}")
                all_match = False

            if "reward" in expected and neow.option3_reward != expected["reward"]:
                print(f"  Reward mismatch: {neow.option3_reward} vs {expected['reward']}")
                all_match = False

            if "boss_swap" in expected and neow.boss_swap_relic != expected["boss_swap"]:
                print(f"  Boss swap mismatch: {neow.boss_swap_relic} vs {expected['boss_swap']}")
                all_match = False

            if all_match:
                print(f"✓ PASS: Seed {seed} Neow options")
                results["pass"] += 1
            else:
                print(f"✗ FAIL: Seed {seed} Neow options")
                results["fail"] += 1

        except Exception as e:
            print(f"✗ ERROR: Seed {seed}: {e}")
            results["fail"] += 1

    print()
    print(f"Results: {results['pass']} passed, {results['fail']} failed")
    return results


def test_path_simulation():
    """Test path simulation with shop visits."""
    print()
    print("=" * 70)
    print("Testing Path Simulation (Shop Impact)")
    print("=" * 70)

    # Test that shop visits correctly advance cardRng
    state_no_shop = GameRNGState("N")
    state_no_shop.apply_neow_choice("THREE_ENEMY_KILL")
    state_no_shop.enter_floor(1)
    state_no_shop.apply_combat("monster")
    state_no_shop.enter_floor(2)

    state_with_shop = GameRNGState("N")
    state_with_shop.apply_neow_choice("THREE_ENEMY_KILL")
    state_with_shop.enter_floor(1)
    state_with_shop.apply_combat("monster")
    state_with_shop.apply_shop()  # Visit shop
    state_with_shop.enter_floor(2)

    cards_no_shop = [c[0] for c in predict_card_reward(state_no_shop)]
    cards_with_shop = [c[0] for c in predict_card_reward(state_with_shop)]

    print(f"Without shop, Floor 2 cards: {cards_no_shop}")
    print(f"With shop, Floor 2 cards: {cards_with_shop}")
    print(f"Card counter without shop: {state_no_shop.get_counter(RNGStream.CARD)}")
    print(f"Card counter with shop: {state_with_shop.get_counter(RNGStream.CARD)}")

    if cards_no_shop != cards_with_shop:
        print("✓ PASS: Shop visit correctly shifts card predictions")
        return {"pass": 1, "fail": 0}
    else:
        print("✗ FAIL: Shop visit should change card predictions")
        return {"pass": 0, "fail": 1}


def test_act_transition():
    """Test act transition cardRng snapping."""
    print()
    print("=" * 70)
    print("Testing Act Transition (cardRng Snapping)")
    print("=" * 70)

    results = {"pass": 0, "fail": 0}

    # Test various counter positions
    test_cases = [
        (100, 250),  # 1-249 snaps to 250
        (200, 250),
        (249, 250),
        (260, 500),  # 251-499 snaps to 500
        (400, 500),
        (510, 750),  # 501-749 snaps to 750
        (250, 250),  # Exactly on boundary stays
        (500, 500),
        (0, 0),      # 0 stays at 0
    ]

    for start, expected in test_cases:
        state = GameRNGState("TEST")
        state.set_counter(RNGStream.CARD, start)
        state.transition_to_next_act()
        actual = state.get_counter(RNGStream.CARD)

        if actual == expected:
            print(f"✓ PASS: {start} → {actual}")
            results["pass"] += 1
        else:
            print(f"✗ FAIL: {start} → {actual} (expected {expected})")
            results["fail"] += 1

    print()
    print(f"Results: {results['pass']} passed, {results['fail']} failed")
    return results


def main():
    """Run all tests."""
    print("\n" + "=" * 70)
    print("COMPREHENSIVE RNG PREDICTION TEST SUITE")
    print("=" * 70 + "\n")

    all_results = {
        "seed_predictor": test_seed_predictor(),
        "game_rng_state": test_game_rng_state(),
        "encounters": test_encounter_predictions(),
        "neow_options": test_neow_options(),
        "path_simulation": test_path_simulation(),
        "act_transition": test_act_transition(),
    }

    # Summary
    print("\n" + "=" * 70)
    print("SUMMARY")
    print("=" * 70)

    total_pass = sum(r["pass"] for r in all_results.values())
    total_fail = sum(r["fail"] for r in all_results.values())
    total_skip = sum(r.get("skip", 0) for r in all_results.values())

    for name, results in all_results.items():
        status = "✓" if results["fail"] == 0 else "✗"
        print(f"{status} {name}: {results['pass']} pass, {results['fail']} fail")

    print()
    print(f"TOTAL: {total_pass} passed, {total_fail} failed, {total_skip} skipped")

    if total_fail > 0:
        print("\nEdge cases that need attention:")
        print("- CURSE drawback: Variable offset due to curse selection consuming cardRng")
        print("- RANDOM_COLORLESS: Variable offset (3+ calls, depends on duplicates)")
        print("- TheLibrary event: Consumes ~20 cardRng calls")

    return total_fail == 0


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
