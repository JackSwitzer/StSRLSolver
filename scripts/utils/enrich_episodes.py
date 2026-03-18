"""Enrich top_episodes.json with computed fields from combat data."""

import json
import sys
from collections import Counter
from pathlib import Path

INPUT_PATH = Path(__file__).resolve().parent.parent / "logs" / "weekend-run" / "top_episodes.json"

# -- Card classification tables --

# Cards that enter Wrath stance
WRATH_CARDS = {
    "Eruption", "Eruption+",
    "Tantrum", "Tantrum+",
    "Crescendo", "Crescendo+",
    "Vengeance", "Vengeance+",  # Simmering Fury (Java ID)
}

# Cards that enter Calm stance
CALM_CARDS = {
    "Vigilance", "Vigilance+",
    "EmptyBody", "EmptyBody+",
    "FearNoEvil", "FearNoEvil+",
    "InnerPeace", "InnerPeace+",
    "ClearTheMind", "ClearTheMind+",  # Tranquility (Java ID)
}

# Cards that add Mantra (toward Divinity)
MANTRA_CARDS = {
    "Worship", "Worship+",
    "Prostrate", "Prostrate+",
    "Pray", "Pray+",
    "Devotion", "Devotion+",
}

# Block cards (for boss fight analysis)
BLOCK_CARDS = {
    "Defend_P", "Defend_P+",
    "Vigilance", "Vigilance+",
    "Halt", "Halt+",
    "Protect", "Protect+",
    "ThirdEye", "ThirdEye+",
    "Perseverance", "Perseverance+",
    "DeceiveReality", "DeceiveReality+",
    "Evaluate", "Evaluate+",
    "EmptyBody", "EmptyBody+",
    "Wallop", "Wallop+",
    "Safety", "Safety+",
}

# Attack cards (for damage estimation) — card_type=ATTACK in the engine
# We use a heuristic: anything not in SKILL/POWER/STATUS/CURSE sets is an attack.
# But it's cleaner to list known attacks from the Watcher pool.
ATTACK_CARDS = {
    "Strike_P", "Strike_P+",
    "Eruption", "Eruption+",
    "BowlingBash", "BowlingBash+",
    "CutThroughFate", "CutThroughFate+",
    "EmptyFist", "EmptyFist+",
    "FlurryOfBlows", "FlurryOfBlows+",
    "FlyingSleeves", "FlyingSleeves+",
    "FollowUp", "FollowUp+",
    "JustLucky", "JustLucky+",
    "SashWhip", "SashWhip+",
    "CrushJoints", "CrushJoints+",
    "Consecrate", "Consecrate+",
    "Tantrum", "Tantrum+",
    "FearNoEvil", "FearNoEvil+",
    "ReachHeaven", "ReachHeaven+",
    "SandsOfTime", "SandsOfTime+",
    "SignatureMove", "SignatureMove+",
    "TalkToTheHand", "TalkToTheHand+",
    "Wallop", "Wallop+",
    "Weave", "Weave+",
    "WheelKick", "WheelKick+",
    "WindmillStrike", "WindmillStrike+",
    "Conclude", "Conclude+",
    "CarveReality", "CarveReality+",
    "Brilliance", "Brilliance+",
    "Judgement", "Judgement+",
    "LessonLearned", "LessonLearned+",
    "Ragnarok", "Ragnarok+",
    "Expunger", "Expunger+",
    "ThroughViolence", "ThroughViolence+",
    "Smite", "Smite+",
}

POWER_CARDS = {
    "MentalFortress", "MentalFortress+",
    "Nirvana", "Nirvana+",
    "Adaptation", "Adaptation+",  # Rushdown (Java ID)
    "Study", "Study+",
    "BattleHymn", "BattleHymn+",
    "Establishment", "Establishment+",
    "LikeWater", "LikeWater+",
    "DevaForm", "DevaForm+",
    "Devotion", "Devotion+",
    "Fasting2", "Fasting2+",
    "MasterReality", "MasterReality+",
    "Omega", "Omega+",
    "Sadistic Nature", "Sadistic Nature+",
}

# Strip upgrade suffix for base name
def base_name(card: str) -> str:
    return card.rstrip("+")


def classify_card(card: str) -> str:
    """Classify a card as attack/skill/power/status/other."""
    if card.startswith("potion:"):
        return "potion"
    if card in ATTACK_CARDS:
        return "attack"
    if card in POWER_CARDS:
        return "power"
    if card in {"Slimed", "Slimed+", "Burn", "Burn+", "Dazed", "Dazed+", "Void", "Void+", "Wound", "Wound+"}:
        return "status"
    # Everything else is a skill (Miracle, Defend, etc.)
    return "skill"


def compute_hp_history(episode: dict) -> list[int]:
    """Compute HP after each floor. Start at 80, subtract hp_lost per combat."""
    hp = 80
    history = [hp]
    combat_by_floor = {c["floor"]: c["hp_lost"] for c in episode["combats"]}
    max_floor = episode["floor"]
    for f in range(1, max_floor + 1):
        if f in combat_by_floor:
            hp -= combat_by_floor[f]
            hp = max(hp, 0)
        history.append(hp)
    return history


def compute_stance_sequence(episode: dict) -> list[dict]:
    """For each combat, track stance entries from cards played."""
    result = []
    for combat in episode["combats"]:
        stances = []
        for turn in combat.get("turns_detail", []):
            for card in turn.get("cards", []):
                if card in WRATH_CARDS:
                    stances.append("Wrath")
                elif card in CALM_CARDS:
                    stances.append("Calm")
                elif card in MANTRA_CARDS:
                    stances.append("Mantra")
        if stances:
            result.append({"floor": combat["floor"], "stances": stances})
    return result


def compute_cards_played_total(episode: dict) -> int:
    """Total cards played across all combats."""
    total = 0
    for combat in episode["combats"]:
        for turn in combat.get("turns_detail", []):
            total += len(turn.get("cards", []))
    return total


def compute_avg_cards_per_turn(episode: dict) -> float:
    """Average cards played per turn across all combats."""
    total_cards = 0
    total_turns = 0
    for combat in episode["combats"]:
        for turn in combat.get("turns_detail", []):
            total_cards += len(turn.get("cards", []))
            total_turns += 1
    return round(total_cards / total_turns, 2) if total_turns > 0 else 0.0


def compute_unique_cards_played(episode: dict) -> list[str]:
    """Set of unique card names played across all combats."""
    unique = set()
    for combat in episode["combats"]:
        for turn in combat.get("turns_detail", []):
            unique.update(turn.get("cards", []))
    return sorted(unique)


def compute_boss_fight_analysis(episode: dict) -> dict | None:
    """Analyze the floor 16 boss fight (Guardian) if present."""
    boss_combat = None
    for c in episode["combats"]:
        if c["floor"] == 16:
            boss_combat = c
            break
    if boss_combat is None:
        return None

    all_cards = []
    cards_by_turn = []
    wrath_turns = []
    block_count = 0
    attack_count = 0

    for turn in boss_combat.get("turns_detail", []):
        turn_cards = turn.get("cards", [])
        all_cards.extend(turn_cards)
        cards_by_turn.append({"turn": turn["turn"], "cards": turn_cards})

        in_wrath = False
        for card in turn_cards:
            if card in WRATH_CARDS:
                in_wrath = True
            if card in BLOCK_CARDS:
                block_count += 1
            if card in ATTACK_CARDS:
                attack_count += 1

        if in_wrath:
            wrath_turns.append(turn["turn"])

    # Rough damage estimate: 6 avg per attack, doubled in Wrath turns
    wrath_turn_set = set(wrath_turns)
    damage_estimate = 0
    for turn in boss_combat.get("turns_detail", []):
        multiplier = 2 if turn["turn"] in wrath_turn_set else 1
        for card in turn.get("cards", []):
            if card in ATTACK_CARDS:
                damage_estimate += 6 * multiplier

    return {
        "total_turns": boss_combat["turns"],
        "total_cards_played": len(all_cards),
        "damage_output_estimate": damage_estimate,
        "block_cards_played": block_count,
        "wrath_turns": wrath_turns,
        "cards_by_turn": cards_by_turn,
        "hp_lost": boss_combat["hp_lost"],
    }


def compute_deck_analysis(episode: dict) -> dict:
    """Categorize deck_final into attacks/skills/powers with counts."""
    attacks = []
    skills = []
    powers = []
    statuses = []
    curses = []

    for card in episode.get("deck_final", []):
        cat = classify_card(card)
        if cat == "attack":
            attacks.append(card)
        elif cat == "power":
            powers.append(card)
        elif cat == "status":
            statuses.append(card)
        else:
            skills.append(card)

    return {
        "attacks": Counter(attacks).most_common(),
        "skills": Counter(skills).most_common(),
        "powers": Counter(powers).most_common(),
        "statuses": Counter(statuses).most_common() if statuses else [],
        "total_attacks": len(attacks),
        "total_skills": len(skills),
        "total_powers": len(powers),
        "total_cards": len(episode.get("deck_final", [])),
    }


def compute_meta(episodes: list[dict]) -> dict:
    """Compute aggregate stats across all episodes."""
    floors = [ep["floor"] for ep in episodes]
    hps = [ep["hp"] for ep in episodes]

    # Global card play counts
    global_card_counter = Counter()
    total_cards_all = 0
    for ep in episodes:
        for combat in ep["combats"]:
            for turn in combat.get("turns_detail", []):
                cards = turn.get("cards", [])
                global_card_counter.update(cards)
                total_cards_all += len(cards)

    # Guardian (floor 16) analysis
    guardian_turns = []
    guardian_hp_lost = []
    guardian_card_counter = Counter()
    guardian_wrath_entries = []

    for ep in episodes:
        boss = ep.get("boss_fight_analysis")
        if boss:
            guardian_turns.append(boss["total_turns"])
            guardian_hp_lost.append(boss["hp_lost"])
            guardian_wrath_entries.append(len(boss["wrath_turns"]))
            for t in boss["cards_by_turn"]:
                guardian_card_counter.update(t["cards"])

    guardian_analysis = None
    if guardian_turns:
        guardian_analysis = {
            "count": len(guardian_turns),
            "avg_turns": round(sum(guardian_turns) / len(guardian_turns), 2),
            "avg_hp_lost": round(sum(guardian_hp_lost) / len(guardian_hp_lost), 2),
            "most_played_cards": guardian_card_counter.most_common(15),
            "avg_wrath_entries": round(sum(guardian_wrath_entries) / len(guardian_wrath_entries), 2),
        }

    return {
        "total_episodes": len(episodes),
        "avg_floor": round(sum(floors) / len(floors), 2),
        "all_f16": all(f >= 16 for f in floors),
        "top_cards_played": global_card_counter.most_common(20),
        "avg_cards_per_game": round(total_cards_all / len(episodes), 2),
        "avg_hp_at_death": round(sum(hps) / len(hps), 2),
        "guardian_analysis": guardian_analysis,
    }


def enrich_episode(ep: dict) -> dict:
    """Add all computed fields to an episode."""
    ep["hp_history"] = compute_hp_history(ep)
    ep["stance_sequence"] = compute_stance_sequence(ep)
    ep["cards_played_total"] = compute_cards_played_total(ep)
    ep["avg_cards_per_turn"] = compute_avg_cards_per_turn(ep)
    ep["unique_cards_played"] = compute_unique_cards_played(ep)
    ep["boss_fight_analysis"] = compute_boss_fight_analysis(ep)
    ep["deck_analysis"] = compute_deck_analysis(ep)
    return ep


def main():
    if not INPUT_PATH.exists():
        print(f"ERROR: {INPUT_PATH} not found", file=sys.stderr)
        sys.exit(1)

    print(f"Reading {INPUT_PATH} ...")
    with open(INPUT_PATH) as f:
        raw = json.load(f)

    # Handle both formats: bare list or {meta, episodes}
    if isinstance(raw, list):
        episodes = raw
    elif isinstance(raw, dict) and "episodes" in raw:
        episodes = raw["episodes"]
    else:
        print("ERROR: unexpected JSON structure", file=sys.stderr)
        sys.exit(1)

    print(f"Enriching {len(episodes)} episodes ...")
    for ep in episodes:
        enrich_episode(ep)

    meta = compute_meta(episodes)

    output = {
        "meta": meta,
        "episodes": episodes,
    }

    print(f"Writing back to {INPUT_PATH} ...")
    with open(INPUT_PATH, "w") as f:
        json.dump(output, f, indent=2)

    # Print summary
    print(f"\nDone. Summary:")
    print(f"  Episodes: {meta['total_episodes']}")
    print(f"  Avg floor: {meta['avg_floor']}")
    print(f"  All reach F16: {meta['all_f16']}")
    print(f"  Avg cards/game: {meta['avg_cards_per_game']}")
    print(f"  Avg HP at death: {meta['avg_hp_at_death']}")
    print(f"  Top 5 cards: {meta['top_cards_played'][:5]}")
    if meta["guardian_analysis"]:
        g = meta["guardian_analysis"]
        print(f"  Guardian fights: {g['count']}")
        print(f"    Avg turns: {g['avg_turns']}")
        print(f"    Avg HP lost: {g['avg_hp_lost']}")
        print(f"    Avg Wrath entries: {g['avg_wrath_entries']}")
        print(f"    Top cards: {g['most_played_cards'][:5]}")


if __name__ == "__main__":
    main()
