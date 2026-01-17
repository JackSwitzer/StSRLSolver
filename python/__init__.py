"""
Slay the Spire RL - Python EV and Mechanics Module

Modules:
- ev_calculator: Core damage/EV calculations
- watcher_mechanics: Comprehensive Watcher card/stance/combo abstractions
- shop_probabilities: Shop optimization and deck purity analysis
"""

from .ev_calculator import (
    Stance,
    PlayerState,
    MonsterState,
    CardState,
    CombatState,
    calculate_player_attack_damage,
    calculate_incoming_damage,
    calculate_total_incoming_damage,
    calculate_turns_to_kill,
    calculate_full_ev_report,
    should_enter_wrath,
    can_lethal_this_turn,
)

from .watcher_mechanics import (
    WATCHER_CARDS,
    WATCHER_RELICS,
    WatcherCard,
    WatcherRelic,
    CardArchetype,
    StanceState,
    can_go_infinite,
    score_deck_archetype,
    get_primary_archetype,
    score_card_for_deck,
    analyze_stance_balance,
    process_stance_change,
    add_mantra,
    evaluate_blasphemy_play,
)

from .shop_probabilities import (
    DeckState,
    ShopState,
    get_removal_cost,
    get_total_removal_cost,
    estimate_shops_for_purity,
    analyze_infinite_purity,
    optimize_gold_usage,
)

__all__ = [
    # EV Calculator
    "Stance", "PlayerState", "MonsterState", "CardState", "CombatState",
    "calculate_player_attack_damage", "calculate_incoming_damage",
    "calculate_total_incoming_damage", "calculate_turns_to_kill",
    "calculate_full_ev_report", "should_enter_wrath", "can_lethal_this_turn",

    # Watcher Mechanics
    "WATCHER_CARDS", "WATCHER_RELICS", "WatcherCard", "WatcherRelic",
    "CardArchetype", "StanceState", "can_go_infinite", "score_deck_archetype",
    "get_primary_archetype", "score_card_for_deck", "analyze_stance_balance",
    "process_stance_change", "add_mantra", "evaluate_blasphemy_play",

    # Shop Probabilities
    "DeckState", "ShopState", "get_removal_cost", "get_total_removal_cost",
    "estimate_shops_for_purity", "analyze_infinite_purity", "optimize_gold_usage",
]
