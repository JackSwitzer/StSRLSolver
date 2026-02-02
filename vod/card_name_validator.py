#!/usr/bin/env python3
"""
Card Name Validation and Correction for VOD Extraction.

Handles:
1. Normalizing card names (case, spacing, special characters)
2. Fuzzy matching to correct OCR errors
3. Mapping between display names and Java IDs
4. Validating cards are in the correct pool
"""

from typing import Optional, List, Tuple, Dict
from difflib import SequenceMatcher

# All valid Watcher card display names (what players see)
# Maps display name -> Java ID
WATCHER_DISPLAY_TO_ID = {
    # Basic cards (NOT in reward pools - for reference only)
    # "Strike": "Strike_P",  # Basic - not in rewards
    # "Defend": "Defend_P",  # Basic - not in rewards
    # "Eruption": "Eruption",  # Basic - not in rewards
    # "Vigilance": "Vigilance",  # Basic - not in rewards
    # "Miracle": "Miracle",  # Special - from Calm

    # Common cards
    "Bowling Bash": "BowlingBash",
    "Cut Through Fate": "CutThroughFate",
    "Empty Fist": "EmptyFist",
    "Flurry of Blows": "FlurryOfBlows",
    "Flying Sleeves": "FlyingSleeves",
    "Follow-Up": "FollowUp",
    "Follow Up": "FollowUp",
    "Halt": "Halt",
    "Just Lucky": "JustLucky",
    "Pressure Points": "PathToVictory",
    "Sash Whip": "SashWhip",
    "Tranquility": "ClearTheMind",
    "Crescendo": "Crescendo",
    "Consecrate": "Consecrate",
    "Crush Joints": "CrushJoints",
    "Empty Body": "EmptyBody",
    "Evaluate": "Evaluate",
    "Protect": "Protect",
    "Third Eye": "ThirdEye",
    "Prostrate": "Prostrate",

    # Uncommon cards
    "Tantrum": "Tantrum",
    "Fear No Evil": "FearNoEvil",
    "Reach Heaven": "ReachHeaven",
    "Sands of Time": "SandsOfTime",
    "Signature Move": "SignatureMove",
    "Talk to the Hand": "TalkToTheHand",
    "Wallop": "Wallop",
    "Weave": "Weave",
    "Wheel Kick": "WheelKick",
    "Windmill Strike": "WindmillStrike",
    "Conclude": "Conclude",
    "Carve Reality": "CarveReality",
    "Collect": "Collect",
    "Deceive Reality": "DeceiveReality",
    "Foresight": "Wireheading",
    "Indignation": "Indignation",
    "Meditate": "Meditate",
    "Perseverance": "Perseverance",
    "Pray": "Pray",
    "Sanctity": "Sanctity",
    "Swivel": "Swivel",
    "Simmering Fury": "Vengeance",
    "Wave of the Hand": "WaveOfTheHand",
    "Worship": "Worship",
    "Wreath of Flame": "WreathOfFlame",
    "Battle Hymn": "BattleHymn",
    "Establishment": "Establishment",
    "Like Water": "LikeWater",
    "Mental Fortress": "MentalFortress",
    "Nirvana": "Nirvana",
    "Rushdown": "Adaptation",
    "Study": "Study",
    "Empty Mind": "EmptyMind",
    "Inner Peace": "InnerPeace",
    "Fasting": "Fasting2",
    "Foreign Influence": "ForeignInfluence",

    # Rare cards
    "Brilliance": "Brilliance",
    "Alpha": "Alpha",
    "Blasphemy": "Blasphemy",
    "Conjure Blade": "ConjureBlade",
    "Deus Ex Machina": "DeusExMachina",
    "Deva Form": "DevaForm",
    "Devotion": "Devotion",
    "Judgement": "Judgement",
    "Judgment": "Judgement",  # Alternate spelling
    "Lesson Learned": "LessonLearned",
    "Master Reality": "MasterReality",
    "Omniscience": "Omniscience",
    "Ragnarok": "Ragnarok",
    "Scrawl": "Scrawl",
    "Spirit Shield": "SpiritShield",
    "Vault": "Vault",
    "Wish": "Wish",
}

# Reverse mapping: Java ID -> display name
WATCHER_ID_TO_DISPLAY = {v: k for k, v in WATCHER_DISPLAY_TO_ID.items()}

# All valid card names as a set for quick lookup
VALID_CARD_NAMES = set(WATCHER_DISPLAY_TO_ID.keys())

# Cards that CANNOT appear in rewards (basics, specials)
INVALID_IN_REWARDS = {
    "Strike", "Strike_P",
    "Defend", "Defend_P",
    "Eruption",
    "Vigilance",
    "Miracle",
    "Ascender's Bane",  # Curse
}

# Common OCR error patterns
# Maps commonly misread text -> correct card name
OCR_CORRECTIONS = {
    # "Defend" is a basic card and CANNOT appear in rewards
    # Most likely Gemini misread "Protect" as "Defend"
    "Defend": "Protect",
    "Defend_P": "Protect",

    # Common misreadings
    "FollowUp": "Follow-Up",
    "Follow up": "Follow-Up",
    "Follow-up": "Follow-Up",
    "Talk To The Hand": "Talk to the Hand",
    "talk to the hand": "Talk to the Hand",
    "talktothehand": "Talk to the Hand",
    "Wreath Of Flame": "Wreath of Flame",
    "WreathOfFlame": "Wreath of Flame",
    "Reach heaven": "Reach Heaven",
    "ReachHeaven": "Reach Heaven",
    "Cutthrough fate": "Cut Through Fate",
    "Cut through fate": "Cut Through Fate",
    "CutThroughFate": "Cut Through Fate",
    "Flying sleeves": "Flying Sleeves",
    "FlyingSleeves": "Flying Sleeves",
    "Flurry Of Blows": "Flurry of Blows",
    "FlurryOfBlows": "Flurry of Blows",
    "Wheel kick": "Wheel Kick",
    "WheelKick": "Wheel Kick",
    "WindmillStrike": "Windmill Strike",
    "Windmill strike": "Windmill Strike",
    "BowlingBash": "Bowling Bash",
    "EmptyFist": "Empty Fist",
    "EmptyBody": "Empty Body",
    "EmptyMind": "Empty Mind",
    "ThirdEye": "Third Eye",
    "InnerPeace": "Inner Peace",
    "MentalFortress": "Mental Fortress",
    "SpiritShield": "Spirit Shield",
    "DevaForm": "Deva Form",
    "FearNoEvil": "Fear No Evil",
    "JustLucky": "Just Lucky",
    "SashWhip": "Sash Whip",
    "CrushJoints": "Crush Joints",
    "DecieveReality": "Deceive Reality",  # Typo
    "CarveReality": "Carve Reality",
    "DeceiveReality": "Deceive Reality",
    "SignatureMove": "Signature Move",
    "SandsOfTime": "Sands of Time",
    "SimmeringFury": "Simmering Fury",
    "WaveOfTheHand": "Wave of the Hand",
    "BattleHymn": "Battle Hymn",
    "LikeWater": "Like Water",
    "PressurePoints": "Pressure Points",
    "ConjureBlade": "Conjure Blade",
    "DeusExMachina": "Deus Ex Machina",
    "LessonLearned": "Lesson Learned",
    "MasterReality": "Master Reality",
    "ForeignInfluence": "Foreign Influence",
}


def normalize_card_name(name: str) -> str:
    """
    Normalize a card name for comparison.

    - Strip whitespace
    - Remove '+' upgrade markers
    - Consistent casing
    """
    name = name.strip()
    name = name.replace("+", "")
    name = name.strip()
    return name


def validate_card_name(name: str) -> Tuple[bool, Optional[str], float]:
    """
    Validate and correct a card name.

    Returns:
        (is_valid, corrected_name, confidence)
        - is_valid: True if card is valid/correctable
        - corrected_name: The correct card name or None
        - confidence: 0.0-1.0 confidence in the correction
    """
    normalized = normalize_card_name(name)

    # Direct match
    if normalized in VALID_CARD_NAMES:
        return True, normalized, 1.0

    # Try OCR corrections
    if normalized in OCR_CORRECTIONS:
        corrected = OCR_CORRECTIONS[normalized]
        return True, corrected, 0.95

    # Case-insensitive match
    normalized_lower = normalized.lower()
    for valid_name in VALID_CARD_NAMES:
        if valid_name.lower() == normalized_lower:
            return True, valid_name, 0.98

    # Fuzzy match
    best_match = None
    best_score = 0.0
    for valid_name in VALID_CARD_NAMES:
        score = SequenceMatcher(None, normalized_lower, valid_name.lower()).ratio()
        if score > best_score:
            best_score = score
            best_match = valid_name

    # Accept fuzzy matches with > 70% similarity
    if best_score >= 0.70:
        return True, best_match, best_score

    # No good match found
    return False, None, 0.0


def correct_card_list(cards: List[str]) -> Tuple[List[str], List[str], float]:
    """
    Correct a list of card names from extraction.

    Returns:
        (corrected_cards, invalid_cards, overall_confidence)
    """
    corrected = []
    invalid = []
    confidences = []

    for card in cards:
        is_valid, corrected_name, confidence = validate_card_name(card)
        if is_valid:
            corrected.append(corrected_name)
            confidences.append(confidence)
        else:
            invalid.append(card)

    overall_conf = sum(confidences) / len(confidences) if confidences else 0.0
    return corrected, invalid, overall_conf


def get_java_id(display_name: str) -> Optional[str]:
    """Get Java card ID from display name."""
    return WATCHER_DISPLAY_TO_ID.get(display_name)


def get_display_name(java_id: str) -> Optional[str]:
    """Get display name from Java card ID."""
    return WATCHER_ID_TO_DISPLAY.get(java_id)


def validate_for_rewards(name: str) -> Tuple[bool, Optional[str], float, str]:
    """
    Validate a card name specifically for reward contexts.

    Returns:
        (is_valid, corrected_name, confidence, issue)
        - issue: Description of any problem detected
    """
    normalized = normalize_card_name(name)

    # Check if it's a card that can't appear in rewards
    if normalized in INVALID_IN_REWARDS or normalized.lower() in [n.lower() for n in INVALID_IN_REWARDS]:
        # Try to correct it
        if normalized in OCR_CORRECTIONS:
            corrected = OCR_CORRECTIONS[normalized]
            return True, corrected, 0.85, f"Corrected invalid reward card '{normalized}'"
        return False, None, 0.0, f"'{normalized}' cannot appear in card rewards (basic/special card)"

    # Normal validation
    is_valid, corrected, conf = validate_card_name(name)
    if is_valid:
        return True, corrected, conf, ""
    return False, None, 0.0, f"'{normalized}' is not a valid Watcher card"


def detect_extraction_issues(extractions: List[dict]) -> List[dict]:
    """
    Detect potential extraction issues like duplicate card rewards.

    Returns list of issues found.
    """
    issues = []
    card_reward_sets = {}  # floor -> set of cards

    for ext in extractions:
        if ext.get("type") == "card_reward":
            floor = ext.get("floor", 0)
            cards = tuple(sorted(ext.get("cards_offered", [])))

            # Check for exact duplicates
            for prev_floor, prev_cards in card_reward_sets.items():
                if cards == prev_cards:
                    issues.append({
                        "type": "duplicate_cards",
                        "floor": floor,
                        "previous_floor": prev_floor,
                        "cards": list(cards),
                        "message": f"Floor {floor} has EXACT same cards as floor {prev_floor} - likely extraction error"
                    })

            card_reward_sets[floor] = cards

    return issues


# =============================================================================
# TESTING
# =============================================================================

if __name__ == "__main__":
    print("=== Card Name Validator Tests ===\n")

    test_cases = [
        # Correct names
        "Protect",
        "Wreath of Flame",
        "Pray",

        # OCR errors from extraction
        "Defend",  # Should correct to Protect
        "Reach Heaven",
        "WreathOfFlame",  # Java ID format

        # Fuzzy matches
        "Protec",  # Typo
        "Proctect",  # Typo
        "Flying Sleves",  # Typo

        # Invalid
        "Fireball",  # Not a Watcher card
        "Backstab",  # Silent card
    ]

    for name in test_cases:
        is_valid, corrected, conf = validate_card_name(name)
        if is_valid:
            print(f"'{name}' -> '{corrected}' (conf: {conf:.2f})")
        else:
            print(f"'{name}' -> INVALID")

    print("\n--- Testing card list from extraction ---")
    extracted = ["Defend", "Reach Heaven", "Pray"]
    corrected, invalid, conf = correct_card_list(extracted)
    print(f"Input: {extracted}")
    print(f"Corrected: {corrected}")
    print(f"Invalid: {invalid}")
    print(f"Confidence: {conf:.2f}")

    print("\n--- Testing floor 14 duplicate detection ---")
    floor3_cards = ["Consecrate", "Fasting", "Pressure Points"]
    floor14_cards = ["Consecrate", "Fasting", "Pressure Points"]
    print(f"Floor 3: {floor3_cards}")
    print(f"Floor 14: {floor14_cards}")
    print(f"Exact duplicate: {floor3_cards == floor14_cards}")
