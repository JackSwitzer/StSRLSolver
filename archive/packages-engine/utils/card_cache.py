"""Pre-computed card filters for efficient lookups."""
from typing import List, Optional

# Lazy import to avoid circular dependencies
_card_cache = None

def _get_all_cards():
    global _card_cache
    if _card_cache is None:
        from ..content.cards import ALL_CARDS, CardType, CardColor
        _card_cache = {
            'ALL_CARDS': ALL_CARDS,
            'CardType': CardType,
            'CardColor': CardColor,
        }
    return _card_cache

def get_cards_by_type(card_type, colors: Optional[List] = None) -> List[str]:
    """Filter cards by type and optional colors."""
    cache = _get_all_cards()
    ALL_CARDS = cache['ALL_CARDS']
    result = []
    for cid, card in ALL_CARDS.items():
        if card_type is not None and card.card_type != card_type:
            continue
        if colors and card.color not in colors:
            continue
        result.append(cid)
    return result

# Lazy-loaded pre-computed filters
def get_watcher_attacks():
    cache = _get_all_cards()
    return get_cards_by_type(cache['CardType'].ATTACK, [cache['CardColor'].PURPLE, cache['CardColor'].COLORLESS])

def get_watcher_skills():
    cache = _get_all_cards()
    return get_cards_by_type(cache['CardType'].SKILL, [cache['CardColor'].PURPLE, cache['CardColor'].COLORLESS])

def get_watcher_powers():
    cache = _get_all_cards()
    return get_cards_by_type(cache['CardType'].POWER, [cache['CardColor'].PURPLE, cache['CardColor'].COLORLESS])

def get_colorless_cards():
    cache = _get_all_cards()
    return get_cards_by_type(None, [cache['CardColor'].COLORLESS])

def get_curse_cards():
    cache = _get_all_cards()
    return [cid for cid, c in cache['ALL_CARDS'].items() if c.card_type == cache['CardType'].CURSE]
