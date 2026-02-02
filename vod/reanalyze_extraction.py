#!/usr/bin/env python3
"""
Re-analyze an existing extraction with card name correction.

This script applies the card name validator to an existing extraction JSON
and recalculates match rates without needing to re-run the video extraction.
"""

import json
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from vod.card_name_validator import (
    correct_card_list,
    detect_extraction_issues,
    normalize_card_name,
)


def reanalyze_extraction(input_path: str, output_path: str = None):
    """
    Re-analyze an extraction with card name correction.

    Args:
        input_path: Path to existing extraction JSON
        output_path: Path to save re-analyzed results (optional)
    """
    print(f"Re-analyzing extraction: {input_path}")
    print("=" * 60)

    with open(input_path) as f:
        data = json.load(f)

    extractions = data.get("extractions", [])
    rng_history = data.get("rng_history", [])

    # Re-analyze card rewards
    card_rewards = [e for e in extractions if e.get("type") == "card_reward"]
    print(f"\nTotal card rewards: {len(card_rewards)}")

    matches = 0
    mismatches = 0
    corrected_matches = 0
    corrections_made = 0

    for i, ext in enumerate(card_rewards):
        observed = ext.get("cards_offered", [])
        predicted = ext.get("predicted_options", [])
        floor = ext.get("floor", "?")

        # Apply correction
        corrected, invalid, confidence = correct_card_list(observed)

        # Check if correction changed anything
        if corrected != observed:
            corrections_made += 1
            ext["cards_corrected"] = corrected
            ext["correction_confidence"] = confidence

        # Normalize for comparison
        pred_norm = set(normalize_card_name(c) for c in predicted)
        obs_norm = set(normalize_card_name(c) for c in observed)
        corr_norm = set(normalize_card_name(c) for c in corrected)

        # Calculate match scores
        original_score = len(pred_norm & obs_norm) / len(pred_norm | obs_norm) if pred_norm | obs_norm else 0
        corrected_score = len(pred_norm & corr_norm) / len(pred_norm | corr_norm) if pred_norm | corr_norm else 0

        ext["original_match_score"] = original_score
        ext["corrected_match_score"] = corrected_score

        # Count matches (using 50% threshold)
        original_match = original_score >= 0.5
        corrected_match = corrected_score >= 0.5

        if original_match:
            matches += 1
        else:
            mismatches += 1

        if corrected_match:
            corrected_matches += 1

        # Print differences
        if corrected_match and not original_match:
            print(f"\n✓ Floor {floor}: IMPROVED by correction")
            print(f"  Predicted: {predicted}")
            print(f"  Observed:  {observed}")
            print(f"  Corrected: {corrected}")
            print(f"  Score: {original_score:.2f} -> {corrected_score:.2f}")
        elif not corrected_match:
            print(f"\n✗ Floor {floor}: Still mismatch")
            print(f"  Predicted: {predicted}")
            print(f"  Observed:  {observed}")
            if corrected != observed:
                print(f"  Corrected: {corrected}")
            print(f"  Score: {original_score:.2f} -> {corrected_score:.2f}")

    # Detect extraction issues
    issues = detect_extraction_issues(extractions)

    # Calculate new rates
    original_rate = matches / len(card_rewards) * 100 if card_rewards else 0
    corrected_rate = corrected_matches / len(card_rewards) * 100 if card_rewards else 0

    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print(f"Card rewards: {len(card_rewards)}")
    print(f"Original matches: {matches} ({original_rate:.1f}%)")
    print(f"After correction: {corrected_matches} ({corrected_rate:.1f}%)")
    print(f"Corrections made: {corrections_made}")
    print(f"Improvement: +{corrected_matches - matches} matches (+{corrected_rate - original_rate:.1f}%)")

    if issues:
        print(f"\n⚠️  Extraction issues detected: {len(issues)}")
        for issue in issues:
            print(f"  - {issue.get('message', str(issue))}")

    # Save updated results
    if output_path:
        data["original_match_rate"] = f"{original_rate:.1f}%"
        data["corrected_match_rate"] = f"{corrected_rate:.1f}%"
        data["corrections_made"] = corrections_made
        data["extraction_issues"] = issues

        with open(output_path, "w") as f:
            json.dump(data, f, indent=2)
        print(f"\nSaved to: {output_path}")

    return {
        "original_rate": original_rate,
        "corrected_rate": corrected_rate,
        "improvements": corrected_matches - matches,
        "issues": issues,
    }


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Re-analyze extraction with card correction")
    parser.add_argument("input", help="Input extraction JSON")
    parser.add_argument("-o", "--output", help="Output path (optional)")

    args = parser.parse_args()

    output = args.output or args.input.replace(".json", "_reanalyzed.json")
    reanalyze_extraction(args.input, output)
