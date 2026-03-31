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
            effects: &["multi_hit", "shuffle_self_into_draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Tantrum+", name: "Tantrum+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 3, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: Some("Wrath"),
            effects: &["multi_hit", "shuffle_self_into_draw"],
        });

        // ---- Common: Consecrate ---- (cost 0, 5 dmg AoE, +3 upgrade)
        Self::insert(&mut cards, CardDef {
            id: "Consecrate", name: "Consecrate", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 0, base_damage: 5, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Consecrate+", name: "Consecrate+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 0, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });

        // ---- Common: Crescendo ---- (cost 1, enter Wrath, exhaust, retain; upgrade: cost 0)
        Self::insert(&mut cards, CardDef {
            id: "Crescendo", name: "Crescendo", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Wrath"),
            effects: &["retain"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Crescendo+", name: "Crescendo+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Wrath"),
            effects: &["retain"],
        });

        // ---- Common: Empty Fist ---- (cost 1, 9 dmg, exit stance; +5 upgrade)
        Self::insert(&mut cards, CardDef {
            id: "EmptyFist", name: "Empty Fist", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["exit_stance"],
        });
        Self::insert(&mut cards, CardDef {
            id: "EmptyFist+", name: "Empty Fist+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 14, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["exit_stance"],
        });

        // ---- Common: Evaluate ---- (cost 1, 6 block, add Insight to draw; +4 block upgrade)
        Self::insert(&mut cards, CardDef {
            id: "Evaluate", name: "Evaluate", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 6,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["insight_to_draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Evaluate+", name: "Evaluate+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 10,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["insight_to_draw"],
        });

        // ---- Common: Just Lucky ---- (cost 0, 3 dmg, 2 block, scry 1; +1/+1/+1 upgrade)
        Self::insert(&mut cards, CardDef {
            id: "JustLucky", name: "Just Lucky", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: 2,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["scry"],
        });
        Self::insert(&mut cards, CardDef {
            id: "JustLucky+", name: "Just Lucky+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: 3,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["scry"],
        });

        // ---- Common: Pressure Points ---- (cost 1, skill, apply 8 Mark, trigger; +3 upgrade)
        // Java ID: PathToVictory, run.rs uses PressurePoints
        Self::insert(&mut cards, CardDef {
            id: "PressurePoints", name: "Pressure Points", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 8, exhaust: false, enter_stance: None,
            effects: &["pressure_points"],
        });
        Self::insert(&mut cards, CardDef {
            id: "PressurePoints+", name: "Pressure Points+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 11, exhaust: false, enter_stance: None,
            effects: &["pressure_points"],
        });

        // ---- Common: Protect ---- (cost 2, 12 block, retain; +4 upgrade)
        Self::insert(&mut cards, CardDef {
            id: "Protect", name: "Protect", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 12,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["retain"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Protect+", name: "Protect+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 16,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["retain"],
        });

        // ---- Common: Sash Whip ---- (cost 1, 8 dmg, weak 1 if last attack; +2 dmg +1 magic upgrade)
        Self::insert(&mut cards, CardDef {
            id: "SashWhip", name: "Sash Whip", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["weak_if_last_attack"],
        });
        Self::insert(&mut cards, CardDef {
            id: "SashWhip+", name: "Sash Whip+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["weak_if_last_attack"],
        });

        // ---- Common: Tranquility ---- (cost 1, enter Calm, exhaust, retain; upgrade: cost 0)
        // Java ID: ClearTheMind, run.rs uses Tranquility
        Self::insert(&mut cards, CardDef {
            id: "Tranquility", name: "Tranquility", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Calm"),
            effects: &["retain"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Tranquility+", name: "Tranquility+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Calm"),
            effects: &["retain"],
        });

        // ---- Common Watcher Cards (continued) ----
        Self::insert(&mut cards, CardDef {
            id: "ThirdEye", name: "Third Eye", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["scry"],
        });
        Self::insert(&mut cards, CardDef {
            id: "ThirdEye+", name: "Third Eye+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 9,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["scry"],
        });

        // ---- Uncommon Watcher Cards ----
        Self::insert(&mut cards, CardDef {
            id: "InnerPeace", name: "Inner Peace", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["if_calm_draw_else_calm"],
        });
        Self::insert(&mut cards, CardDef {
            id: "InnerPeace+", name: "Inner Peace+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["if_calm_draw_else_calm"],
        });
        Self::insert(&mut cards, CardDef {
            id: "WheelKick", name: "Wheel Kick", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 15, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "WheelKick+", name: "Wheel Kick+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 20, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw"],
        });
        // ---- Uncommon: Battle Hymn ---- (cost 1, power, add Smite to hand each turn; upgrade: innate)
        Self::insert(&mut cards, CardDef {
            id: "BattleHymn", name: "Battle Hymn", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["battle_hymn"],
        });
        Self::insert(&mut cards, CardDef {
            id: "BattleHymn+", name: "Battle Hymn+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["battle_hymn", "innate"],
        });

        // ---- Uncommon: Carve Reality ---- (cost 1, 6 dmg, add Smite to hand; +4 dmg upgrade)
        Self::insert(&mut cards, CardDef {
            id: "CarveReality", name: "Carve Reality", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_smite_to_hand"],
        });
        Self::insert(&mut cards, CardDef {
            id: "CarveReality+", name: "Carve Reality+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_smite_to_hand"],
        });

        // ---- Uncommon: Deceive Reality ---- (cost 1, 4 block, add Safety to hand; +3 block upgrade)
        Self::insert(&mut cards, CardDef {
            id: "DeceiveReality", name: "Deceive Reality", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 4,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_safety_to_hand"],
        });
        Self::insert(&mut cards, CardDef {
            id: "DeceiveReality+", name: "Deceive Reality+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_safety_to_hand"],
        });

        // ---- Uncommon: Empty Mind ---- (cost 1, draw 2, exit stance; +1 draw upgrade)
        Self::insert(&mut cards, CardDef {
            id: "EmptyMind", name: "Empty Mind", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["draw", "exit_stance"],
        });
        Self::insert(&mut cards, CardDef {
            id: "EmptyMind+", name: "Empty Mind+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["draw", "exit_stance"],
        });

        // ---- Uncommon: Fear No Evil ---- (cost 1, 8 dmg, enter Calm if enemy attacking; +3 dmg upgrade)
        Self::insert(&mut cards, CardDef {
            id: "FearNoEvil", name: "Fear No Evil", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["calm_if_enemy_attacking"],
        });
        Self::insert(&mut cards, CardDef {
            id: "FearNoEvil+", name: "Fear No Evil+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["calm_if_enemy_attacking"],
        });

        // ---- Uncommon: Foreign Influence ---- (cost 0, skill, exhaust, choose attack from other class; upgrade: upgraded choices)
        Self::insert(&mut cards, CardDef {
            id: "ForeignInfluence", name: "Foreign Influence", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["foreign_influence"],
        });
        Self::insert(&mut cards, CardDef {
            id: "ForeignInfluence+", name: "Foreign Influence+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["foreign_influence"],
        });

        // ---- Uncommon: Indignation ---- (cost 1, if in Wrath apply 3 vuln to all, else enter Wrath; +2 magic upgrade)
        Self::insert(&mut cards, CardDef {
            id: "Indignation", name: "Indignation", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["indignation"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Indignation+", name: "Indignation+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["indignation"],
        });

        // ---- Uncommon: Like Water ---- (cost 1, power, if in Calm at end of turn gain 5 block; +2 magic upgrade)
        Self::insert(&mut cards, CardDef {
            id: "LikeWater", name: "Like Water", card_type: CardType::Power,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["like_water"],
        });
        Self::insert(&mut cards, CardDef {
            id: "LikeWater+", name: "Like Water+", card_type: CardType::Power,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 7, exhaust: false, enter_stance: None,
            effects: &["like_water"],
        });

        // ---- Uncommon: Meditate ---- (cost 1, put 1 card from discard into hand + retain it, enter Calm, end turn; +1 magic upgrade)
        Self::insert(&mut cards, CardDef {
            id: "Meditate", name: "Meditate", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: Some("Calm"),
            effects: &["meditate", "end_turn"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Meditate+", name: "Meditate+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: Some("Calm"),
            effects: &["meditate", "end_turn"],
        });

        // ---- Uncommon: Nirvana ---- (cost 1, power, gain 3 block whenever you Scry; +1 magic upgrade)
        Self::insert(&mut cards, CardDef {
            id: "Nirvana", name: "Nirvana", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["on_scry_block"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Nirvana+", name: "Nirvana+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["on_scry_block"],
        });

        // ---- Uncommon: Perseverance ---- (cost 1, 5 block, retain, block grows by 2 each retain; +2 block +1 magic upgrade)
        Self::insert(&mut cards, CardDef {
            id: "Perseverance", name: "Perseverance", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["retain", "grow_block_on_retain"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Perseverance+", name: "Perseverance+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["retain", "grow_block_on_retain"],
        });

        // ---- Uncommon: Reach Heaven ---- (cost 2, 10 dmg, shuffle Through Violence into draw; +5 dmg upgrade)
        Self::insert(&mut cards, CardDef {
            id: "ReachHeaven", name: "Reach Heaven", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_through_violence_to_draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "ReachHeaven+", name: "Reach Heaven+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 15, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_through_violence_to_draw"],
        });

        // ---- Uncommon: Sands of Time ---- (cost 4, 20 dmg, retain, cost -1 each retain; +6 dmg upgrade)
        Self::insert(&mut cards, CardDef {
            id: "SandsOfTime", name: "Sands of Time", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 4, base_damage: 20, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["retain", "reduce_cost_on_retain"],
        });
        Self::insert(&mut cards, CardDef {
            id: "SandsOfTime+", name: "Sands of Time+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 4, base_damage: 26, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["retain", "reduce_cost_on_retain"],
        });

        // ---- Uncommon: Signature Move ---- (cost 2, 30 dmg, only playable if no other attacks in hand; +10 dmg upgrade)
        Self::insert(&mut cards, CardDef {
            id: "SignatureMove", name: "Signature Move", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 30, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["only_attack_in_hand"],
        });
        Self::insert(&mut cards, CardDef {
            id: "SignatureMove+", name: "Signature Move+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 40, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["only_attack_in_hand"],
        });

        // ---- Uncommon: Study ---- (cost 2, power, add Insight to draw at end of turn; upgrade: cost 1)
        Self::insert(&mut cards, CardDef {
            id: "Study", name: "Study", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["study"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Study+", name: "Study+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["study"],
        });

        // ---- Uncommon: Swivel ---- (cost 2, 8 block, next attack costs 0; +3 block upgrade)
        Self::insert(&mut cards, CardDef {
            id: "Swivel", name: "Swivel", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["next_attack_free"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Swivel+", name: "Swivel+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 11,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["next_attack_free"],
        });

        // ---- Uncommon: Wallop ---- (cost 2, 9 dmg, gain block equal to unblocked damage; +3 dmg upgrade)
        Self::insert(&mut cards, CardDef {
            id: "Wallop", name: "Wallop", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_from_damage"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Wallop+", name: "Wallop+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_from_damage"],
        });

        // ---- Uncommon: Wave of the Hand ---- (cost 1, skill, whenever you gain block this turn apply 1 Weak; +1 magic upgrade)
        Self::insert(&mut cards, CardDef {
            id: "WaveOfTheHand", name: "Wave of the Hand", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["wave_of_the_hand"],
        });
        Self::insert(&mut cards, CardDef {
            id: "WaveOfTheHand+", name: "Wave of the Hand+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["wave_of_the_hand"],
        });

        // ---- Uncommon: Weave ---- (cost 0, 4 dmg, returns to hand on Scry; +2 dmg upgrade)
        Self::insert(&mut cards, CardDef {
            id: "Weave", name: "Weave", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["return_on_scry"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Weave+", name: "Weave+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["return_on_scry"],
        });

        // ---- Uncommon: Windmill Strike ---- (cost 2, 7 dmg, retain, +4 dmg each retain; +3 dmg +1 magic upgrade)
        Self::insert(&mut cards, CardDef {
            id: "WindmillStrike", name: "Windmill Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 7, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["retain", "grow_damage_on_retain"],
        });
        Self::insert(&mut cards, CardDef {
            id: "WindmillStrike+", name: "Windmill Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["retain", "grow_damage_on_retain"],
        });

        // ---- Uncommon: Wreath of Flame ---- (cost 1, gain 5 Vigor; +3 magic upgrade)
        Self::insert(&mut cards, CardDef {
            id: "WreathOfFlame", name: "Wreath of Flame", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["vigor"],
        });
        Self::insert(&mut cards, CardDef {
            id: "WreathOfFlame+", name: "Wreath of Flame+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 8, exhaust: false, enter_stance: None,
            effects: &["vigor"],
        });

        Self::insert(&mut cards, CardDef {
            id: "Conclude", name: "Conclude", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["end_turn"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Conclude+", name: "Conclude+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 16, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["end_turn"],
        });
        Self::insert(&mut cards, CardDef {
            id: "TalkToTheHand", name: "Talk to the Hand", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 5, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["apply_block_return"],
        });
        Self::insert(&mut cards, CardDef {
            id: "TalkToTheHand+", name: "Talk to the Hand+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["apply_block_return"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Pray", name: "Pray", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["mantra"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Pray+", name: "Pray+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["mantra"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Worship", name: "Worship", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["mantra"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Worship+", name: "Worship+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["mantra", "retain"],
        });

        // ---- Power Cards ----
        Self::insert(&mut cards, CardDef {
            id: "Adaptation", name: "Rushdown", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["on_wrath_draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Adaptation+", name: "Rushdown+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["on_wrath_draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "MentalFortress", name: "Mental Fortress", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["on_stance_change_block"],
        });
        Self::insert(&mut cards, CardDef {
            id: "MentalFortress+", name: "Mental Fortress+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 6, exhaust: false, enter_stance: None,
            effects: &["on_stance_change_block"],
        });

        // ---- Rare Watcher Cards ----
        Self::insert(&mut cards, CardDef {
            id: "Ragnarok", name: "Ragnarok", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 3, base_damage: 5, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: Some("Wrath"),
            effects: &["damage_random_x_times"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Ragnarok+", name: "Ragnarok+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 3, base_damage: 6, base_block: -1,
            base_magic: 6, exhaust: false, enter_stance: Some("Wrath"),
            effects: &["damage_random_x_times"],
        });

        // ---- Rare: Alpha ---- (cost 1, skill, exhaust, shuffle Beta into draw; upgrade: innate)
        Self::insert(&mut cards, CardDef {
            id: "Alpha", name: "Alpha", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["add_beta_to_draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Alpha+", name: "Alpha+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["add_beta_to_draw", "innate"],
        });

        // ---- Rare: Blasphemy ---- (cost 1, skill, exhaust, enter Divinity, die next turn; upgrade: retain)
        Self::insert(&mut cards, CardDef {
            id: "Blasphemy", name: "Blasphemy", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Divinity"),
            effects: &["die_next_turn"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Blasphemy+", name: "Blasphemy+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Divinity"),
            effects: &["die_next_turn", "retain"],
        });

        // ---- Rare: Brilliance ---- (cost 1, 12 dmg + mantra gained this combat; +4 dmg upgrade)
        Self::insert(&mut cards, CardDef {
            id: "Brilliance", name: "Brilliance", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["damage_plus_mantra"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Brilliance+", name: "Brilliance+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 16, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["damage_plus_mantra"],
        });

        // ---- Rare: Conjure Blade ---- (cost X, skill, exhaust, create Expunger with X hits; upgrade: X+1 hits)
        Self::insert(&mut cards, CardDef {
            id: "ConjureBlade", name: "Conjure Blade", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["conjure_blade"], // TODO: full X-cost + Expunger creation
        });
        Self::insert(&mut cards, CardDef {
            id: "ConjureBlade+", name: "Conjure Blade+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["conjure_blade"],
        });

        // ---- Rare: Deva Form ---- (cost 3, power, ethereal, gain 1 energy each turn (stacks); upgrade: no ethereal)
        Self::insert(&mut cards, CardDef {
            id: "DevaForm", name: "Deva Form", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["deva_form", "ethereal"],
        });
        Self::insert(&mut cards, CardDef {
            id: "DevaForm+", name: "Deva Form+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["deva_form"],
        });

        // ---- Rare: Devotion ---- (cost 1, power, gain 2 Mantra at start of each turn; +1 magic upgrade)
        Self::insert(&mut cards, CardDef {
            id: "Devotion", name: "Devotion", card_type: CardType::Power,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["devotion"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Devotion+", name: "Devotion+", card_type: CardType::Power,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["devotion"],
        });

        // ---- Rare: Establishment ---- (cost 1, power, retained cards cost 1 less; upgrade: innate)
        Self::insert(&mut cards, CardDef {
            id: "Establishment", name: "Establishment", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["establishment"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Establishment+", name: "Establishment+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["establishment", "innate"],
        });

        // ---- Rare (listed): Fasting ---- (Java: Uncommon, cost 2, power, +3 str/dex, -1 energy; +1 magic upgrade)
        Self::insert(&mut cards, CardDef {
            id: "Fasting", name: "Fasting", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["fasting"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Fasting+", name: "Fasting+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["fasting"],
        });

        // ---- Rare: Judgement ---- (cost 1, skill, if enemy HP <= 30, kill it; +10 magic upgrade)
        Self::insert(&mut cards, CardDef {
            id: "Judgement", name: "Judgement", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 30, exhaust: false, enter_stance: None,
            effects: &["judgement"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Judgement+", name: "Judgement+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 40, exhaust: false, enter_stance: None,
            effects: &["judgement"],
        });

        // ---- Rare: Lesson Learned ---- (cost 2, 10 dmg, exhaust, if kill upgrade a random card; +3 dmg upgrade)
        Self::insert(&mut cards, CardDef {
            id: "LessonLearned", name: "Lesson Learned", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["lesson_learned"],
        });
        Self::insert(&mut cards, CardDef {
            id: "LessonLearned+", name: "Lesson Learned+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 13, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["lesson_learned"],
        });

        // ---- Rare: Master Reality ---- (cost 1, power, created cards are upgraded; upgrade: cost 0)
        Self::insert(&mut cards, CardDef {
            id: "MasterReality", name: "Master Reality", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["master_reality"],
        });
        Self::insert(&mut cards, CardDef {
            id: "MasterReality+", name: "Master Reality+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["master_reality"],
        });

        // ---- Rare: Omniscience ---- (cost 4, skill, exhaust, choose card from draw pile play it twice; upgrade: cost 3)
        // TODO: Full effect requires choosing a card from draw pile and playing it twice
        Self::insert(&mut cards, CardDef {
            id: "Omniscience", name: "Omniscience", card_type: CardType::Skill,
            target: CardTarget::None, cost: 4, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["omniscience"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Omniscience+", name: "Omniscience+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["omniscience"],
        });

        // ---- Rare: Scrawl ---- (cost 1, skill, exhaust, draw until you have 10 cards; upgrade: cost 0)
        Self::insert(&mut cards, CardDef {
            id: "Scrawl", name: "Scrawl", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["draw_to_ten"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Scrawl+", name: "Scrawl+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["draw_to_ten"],
        });

        // ---- Rare: Spirit Shield ---- (cost 2, skill, gain 3 block per card in hand; +1 magic upgrade)
        Self::insert(&mut cards, CardDef {
            id: "SpiritShield", name: "Spirit Shield", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["block_per_card_in_hand"],
        });
        Self::insert(&mut cards, CardDef {
            id: "SpiritShield+", name: "Spirit Shield+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["block_per_card_in_hand"],
        });

        // ---- Rare: Vault ---- (cost 3, skill, exhaust, skip enemy turn, end turn; upgrade: cost 2)
        Self::insert(&mut cards, CardDef {
            id: "Vault", name: "Vault", card_type: CardType::Skill,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["skip_enemy_turn", "end_turn"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Vault+", name: "Vault+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["skip_enemy_turn", "end_turn"],
        });

        // ---- Rare: Wish ---- (cost 3, skill, exhaust, choose: +3 str, or 25 gold, or 6 block; upgrade: +1/+5/+2)
        // TODO: Full effect requires ChooseOne UI (BecomeAlmighty, FameAndFortune, LiveForever)
        Self::insert(&mut cards, CardDef {
            id: "Wish", name: "Wish", card_type: CardType::Skill,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["wish"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Wish+", name: "Wish+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: true, enter_stance: None,
            effects: &["wish"],
        });

        // ---- Special Cards ----
        Self::insert(&mut cards, CardDef {
            id: "Miracle", name: "Miracle", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["gain_energy"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Miracle+", name: "Miracle+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["gain_energy"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Smite", name: "Smite", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Smite+", name: "Smite+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 16, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"],
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
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["unplayable", "end_turn_damage"],
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

        // ====================================================================
        // DEFECT (Blue) Cards
        // ====================================================================

        // ---- Defect Basic Cards ----
        Self::insert(&mut cards, CardDef {
            id: "Strike_B", name: "Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Strike_B+", name: "Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Defend_B", name: "Defend", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Defend_B+", name: "Defend+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Zap", name: "Zap", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_lightning"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Zap+", name: "Zap+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_lightning"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Dualcast", name: "Dualcast", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["evoke_orb", "evoke_orb"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Dualcast+", name: "Dualcast+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["evoke_orb", "evoke_orb"],
        });

        // ---- Defect Common Cards ----
        // Ball Lightning: 1 cost, 7 dmg, channel 1 Lightning
        Self::insert(&mut cards, CardDef {
            id: "Ball Lightning", name: "Ball Lightning", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_lightning"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Ball Lightning+", name: "Ball Lightning+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_lightning"],
        });
        // Barrage: 1 cost, 4 dmg x orbs
        Self::insert(&mut cards, CardDef {
            id: "Barrage", name: "Barrage", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 4, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_per_orb"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Barrage+", name: "Barrage+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_per_orb"],
        });
        // Beam Cell: 0 cost, 3 dmg, 1 vuln
        Self::insert(&mut cards, CardDef {
            id: "Beam Cell", name: "Beam Cell", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["apply_vulnerable"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Beam Cell+", name: "Beam Cell+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["apply_vulnerable"],
        });
        // Cold Snap: 1 cost, 6 dmg, channel 1 Frost
        Self::insert(&mut cards, CardDef {
            id: "Cold Snap", name: "Cold Snap", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_frost"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Cold Snap+", name: "Cold Snap+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_frost"],
        });
        // Compile Driver: 1 cost, 7 dmg, draw 1 per unique orb
        Self::insert(&mut cards, CardDef {
            id: "Compile Driver", name: "Compile Driver", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["draw_per_unique_orb"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Compile Driver+", name: "Compile Driver+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["draw_per_unique_orb"],
        });
        // Conserve Battery: 1 cost, 7 block, next turn gain 1 energy (via Energized)
        Self::insert(&mut cards, CardDef {
            id: "Conserve Battery", name: "Conserve Battery", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["next_turn_energy"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Conserve Battery+", name: "Conserve Battery+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 10,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["next_turn_energy"],
        });
        // Coolheaded: 1 cost, channel Frost, draw 1
        Self::insert(&mut cards, CardDef {
            id: "Coolheaded", name: "Coolheaded", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_frost", "draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Coolheaded+", name: "Coolheaded+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["channel_frost", "draw"],
        });
        // Go for the Eyes: 0 cost, 3 dmg, apply Weak if attacking
        Self::insert(&mut cards, CardDef {
            id: "Go for the Eyes", name: "Go for the Eyes", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["weak_if_attacking"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Go for the Eyes+", name: "Go for the Eyes+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["weak_if_attacking"],
        });
        // Hologram: 1 cost, 3 block, put card from discard into hand, exhaust (upgrade: no exhaust)
        Self::insert(&mut cards, CardDef {
            id: "Hologram", name: "Hologram", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 3,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["return_from_discard"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Hologram+", name: "Hologram+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["return_from_discard"],
        });
        // Leap: 1 cost, 9 block
        Self::insert(&mut cards, CardDef {
            id: "Leap", name: "Leap", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 9,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Leap+", name: "Leap+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 12,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        // Rebound: 1 cost, 9 dmg, next card drawn goes to top of draw pile
        Self::insert(&mut cards, CardDef {
            id: "Rebound", name: "Rebound", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["next_card_to_top"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Rebound+", name: "Rebound+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["next_card_to_top"],
        });
        // Stack: 1 cost, block = discard pile size (upgrade: +3)
        Self::insert(&mut cards, CardDef {
            id: "Stack", name: "Stack", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 0,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_from_discard"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Stack+", name: "Stack+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 3,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_from_discard"],
        });
        // Steam Barrier (SteamBarrier): 0 cost, 6 block, loses 1 block each play
        Self::insert(&mut cards, CardDef {
            id: "Steam", name: "Steam Barrier", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 6,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["lose_block_each_play"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Steam+", name: "Steam Barrier+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["lose_block_each_play"],
        });
        // Streamline: 2 cost, 15 dmg, costs 1 less each play
        Self::insert(&mut cards, CardDef {
            id: "Streamline", name: "Streamline", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 15, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["reduce_cost_each_play"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Streamline+", name: "Streamline+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 20, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["reduce_cost_each_play"],
        });
        // Sweeping Beam: 1 cost, 6 dmg AoE, draw 1
        Self::insert(&mut cards, CardDef {
            id: "Sweeping Beam", name: "Sweeping Beam", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Sweeping Beam+", name: "Sweeping Beam+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["draw"],
        });
        // Turbo: 0 cost, gain 2 energy, add Void to discard
        Self::insert(&mut cards, CardDef {
            id: "Turbo", name: "Turbo", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["gain_energy", "add_void_to_discard"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Turbo+", name: "Turbo+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["gain_energy", "add_void_to_discard"],
        });
        // Claw (Java ID: Gash): 0 cost, 3 dmg, all Claw dmg +2 for rest of combat
        Self::insert(&mut cards, CardDef {
            id: "Gash", name: "Claw", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["claw_scaling"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Gash+", name: "Claw+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 5, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["claw_scaling"],
        });

        // ---- Defect Uncommon Cards ----
        // Aggregate: 1 cost, gain 1 energy per 4 cards in draw pile (upgrade: per 3)
        Self::insert(&mut cards, CardDef {
            id: "Aggregate", name: "Aggregate", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["energy_per_cards_in_draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Aggregate+", name: "Aggregate+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["energy_per_cards_in_draw"],
        });
        // Auto Shields: 1 cost, 11 block only if no block
        Self::insert(&mut cards, CardDef {
            id: "Auto Shields", name: "Auto-Shields", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 11,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_if_no_block"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Auto Shields+", name: "Auto-Shields+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 15,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_if_no_block"],
        });
        // Blizzard: 1 cost, dmg = 2 * frost channeled this combat, AoE
        Self::insert(&mut cards, CardDef {
            id: "Blizzard", name: "Blizzard", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 0, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["damage_per_frost_channeled"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Blizzard+", name: "Blizzard+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 0, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["damage_per_frost_channeled"],
        });
        // Boot Sequence: 0 cost, 10 block, innate, exhaust
        Self::insert(&mut cards, CardDef {
            id: "BootSequence", name: "Boot Sequence", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 10,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["innate"],
        });
        Self::insert(&mut cards, CardDef {
            id: "BootSequence+", name: "Boot Sequence+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 13,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["innate"],
        });
        // Capacitor: 1 cost, power, gain 2 orb slots
        Self::insert(&mut cards, CardDef {
            id: "Capacitor", name: "Capacitor", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["gain_orb_slots"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Capacitor+", name: "Capacitor+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["gain_orb_slots"],
        });
        // Chaos: 1 cost, channel 1 random orb (upgrade: 2)
        Self::insert(&mut cards, CardDef {
            id: "Chaos", name: "Chaos", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_random"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Chaos+", name: "Chaos+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["channel_random"],
        });
        // Chill: 0 cost, channel 1 Frost per enemy, exhaust (upgrade: innate)
        Self::insert(&mut cards, CardDef {
            id: "Chill", name: "Chill", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["channel_frost_per_enemy"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Chill+", name: "Chill+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["channel_frost_per_enemy", "innate"],
        });
        // Consume: 2 cost, remove 1 orb slot, gain 2 focus
        Self::insert(&mut cards, CardDef {
            id: "Consume", name: "Consume", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["gain_focus", "lose_orb_slot"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Consume+", name: "Consume+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["gain_focus", "lose_orb_slot"],
        });
        // Darkness: 1 cost, channel 1 Dark (upgrade: also trigger Dark passive)
        Self::insert(&mut cards, CardDef {
            id: "Darkness", name: "Darkness", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_dark"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Darkness+", name: "Darkness+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_dark", "trigger_dark_passive"],
        });
        // Defragment: 1 cost, power, gain 1 focus
        Self::insert(&mut cards, CardDef {
            id: "Defragment", name: "Defragment", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["gain_focus"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Defragment+", name: "Defragment+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["gain_focus"],
        });
        // Doom and Gloom: 2 cost, 10 dmg AoE, channel 1 Dark
        Self::insert(&mut cards, CardDef {
            id: "Doom and Gloom", name: "Doom and Gloom", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 2, base_damage: 10, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_dark"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Doom and Gloom+", name: "Doom and Gloom+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 2, base_damage: 14, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_dark"],
        });
        // Double Energy: 1 cost, double your energy, exhaust (upgrade: cost 0)
        Self::insert(&mut cards, CardDef {
            id: "Double Energy", name: "Double Energy", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["double_energy"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Double Energy+", name: "Double Energy+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["double_energy"],
        });
        // Equilibrium (Java ID: Undo): 2 cost, 13 block, retain hand this turn
        Self::insert(&mut cards, CardDef {
            id: "Undo", name: "Equilibrium", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 13,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["retain_hand"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Undo+", name: "Equilibrium+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 16,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["retain_hand"],
        });
        // Force Field: 4 cost, 12 block, costs 1 less per power played
        Self::insert(&mut cards, CardDef {
            id: "Force Field", name: "Force Field", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 4, base_damage: -1, base_block: 12,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["reduce_cost_per_power"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Force Field+", name: "Force Field+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 4, base_damage: -1, base_block: 16,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["reduce_cost_per_power"],
        });
        // FTL: 0 cost, 5 dmg, draw 1 if <3 cards played this turn
        Self::insert(&mut cards, CardDef {
            id: "FTL", name: "FTL", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 5, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["draw_if_few_cards_played"],
        });
        Self::insert(&mut cards, CardDef {
            id: "FTL+", name: "FTL+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["draw_if_few_cards_played"],
        });
        // Fusion: 2 cost, channel 1 Plasma (upgrade: cost 1)
        Self::insert(&mut cards, CardDef {
            id: "Fusion", name: "Fusion", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_plasma"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Fusion+", name: "Fusion+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_plasma"],
        });
        // Genetic Algorithm: 1 cost, block from misc (starts 0), grows +2 per combat, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Genetic Algorithm", name: "Genetic Algorithm", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 0,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["genetic_algorithm"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Genetic Algorithm+", name: "Genetic Algorithm+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 0,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["genetic_algorithm"],
        });
        // Glacier: 2 cost, 7 block, channel 2 Frost
        Self::insert(&mut cards, CardDef {
            id: "Glacier", name: "Glacier", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 7,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["channel_frost"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Glacier+", name: "Glacier+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 10,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["channel_frost"],
        });
        // Heatsinks: 1 cost, power, whenever you play a power draw 1 card
        Self::insert(&mut cards, CardDef {
            id: "Heatsinks", name: "Heatsinks", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["draw_on_power_play"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Heatsinks+", name: "Heatsinks+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw_on_power_play"],
        });
        // Hello World: 1 cost, power, add random common card to hand each turn (upgrade: innate)
        Self::insert(&mut cards, CardDef {
            id: "Hello World", name: "Hello World", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["hello_world"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Hello World+", name: "Hello World+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["hello_world", "innate"],
        });
        // Impulse: 1 cost, trigger all orb passives, exhaust (upgrade: no exhaust)
        Self::insert(&mut cards, CardDef {
            id: "Impulse", name: "Impulse", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["trigger_all_passives"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Impulse+", name: "Impulse+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["trigger_all_passives"],
        });
        // Lock-On (Java ID: Lockon): 1 cost, 8 dmg, apply 2 Lock-On
        Self::insert(&mut cards, CardDef {
            id: "Lockon", name: "Lock-On", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["apply_lock_on"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Lockon+", name: "Lock-On+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["apply_lock_on"],
        });
        // Loop: 1 cost, power, trigger frontmost orb passive at start of turn
        Self::insert(&mut cards, CardDef {
            id: "Loop", name: "Loop", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["loop_orb"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Loop+", name: "Loop+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["loop_orb"],
        });
        // Melter: 1 cost, 10 dmg, remove all enemy block
        Self::insert(&mut cards, CardDef {
            id: "Melter", name: "Melter", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["remove_enemy_block"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Melter+", name: "Melter+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 14, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["remove_enemy_block"],
        });
        // Overclock (Java ID: Steam Power): 0 cost, draw 2, add Burn to discard
        Self::insert(&mut cards, CardDef {
            id: "Steam Power", name: "Overclock", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw", "add_burn_to_discard"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Steam Power+", name: "Overclock+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["draw", "add_burn_to_discard"],
        });
        // Recycle: 1 cost, exhaust a card, gain energy equal to its cost (upgrade: cost 0)
        Self::insert(&mut cards, CardDef {
            id: "Recycle", name: "Recycle", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["recycle"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Recycle+", name: "Recycle+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["recycle"],
        });
        // Recursion (Java ID: Redo): 1 cost, evoke frontmost, channel it back (upgrade: cost 0)
        Self::insert(&mut cards, CardDef {
            id: "Redo", name: "Recursion", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["evoke_orb", "channel_evoked"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Redo+", name: "Recursion+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["evoke_orb", "channel_evoked"],
        });
        // Reinforced Body: X cost, gain 7 block X times
        Self::insert(&mut cards, CardDef {
            id: "Reinforced Body", name: "Reinforced Body", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: 7,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_x_times"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Reinforced Body+", name: "Reinforced Body+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: 9,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_x_times"],
        });
        // Reprogram: 1 cost, lose 1 focus, gain 1 str and 1 dex
        Self::insert(&mut cards, CardDef {
            id: "Reprogram", name: "Reprogram", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["reprogram"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Reprogram+", name: "Reprogram+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["reprogram"],
        });
        // Rip and Tear: 1 cost, deal 7 dmg twice to random enemies
        Self::insert(&mut cards, CardDef {
            id: "Rip and Tear", name: "Rip and Tear", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["damage_random_x_times"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Rip and Tear+", name: "Rip and Tear+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["damage_random_x_times"],
        });
        // Scrape: 1 cost, 7 dmg, draw 4 then discard non-0-cost cards drawn
        Self::insert(&mut cards, CardDef {
            id: "Scrape", name: "Scrape", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["draw_discard_non_zero"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Scrape+", name: "Scrape+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["draw_discard_non_zero"],
        });
        // Self Repair: 1 cost, power, heal 7 HP at end of combat
        Self::insert(&mut cards, CardDef {
            id: "Self Repair", name: "Self Repair", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 7, exhaust: false, enter_stance: None,
            effects: &["heal_end_of_combat"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Self Repair+", name: "Self Repair+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 10, exhaust: false, enter_stance: None,
            effects: &["heal_end_of_combat"],
        });
        // Skim: 1 cost, draw 3 cards
        Self::insert(&mut cards, CardDef {
            id: "Skim", name: "Skim", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Skim+", name: "Skim+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["draw"],
        });
        // Static Discharge: 1 cost, power, channel 1 Lightning whenever you take unblocked damage
        Self::insert(&mut cards, CardDef {
            id: "Static Discharge", name: "Static Discharge", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_lightning_on_damage"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Static Discharge+", name: "Static Discharge+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["channel_lightning_on_damage"],
        });
        // Storm: 1 cost, power, channel 1 Lightning on power play (upgrade: innate)
        Self::insert(&mut cards, CardDef {
            id: "Storm", name: "Storm", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_lightning_on_power"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Storm+", name: "Storm+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["channel_lightning_on_power", "innate"],
        });
        // Sunder: 3 cost, 24 dmg, gain 3 energy if this kills
        Self::insert(&mut cards, CardDef {
            id: "Sunder", name: "Sunder", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 3, base_damage: 24, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["energy_on_kill"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Sunder+", name: "Sunder+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 3, base_damage: 32, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["energy_on_kill"],
        });
        // Tempest: X cost, channel X Lightning orbs, exhaust (upgrade: +1)
        Self::insert(&mut cards, CardDef {
            id: "Tempest", name: "Tempest", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["channel_lightning_x"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Tempest+", name: "Tempest+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["channel_lightning_x_plus_1"],
        });
        // White Noise: 1 cost, add random Power to hand, exhaust (upgrade: cost 0)
        Self::insert(&mut cards, CardDef {
            id: "White Noise", name: "White Noise", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["add_random_power"],
        });
        Self::insert(&mut cards, CardDef {
            id: "White Noise+", name: "White Noise+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["add_random_power"],
        });

        // ---- Defect Rare Cards ----
        // All For One: 2 cost, 10 dmg, return all 0-cost cards from discard to hand
        Self::insert(&mut cards, CardDef {
            id: "All For One", name: "All For One", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["return_zero_cost_from_discard"],
        });
        Self::insert(&mut cards, CardDef {
            id: "All For One+", name: "All For One+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 14, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["return_zero_cost_from_discard"],
        });
        // Amplify: 1 cost, next power played this turn is played twice
        Self::insert(&mut cards, CardDef {
            id: "Amplify", name: "Amplify", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["amplify_power"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Amplify+", name: "Amplify+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["amplify_power"],
        });
        // Biased Cognition: 1 cost, power, gain 4 focus, lose 1 focus each turn
        Self::insert(&mut cards, CardDef {
            id: "Biased Cognition", name: "Biased Cognition", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["gain_focus", "lose_focus_each_turn"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Biased Cognition+", name: "Biased Cognition+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["gain_focus", "lose_focus_each_turn"],
        });
        // Buffer: 2 cost, power, prevent next X HP loss
        Self::insert(&mut cards, CardDef {
            id: "Buffer", name: "Buffer", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["buffer"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Buffer+", name: "Buffer+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["buffer"],
        });
        // Core Surge: 1 cost, 11 dmg, gain 1 Artifact, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Core Surge", name: "Core Surge", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["gain_artifact"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Core Surge+", name: "Core Surge+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["gain_artifact"],
        });
        // Creative AI: 3 cost, power, add random Power to hand each turn (upgrade: cost 2)
        Self::insert(&mut cards, CardDef {
            id: "Creative AI", name: "Creative AI", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["creative_ai"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Creative AI+", name: "Creative AI+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["creative_ai"],
        });
        // Echo Form: 3 cost, power, ethereal, first card each turn played twice (upgrade: no ethereal)
        Self::insert(&mut cards, CardDef {
            id: "Echo Form", name: "Echo Form", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["echo_form", "ethereal"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Echo Form+", name: "Echo Form+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["echo_form"],
        });
        // Electrodynamics: 2 cost, power, Lightning hits all enemies, channel 2 Lightning
        Self::insert(&mut cards, CardDef {
            id: "Electrodynamics", name: "Electrodynamics", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["lightning_hits_all", "channel_lightning"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Electrodynamics+", name: "Electrodynamics+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["lightning_hits_all", "channel_lightning"],
        });
        // Fission: 0 cost, remove all orbs, gain energy+draw per orb, exhaust (upgrade: evoke instead of remove)
        Self::insert(&mut cards, CardDef {
            id: "Fission", name: "Fission", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["fission"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Fission+", name: "Fission+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["fission_evoke"],
        });
        // Hyperbeam: 2 cost, 26 dmg AoE, lose 3 focus
        Self::insert(&mut cards, CardDef {
            id: "Hyperbeam", name: "Hyperbeam", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 2, base_damage: 26, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["lose_focus"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Hyperbeam+", name: "Hyperbeam+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 2, base_damage: 34, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["lose_focus"],
        });
        // Machine Learning: 1 cost, power, draw 1 extra card each turn (upgrade: innate)
        Self::insert(&mut cards, CardDef {
            id: "Machine Learning", name: "Machine Learning", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["extra_draw_each_turn"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Machine Learning+", name: "Machine Learning+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["extra_draw_each_turn", "innate"],
        });
        // Meteor Strike: 5 cost, 24 dmg, channel 3 Plasma
        Self::insert(&mut cards, CardDef {
            id: "Meteor Strike", name: "Meteor Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 5, base_damage: 24, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["channel_plasma"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Meteor Strike+", name: "Meteor Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 5, base_damage: 30, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["channel_plasma"],
        });
        // Multi-Cast: X cost, evoke frontmost orb X times (upgrade: X+1)
        Self::insert(&mut cards, CardDef {
            id: "Multi-Cast", name: "Multi-Cast", card_type: CardType::Skill,
            target: CardTarget::None, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["evoke_orb_x"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Multi-Cast+", name: "Multi-Cast+", card_type: CardType::Skill,
            target: CardTarget::None, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["evoke_orb_x_plus_1"],
        });
        // Rainbow: 2 cost, channel Lightning+Frost+Dark, exhaust (upgrade: no exhaust)
        Self::insert(&mut cards, CardDef {
            id: "Rainbow", name: "Rainbow", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["channel_lightning", "channel_frost", "channel_dark"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Rainbow+", name: "Rainbow+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["channel_lightning", "channel_frost", "channel_dark"],
        });
        // Reboot: 0 cost, shuffle hand+discard into draw, draw 4, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Reboot", name: "Reboot", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: true, enter_stance: None,
            effects: &["reboot"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Reboot+", name: "Reboot+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 6, exhaust: true, enter_stance: None,
            effects: &["reboot"],
        });
        // Seek: 0 cost, choose 1 card from draw pile and put into hand, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Seek", name: "Seek", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["seek"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Seek+", name: "Seek+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["seek"],
        });
        // Thunder Strike: 3 cost, deal 7 dmg for each Lightning channeled this combat
        Self::insert(&mut cards, CardDef {
            id: "Thunder Strike", name: "Thunder Strike", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 3, base_damage: 7, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["damage_per_lightning_channeled"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Thunder Strike+", name: "Thunder Strike+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 3, base_damage: 9, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["damage_per_lightning_channeled"],
        });

        // ====================================================================
        // COLORLESS Cards
        // ====================================================================

        // ---- Colorless Uncommon ----
        // Bandage Up: 0 cost, heal 4, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Bandage Up", name: "Bandage Up", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: true, enter_stance: None,
            effects: &["heal"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Bandage Up+", name: "Bandage Up+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 6, exhaust: true, enter_stance: None,
            effects: &["heal"],
        });
        // Blind: 0 cost, apply 2 Weak to enemy (upgrade: target all)
        Self::insert(&mut cards, CardDef {
            id: "Blind", name: "Blind", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["apply_weak"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Blind+", name: "Blind+", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["apply_weak"],
        });
        // Dark Shackles: 0 cost, reduce enemy str by 9 for one turn, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Dark Shackles", name: "Dark Shackles", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 9, exhaust: true, enter_stance: None,
            effects: &["reduce_str_this_turn"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Dark Shackles+", name: "Dark Shackles+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 15, exhaust: true, enter_stance: None,
            effects: &["reduce_str_this_turn"],
        });
        // Deep Breath: 0 cost, shuffle discard into draw, draw 1
        Self::insert(&mut cards, CardDef {
            id: "Deep Breath", name: "Deep Breath", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["shuffle_discard_into_draw", "draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Deep Breath+", name: "Deep Breath+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["shuffle_discard_into_draw", "draw"],
        });
        // Discovery: 1 cost, choose 1 of 3 cards to add to hand, exhaust (upgrade: no exhaust)
        Self::insert(&mut cards, CardDef {
            id: "Discovery", name: "Discovery", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["discovery"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Discovery+", name: "Discovery+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["discovery"],
        });
        // Dramatic Entrance: 0 cost, 8 dmg AoE, innate, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Dramatic Entrance", name: "Dramatic Entrance", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 0, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["innate"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Dramatic Entrance+", name: "Dramatic Entrance+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 0, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["innate"],
        });
        // Enlightenment: 0 cost, reduce cost of all cards in hand to 1 (this turn, upgrade: permanent)
        Self::insert(&mut cards, CardDef {
            id: "Enlightenment", name: "Enlightenment", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["enlightenment_this_turn"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Enlightenment+", name: "Enlightenment+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["enlightenment_permanent"],
        });
        // Finesse: 0 cost, 2 block, draw 1
        Self::insert(&mut cards, CardDef {
            id: "Finesse", name: "Finesse", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 2,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Finesse+", name: "Finesse+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 4,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["draw"],
        });
        // Flash of Steel: 0 cost, 3 dmg, draw 1
        Self::insert(&mut cards, CardDef {
            id: "Flash of Steel", name: "Flash of Steel", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Flash of Steel+", name: "Flash of Steel+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["draw"],
        });
        // Forethought: 0 cost, put card from hand to bottom of draw pile at 0 cost
        Self::insert(&mut cards, CardDef {
            id: "Forethought", name: "Forethought", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["forethought"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Forethought+", name: "Forethought+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["forethought_all"],
        });
        // Good Instincts: 0 cost, 6 block
        Self::insert(&mut cards, CardDef {
            id: "Good Instincts", name: "Good Instincts", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 6,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Good Instincts+", name: "Good Instincts+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 9,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        // Impatience: 0 cost, draw 2 if no attacks in hand
        Self::insert(&mut cards, CardDef {
            id: "Impatience", name: "Impatience", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw_if_no_attacks"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Impatience+", name: "Impatience+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["draw_if_no_attacks"],
        });
        // Jack of All Trades: 0 cost, add 1 random colorless card to hand, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Jack Of All Trades", name: "Jack Of All Trades", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["add_random_colorless"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Jack Of All Trades+", name: "Jack Of All Trades+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["add_random_colorless"],
        });
        // Madness: 1 cost, reduce random card in hand to 0 cost, exhaust (upgrade: cost 0)
        Self::insert(&mut cards, CardDef {
            id: "Madness", name: "Madness", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["madness"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Madness+", name: "Madness+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["madness"],
        });
        // Mind Blast: 2 cost, dmg = draw pile size, innate (upgrade: cost 1)
        Self::insert(&mut cards, CardDef {
            id: "Mind Blast", name: "Mind Blast", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 0, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_from_draw_pile", "innate"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Mind Blast+", name: "Mind Blast+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 0, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_from_draw_pile", "innate"],
        });
        // Panacea: 0 cost, gain 1 Artifact, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Panacea", name: "Panacea", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["gain_artifact"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Panacea+", name: "Panacea+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["gain_artifact"],
        });
        // Panic Button: 0 cost, 30 block, no block next 2 turns, exhaust
        Self::insert(&mut cards, CardDef {
            id: "PanicButton", name: "Panic Button", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 30,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["no_block_next_turns"],
        });
        Self::insert(&mut cards, CardDef {
            id: "PanicButton+", name: "Panic Button+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 40,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["no_block_next_turns"],
        });
        // Purity: 0 cost, exhaust up to 3 cards from hand, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Purity", name: "Purity", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["exhaust_from_hand"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Purity+", name: "Purity+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: true, enter_stance: None,
            effects: &["exhaust_from_hand"],
        });
        // Swift Strike: 0 cost, 7 dmg
        Self::insert(&mut cards, CardDef {
            id: "Swift Strike", name: "Swift Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 7, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Swift Strike+", name: "Swift Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        // Trip: 0 cost, apply 2 Vulnerable (upgrade: target all)
        Self::insert(&mut cards, CardDef {
            id: "Trip", name: "Trip", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["apply_vulnerable"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Trip+", name: "Trip+", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["apply_vulnerable"],
        });

        // ---- Colorless Rare ----
        // Apotheosis: 2 cost, upgrade all cards in deck, exhaust (upgrade: cost 1)
        Self::insert(&mut cards, CardDef {
            id: "Apotheosis", name: "Apotheosis", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["upgrade_all_cards"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Apotheosis+", name: "Apotheosis+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["upgrade_all_cards"],
        });
        // Chrysalis: 2 cost, shuffle 3 random upgraded Skills into draw pile, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Chrysalis", name: "Chrysalis", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["add_random_skills_to_draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Chrysalis+", name: "Chrysalis+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: true, enter_stance: None,
            effects: &["add_random_skills_to_draw"],
        });
        // Hand of Greed: 2 cost, 20 dmg, if kill gain 20 gold
        Self::insert(&mut cards, CardDef {
            id: "HandOfGreed", name: "Hand of Greed", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 20, base_block: -1,
            base_magic: 20, exhaust: false, enter_stance: None,
            effects: &["gold_on_kill"],
        });
        Self::insert(&mut cards, CardDef {
            id: "HandOfGreed+", name: "Hand of Greed+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 25, base_block: -1,
            base_magic: 25, exhaust: false, enter_stance: None,
            effects: &["gold_on_kill"],
        });
        // Magnetism: 2 cost, power, add random colorless card to hand each turn (upgrade: cost 1)
        Self::insert(&mut cards, CardDef {
            id: "Magnetism", name: "Magnetism", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["magnetism"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Magnetism+", name: "Magnetism+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["magnetism"],
        });
        // Master of Strategy: 0 cost, draw 3, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Master of Strategy", name: "Master of Strategy", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Master of Strategy+", name: "Master of Strategy+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: true, enter_stance: None,
            effects: &["draw"],
        });
        // Mayhem: 2 cost, power, auto-play top card of draw pile each turn (upgrade: cost 1)
        Self::insert(&mut cards, CardDef {
            id: "Mayhem", name: "Mayhem", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["mayhem"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Mayhem+", name: "Mayhem+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["mayhem"],
        });
        // Metamorphosis: 2 cost, shuffle 3 random upgraded Attacks into draw pile, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Metamorphosis", name: "Metamorphosis", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["add_random_attacks_to_draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Metamorphosis+", name: "Metamorphosis+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: true, enter_stance: None,
            effects: &["add_random_attacks_to_draw"],
        });
        // Panache: 0 cost, power, deal 10 dmg to all every 5th card played per turn
        Self::insert(&mut cards, CardDef {
            id: "Panache", name: "Panache", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 10, exhaust: false, enter_stance: None,
            effects: &["panache"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Panache+", name: "Panache+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 14, exhaust: false, enter_stance: None,
            effects: &["panache"],
        });
        // Sadistic Nature: 0 cost, power, deal 5 dmg whenever you apply debuff
        Self::insert(&mut cards, CardDef {
            id: "Sadistic Nature", name: "Sadistic Nature", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["sadistic_nature"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Sadistic Nature+", name: "Sadistic Nature+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 7, exhaust: false, enter_stance: None,
            effects: &["sadistic_nature"],
        });
        // Secret Technique: 0 cost, choose Skill from draw pile, put in hand, exhaust (upgrade: no exhaust)
        Self::insert(&mut cards, CardDef {
            id: "Secret Technique", name: "Secret Technique", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["search_skill"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Secret Technique+", name: "Secret Technique+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["search_skill"],
        });
        // Secret Weapon: 0 cost, choose Attack from draw pile, put in hand, exhaust (upgrade: no exhaust)
        Self::insert(&mut cards, CardDef {
            id: "Secret Weapon", name: "Secret Weapon", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["search_attack"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Secret Weapon+", name: "Secret Weapon+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["search_attack"],
        });
        // The Bomb: 2 cost, deal 40 dmg to all enemies in 3 turns
        Self::insert(&mut cards, CardDef {
            id: "The Bomb", name: "The Bomb", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 40, exhaust: false, enter_stance: None,
            effects: &["the_bomb"],
        });
        Self::insert(&mut cards, CardDef {
            id: "The Bomb+", name: "The Bomb+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 50, exhaust: false, enter_stance: None,
            effects: &["the_bomb"],
        });
        // Thinking Ahead: 0 cost, draw 2, put 1 card from hand on top of draw, exhaust (upgrade: no exhaust)
        Self::insert(&mut cards, CardDef {
            id: "Thinking Ahead", name: "Thinking Ahead", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["thinking_ahead"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Thinking Ahead+", name: "Thinking Ahead+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["thinking_ahead"],
        });
        // Transmutation: X cost, add X random colorless cards to hand, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Transmutation", name: "Transmutation", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["transmutation"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Transmutation+", name: "Transmutation+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["transmutation"],
        });
        // Violence: 0 cost, put 3 random Attacks from draw pile into hand, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Violence", name: "Violence", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["draw_attacks_from_draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Violence+", name: "Violence+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: true, enter_stance: None,
            effects: &["draw_attacks_from_draw"],
        });

        // ---- Colorless Special ----
        // Apparition (Java ID: Ghostly): 1 cost, gain 1 Intangible, exhaust, ethereal
        Self::insert(&mut cards, CardDef {
            id: "Ghostly", name: "Apparition", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["intangible", "ethereal"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Ghostly+", name: "Apparition+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["intangible"],
        });
        // Bite: 1 cost, 7 dmg, heal 2
        Self::insert(&mut cards, CardDef {
            id: "Bite", name: "Bite", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["heal_on_play"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Bite+", name: "Bite+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["heal_on_play"],
        });
        // J.A.X.: 0 cost, lose 3 HP, gain 2 str
        Self::insert(&mut cards, CardDef {
            id: "J.A.X.", name: "J.A.X.", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["lose_hp_gain_str"],
        });
        Self::insert(&mut cards, CardDef {
            id: "J.A.X.+", name: "J.A.X.+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["lose_hp_gain_str"],
        });
        // Ritual Dagger: 1 cost, dmg from misc, gain 3 per kill, exhaust
        Self::insert(&mut cards, CardDef {
            id: "RitualDagger", name: "Ritual Dagger", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["ritual_dagger"],
        });
        Self::insert(&mut cards, CardDef {
            id: "RitualDagger+", name: "Ritual Dagger+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
            base_magic: 5, exhaust: true, enter_stance: None,
            effects: &["ritual_dagger"],
        });

        // ====================================================================
        // CURSE Cards
        // ====================================================================
        // AscendersBane already registered above

        // Clumsy: unplayable, ethereal
        Self::insert(&mut cards, CardDef {
            id: "Clumsy", name: "Clumsy", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "ethereal"],
        });
        // CurseOfTheBell: unplayable, cannot be removed
        Self::insert(&mut cards, CardDef {
            id: "CurseOfTheBell", name: "Curse of the Bell", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable"],
        });
        // Decay: unplayable, deal 2 dmg to player at end of turn
        Self::insert(&mut cards, CardDef {
            id: "Decay", name: "Decay", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "end_turn_damage"],
        });
        // Doubt: unplayable, apply 1 Weak at end of turn
        Self::insert(&mut cards, CardDef {
            id: "Doubt", name: "Doubt", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "end_turn_weak"],
        });
        // Injury: unplayable
        Self::insert(&mut cards, CardDef {
            id: "Injury", name: "Injury", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable"],
        });
        // Necronomicurse: unplayable, cannot be removed
        Self::insert(&mut cards, CardDef {
            id: "Necronomicurse", name: "Necronomicurse", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "unremovable"],
        });
        // Normality: unplayable, can only play 3 cards per turn
        Self::insert(&mut cards, CardDef {
            id: "Normality", name: "Normality", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "limit_cards_per_turn"],
        });
        // Pain: unplayable, lose 1 HP when played from hand
        Self::insert(&mut cards, CardDef {
            id: "Pain", name: "Pain", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "damage_on_draw"],
        });
        // Parasite: unplayable, lose 3 max HP if removed
        Self::insert(&mut cards, CardDef {
            id: "Parasite", name: "Parasite", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "lose_max_hp_on_remove"],
        });
        // Pride: 1 cost, exhaust, innate, add copy to draw pile at end of turn
        Self::insert(&mut cards, CardDef {
            id: "Pride", name: "Pride", card_type: CardType::Curse,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["innate", "add_copy_end_turn"],
        });
        // Regret: unplayable, lose HP equal to cards in hand at end of turn
        Self::insert(&mut cards, CardDef {
            id: "Regret", name: "Regret", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "end_turn_hp_loss_per_card"],
        });
        // Shame: unplayable, apply 1 Frail at end of turn
        Self::insert(&mut cards, CardDef {
            id: "Shame", name: "Shame", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "end_turn_frail"],
        });
        // Writhe: unplayable, innate
        Self::insert(&mut cards, CardDef {
            id: "Writhe", name: "Writhe", card_type: CardType::Curse,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "innate"],
        });

        // ====================================================================
        // STATUS Cards (some already registered above: Slimed, Wound, Daze, Burn)
        // ====================================================================
        // Burn+: unplayable, 4 end-of-turn damage (upgraded from 2)
        Self::insert(&mut cards, CardDef {
            id: "Burn+", name: "Burn+", card_type: CardType::Status,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["unplayable", "end_turn_damage"],
        });
        // Void: unplayable, ethereal, lose 1 energy on draw
        Self::insert(&mut cards, CardDef {
            id: "Void", name: "Void", card_type: CardType::Status,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "ethereal", "lose_energy_on_draw"],
        });

        // ====================================================================
        // TEMP Cards (some already registered: Miracle, Smite)
        // ====================================================================
        // Beta: 2 cost, shuffle Omega into draw pile, exhaust (upgrade: cost 1)
        Self::insert(&mut cards, CardDef {
            id: "Beta", name: "Beta", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["add_omega_to_draw"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Beta+", name: "Beta+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["add_omega_to_draw"],
        });
        // Omega: 3 cost, power, deal 50 dmg to all enemies at end of each turn
        Self::insert(&mut cards, CardDef {
            id: "Omega", name: "Omega", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 50, exhaust: false, enter_stance: None,
            effects: &["omega"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Omega+", name: "Omega+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 60, exhaust: false, enter_stance: None,
            effects: &["omega"],
        });
        // Expunger: 1 cost, 9 dmg x magic (from Conjure Blade)
        Self::insert(&mut cards, CardDef {
            id: "Expunger", name: "Expunger", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["multi_hit"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Expunger+", name: "Expunger+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["multi_hit"],
        });
        // Insight: 0 cost, draw 2, retain, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Insight", name: "Insight", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["draw", "retain"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Insight+", name: "Insight+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["draw", "retain"],
        });
        // Safety: 1 cost, 12 block, retain, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Safety", name: "Safety", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 12,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"],
        });
        Self::insert(&mut cards, CardDef {
            id: "Safety+", name: "Safety+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 16,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"],
        });
        // Through Violence: 0 cost, 20 dmg, retain, exhaust
        Self::insert(&mut cards, CardDef {
            id: "ThroughViolence", name: "Through Violence", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 20, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"],
        });
        Self::insert(&mut cards, CardDef {
            id: "ThroughViolence+", name: "Through Violence+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 30, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"],
        });
        // Shiv: 0 cost, 4 dmg, exhaust
        Self::insert(&mut cards, CardDef {
            id: "Shiv", name: "Shiv", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[],
        });
        Self::insert(&mut cards, CardDef {
            id: "Shiv+", name: "Shiv+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[],
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
