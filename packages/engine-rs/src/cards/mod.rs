//! Card data and effects — minimal card registry for the core turn loop.
//!
//! Only implements cards needed for the fast MCTS path. The Python engine
//! handles the full ~350 card catalog with all edge cases.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod watcher;
mod ironclad;
mod silent;
mod defect;
mod colorless;
mod curses;
mod status;
mod temp;


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
#[derive(Clone)]
pub struct CardRegistry {
    cards: HashMap<&'static str, CardDef>,
}


impl CardRegistry {
    pub fn new() -> Self {
        let mut cards = HashMap::new();

        watcher::register_watcher(&mut cards);
        ironclad::register_ironclad(&mut cards);
        silent::register_silent(&mut cards);
        defect::register_defect(&mut cards);
        colorless::register_colorless(&mut cards);
        curses::register_curses(&mut cards);
        status::register_status(&mut cards);
        temp::register_temp(&mut cards);

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

    // -----------------------------------------------------------------------
    // Helper: assert a card exists with expected base + upgraded stats
    // -----------------------------------------------------------------------
    fn assert_card(reg: &CardRegistry, id: &str, cost: i32, dmg: i32, blk: i32, mag: i32, ct: CardType) {
        let card = reg.get(id).unwrap_or_else(|| panic!("Card '{}' not found in registry", id));
        assert_eq!(card.cost, cost, "{} cost", id);
        assert_eq!(card.base_damage, dmg, "{} damage", id);
        assert_eq!(card.base_block, blk, "{} block", id);
        assert_eq!(card.base_magic, mag, "{} magic", id);
        assert_eq!(card.card_type, ct, "{} type", id);
    }

    fn assert_has_effect(reg: &CardRegistry, id: &str, effect: &str) {
        let card = reg.get(id).unwrap_or_else(|| panic!("Card '{}' not found", id));
        assert!(card.effects.contains(&effect), "{} should have effect '{}'", id, effect);
    }

    // -----------------------------------------------------------------------
    // All cards in reward pools must be registered (no fallback to Unknown)
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_pool_cards_registered() {
        let reg = CardRegistry::new();
        let pool_cards = [
            // Common
            "BowlingBash", "Consecrate", "Crescendo", "CrushJoints",
            "CutThroughFate", "EmptyBody", "EmptyFist", "Evaluate",
            "Flurry", "FlyingSleeves", "FollowUp", "Halt",
            "JustLucky", "PressurePoints", "Prostrate",
            "Protect", "SashWhip", "Tranquility",
            // Uncommon
            "BattleHymn", "CarveReality", "Conclude", "DeceiveReality",
            "EmptyMind", "FearNoEvil", "ForeignInfluence", "Indignation",
            "InnerPeace", "LikeWater", "Meditate", "Nirvana",
            "Perseverance", "ReachHeaven", "SandsOfTime", "SignatureMove",
            "Smite", "Study", "Swivel", "TalkToTheHand",
            "Tantrum", "ThirdEye", "Wallop", "WaveOfTheHand",
            "Weave", "WheelKick", "WindmillStrike", "WreathOfFlame",
            // Rare
            "Alpha", "Blasphemy", "Brilliance", "ConjureBlade",
            "DevaForm", "Devotion", "Establishment", "Fasting",
            "Judgement", "LessonLearned", "MasterReality",
            "MentalFortress", "Omniscience", "Ragnarok",
            "Adaptation", "Scrawl", "SpiritShield", "Vault", "Wish",
        ];
        for id in &pool_cards {
            assert!(reg.get(id).is_some(), "Card '{}' missing from registry", id);
            let upgraded = format!("{}+", id);
            assert!(reg.get(&upgraded).is_some(), "Card '{}' missing from registry", upgraded);
        }
    }

    // -----------------------------------------------------------------------
    // Common card stats (base + upgraded)
    // -----------------------------------------------------------------------
    #[test]
    fn test_consecrate_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Consecrate", 0, 5, -1, -1, CardType::Attack);
        assert_card(&reg, "Consecrate+", 0, 8, -1, -1, CardType::Attack);
    }

    #[test]
    fn test_crescendo_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Crescendo", 1, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "Crescendo+", 0, -1, -1, -1, CardType::Skill);
        assert!(reg.get("Crescendo").unwrap().exhaust);
        assert_has_effect(&reg, "Crescendo", "retain");
        assert_eq!(reg.get("Crescendo").unwrap().enter_stance, Some("Wrath"));
    }

    #[test]
    fn test_empty_fist_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "EmptyFist", 1, 9, -1, -1, CardType::Attack);
        assert_card(&reg, "EmptyFist+", 1, 14, -1, -1, CardType::Attack);
        assert_eq!(reg.get("EmptyFist").unwrap().enter_stance, Some("Neutral"));
    }

    #[test]
    fn test_evaluate_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Evaluate", 1, -1, 6, -1, CardType::Skill);
        assert_card(&reg, "Evaluate+", 1, -1, 10, -1, CardType::Skill);
    }

    #[test]
    fn test_just_lucky_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "JustLucky", 0, 3, 2, 1, CardType::Attack);
        assert_card(&reg, "JustLucky+", 0, 4, 3, 2, CardType::Attack);
    }

    #[test]
    fn test_pressure_points_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "PressurePoints", 1, -1, -1, 8, CardType::Skill);
        assert_card(&reg, "PressurePoints+", 1, -1, -1, 11, CardType::Skill);
    }

    #[test]
    fn test_protect_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Protect", 2, -1, 12, -1, CardType::Skill);
        assert_card(&reg, "Protect+", 2, -1, 16, -1, CardType::Skill);
        assert_has_effect(&reg, "Protect", "retain");
    }

    #[test]
    fn test_sash_whip_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "SashWhip", 1, 8, -1, 1, CardType::Attack);
        assert_card(&reg, "SashWhip+", 1, 10, -1, 2, CardType::Attack);
    }

    #[test]
    fn test_tranquility_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Tranquility", 1, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "Tranquility+", 0, -1, -1, -1, CardType::Skill);
        assert!(reg.get("Tranquility").unwrap().exhaust);
        assert_eq!(reg.get("Tranquility").unwrap().enter_stance, Some("Calm"));
    }

    // -----------------------------------------------------------------------
    // Uncommon card stats (base + upgraded)
    // -----------------------------------------------------------------------
    #[test]
    fn test_battle_hymn_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "BattleHymn", 1, -1, -1, 1, CardType::Power);
        assert_card(&reg, "BattleHymn+", 1, -1, -1, 1, CardType::Power);
        assert_has_effect(&reg, "BattleHymn+", "innate");
    }

    #[test]
    fn test_carve_reality_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "CarveReality", 1, 6, -1, -1, CardType::Attack);
        assert_card(&reg, "CarveReality+", 1, 10, -1, -1, CardType::Attack);
    }

    #[test]
    fn test_deceive_reality_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "DeceiveReality", 1, -1, 4, -1, CardType::Skill);
        assert_card(&reg, "DeceiveReality+", 1, -1, 7, -1, CardType::Skill);
    }

    #[test]
    fn test_empty_mind_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "EmptyMind", 1, -1, -1, 2, CardType::Skill);
        assert_card(&reg, "EmptyMind+", 1, -1, -1, 3, CardType::Skill);
        assert_eq!(reg.get("EmptyMind").unwrap().enter_stance, Some("Neutral"));
    }

    #[test]
    fn test_fear_no_evil_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "FearNoEvil", 1, 8, -1, -1, CardType::Attack);
        assert_card(&reg, "FearNoEvil+", 1, 11, -1, -1, CardType::Attack);
    }

    #[test]
    fn test_foreign_influence_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "ForeignInfluence", 0, -1, -1, -1, CardType::Skill);
        assert!(reg.get("ForeignInfluence").unwrap().exhaust);
    }

    #[test]
    fn test_indignation_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Indignation", 1, -1, -1, 3, CardType::Skill);
        assert_card(&reg, "Indignation+", 1, -1, -1, 5, CardType::Skill);
    }

    #[test]
    fn test_like_water_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "LikeWater", 1, -1, -1, 5, CardType::Power);
        assert_card(&reg, "LikeWater+", 1, -1, -1, 7, CardType::Power);
    }

    #[test]
    fn test_meditate_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Meditate", 1, -1, -1, 1, CardType::Skill);
        assert_card(&reg, "Meditate+", 1, -1, -1, 2, CardType::Skill);
        assert_eq!(reg.get("Meditate").unwrap().enter_stance, Some("Calm"));
    }

    #[test]
    fn test_nirvana_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Nirvana", 1, -1, -1, 3, CardType::Power);
        assert_card(&reg, "Nirvana+", 1, -1, -1, 4, CardType::Power);
    }

    #[test]
    fn test_perseverance_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Perseverance", 1, -1, 5, 2, CardType::Skill);
        assert_card(&reg, "Perseverance+", 1, -1, 7, 3, CardType::Skill);
        assert_has_effect(&reg, "Perseverance", "retain");
    }

    #[test]
    fn test_reach_heaven_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "ReachHeaven", 2, 10, -1, -1, CardType::Attack);
        assert_card(&reg, "ReachHeaven+", 2, 15, -1, -1, CardType::Attack);
    }

    #[test]
    fn test_sands_of_time_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "SandsOfTime", 4, 20, -1, -1, CardType::Attack);
        assert_card(&reg, "SandsOfTime+", 4, 26, -1, -1, CardType::Attack);
        assert_has_effect(&reg, "SandsOfTime", "retain");
    }

    #[test]
    fn test_signature_move_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "SignatureMove", 2, 30, -1, -1, CardType::Attack);
        assert_card(&reg, "SignatureMove+", 2, 40, -1, -1, CardType::Attack);
    }

    #[test]
    fn test_study_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Study", 2, -1, -1, 1, CardType::Power);
        assert_card(&reg, "Study+", 1, -1, -1, 1, CardType::Power);
    }

    #[test]
    fn test_swivel_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Swivel", 2, -1, 8, -1, CardType::Skill);
        assert_card(&reg, "Swivel+", 2, -1, 11, -1, CardType::Skill);
    }

    #[test]
    fn test_wallop_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Wallop", 2, 9, -1, -1, CardType::Attack);
        assert_card(&reg, "Wallop+", 2, 12, -1, -1, CardType::Attack);
    }

    #[test]
    fn test_wave_of_the_hand_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "WaveOfTheHand", 1, -1, -1, 1, CardType::Skill);
        assert_card(&reg, "WaveOfTheHand+", 1, -1, -1, 2, CardType::Skill);
    }

    #[test]
    fn test_weave_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Weave", 0, 4, -1, -1, CardType::Attack);
        assert_card(&reg, "Weave+", 0, 6, -1, -1, CardType::Attack);
    }

    #[test]
    fn test_windmill_strike_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "WindmillStrike", 2, 7, -1, 4, CardType::Attack);
        assert_card(&reg, "WindmillStrike+", 2, 10, -1, 5, CardType::Attack);
        assert_has_effect(&reg, "WindmillStrike", "retain");
    }

    #[test]
    fn test_wreath_of_flame_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "WreathOfFlame", 1, -1, -1, 5, CardType::Skill);
        assert_card(&reg, "WreathOfFlame+", 1, -1, -1, 8, CardType::Skill);
    }

    // -----------------------------------------------------------------------
    // Rare card stats (base + upgraded)
    // -----------------------------------------------------------------------
    #[test]
    fn test_alpha_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Alpha", 1, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "Alpha+", 1, -1, -1, -1, CardType::Skill);
        assert!(reg.get("Alpha").unwrap().exhaust);
        assert_has_effect(&reg, "Alpha+", "innate");
    }

    #[test]
    fn test_blasphemy_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Blasphemy", 1, -1, -1, -1, CardType::Skill);
        assert!(reg.get("Blasphemy").unwrap().exhaust);
        assert_eq!(reg.get("Blasphemy").unwrap().enter_stance, Some("Divinity"));
        assert_has_effect(&reg, "Blasphemy+", "retain");
    }

    #[test]
    fn test_brilliance_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Brilliance", 1, 12, -1, 0, CardType::Attack);
        assert_card(&reg, "Brilliance+", 1, 16, -1, 0, CardType::Attack);
    }

    #[test]
    fn test_conjure_blade_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "ConjureBlade", -1, -1, -1, -1, CardType::Skill);
        assert!(reg.get("ConjureBlade").unwrap().exhaust);
    }

    #[test]
    fn test_deva_form_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "DevaForm", 3, -1, -1, 1, CardType::Power);
        assert_card(&reg, "DevaForm+", 3, -1, -1, 1, CardType::Power);
        assert_has_effect(&reg, "DevaForm", "ethereal");
        assert!(!reg.get("DevaForm+").unwrap().effects.contains(&"ethereal"));
    }

    #[test]
    fn test_devotion_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Devotion", 1, -1, -1, 2, CardType::Power);
        assert_card(&reg, "Devotion+", 1, -1, -1, 3, CardType::Power);
    }

    #[test]
    fn test_establishment_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Establishment", 1, -1, -1, 1, CardType::Power);
        assert_has_effect(&reg, "Establishment+", "innate");
    }

    #[test]
    fn test_fasting_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Fasting", 2, -1, -1, 3, CardType::Power);
        assert_card(&reg, "Fasting+", 2, -1, -1, 4, CardType::Power);
    }

    #[test]
    fn test_judgement_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Judgement", 1, -1, -1, 30, CardType::Skill);
        assert_card(&reg, "Judgement+", 1, -1, -1, 40, CardType::Skill);
    }

    #[test]
    fn test_lesson_learned_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "LessonLearned", 2, 10, -1, -1, CardType::Attack);
        assert_card(&reg, "LessonLearned+", 2, 13, -1, -1, CardType::Attack);
        assert!(reg.get("LessonLearned").unwrap().exhaust);
    }

    #[test]
    fn test_master_reality_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "MasterReality", 1, -1, -1, -1, CardType::Power);
        assert_card(&reg, "MasterReality+", 0, -1, -1, -1, CardType::Power);
    }

    #[test]
    fn test_omniscience_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Omniscience", 4, -1, -1, 2, CardType::Skill);
        assert_card(&reg, "Omniscience+", 3, -1, -1, 2, CardType::Skill);
        assert!(reg.get("Omniscience").unwrap().exhaust);
    }

    #[test]
    fn test_scrawl_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Scrawl", 1, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "Scrawl+", 0, -1, -1, -1, CardType::Skill);
        assert!(reg.get("Scrawl").unwrap().exhaust);
    }

    #[test]
    fn test_spirit_shield_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "SpiritShield", 2, -1, -1, 3, CardType::Skill);
        assert_card(&reg, "SpiritShield+", 2, -1, -1, 4, CardType::Skill);
    }

    #[test]
    fn test_vault_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Vault", 3, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "Vault+", 2, -1, -1, -1, CardType::Skill);
        assert!(reg.get("Vault").unwrap().exhaust);
    }

    #[test]
    fn test_wish_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Wish", 3, -1, -1, 3, CardType::Skill);
        assert_card(&reg, "Wish+", 3, -1, -1, 4, CardType::Skill);
        assert!(reg.get("Wish").unwrap().exhaust);
    }

    // -----------------------------------------------------------------------
    // Bug fixes: Tantrum shuffle + Smite exhaust
    // -----------------------------------------------------------------------
    #[test]
    fn test_tantrum_shuffle_into_draw() {
        let reg = CardRegistry::new();
        assert_has_effect(&reg, "Tantrum", "shuffle_self_into_draw");
        assert_has_effect(&reg, "Tantrum+", "shuffle_self_into_draw");
    }

    #[test]
    fn test_smite_exhaust() {
        let reg = CardRegistry::new();
        assert!(reg.get("Smite").unwrap().exhaust, "Smite should exhaust");
        assert!(reg.get("Smite+").unwrap().exhaust, "Smite+ should exhaust");
        assert_has_effect(&reg, "Smite", "retain");
    }

    // -----------------------------------------------------------------------
    // All Ironclad cards in reward pools must be registered
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_ironclad_cards_registered() {
        let reg = CardRegistry::new();
        let ironclad_cards = [
            // Basic
            "Strike_R", "Defend_R", "Bash",
            // Common
            "Anger", "Armaments", "Body Slam", "Clash", "Cleave",
            "Clothesline", "Flex", "Havoc", "Headbutt", "Heavy Blade",
            "Iron Wave", "Perfected Strike", "Pommel Strike", "Shrug It Off",
            "Sword Boomerang", "Thunderclap", "True Grit", "Twin Strike",
            "Warcry", "Wild Strike",
            // Uncommon
            "Battle Trance", "Blood for Blood", "Bloodletting", "Burning Pact",
            "Carnage", "Combust", "Dark Embrace", "Disarm", "Dropkick",
            "Dual Wield", "Entrench", "Evolve", "Feel No Pain", "Fire Breathing",
            "Flame Barrier", "Ghostly Armor", "Hemokinesis", "Infernal Blade",
            "Inflame", "Intimidate", "Metallicize", "Power Through", "Pummel",
            "Rage", "Rampage", "Reckless Charge", "Rupture", "Searing Blow",
            "Second Wind", "Seeing Red", "Sentinel", "Sever Soul", "Shockwave",
            "Spot Weakness", "Uppercut", "Whirlwind",
            // Rare
            "Barricade", "Berserk", "Bludgeon", "Brutality", "Corruption",
            "Demon Form", "Double Tap", "Exhume", "Feed", "Fiend Fire",
            "Immolate", "Impervious", "Juggernaut", "Limit Break", "Offering",
            "Reaper",
        ];
        for id in &ironclad_cards {
            assert!(reg.get(id).is_some(), "Ironclad card '{}' missing from registry", id);
            let upgraded = format!("{}+", id);
            assert!(reg.get(&upgraded).is_some(), "Ironclad card '{}' missing from registry", upgraded);
        }
        // Verify count: 3 basic + 20 common + 36 uncommon + 16 rare = 75
        assert_eq!(ironclad_cards.len(), 75, "Should have exactly 75 Ironclad cards");
    }

    // -----------------------------------------------------------------------
    // All Silent cards in reward pools must be registered
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_silent_cards_registered() {
        let reg = CardRegistry::new();
        let silent_cards = [
            // Basic
            "Strike_G", "Defend_G", "Neutralize", "Survivor",
            // Common
            "Acrobatics", "Backflip", "Bane", "Blade Dance", "Cloak and Dagger",
            "Dagger Spray", "Dagger Throw", "Deadly Poison", "Deflect",
            "Dodge and Roll", "Flying Knee", "Outmaneuver", "Piercing Wail",
            "Poisoned Stab", "Prepared", "Quick Slash", "Slice",
            "Sneaky Strike", "Sucker Punch",
            // Uncommon
            "Accuracy", "All-Out Attack", "Backstab", "Blur", "Bouncing Flask",
            "Calculated Gamble", "Caltrops", "Catalyst", "Choke", "Concentrate",
            "Crippling Cloud", "Dash", "Distraction", "Endless Agony", "Envenom",
            "Escape Plan", "Eviscerate", "Expertise", "Finisher", "Flechettes",
            "Footwork", "Heel Hook", "Infinite Blades", "Leg Sweep",
            "Masterful Stab", "Noxious Fumes", "Predator", "Reflex",
            "Riddle with Holes", "Setup", "Skewer", "Tactician", "Terror",
            "Well-Laid Plans",
            // Rare
            "A Thousand Cuts", "Adrenaline", "After Image", "Alchemize",
            "Bullet Time", "Burst", "Corpse Explosion", "Die Die Die",
            "Doppelganger", "Glass Knife", "Grand Finale", "Malaise",
            "Nightmare", "Phantasmal Killer", "Storm of Steel",
            "Tools of the Trade", "Unload", "Wraith Form",
        ];
        for id in &silent_cards {
            assert!(reg.get(id).is_some(), "Silent card '{}' missing from registry", id);
            let upgraded = format!("{}+", id);
            assert!(reg.get(&upgraded).is_some(), "Silent card '{}' missing from registry", upgraded);
        }
        // Verify count: 4 basic + 19 common + 34 uncommon + 18 rare = 75
        assert_eq!(silent_cards.len(), 75, "Should have exactly 75 Silent cards");
    }

    // -----------------------------------------------------------------------
    // Spot-check Ironclad card stats
    // -----------------------------------------------------------------------
    #[test]
    fn test_bash_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Bash", 2, 8, -1, 2, CardType::Attack);
        assert_card(&reg, "Bash+", 2, 10, -1, 3, CardType::Attack);
        assert_has_effect(&reg, "Bash", "vulnerable");
    }

    #[test]
    fn test_impervious_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Impervious", 2, -1, 30, -1, CardType::Skill);
        assert_card(&reg, "Impervious+", 2, -1, 40, -1, CardType::Skill);
        assert!(reg.get("Impervious").unwrap().exhaust);
    }

    #[test]
    fn test_corruption_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Corruption", 3, -1, -1, -1, CardType::Power);
        assert_card(&reg, "Corruption+", 2, -1, -1, -1, CardType::Power);
        assert_has_effect(&reg, "Corruption", "corruption");
    }

    // -----------------------------------------------------------------------
    // Spot-check Silent card stats
    // -----------------------------------------------------------------------
    #[test]
    fn test_neutralize_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Neutralize", 0, 3, -1, 1, CardType::Attack);
        assert_card(&reg, "Neutralize+", 0, 4, -1, 2, CardType::Attack);
        assert_has_effect(&reg, "Neutralize", "weak");
    }

    #[test]
    fn test_wraith_form_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Wraith Form", 3, -1, -1, 2, CardType::Power);
        assert_card(&reg, "Wraith Form+", 3, -1, -1, 3, CardType::Power);
        assert_has_effect(&reg, "Wraith Form", "wraith_form");
    }

    #[test]
    fn test_deadly_poison_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Deadly Poison", 1, -1, -1, 5, CardType::Skill);
        assert_card(&reg, "Deadly Poison+", 1, -1, -1, 7, CardType::Skill);
        assert_has_effect(&reg, "Deadly Poison", "poison");
    }

    // Defect card registration tests
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_defect_cards_registered() {
        let reg = CardRegistry::new();
        let defect_cards = [
            // Basic
            "Strike_B", "Defend_B", "Zap", "Dualcast",
            // Common
            "Ball Lightning", "Barrage", "Beam Cell", "Cold Snap",
            "Compile Driver", "Conserve Battery", "Coolheaded",
            "Go for the Eyes", "Hologram", "Leap", "Rebound",
            "Stack", "Steam", "Streamline", "Sweeping Beam", "Turbo", "Gash",
            // Uncommon
            "Aggregate", "Auto Shields", "Blizzard", "BootSequence",
            "Capacitor", "Chaos", "Chill", "Consume", "Darkness",
            "Defragment", "Doom and Gloom", "Double Energy", "Undo",
            "Force Field", "FTL", "Fusion", "Genetic Algorithm", "Glacier",
            "Heatsinks", "Hello World", "Impulse", "Lockon", "Loop",
            "Melter", "Steam Power", "Recycle", "Redo",
            "Reinforced Body", "Reprogram", "Rip and Tear", "Scrape",
            "Self Repair", "Skim", "Static Discharge", "Storm",
            "Sunder", "Tempest", "White Noise",
            // Rare
            "All For One", "Amplify", "Biased Cognition", "Buffer",
            "Core Surge", "Creative AI", "Echo Form", "Electrodynamics",
            "Fission", "Hyperbeam", "Machine Learning", "Meteor Strike",
            "Multi-Cast", "Rainbow", "Reboot", "Seek", "Thunder Strike",
        ];
        for id in &defect_cards {
            assert!(reg.get(id).is_some(), "Defect card '{}' missing", id);
            let upgraded = format!("{}+", id);
            assert!(reg.get(&upgraded).is_some(), "Defect card '{}' missing", upgraded);
        }
    }

    #[test]
    fn test_defect_orb_effects() {
        let reg = CardRegistry::new();
        assert_has_effect(&reg, "Zap", "channel_lightning");
        assert_has_effect(&reg, "Ball Lightning", "channel_lightning");
        assert_has_effect(&reg, "Cold Snap", "channel_frost");
        assert_has_effect(&reg, "Coolheaded", "channel_frost");
        assert_has_effect(&reg, "Darkness", "channel_dark");
        assert_has_effect(&reg, "Fusion", "channel_plasma");
        assert_has_effect(&reg, "Dualcast", "evoke_orb");
        assert_has_effect(&reg, "Defragment", "gain_focus");
    }

    #[test]
    fn test_defect_card_stats() {
        let reg = CardRegistry::new();
        // Basic
        assert_card(&reg, "Strike_B", 1, 6, -1, -1, CardType::Attack);
        assert_card(&reg, "Strike_B+", 1, 9, -1, -1, CardType::Attack);
        assert_card(&reg, "Defend_B", 1, -1, 5, -1, CardType::Skill);
        assert_card(&reg, "Defend_B+", 1, -1, 8, -1, CardType::Skill);
        assert_card(&reg, "Zap", 1, -1, -1, 1, CardType::Skill);
        assert_card(&reg, "Zap+", 0, -1, -1, 1, CardType::Skill);
        assert_card(&reg, "Dualcast", 1, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "Dualcast+", 0, -1, -1, -1, CardType::Skill);
        // Key uncommon/rare
        assert_card(&reg, "Glacier", 2, -1, 7, 2, CardType::Skill);
        assert_card(&reg, "Glacier+", 2, -1, 10, 2, CardType::Skill);
        assert_card(&reg, "Hyperbeam", 2, 26, -1, 3, CardType::Attack);
        assert_card(&reg, "Hyperbeam+", 2, 34, -1, 3, CardType::Attack);
        assert_card(&reg, "Echo Form", 3, -1, -1, -1, CardType::Power);
        assert_has_effect(&reg, "Echo Form", "ethereal");
        assert!(!reg.get("Echo Form+").unwrap().effects.contains(&"ethereal"));
        assert_card(&reg, "Meteor Strike", 5, 24, -1, 3, CardType::Attack);
        assert_card(&reg, "Biased Cognition", 1, -1, -1, 4, CardType::Power);
        assert_card(&reg, "Biased Cognition+", 1, -1, -1, 5, CardType::Power);
    }

    // -----------------------------------------------------------------------
    // Colorless card registration tests
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_colorless_cards_registered() {
        let reg = CardRegistry::new();
        let colorless_cards = [
            // Uncommon
            "Bandage Up", "Blind", "Dark Shackles", "Deep Breath",
            "Discovery", "Dramatic Entrance", "Enlightenment", "Finesse",
            "Flash of Steel", "Forethought", "Good Instincts", "Impatience",
            "Jack Of All Trades", "Madness", "Mind Blast", "Panacea",
            "PanicButton", "Purity", "Swift Strike", "Trip",
            // Rare
            "Apotheosis", "Chrysalis", "HandOfGreed", "Magnetism",
            "Master of Strategy", "Mayhem", "Metamorphosis", "Panache",
            "Sadistic Nature", "Secret Technique", "Secret Weapon",
            "The Bomb", "Thinking Ahead", "Transmutation", "Violence",
            // Special
            "Ghostly", "Bite", "J.A.X.", "RitualDagger",
        ];
        for id in &colorless_cards {
            assert!(reg.get(id).is_some(), "Colorless card '{}' missing", id);
            let upgraded = format!("{}+", id);
            assert!(reg.get(&upgraded).is_some(), "Colorless card '{}' missing", upgraded);
        }
    }

    #[test]
    fn test_colorless_card_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Apotheosis", 2, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "Apotheosis+", 1, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "HandOfGreed", 2, 20, -1, 20, CardType::Attack);
        assert_card(&reg, "HandOfGreed+", 2, 25, -1, 25, CardType::Attack);
        assert_card(&reg, "Swift Strike", 0, 7, -1, -1, CardType::Attack);
        assert_card(&reg, "Ghostly", 1, -1, -1, -1, CardType::Skill);
        assert_has_effect(&reg, "Ghostly", "ethereal");
        assert!(!reg.get("Ghostly+").unwrap().effects.contains(&"ethereal"));
        assert_card(&reg, "Panache", 0, -1, -1, 10, CardType::Power);
        assert_card(&reg, "Panache+", 0, -1, -1, 14, CardType::Power);
    }

    // -----------------------------------------------------------------------
    // Curse card registration tests
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_curse_cards_registered() {
        let reg = CardRegistry::new();
        let curse_cards = [
            "AscendersBane", "Clumsy", "CurseOfTheBell", "Decay",
            "Doubt", "Injury", "Necronomicurse", "Normality",
            "Pain", "Parasite", "Pride", "Regret", "Shame", "Writhe",
        ];
        for id in &curse_cards {
            let card = reg.get(id).unwrap_or_else(|| panic!("Curse '{}' missing", id));
            assert_eq!(card.card_type, CardType::Curse, "{} should be Curse type", id);
            assert!(card.effects.contains(&"unplayable") || card.cost >= 0,
                "{} should be unplayable or have a cost", id);
        }
    }

    #[test]
    fn test_curse_effects() {
        let reg = CardRegistry::new();
        assert_has_effect(&reg, "Decay", "end_turn_damage");
        assert_has_effect(&reg, "Doubt", "end_turn_weak");
        assert_has_effect(&reg, "Shame", "end_turn_frail");
        assert_has_effect(&reg, "Normality", "limit_cards_per_turn");
        assert_has_effect(&reg, "Writhe", "innate");
        assert_has_effect(&reg, "Clumsy", "ethereal");
        assert_has_effect(&reg, "Necronomicurse", "unremovable");
    }

    // -----------------------------------------------------------------------
    // Status card registration tests
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_status_cards_registered() {
        let reg = CardRegistry::new();
        let status_cards = ["Slimed", "Wound", "Daze", "Burn", "Void"];
        for id in &status_cards {
            let card = reg.get(id).unwrap_or_else(|| panic!("Status '{}' missing", id));
            assert_eq!(card.card_type, CardType::Status, "{} should be Status type", id);
        }
    }

    #[test]
    fn test_status_effects() {
        let reg = CardRegistry::new();
        assert_has_effect(&reg, "Burn", "end_turn_damage");
        assert_eq!(reg.get("Burn").unwrap().base_magic, 2);
        assert_has_effect(&reg, "Burn+", "end_turn_damage");
        assert_eq!(reg.get("Burn+").unwrap().base_magic, 4);
        assert_has_effect(&reg, "Void", "lose_energy_on_draw");
        assert_has_effect(&reg, "Void", "ethereal");
        assert_has_effect(&reg, "Daze", "ethereal");
    }

    // -----------------------------------------------------------------------
    // Temp card registration tests
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_temp_cards_registered() {
        let reg = CardRegistry::new();
        let temp_cards = [
            "Miracle", "Smite", "Beta", "Omega", "Expunger",
            "Insight", "Safety", "ThroughViolence", "Shiv",
        ];
        for id in &temp_cards {
            assert!(reg.get(id).is_some(), "Temp card '{}' missing", id);
            let upgraded = format!("{}+", id);
            assert!(reg.get(&upgraded).is_some(), "Temp card '{}' missing", upgraded);
        }
    }

    #[test]
    fn test_temp_card_stats() {
        let reg = CardRegistry::new();
        assert_card(&reg, "Beta", 2, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "Beta+", 1, -1, -1, -1, CardType::Skill);
        assert_card(&reg, "Omega", 3, -1, -1, 50, CardType::Power);
        assert_card(&reg, "Omega+", 3, -1, -1, 60, CardType::Power);
        assert_card(&reg, "Shiv", 0, 4, -1, -1, CardType::Attack);
        assert_card(&reg, "Shiv+", 0, 6, -1, -1, CardType::Attack);
        assert!(reg.get("Shiv").unwrap().exhaust);
        assert_card(&reg, "Safety", 1, -1, 12, -1, CardType::Skill);
        assert_card(&reg, "Safety+", 1, -1, 16, -1, CardType::Skill);
        assert_has_effect(&reg, "Safety", "retain");
        assert_card(&reg, "ThroughViolence", 0, 20, -1, -1, CardType::Attack);
        assert_card(&reg, "ThroughViolence+", 0, 30, -1, -1, CardType::Attack);
        assert_has_effect(&reg, "ThroughViolence", "retain");
    }
}
