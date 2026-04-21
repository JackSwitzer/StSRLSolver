"""Manual Act 1 imports for the fixed Watcher validation seed suite."""

from __future__ import annotations

from dataclasses import asdict, dataclass
from typing import Any

from .corpus import WATCHER_STARTER_DECK
from .entity_catalog import (
    canonicalize_potion_id,
    canonicalize_relic_id,
    canonicalize_watcher_card_id,
)


def _normalize_card(name: str) -> str:
    return canonicalize_watcher_card_id(name)


def _normalize_relic(name: str) -> str:
    return canonicalize_relic_id(name)


def _normalize_potion(name: str) -> str:
    return canonicalize_potion_id(name)


@dataclass(frozen=True)
class ImportedAct1Floor:
    floor: int
    room_kind: str
    summary: str
    post_hp: int
    max_hp: int
    post_gold: int
    encounter: str | None = None
    damage_taken: int = 0
    turns: int | None = None
    option_taken: str | None = None
    picked_card: str | None = None
    skipped_cards: tuple[str, ...] = ()
    relics_gained: tuple[str, ...] = ()
    relics_skipped: tuple[str, ...] = ()
    cards_removed: tuple[str, ...] = ()
    upgraded_cards: tuple[str, ...] = ()
    potions_gained: tuple[str, ...] = ()
    potions_used: tuple[str, ...] = ()
    potions_discarded: tuple[str, ...] = ()
    max_hp_lost: int = 0
    max_hp_gained: int = 0
    gold_lost: int = 0
    healed: int = 0

    @property
    def is_combat(self) -> bool:
        return self.encounter is not None

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True)
class ImportedAct1Script:
    label: str
    seed: str
    source_url: str
    source_ascension: int
    eval_ascension: int
    exact_available: bool
    exact_issue: str | None = None
    neow_choice: str | None = None
    starting_max_hp: int | None = None
    starting_current_hp: int | None = None
    starting_gold: int = 99
    starting_relics: tuple[str, ...] = ("PureWater",)
    starting_removed_cards: tuple[str, ...] = ()
    starting_added_cards: tuple[str, ...] = ()
    floors: tuple[ImportedAct1Floor, ...] = ()

    def to_dict(self) -> dict[str, Any]:
        payload = asdict(self)
        payload["floors"] = [floor.to_dict() for floor in self.floors]
        return payload


@dataclass(frozen=True)
class ImportedCombatCase:
    seed_label: str
    seed: str
    source_url: str
    floor: int
    encounter: str
    room_kind: str
    current_hp: int
    max_hp: int
    gold: int
    deck: tuple[str, ...]
    relics: tuple[str, ...]
    potions: tuple[str, ...]
    notes: tuple[str, ...] = ()

    def to_dict(self) -> dict[str, Any]:
        payload = asdict(self)
        payload["deck_size"] = len(self.deck)
        payload["relic_count"] = len(self.relics)
        payload["potion_count"] = len(self.potions)
        return payload


def _remove_cards(deck: list[str], cards: tuple[str, ...]) -> None:
    for card in cards:
        normalized = _normalize_card(card)
        if normalized in deck:
            deck.remove(normalized)


def _add_card(deck: list[str], card: str) -> None:
    deck.append(_normalize_card(card))


def _upgrade_card(deck: list[str], card: str) -> None:
    normalized = _normalize_card(card)
    upgraded = f"{normalized}+"
    for index, deck_card in enumerate(deck):
        if deck_card == normalized:
            deck[index] = upgraded
            return
    if upgraded not in deck:
        deck.append(upgraded)


def _consume_potion(potions: list[str], potion: str) -> None:
    normalized = _normalize_potion(potion)
    if normalized in potions:
        potions.remove(normalized)


def _gain_potion(potions: list[str], potion: str, *, slots: int) -> None:
    normalized = _normalize_potion(potion)
    if len(potions) < slots:
        potions.append(normalized)


def _starting_deck(script: ImportedAct1Script) -> list[str]:
    deck = list(WATCHER_STARTER_DECK)
    _remove_cards(deck, script.starting_removed_cards)
    for card in script.starting_added_cards:
        _add_card(deck, card)
    return deck


def build_imported_combat_cases(
    scripts: tuple[ImportedAct1Script, ...] | None = None,
) -> tuple[ImportedCombatCase, ...]:
    active_scripts = scripts or default_imported_act1_scripts()
    cases: list[ImportedCombatCase] = []
    for script in active_scripts:
        if not script.exact_available:
            continue

        deck = _starting_deck(script)
        relics = [_normalize_relic(name) for name in script.starting_relics]
        potions: list[str] = []
        potion_slots = 3
        current_hp = script.starting_current_hp
        max_hp = script.starting_max_hp
        gold = script.starting_gold

        for floor in script.floors:
            if current_hp is None:
                current_hp = floor.post_hp + floor.damage_taken
            if max_hp is None:
                max_hp = floor.max_hp

            if floor.is_combat:
                cases.append(
                    ImportedCombatCase(
                        seed_label=script.label,
                        seed=script.seed,
                        source_url=script.source_url,
                        floor=floor.floor,
                        encounter=floor.encounter or "unknown",
                        room_kind="imported_seed",
                        current_hp=current_hp,
                        max_hp=max_hp,
                        gold=gold,
                        deck=tuple(deck),
                        relics=tuple(relics),
                        potions=tuple(potions),
                        notes=(floor.summary,),
                    )
                )

            for potion in floor.potions_used:
                _consume_potion(potions, potion)
            for potion in floor.potions_discarded:
                _consume_potion(potions, potion)
            for potion in floor.potions_gained:
                _gain_potion(potions, potion, slots=potion_slots)

            _remove_cards(deck, floor.cards_removed)
            for card in floor.upgraded_cards:
                _upgrade_card(deck, card)
            if floor.picked_card:
                _add_card(deck, floor.picked_card)
            for relic in floor.relics_gained:
                normalized = _normalize_relic(relic)
                if normalized not in relics:
                    relics.append(normalized)
                if normalized == "PotionBelt":
                    potion_slots = 5

            current_hp = floor.post_hp
            max_hp = floor.max_hp
            gold = floor.post_gold
        # end floor loop
    return tuple(cases)


def default_imported_act1_scripts() -> tuple[ImportedAct1Script, ...]:
    minimalist_remove = ImportedAct1Script(
        label="minimalist_remove",
        seed="4AWM3ECVQDEWJ",
        source_url="https://baalorlord.tv/runs/1736881318",
        source_ascension=20,
        eval_ascension=0,
        exact_available=True,
        neow_choice="Lose 6 Max HP, then remove Defend and Defend",
        starting_max_hp=62,
        starting_current_hp=61,
        starting_removed_cards=("Defend", "Defend"),
        floors=(
            ImportedAct1Floor(1, "Enemy", "Cultist reward: Follow-Up", 61, 62, 115, encounter="Cultist", damage_taken=0, turns=3, picked_card="Follow-Up", skipped_cards=("Empty Fist", "Crescendo")),
            ImportedAct1Floor(2, "Enemy", "Jaw Worm reward: Third Eye", 49, 62, 132, encounter="Jaw Worm", damage_taken=12, turns=3, picked_card="Third Eye", skipped_cards=("Mental Fortress", "Nirvana")),
            ImportedAct1Floor(3, "Enemy", "2 Louse reward: Flurry of Blows", 49, 62, 148, encounter="2 Louse", damage_taken=0, turns=2, picked_card="Flurry of Blows", potions_gained=("Energy Potion",), skipped_cards=("Follow-Up", "Tranquility")),
            ImportedAct1Floor(4, "Unknown", "Golden Idol event, lose max HP", 49, 56, 148, option_taken="Lose Max HP", relics_gained=("Golden Idol",), max_hp_lost=6),
            ImportedAct1Floor(5, "Enemy", "Gremlin Gang reward: Indignation", 49, 56, 171, encounter="Gremlin Gang", damage_taken=0, turns=4, picked_card="Indignation", potions_gained=("Bottled Miracle",), skipped_cards=("Perseverance", "Spirit Shield")),
            ImportedAct1Floor(6, "Rest Site", "Upgrade Follow-Up", 49, 56, 171, upgraded_cards=("Follow-Up",)),
            ImportedAct1Floor(7, "Elite", "Lagavulin reward: Ragnarok, Emerald key", 37, 56, 204, encounter="Lagavulin", damage_taken=12, turns=5, picked_card="Ragnarok", potions_gained=("Explosive Potion",), potions_used=("Bottled Miracle",), relics_gained=("Gambling Chip",), skipped_cards=("Crescendo", "Bowling Bash")),
            ImportedAct1Floor(8, "Enemy", "2 Fungi Beasts reward skipped", 37, 56, 229, encounter="2 Fungi Beasts", damage_taken=0, turns=2, skipped_cards=("Crescendo", "Protect", "Carve Reality")),
            ImportedAct1Floor(9, "Treasure", "Treasure chest: Akabeko", 37, 56, 256, relics_gained=("Akabeko",)),
            ImportedAct1Floor(10, "Merchant", "Merchant: buy Frozen Eye, remove Defend", 37, 56, 10, relics_gained=("Frozen Eye",), cards_removed=("Defend",)),
            ImportedAct1Floor(11, "Rest Site", "Upgrade Ragnarok", 37, 56, 10, upgraded_cards=("Ragnarok",)),
            ImportedAct1Floor(12, "Elite", "Gremlin Nob reward skipped", 37, 56, 46, encounter="Gremlin Nob", damage_taken=0, turns=1, relics_gained=("Fossilized Helix",), skipped_cards=("Nirvana", "Signature Move", "Just Lucky")),
            ImportedAct1Floor(13, "Unknown", "Mushroom Lair reward skipped", 37, 56, 74, encounter="2 Fungi Beasts", damage_taken=0, turns=1, option_taken="Fought Mushrooms", relics_gained=("Odd Mushroom",), skipped_cards=("Sash Whip", "Flying Sleeves", "Pressure Points")),
            ImportedAct1Floor(14, "Elite", "Sentries reward skipped", 37, 56, 113, encounter="3 Sentries", damage_taken=0, turns=2, potions_gained=("Block Potion",), relics_gained=("Potion Belt",), skipped_cards=("Foreign Influence", "Prostrate", "Tranquility")),
            ImportedAct1Floor(15, "Rest Site", "Upgrade Eruption", 37, 56, 113, upgraded_cards=("Eruption",)),
            ImportedAct1Floor(16, "Boss", "Slime Boss reward: Deus Ex Machina", 37, 56, 211, encounter="Slime Boss", damage_taken=0, turns=1, picked_card="Deus Ex Machina", skipped_cards=("Alpha", "Lesson Learned")),
            ImportedAct1Floor(17, "Boss Chest", "Boss chest: Holy Water", 51, 56, 211, relics_gained=("Holy Water",), relics_skipped=("Cursed Key", "Black Star")),
        ),
    )
    lesson_learned_shell = ImportedAct1Script(
        label="lesson_learned_shell",
        seed="4VM6JKC3KR3TD",
        source_url="https://baalorlord.tv/runs/1744916840",
        source_ascension=20,
        eval_ascension=0,
        exact_available=True,
        neow_choice="Lose 6 Max HP, then pick Lesson Learned over Establishment and Wish",
        starting_max_hp=62,
        starting_current_hp=61,
        starting_added_cards=("Lesson Learned",),
        floors=(
            ImportedAct1Floor(1, "Enemy", "Jaw Worm reward: Tantrum", 54, 62, 115, encounter="Jaw Worm", damage_taken=7, turns=3, picked_card="Tantrum", skipped_cards=("Carve Reality", "Bowling Bash")),
            ImportedAct1Floor(2, "Enemy", "Small Slimes reward: Rushdown", 49, 62, 128, encounter="Small Slimes", damage_taken=5, turns=6, picked_card="Rushdown", potions_gained=("Bottled Miracle",), skipped_cards=("Weave", "Pressure Points")),
            ImportedAct1Floor(3, "Unknown", "Wing Statue remove Defend", 42, 62, 128, option_taken="Card Removal", cards_removed=("Defend",), damage_taken=7),
            ImportedAct1Floor(4, "Unknown (Enemy)", "2 Louse reward: Third Eye", 35, 62, 146, encounter="2 Louse", damage_taken=7, turns=2, picked_card="Third Eye", potions_gained=("Fruit Juice",), skipped_cards=("Protect", "Evaluate")),
            ImportedAct1Floor(5, "Enemy", "Red Slaver reward: Talk to the Hand", 34, 62, 163, encounter="Red Slaver", damage_taken=1, turns=3, picked_card="Talk to the Hand", skipped_cards=("Protect", "Cut Through Fate")),
            ImportedAct1Floor(6, "Rest Site", "Rested and used Fruit Juice", 59, 67, 163, potions_used=("Fruit Juice",), healed=25, max_hp_gained=5),
            ImportedAct1Floor(7, "Unknown", "Shining Light ignored", 59, 67, 163, option_taken="Ignored"),
            ImportedAct1Floor(8, "Elite", "Lagavulin reward: Mental Fortress", 59, 67, 189, encounter="Lagavulin", damage_taken=0, turns=5, picked_card="Mental Fortress", relics_gained=("Akabeko",), skipped_cards=("Wave of the Hand", "Reach Heaven")),
            ImportedAct1Floor(9, "Treasure", "Take Sapphire key over Bronze Scales", 59, 67, 238),
            ImportedAct1Floor(10, "Elite", "Gremlin Nob reward: Bowling Bash", 48, 67, 270, encounter="Gremlin Nob", damage_taken=11, turns=3, picked_card="Bowling Bash", potions_gained=("Speed Potion",), relics_gained=("Pocketwatch",), skipped_cards=("Fasting", "Nirvana")),
            ImportedAct1Floor(11, "Rest Site", "Upgrade Lesson Learned", 48, 67, 270, upgraded_cards=("Lesson Learned",)),
            ImportedAct1Floor(12, "Enemy", "Looter reward: Inner Peace", 48, 67, 282, encounter="Looter", damage_taken=0, turns=2, picked_card="Inner Peace", potions_gained=("Distilled Chaos",), potions_discarded=("Speed Potion",), skipped_cards=("Bowling Bash", "Halt")),
            ImportedAct1Floor(13, "Enemy", "2 Fungi Beasts reward skipped", 44, 67, 293, encounter="2 Fungi Beasts", damage_taken=4, turns=2, skipped_cards=("Empty Body", "Protect", "Bowling Bash")),
            ImportedAct1Floor(14, "Enemy", "Blue Slaver reward skipped", 44, 67, 308, encounter="Blue Slaver", damage_taken=0, turns=2, skipped_cards=("Pressure Points", "Foreign Influence", "Just Lucky")),
            ImportedAct1Floor(15, "Rest Site", "Upgrade Bowling Bash", 44, 67, 308, upgraded_cards=("Bowling Bash",)),
            ImportedAct1Floor(16, "Boss", "Slime Boss reward: Ragnarok", 44, 67, 387, encounter="Slime Boss", damage_taken=0, turns=3, picked_card="Ragnarok", skipped_cards=("Deva Form", "Deus Ex Machina")),
            ImportedAct1Floor(17, "Boss Chest", "Boss chest: Empty Cage", 61, 67, 387, relics_gained=("Empty Cage",), relics_skipped=("Sozu", "Astrolabe")),
        ),
    )
    icecream_runic_pyramid = ImportedAct1Script(
        label="icecream_runic_pyramid",
        seed="1TPMUARFP690B",
        source_url="https://steamcommunity.com/app/646570/discussions/0/3667553591708386502/?l=english",
        source_ascension=20,
        eval_ascension=0,
        exact_available=False,
        exact_issue="public source only confirms Neow Ice Cream and first boss Runic Pyramid; floor-by-floor Act 1 route is not recoverable from the thread",
        neow_choice="Take Ice Cream; first boss chest gives Runic Pyramid",
        starting_relics=("Ice Cream",),
    )
    return (minimalist_remove, lesson_learned_shell, icecream_runic_pyramid)
