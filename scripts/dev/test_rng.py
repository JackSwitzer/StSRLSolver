#!/usr/bin/env python3
"""
STS RNG Prediction Tester

Test RNG predictions against known seeds and expected values.
Useful for verifying the prediction system works correctly.

Usage:
    uv run scripts/dev/test_rng.py                  # Run all tests
    uv run scripts/dev/test_rng.py neow             # Test Neow predictions
    uv run scripts/dev/test_rng.py bosses           # Test boss predictions
    uv run scripts/dev/test_rng.py cards            # Test card reward predictions
    uv run scripts/dev/test_rng.py --seed ABC12     # Test with specific seed
"""

import argparse
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

# Project setup
PROJECT_ROOT = Path(__file__).parent.parent.parent
sys.path.insert(0, str(PROJECT_ROOT))


def test_neow_predictions(seed_str: Optional[str] = None, verbose: bool = True) -> Dict[str, Any]:
    """Test Neow option predictions."""
    from core.state.rng import seed_to_long
    from web.server import predict_neow_options

    results = {
        "passed": 0,
        "failed": 0,
        "errors": [],
        "predictions": [],
    }

    # Test seeds (if none specified, use defaults)
    test_seeds = [seed_str] if seed_str else ["ABC12", "XYZ99", "3ERSKZUJDGLE5"]

    for seed in test_seeds:
        try:
            seed_long = seed_to_long(seed)
            options = predict_neow_options(seed_long, "WATCHER")

            prediction = {
                "seed": seed,
                "seed_long": seed_long,
                "options": options,
            }
            results["predictions"].append(prediction)

            # Validate structure
            if len(options) != 4:
                results["errors"].append(f"{seed}: Expected 4 options, got {len(options)}")
                results["failed"] += 1
            else:
                # Check each option has required fields
                for i, opt in enumerate(options):
                    required = ["slot", "option", "name", "category"]
                    missing = [f for f in required if f not in opt]
                    if missing:
                        results["errors"].append(f"{seed} option {i+1}: missing {missing}")
                        results["failed"] += 1
                    else:
                        results["passed"] += 1

            if verbose:
                print(f"\nSeed: {seed} ({seed_long})")
                for opt in options:
                    name = opt.get("name", "?")
                    drawback = opt.get("drawback")
                    if drawback:
                        print(f"  {opt.get('slot', '?')}. {name} ({drawback})")
                    else:
                        print(f"  {opt.get('slot', '?')}. {name}")

        except Exception as e:
            results["errors"].append(f"{seed}: {e}")
            results["failed"] += 1

    return results


def test_boss_predictions(seed_str: Optional[str] = None, verbose: bool = True) -> Dict[str, Any]:
    """Test boss predictions for all acts."""
    from core.state.rng import seed_to_long
    from core.generation.encounters import predict_all_bosses_extended

    results = {
        "passed": 0,
        "failed": 0,
        "errors": [],
        "predictions": [],
    }

    test_seeds = [seed_str] if seed_str else ["ABC12", "XYZ99"]

    for seed in test_seeds:
        try:
            seed_long = seed_to_long(seed)

            # Test at different ascension levels
            for asc in [0, 20]:
                bosses = predict_all_bosses_extended(seed_long, ascension=asc)

                prediction = {
                    "seed": seed,
                    "ascension": asc,
                    "bosses": bosses,
                }
                results["predictions"].append(prediction)

                # Validate we have all 3 acts
                expected_acts = [1, 2, 3]
                actual_acts = list(bosses.keys())

                for act in expected_acts:
                    if act not in actual_acts:
                        results["errors"].append(f"{seed} A{asc}: missing act {act}")
                        results["failed"] += 1
                    elif not bosses[act]:
                        results["errors"].append(f"{seed} A{asc}: act {act} has no bosses")
                        results["failed"] += 1
                    else:
                        results["passed"] += 1

                if verbose:
                    print(f"\nSeed: {seed} (A{asc})")
                    for act, boss_list in sorted(bosses.items()):
                        boss_str = " + ".join(boss_list) if isinstance(boss_list, list) else str(boss_list)
                        print(f"  Act {act}: {boss_str}")

        except Exception as e:
            results["errors"].append(f"{seed}: {e}")
            results["failed"] += 1

    return results


def test_card_predictions(seed_str: Optional[str] = None, verbose: bool = True) -> Dict[str, Any]:
    """Test card reward predictions."""
    from core.comparison.full_rng_tracker import predict_card_reward
    from core.state.rng import seed_to_long, long_to_seed

    results = {
        "passed": 0,
        "failed": 0,
        "errors": [],
        "predictions": [],
    }

    test_seeds = [seed_str] if seed_str else ["ABC12", "XYZ99"]

    for seed in test_seeds:
        try:
            seed_long = seed_to_long(seed)
            seed_formatted = long_to_seed(seed_long)

            # Test card reward at different counter values
            for counter in [0, 9, 50]:
                for room in ["normal", "elite"]:
                    cards, new_counter = predict_card_reward(
                        seed_formatted,
                        counter,
                        act=1,
                        room_type=room,
                        card_blizzard=5,
                        relics=["PureWater"],
                    )

                    prediction = {
                        "seed": seed,
                        "counter": counter,
                        "room": room,
                        "cards": cards,
                        "new_counter": new_counter,
                    }
                    results["predictions"].append(prediction)

                    # Validate
                    if not cards or len(cards) < 3:
                        results["errors"].append(f"{seed} c{counter} {room}: expected 3 cards, got {len(cards)}")
                        results["failed"] += 1
                    elif new_counter <= counter:
                        results["errors"].append(f"{seed} c{counter} {room}: counter didn't advance")
                        results["failed"] += 1
                    else:
                        results["passed"] += 1

                    if verbose:
                        print(f"\nSeed: {seed}, counter={counter}, room={room}")
                        print(f"  Cards: {', '.join(cards)}")
                        print(f"  Counter: {counter} -> {new_counter}")

        except Exception as e:
            results["errors"].append(f"{seed}: {e}")
            results["failed"] += 1

    return results


def test_boss_relics(seed_str: Optional[str] = None, verbose: bool = True) -> Dict[str, Any]:
    """Test boss relic predictions."""
    from core.state.rng import seed_to_long
    from core.comparison.full_rng_tracker import predict_boss_relics

    results = {
        "passed": 0,
        "failed": 0,
        "errors": [],
        "predictions": [],
    }

    test_seeds = [seed_str] if seed_str else ["ABC12", "XYZ99"]

    for seed in test_seeds:
        try:
            seed_long = seed_to_long(seed)

            for relics_taken in [0, 1, 2]:
                boss_relics = predict_boss_relics(
                    seed_long,
                    player_class="WATCHER",
                    has_starter_relic=True,
                    relics_already_taken=relics_taken,
                    already_owned_relics=["PureWater"],
                )

                prediction = {
                    "seed": seed,
                    "relics_taken": relics_taken,
                    "boss_relics": boss_relics,
                }
                results["predictions"].append(prediction)

                # Validate
                if not boss_relics or len(boss_relics) < 3:
                    results["errors"].append(f"{seed} taken={relics_taken}: expected 3 relics, got {len(boss_relics)}")
                    results["failed"] += 1
                else:
                    results["passed"] += 1

                if verbose:
                    print(f"\nSeed: {seed}, relics_taken={relics_taken}")
                    print(f"  Boss relics: {', '.join(boss_relics)}")

        except Exception as e:
            results["errors"].append(f"{seed}: {e}")
            results["failed"] += 1

    return results


def test_rng_accuracy(seed_str: Optional[str] = None, verbose: bool = True) -> Dict[str, Any]:
    """Test RNG accuracy calculation function."""
    from core.state.rng import seed_to_long, long_to_seed
    from web.server import calculate_rng_accuracy

    results = {
        "passed": 0,
        "failed": 0,
        "errors": [],
    }

    test_seed = seed_str or "ABC12"
    seed_long = seed_to_long(test_seed)
    seed_formatted = long_to_seed(seed_long)

    # Create mock save data with some card choices
    mock_save = {
        "seed": seed_long,
        "metric_card_choices": [
            {
                "floor": 1,
                "picked": "Eruption",
                "not_picked": ["Crescendo", "InnerPeace"],
            },
        ],
        "metric_path_per_floor": ["M"],
        "boss_relics": [],
        "relics": ["PureWater"],
        "relics_obtained": [],
    }

    try:
        accuracy = calculate_rng_accuracy(mock_save, seed_formatted, "WATCHER")

        if verbose:
            print(f"\nRNG Accuracy Test (seed: {test_seed})")
            print(f"  Overall: {accuracy['overall']['ratio']*100:.0f}%")
            print(f"  Cards: {accuracy['cards']['correct']}/{accuracy['cards']['total']}")
            if accuracy.get("mismatches"):
                print(f"  Mismatches: {len(accuracy['mismatches'])}")

        # Basic validation
        if "overall" not in accuracy:
            results["errors"].append("Missing 'overall' key")
            results["failed"] += 1
        else:
            results["passed"] += 1

    except Exception as e:
        results["errors"].append(f"calculate_rng_accuracy failed: {e}")
        results["failed"] += 1

    return results


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Test STS RNG predictions",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
    uv run scripts/dev/test_rng.py                  # Run all tests
    uv run scripts/dev/test_rng.py neow             # Just Neow tests
    uv run scripts/dev/test_rng.py --seed ABC12     # Test specific seed
"""
    )

    parser.add_argument("test", nargs="?", default="all",
                        choices=["all", "neow", "bosses", "cards", "relics", "accuracy"],
                        help="Which test to run")
    parser.add_argument("--seed", "-s", help="Specific seed to test")
    parser.add_argument("-q", "--quiet", action="store_true", help="Minimal output")

    args = parser.parse_args()
    verbose = not args.quiet

    print("=" * 50)
    print("STS RNG Prediction Tests")
    print("=" * 50)

    total_passed = 0
    total_failed = 0
    all_errors = []

    tests = {
        "neow": ("Neow Options", test_neow_predictions),
        "bosses": ("Boss Predictions", test_boss_predictions),
        "cards": ("Card Rewards", test_card_predictions),
        "relics": ("Boss Relics", test_boss_relics),
        "accuracy": ("RNG Accuracy", test_rng_accuracy),
    }

    if args.test == "all":
        tests_to_run = tests
    else:
        tests_to_run = {args.test: tests[args.test]}

    for test_key, (test_name, test_func) in tests_to_run.items():
        print(f"\n{'='*50}")
        print(f"TEST: {test_name}")
        print("=" * 50)

        results = test_func(args.seed, verbose)

        total_passed += results["passed"]
        total_failed += results["failed"]
        all_errors.extend(results["errors"])

        print(f"\n  Passed: {results['passed']}, Failed: {results['failed']}")

    # Summary
    print("\n" + "=" * 50)
    print("SUMMARY")
    print("=" * 50)
    print(f"Total: {total_passed} passed, {total_failed} failed")

    if all_errors:
        print(f"\nErrors ({len(all_errors)}):")
        for err in all_errors:
            print(f"  - {err}")

    if total_failed == 0:
        print("\nAll tests passed!")
        return 0
    else:
        print(f"\n{total_failed} tests failed!")
        return 1


if __name__ == "__main__":
    sys.exit(main())
