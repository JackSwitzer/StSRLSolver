//! Card data and effects — minimal card registry for the core turn loop.
//!
//! Only implements cards needed for the fast MCTS path. The Python engine
//! handles the full ~350 card catalog with all edge cases.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Card types (match Python enums)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardType {
    Attack,
    Skill,
    Power,
    Status,
    Curse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardTarget {
    /// Single enemy (requires target selection)
    Enemy,
    /// All enemies (no target needed)
    AllEnemy,
    /// Self only
    SelfTarget,
    /// No target
    None,
}

// ---------------------------------------------------------------------------
// Card definition — static data, no mutation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct CardDef {
    pub id: &'static str,
    pub name: &'static str,
    pub card_type: CardType,
    pub target: CardTarget,
    pub cost: i32,
    pub base_damage: i32,
    pub base_block: i32,
    pub base_magic: i32,
    /// Does this card exhaust when played?
    pub exhaust: bool,
    /// Does this card change stance?
    pub enter_stance: Option<&'static str>,
    /// Special effect tags for the engine to check
    pub effects: &'static [&'static str],
}

impl CardDef {
    /// Is this card an unplayable status/curse?
    pub fn is_unplayable(&self) -> bool {
        self.cost == -2
    }
}

// ---------------------------------------------------------------------------
// Card registry — lookup by ID (including "+" suffix for upgrades)
// ---------------------------------------------------------------------------

/// Static card registry. Populated with core Watcher cards + universals.
/// Cards not in the registry fall back to defaults (cost 1, attack, enemy target).
pub struct CardRegistry {
    cards: HashMap<&'static str, CardDef>,
}

impl CardRegistry {
    pub fn new() -> Self {
        let mut cards = HashMap::new();

        // ---- Watcher Basic Cards ----
        Self::insert(&mut cards, CardDef {
            id: "Strike_P", name: "Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Strike_P+", name: "Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Defend_P", name: "Defend", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Defend_P+", name: "Defend+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Eruption", name: "Eruption", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: Some("Wrath"), effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Eruption+", name: "Eruption+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: Some("Wrath"), effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Vigilance", name: "Vigilance", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: Some("Calm"), effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Vigilance+", name: "Vigilance+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 12,
            base_magic: -1, exhaust: false, enter_stance: Some("Calm"), effects: &[],
        });

        // ---- Common Watcher Cards ----
        Self::insert(&mut cards, CardDef {
            id: "BowlingBash", name: "Bowling Bash", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_per_enemy"],
        });
        Self::insert(&mut cards, CardDef {
            id: "BowlingBash+", name: "Bowling Bash+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_per_enemy"],
        });
        Self::insert(&mut cards, CardDef {
            id: "CrushJoints", name: "Crush Joints", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["vuln_if_last_skill"],
        });
        Self::insert(&mut cards, CardDef {
            id: "CrushJoints+", name: "Crush Joints+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["vuln_if_last_skill"],
        });
        Self::insert(&mut cards, CardDef {
            id: "CutThroughFate", name: "Cut Through Fate", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["scry", "draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "CutThroughFate+", name: "Cut Through Fate+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["scry", "draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "EmptyBody", name: "Empty Body", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: -1, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["exit_stance"],
        });
        Self::insert(&mut cards, CardDef {
            id: "EmptyBody+", name: "Empty Body+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 11,
            base_magic: -1, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["exit_stance"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Flurry", name: "Flurry of Blows", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Flurry+", name: "Flurry of Blows+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "FlyingSleeves", name: "Flying Sleeves", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 4, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["multi_hit"],
        });
        Self::insert(&mut cards, CardDef {
            id: "FlyingSleeves+", name: "Flying Sleeves+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["multi_hit"],
        });
        Self::insert(&mut cards, CardDef {
            id: "FollowUp", name: "Follow-Up", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["energy_if_last_attack"],
        });
        Self::insert(&mut cards, CardDef {
            id: "FollowUp+", name: "Follow-Up+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["energy_if_last_attack"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Halt", name: "Halt", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 3,
            base_magic: 9, exhaust: false, enter_stance: None,
            effects: &["extra_block_in_wrath"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Halt+", name: "Halt+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 4,
            base_magic: 14, exhaust: false, enter_stance: None,
            effects: &["extra_block_in_wrath"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Prostrate", name: "Prostrate", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 4,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["mantra"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Prostrate+", name: "Prostrate+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 4,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["mantra"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Tantrum", name: "Tantrum", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 3, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: Some("Wrath"),
            effects: &["multi_hit"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Tantrum+", name: "Tantrum+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 3, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: Some("Wrath"),
            effects: &["multi_hit"],
        });

        // ---- Universal Status/Curse Cards ----
        Self::insert(&mut cards, CardDef {
            id: "Slimed", name: "Slimed", card_type: CardType::Status,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Wound", name: "Wound", card_type: CardType::Status,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &["unplayable"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Daze", name: "Daze", card_type: CardType::Status,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "ethereal"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Burn", name: "Burn", card_type: CardType::Status,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &["unplayable"],
        });
        Self::insert(&mut cards, CardDef {
            id: "AscendersBane", name: "Ascender's Bane", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "ethereal"],
        });

        // ---- Colorless basics (Strike/Defend aliases for other characters) ----
        Self::insert(&mut cards, CardDef {
            id: "Strike_R", name: "Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Defend_R", name: "Defend", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });

        CardRegistry { cards }
    }

    fn insert(map: &mut HashMap<&'static str, CardDef>, card: CardDef) {
        map.insert(card.id, card);
    }

    /// Look up a card by ID. Falls back to a default attack if not found.
    pub fn get(&self, card_id: &str) -> Option<&CardDef> {
        self.cards.get(card_id)
    }

    /// Get card or a sensible default for unknown cards.
    pub fn get_or_default(&self, card_id: &str) -> CardDef {
        if let Some(card) = self.cards.get(card_id) {
            card.clone()
        } else {
            // Unknown card: default to 1-cost attack targeting enemy, 6 damage
            CardDef {
                id: "Unknown",
                name: "Unknown",
                card_type: CardType::Attack,
                target: CardTarget::Enemy,
                cost: 1,
                base_damage: 6,
                base_block: -1,
                base_magic: -1,
                exhaust: false,
                enter_stance: None,
                effects: &[],
            }
        }
    }

    /// Check if a card ID is a known upgrade ("+" suffix).
    pub fn is_upgraded(card_id: &str) -> bool {
        card_id.ends_with('+')
    }

    /// Get the base ID (strip "+" suffix).
    pub fn base_id(card_id: &str) -> &str {
        card_id.trim_end_matches('+')
    }
}

impl Default for CardRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_lookup() {
        let reg = CardRegistry::new();
        let strike = reg.get("Strike_P").unwrap();
        assert_eq!(strike.base_damage, 6);
        assert_eq!(strike.cost, 1);
        assert_eq!(strike.card_type, CardType::Attack);
    }

    #[test]
    fn test_upgraded_lookup() {
        let reg = CardRegistry::new();
        let strike_plus = reg.get("Strike_P+").unwrap();
        assert_eq!(strike_plus.base_damage, 9);
    }

    #[test]
    fn test_eruption_stance() {
        let reg = CardRegistry::new();
        let eruption = reg.get("Eruption").unwrap();
        assert_eq!(eruption.enter_stance, Some("Wrath"));
        assert_eq!(eruption.cost, 2);

        let eruption_plus = reg.get("Eruption+").unwrap();
        assert_eq!(eruption_plus.cost, 1); // Upgrade reduces cost
    }

    #[test]
    fn test_unknown_card_default() {
        let reg = CardRegistry::new();
        let unknown = reg.get_or_default("SomeWeirdCard");
        assert_eq!(unknown.cost, 1);
        assert_eq!(unknown.card_type, CardType::Attack);
    }

    #[test]
    fn test_is_upgraded() {
        assert!(CardRegistry::is_upgraded("Strike_P+"));
        assert!(!CardRegistry::is_upgraded("Strike_P"));
    }
}
