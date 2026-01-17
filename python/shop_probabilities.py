"""
Shop and Card Removal Probability Calculations for Slay the Spire

Models shop offerings, card removal costs, and deck purity optimization.
Key insight from Lifecoach: deck thinning to 5-6 cards enables infinite combos.
"""

import numpy as np
from dataclasses import dataclass, field
from typing import List, Dict, Optional, Tuple
from enum import Enum

# ============ SHOP CONSTANTS ============

# Base gold costs
CARD_REMOVAL_BASE_COST = 75
CARD_REMOVAL_SCALING = 25  # +25 per removal

# Shop card costs by rarity (Watcher)
CARD_COSTS = {
    "COMMON": {"base": 45, "range": (45, 55)},      # 45-55
    "UNCOMMON": {"base": 75, "range": (68, 82)},    # ~75
    "RARE": {"base": 150, "range": (135, 165)},     # ~150
}

# Shop relic costs by rarity
RELIC_COSTS = {
    "COMMON": {"base": 150, "range": (143, 157)},
    "UNCOMMON": {"base": 250, "range": (238, 263)},
    "RARE": {"base": 300, "range": (285, 315)},
    "SHOP": {"base": 150, "range": (143, 157)},
}

# Potion costs
POTION_COSTS = {
    "COMMON": {"base": 50, "range": (48, 52)},
    "UNCOMMON": {"base": 75, "range": (71, 79)},
    "RARE": {"base": 100, "range": (95, 105)},
}

# Shop structure
SHOP_CARDS = 7        # 5 character cards + 2 colorless
SHOP_RELICS = 3       # 2 shop relics + 1 random
SHOP_POTIONS = 3
SHOP_REMOVAL = 1

# Colorless card distribution
COLORLESS_DISTRIBUTION = {
    "COMMON": 0.5,
    "UNCOMMON": 0.35,
    "RARE": 0.15,
}

# ============ DATA CLASSES ============

@dataclass
class ShopState:
    """Current shop offering state."""
    cards: List[Dict] = field(default_factory=list)
    relics: List[Dict] = field(default_factory=list)
    potions: List[Dict] = field(default_factory=list)
    removal_cost: int = 75
    player_gold: int = 0

@dataclass
class DeckState:
    """Deck composition for removal planning."""
    total_cards: int
    strikes: int
    defends: int
    curses: int = 0
    core_cards: int = 0  # Cards essential for combo
    filler_cards: int = 0  # Non-essential cards
    removals_done: int = 0

# ============ CARD REMOVAL CALCULATIONS ============

def get_removal_cost(removals_done: int, has_smiling_mask: bool = False) -> int:
    """Calculate current card removal cost."""
    if has_smiling_mask:
        return 50  # Smiling Mask fixes cost at 50
    return CARD_REMOVAL_BASE_COST + (removals_done * CARD_REMOVAL_SCALING)

def get_total_removal_cost(
    removals_needed: int,
    removals_done: int = 0,
    has_smiling_mask: bool = False
) -> int:
    """Calculate total gold needed for N removals."""
    if has_smiling_mask:
        return removals_needed * 50

    total = 0
    for i in range(removals_needed):
        total += CARD_REMOVAL_BASE_COST + ((removals_done + i) * CARD_REMOVAL_SCALING)
    return total

def estimate_shops_for_purity(
    deck: DeckState,
    target_size: int = 6,
    avg_gold_per_floor: int = 25,
    has_smiling_mask: bool = False,
    has_membership_card: bool = False
) -> Dict[str, any]:
    """
    Estimate shops/floors needed to reach target deck size.

    Returns analysis of deck purity path.
    """
    current_size = deck.total_cards
    removals_needed = current_size - target_size

    if removals_needed <= 0:
        return {
            "removals_needed": 0,
            "gold_needed": 0,
            "shops_needed": 0,
            "already_pure": True,
        }

    # Calculate gold needed
    total_gold = get_total_removal_cost(
        removals_needed,
        deck.removals_done,
        has_smiling_mask
    )

    # Membership Card gives 50% off
    if has_membership_card:
        total_gold = int(total_gold * 0.5)

    # Estimate floors to accumulate gold (rough)
    # Shops appear ~every 12 floors, average gold income varies
    floors_for_gold = total_gold / avg_gold_per_floor
    shops_needed = max(removals_needed, int(np.ceil(floors_for_gold / 12)))

    # Priority order for removal
    removal_priority = []
    if deck.curses > 0:
        removal_priority.extend(["Curse"] * deck.curses)
    removal_priority.extend(["Strike"] * deck.strikes)
    removal_priority.extend(["Defend"] * deck.defends)
    removal_priority.extend(["Filler"] * deck.filler_cards)

    return {
        "removals_needed": removals_needed,
        "gold_needed": total_gold,
        "shops_needed": shops_needed,
        "floors_estimated": shops_needed * 12,
        "removal_priority": removal_priority[:removals_needed],
        "already_pure": False,
        "target_deck": target_size,
        "current_deck": current_size,
    }

# ============ SHOP OFFERING PROBABILITIES ============

def get_shop_card_probabilities(
    cards_seen: List[str],
    cards_in_deck: List[str]
) -> Dict[str, float]:
    """
    Calculate probability of specific cards appearing in shop.

    Shop excludes cards already seen and duplicates in deck.
    """
    # Simplified model - actual game has complex weighting
    # Cards seen this run are excluded from shop pool
    # Basic/starter cards never appear in shop

    # This would need the full card pool to be accurate
    # For now, return uniform probability over remaining pool
    pass

def calculate_shop_ev(
    shop: ShopState,
    deck: DeckState,
    current_gold: int,
    infinite_combo_missing: List[str]
) -> Dict[str, float]:
    """
    Calculate EV of each shop purchase option.

    Priority: Infinite combo pieces > Card removal > Block cards > Other
    """
    ev_scores = {}

    # Card removal EV
    if shop.removal_cost <= current_gold and deck.total_cards > 6:
        # High EV for deck thinning toward infinite
        removal_ev = 10.0 - (deck.total_cards - 6) * 0.5
        if deck.strikes > 0:
            removal_ev += 2.0  # Extra value for removing Strikes
        ev_scores["removal"] = removal_ev

    # Card purchase EV
    for card in shop.cards:
        card_id = card.get("id", "")
        card_cost = card.get("cost", 0)

        if card_cost > current_gold:
            continue

        base_ev = 0.0

        # Check if card completes infinite
        if card_id in infinite_combo_missing:
            base_ev += 15.0  # Very high priority

        # General card quality (would use tier list)
        # Simplified here
        ev_scores[f"card_{card_id}"] = base_ev

    return ev_scores

# ============ INFINITE DECK ANALYSIS ============

def analyze_infinite_purity(
    deck_cards: List[str],
    required_cards: List[str]
) -> Dict[str, any]:
    """
    Analyze how close deck is to infinite-capable purity.

    Lifecoach target: 5-6 card deck with:
    - Rushdown
    - Eruption+
    - Inner Peace or equivalent
    - Optionally Flurry of Blows
    """
    has_required = {card: (card in deck_cards) for card in required_cards}
    missing = [card for card in required_cards if card not in deck_cards]

    excess_cards = len(deck_cards) - len(required_cards)
    excess_list = [c for c in deck_cards if c not in required_cards]

    # Check for dead cards (Strikes, Defends, Curses)
    dead_cards = [c for c in deck_cards if c in ["Strike", "Strike_P", "Defend", "Defend_P"] or "Curse" in c]

    return {
        "is_pure": len(deck_cards) <= 6 and len(missing) == 0,
        "deck_size": len(deck_cards),
        "target_size": max(len(required_cards), 5),
        "has_required": has_required,
        "missing_cards": missing,
        "excess_cards": excess_cards,
        "excess_list": excess_list,
        "dead_cards": dead_cards,
        "dead_card_count": len(dead_cards),
        "removals_to_pure": max(0, len(deck_cards) - max(len(required_cards), 5)),
    }

# ============ GOLD OPTIMIZATION ============

def optimize_gold_usage(
    gold: int,
    deck: DeckState,
    shop: ShopState,
    missing_combo_cards: List[str],
    has_membership_card: bool = False
) -> List[Dict]:
    """
    Optimize gold usage in shop for maximum infinite potential.

    Priority order:
    1. Buy missing infinite combo pieces
    2. Remove dead cards (Strikes first)
    3. Buy strong synergy cards
    4. Save for future shops
    """
    actions = []
    remaining_gold = gold

    # Discount for Membership Card
    discount = 0.5 if has_membership_card else 1.0

    # 1. Check for missing combo pieces in shop
    for card in shop.cards:
        card_id = card.get("id", "")
        card_cost = int(card.get("cost", 0) * discount)

        if card_id in missing_combo_cards and card_cost <= remaining_gold:
            actions.append({
                "action": "buy_card",
                "card": card_id,
                "cost": card_cost,
                "priority": "CRITICAL",
                "reason": "Missing infinite combo piece",
            })
            remaining_gold -= card_cost
            missing_combo_cards.remove(card_id)

    # 2. Card removal if deck > target
    removal_cost = int(shop.removal_cost * discount)
    while deck.total_cards > 6 and removal_cost <= remaining_gold:
        # Pick worst card
        if deck.strikes > 0:
            target = "Strike"
            deck.strikes -= 1
        elif deck.defends > 0:
            target = "Defend"
            deck.defends -= 1
        elif deck.filler_cards > 0:
            target = "Filler"
            deck.filler_cards -= 1
        else:
            break

        actions.append({
            "action": "remove_card",
            "card": target,
            "cost": removal_cost,
            "priority": "HIGH",
            "reason": "Deck purity for infinite",
        })
        remaining_gold -= removal_cost
        deck.total_cards -= 1
        deck.removals_done += 1

        # Update removal cost
        removal_cost = int(get_removal_cost(deck.removals_done) * discount)

    # 3. Save recommendation
    if remaining_gold > 0:
        actions.append({
            "action": "save",
            "gold": remaining_gold,
            "priority": "LOW",
            "reason": "Reserve for future shops/events",
        })

    return actions

# ============ SHOP FREQUENCY ANALYSIS ============

def estimate_shop_visits(act: int, path_type: str = "balanced") -> Dict[str, any]:
    """
    Estimate shop visits per act based on pathing.

    path_type: "aggressive" (elites), "balanced", "conservative" (shops)
    """
    # Standard map has ~15-17 floors per act
    # Shops typically at floors 8-9 and 16-17 (before boss)

    shops_per_act = {
        "aggressive": 1,     # Minimal shops, focus elites
        "balanced": 2,       # Standard 2 shops
        "conservative": 2,   # Also 2, but prioritize hitting them
    }

    base_shops = shops_per_act.get(path_type, 2)

    # Act 1: Usually 2 shops
    # Act 2: Usually 2 shops
    # Act 3: Usually 2 shops + final shop before Heart
    # Act 4: 0-1 shop (if map allows)

    act_shops = {
        1: base_shops,
        2: base_shops,
        3: base_shops + 1,  # Extra before Heart
        4: 0,
    }

    return {
        "act": act,
        "path_type": path_type,
        "expected_shops": act_shops.get(act, 2),
        "total_run_shops": sum(act_shops.values()),
        "total_removal_opportunities": sum(act_shops.values()),
    }

# ============ TESTING ============

if __name__ == "__main__":
    print("=== Shop Probability Analysis ===\n")

    # Test deck purity analysis
    deck = DeckState(
        total_cards=15,
        strikes=4,
        defends=4,
        curses=0,
        core_cards=3,  # Rushdown, Eruption, Inner Peace
        filler_cards=4,
        removals_done=0
    )

    print("Starting deck analysis:")
    print(f"  Total cards: {deck.total_cards}")
    print(f"  Strikes: {deck.strikes}, Defends: {deck.defends}")

    # Calculate path to purity
    purity = estimate_shops_for_purity(deck, target_size=6)
    print(f"\nPath to 6-card infinite deck:")
    print(f"  Removals needed: {purity['removals_needed']}")
    print(f"  Gold needed: {purity['gold_needed']}")
    print(f"  Shops needed: {purity['shops_needed']}")
    print(f"  Removal priority: {purity['removal_priority'][:5]}...")

    # Test removal costs
    print("\n=== Removal Cost Scaling ===")
    for i in range(10):
        cost = get_removal_cost(i)
        print(f"  Removal #{i+1}: {cost} gold")

    # Test with Smiling Mask
    print("\n=== With Smiling Mask ===")
    purity_mask = estimate_shops_for_purity(deck, target_size=6, has_smiling_mask=True)
    print(f"  Gold needed: {purity_mask['gold_needed']} (vs {purity['gold_needed']} without)")

    # Test with Membership Card
    print("\n=== With Membership Card ===")
    purity_member = estimate_shops_for_purity(deck, target_size=6, has_membership_card=True)
    print(f"  Gold needed: {purity_member['gold_needed']} (50% off)")

    # Analyze specific infinite deck
    print("\n=== Infinite Deck Analysis ===")
    current_deck = [
        "Strike_P", "Strike_P", "Strike_P", "Strike_P",
        "Defend_P", "Defend_P", "Defend_P", "Defend_P",
        "Eruption", "Vigilance",
        "Rushdown", "InnerPeace", "FlurryOfBlows"
    ]
    required = ["Rushdown", "Eruption", "InnerPeace"]

    analysis = analyze_infinite_purity(current_deck, required)
    print(f"  Deck size: {analysis['deck_size']}")
    print(f"  Has required: {analysis['has_required']}")
    print(f"  Dead cards: {analysis['dead_card_count']}")
    print(f"  Removals to pure: {analysis['removals_to_pure']}")
