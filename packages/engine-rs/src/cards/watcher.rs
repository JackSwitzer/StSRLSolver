use std::collections::HashMap;
use super::{CardDef, CardType, CardTarget};

pub fn register_watcher(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Watcher Basic Cards ----
        insert(cards, CardDef {
            id: "Strike_P", name: "Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "Strike_P+", name: "Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "Defend_P", name: "Defend", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "Defend_P+", name: "Defend+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "Eruption", name: "Eruption", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: Some("Wrath"), effects: &[],
        });
        insert(cards, CardDef {
            id: "Eruption+", name: "Eruption+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: Some("Wrath"), effects: &[],
        });
        insert(cards, CardDef {
            id: "Vigilance", name: "Vigilance", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: Some("Calm"), effects: &[],
        });
        insert(cards, CardDef {
            id: "Vigilance+", name: "Vigilance+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 12,
            base_magic: -1, exhaust: false, enter_stance: Some("Calm"), effects: &[],
        });

        // ---- Common Watcher Cards ----
        insert(cards, CardDef {
            id: "BowlingBash", name: "Bowling Bash", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_per_enemy"],
        });
        insert(cards, CardDef {
            id: "BowlingBash+", name: "Bowling Bash+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_per_enemy"],
        });
        insert(cards, CardDef {
            id: "CrushJoints", name: "Crush Joints", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["vuln_if_last_skill"],
        });
        insert(cards, CardDef {
            id: "CrushJoints+", name: "Crush Joints+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["vuln_if_last_skill"],
        });
        insert(cards, CardDef {
            id: "CutThroughFate", name: "Cut Through Fate", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["scry", "draw"],
        });
        insert(cards, CardDef {
            id: "CutThroughFate+", name: "Cut Through Fate+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["scry", "draw"],
        });
        insert(cards, CardDef {
            id: "EmptyBody", name: "Empty Body", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: -1, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["exit_stance"],
        });
        insert(cards, CardDef {
            id: "EmptyBody+", name: "Empty Body+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 11,
            base_magic: -1, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["exit_stance"],
        });
        insert(cards, CardDef {
            id: "Flurry", name: "Flurry of Blows", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "Flurry+", name: "Flurry of Blows+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "FlyingSleeves", name: "Flying Sleeves", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 4, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["multi_hit"],
        });
        insert(cards, CardDef {
            id: "FlyingSleeves+", name: "Flying Sleeves+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["multi_hit"],
        });
        insert(cards, CardDef {
            id: "FollowUp", name: "Follow-Up", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["energy_if_last_attack"],
        });
        insert(cards, CardDef {
            id: "FollowUp+", name: "Follow-Up+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["energy_if_last_attack"],
        });
        insert(cards, CardDef {
            id: "Halt", name: "Halt", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 3,
            base_magic: 9, exhaust: false, enter_stance: None,
            effects: &["extra_block_in_wrath"],
        });
        insert(cards, CardDef {
            id: "Halt+", name: "Halt+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 4,
            base_magic: 14, exhaust: false, enter_stance: None,
            effects: &["extra_block_in_wrath"],
        });
        insert(cards, CardDef {
            id: "Prostrate", name: "Prostrate", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 4,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["mantra"],
        });
        insert(cards, CardDef {
            id: "Prostrate+", name: "Prostrate+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 4,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["mantra"],
        });
        insert(cards, CardDef {
            id: "Tantrum", name: "Tantrum", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 3, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: Some("Wrath"),
            effects: &["multi_hit", "shuffle_self_into_draw"],
        });
        insert(cards, CardDef {
            id: "Tantrum+", name: "Tantrum+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 3, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: Some("Wrath"),
            effects: &["multi_hit", "shuffle_self_into_draw"],
        });

        // ---- Common: Consecrate ---- (cost 0, 5 dmg AoE, +3 upgrade)
        insert(cards, CardDef {
            id: "Consecrate", name: "Consecrate", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 0, base_damage: 5, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "Consecrate+", name: "Consecrate+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 0, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });

        // ---- Common: Crescendo ---- (cost 1, enter Wrath, exhaust, retain; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Crescendo", name: "Crescendo", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Wrath"),
            effects: &["retain"],
        });
        insert(cards, CardDef {
            id: "Crescendo+", name: "Crescendo+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Wrath"),
            effects: &["retain"],
        });

        // ---- Common: Empty Fist ---- (cost 1, 9 dmg, exit stance; +5 upgrade)
        insert(cards, CardDef {
            id: "EmptyFist", name: "Empty Fist", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["exit_stance"],
        });
        insert(cards, CardDef {
            id: "EmptyFist+", name: "Empty Fist+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 14, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["exit_stance"],
        });

        // ---- Common: Evaluate ---- (cost 1, 6 block, add Insight to draw; +4 block upgrade)
        insert(cards, CardDef {
            id: "Evaluate", name: "Evaluate", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 6,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["insight_to_draw"],
        });
        insert(cards, CardDef {
            id: "Evaluate+", name: "Evaluate+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 10,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["insight_to_draw"],
        });

        // ---- Common: Just Lucky ---- (cost 0, 3 dmg, 2 block, scry 1; +1/+1/+1 upgrade)
        insert(cards, CardDef {
            id: "JustLucky", name: "Just Lucky", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: 2,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["scry"],
        });
        insert(cards, CardDef {
            id: "JustLucky+", name: "Just Lucky+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: 3,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["scry"],
        });

        // ---- Common: Pressure Points ---- (cost 1, skill, apply 8 Mark, trigger; +3 upgrade)
        // Java ID: PathToVictory, run.rs uses PressurePoints
        insert(cards, CardDef {
            id: "PressurePoints", name: "Pressure Points", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 8, exhaust: false, enter_stance: None,
            effects: &["pressure_points"],
        });
        insert(cards, CardDef {
            id: "PressurePoints+", name: "Pressure Points+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 11, exhaust: false, enter_stance: None,
            effects: &["pressure_points"],
        });

        // ---- Common: Protect ---- (cost 2, 12 block, retain; +4 upgrade)
        insert(cards, CardDef {
            id: "Protect", name: "Protect", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 12,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["retain"],
        });
        insert(cards, CardDef {
            id: "Protect+", name: "Protect+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 16,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["retain"],
        });

        // ---- Common: Sash Whip ---- (cost 1, 8 dmg, weak 1 if last attack; +2 dmg +1 magic upgrade)
        insert(cards, CardDef {
            id: "SashWhip", name: "Sash Whip", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["weak_if_last_attack"],
        });
        insert(cards, CardDef {
            id: "SashWhip+", name: "Sash Whip+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["weak_if_last_attack"],
        });

        // ---- Common: Tranquility ---- (cost 1, enter Calm, exhaust, retain; upgrade: cost 0)
        // Java ID: ClearTheMind, run.rs uses Tranquility
        insert(cards, CardDef {
            id: "Tranquility", name: "Tranquility", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Calm"),
            effects: &["retain"],
        });
        insert(cards, CardDef {
            id: "Tranquility+", name: "Tranquility+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Calm"),
            effects: &["retain"],
        });

        // ---- Common Watcher Cards (continued) ----
        insert(cards, CardDef {
            id: "ThirdEye", name: "Third Eye", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["scry"],
        });
        insert(cards, CardDef {
            id: "ThirdEye+", name: "Third Eye+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 9,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["scry"],
        });

        // ---- Uncommon Watcher Cards ----
        insert(cards, CardDef {
            id: "InnerPeace", name: "Inner Peace", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["if_calm_draw_else_calm"],
        });
        insert(cards, CardDef {
            id: "InnerPeace+", name: "Inner Peace+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["if_calm_draw_else_calm"],
        });
        insert(cards, CardDef {
            id: "WheelKick", name: "Wheel Kick", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 15, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw"],
        });
        insert(cards, CardDef {
            id: "WheelKick+", name: "Wheel Kick+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 20, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw"],
        });
        // ---- Uncommon: Battle Hymn ---- (cost 1, power, add Smite to hand each turn; upgrade: innate)
        insert(cards, CardDef {
            id: "BattleHymn", name: "Battle Hymn", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["battle_hymn"],
        });
        insert(cards, CardDef {
            id: "BattleHymn+", name: "Battle Hymn+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["battle_hymn", "innate"],
        });

        // ---- Uncommon: Carve Reality ---- (cost 1, 6 dmg, add Smite to hand; +4 dmg upgrade)
        insert(cards, CardDef {
            id: "CarveReality", name: "Carve Reality", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_smite_to_hand"],
        });
        insert(cards, CardDef {
            id: "CarveReality+", name: "Carve Reality+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_smite_to_hand"],
        });

        // ---- Uncommon: Deceive Reality ---- (cost 1, 4 block, add Safety to hand; +3 block upgrade)
        insert(cards, CardDef {
            id: "DeceiveReality", name: "Deceive Reality", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 4,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_safety_to_hand"],
        });
        insert(cards, CardDef {
            id: "DeceiveReality+", name: "Deceive Reality+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_safety_to_hand"],
        });

        // ---- Uncommon: Empty Mind ---- (cost 1, draw 2, exit stance; +1 draw upgrade)
        insert(cards, CardDef {
            id: "EmptyMind", name: "Empty Mind", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["draw", "exit_stance"],
        });
        insert(cards, CardDef {
            id: "EmptyMind+", name: "Empty Mind+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["draw", "exit_stance"],
        });

        // ---- Uncommon: Fear No Evil ---- (cost 1, 8 dmg, enter Calm if enemy attacking; +3 dmg upgrade)
        insert(cards, CardDef {
            id: "FearNoEvil", name: "Fear No Evil", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["calm_if_enemy_attacking"],
        });
        insert(cards, CardDef {
            id: "FearNoEvil+", name: "Fear No Evil+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["calm_if_enemy_attacking"],
        });

        // ---- Uncommon: Foreign Influence ---- (cost 0, skill, exhaust, choose attack from other class; upgrade: upgraded choices)
        insert(cards, CardDef {
            id: "ForeignInfluence", name: "Foreign Influence", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["foreign_influence"],
        });
        insert(cards, CardDef {
            id: "ForeignInfluence+", name: "Foreign Influence+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["foreign_influence"],
        });

        // ---- Uncommon: Indignation ---- (cost 1, if in Wrath apply 3 vuln to all, else enter Wrath; +2 magic upgrade)
        insert(cards, CardDef {
            id: "Indignation", name: "Indignation", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["indignation"],
        });
        insert(cards, CardDef {
            id: "Indignation+", name: "Indignation+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["indignation"],
        });

        // ---- Uncommon: Like Water ---- (cost 1, power, if in Calm at end of turn gain 5 block; +2 magic upgrade)
        insert(cards, CardDef {
            id: "LikeWater", name: "Like Water", card_type: CardType::Power,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["like_water"],
        });
        insert(cards, CardDef {
            id: "LikeWater+", name: "Like Water+", card_type: CardType::Power,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 7, exhaust: false, enter_stance: None,
            effects: &["like_water"],
        });

        // ---- Uncommon: Meditate ---- (cost 1, put 1 card from discard into hand + retain it, enter Calm, end turn; +1 magic upgrade)
        insert(cards, CardDef {
            id: "Meditate", name: "Meditate", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: Some("Calm"),
            effects: &["meditate", "end_turn"],
        });
        insert(cards, CardDef {
            id: "Meditate+", name: "Meditate+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: Some("Calm"),
            effects: &["meditate", "end_turn"],
        });

        // ---- Uncommon: Nirvana ---- (cost 1, power, gain 3 block whenever you Scry; +1 magic upgrade)
        insert(cards, CardDef {
            id: "Nirvana", name: "Nirvana", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["on_scry_block"],
        });
        insert(cards, CardDef {
            id: "Nirvana+", name: "Nirvana+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["on_scry_block"],
        });

        // ---- Uncommon: Perseverance ---- (cost 1, 5 block, retain, block grows by 2 each retain; +2 block +1 magic upgrade)
        insert(cards, CardDef {
            id: "Perseverance", name: "Perseverance", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["retain", "grow_block_on_retain"],
        });
        insert(cards, CardDef {
            id: "Perseverance+", name: "Perseverance+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["retain", "grow_block_on_retain"],
        });

        // ---- Uncommon: Reach Heaven ---- (cost 2, 10 dmg, shuffle Through Violence into draw; +5 dmg upgrade)
        insert(cards, CardDef {
            id: "ReachHeaven", name: "Reach Heaven", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_through_violence_to_draw"],
        });
        insert(cards, CardDef {
            id: "ReachHeaven+", name: "Reach Heaven+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 15, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_through_violence_to_draw"],
        });

        // ---- Uncommon: Sands of Time ---- (cost 4, 20 dmg, retain, cost -1 each retain; +6 dmg upgrade)
        insert(cards, CardDef {
            id: "SandsOfTime", name: "Sands of Time", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 4, base_damage: 20, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["retain", "reduce_cost_on_retain"],
        });
        insert(cards, CardDef {
            id: "SandsOfTime+", name: "Sands of Time+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 4, base_damage: 26, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["retain", "reduce_cost_on_retain"],
        });

        // ---- Uncommon: Signature Move ---- (cost 2, 30 dmg, only playable if no other attacks in hand; +10 dmg upgrade)
        insert(cards, CardDef {
            id: "SignatureMove", name: "Signature Move", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 30, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["only_attack_in_hand"],
        });
        insert(cards, CardDef {
            id: "SignatureMove+", name: "Signature Move+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 40, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["only_attack_in_hand"],
        });

        // ---- Uncommon: Study ---- (cost 2, power, add Insight to draw at end of turn; upgrade: cost 1)
        insert(cards, CardDef {
            id: "Study", name: "Study", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["study"],
        });
        insert(cards, CardDef {
            id: "Study+", name: "Study+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["study"],
        });

        // ---- Uncommon: Swivel ---- (cost 2, 8 block, next attack costs 0; +3 block upgrade)
        insert(cards, CardDef {
            id: "Swivel", name: "Swivel", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["next_attack_free"],
        });
        insert(cards, CardDef {
            id: "Swivel+", name: "Swivel+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 11,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["next_attack_free"],
        });

        // ---- Uncommon: Wallop ---- (cost 2, 9 dmg, gain block equal to unblocked damage; +3 dmg upgrade)
        insert(cards, CardDef {
            id: "Wallop", name: "Wallop", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_from_damage"],
        });
        insert(cards, CardDef {
            id: "Wallop+", name: "Wallop+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_from_damage"],
        });

        // ---- Uncommon: Wave of the Hand ---- (cost 1, skill, whenever you gain block this turn apply 1 Weak; +1 magic upgrade)
        insert(cards, CardDef {
            id: "WaveOfTheHand", name: "Wave of the Hand", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["wave_of_the_hand"],
        });
        insert(cards, CardDef {
            id: "WaveOfTheHand+", name: "Wave of the Hand+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["wave_of_the_hand"],
        });

        // ---- Uncommon: Weave ---- (cost 0, 4 dmg, returns to hand on Scry; +2 dmg upgrade)
        insert(cards, CardDef {
            id: "Weave", name: "Weave", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["return_on_scry"],
        });
        insert(cards, CardDef {
            id: "Weave+", name: "Weave+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["return_on_scry"],
        });

        // ---- Uncommon: Windmill Strike ---- (cost 2, 7 dmg, retain, +4 dmg each retain; +3 dmg +1 magic upgrade)
        insert(cards, CardDef {
            id: "WindmillStrike", name: "Windmill Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 7, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["retain", "grow_damage_on_retain"],
        });
        insert(cards, CardDef {
            id: "WindmillStrike+", name: "Windmill Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["retain", "grow_damage_on_retain"],
        });

        // ---- Uncommon: Sanctity ---- (cost 1, 6 block, draw 2 if last card played was Skill; +3 block upgrade)
        insert(cards, CardDef {
            id: "Sanctity", name: "Sanctity", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 6,
            base_magic: 2, exhaust: false, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "Sanctity+", name: "Sanctity+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 9,
            base_magic: 2, exhaust: false, enter_stance: None, effects: &[],
        });

        // ---- Uncommon: Simmering Fury ---- (Java ID: Vengeance, cost 1, next turn enter Wrath + draw 2; +1 magic upgrade)
        insert(cards, CardDef {
            id: "Vengeance", name: "Simmering Fury", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "Vengeance+", name: "Simmering Fury+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None, effects: &[],
        });

        // ---- Uncommon: Foresight ---- (Java ID: Wireheading, cost 1, power, scry 3 at start of turn; +1 magic upgrade)
        insert(cards, CardDef {
            id: "Wireheading", name: "Foresight", card_type: CardType::Power,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "Wireheading+", name: "Foresight+", card_type: CardType::Power,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None, effects: &[],
        });

        // ---- Uncommon: Collect ---- (cost X, skill, exhaust, gain X Miracles next turn; upgrade: X+1)
        insert(cards, CardDef {
            id: "Collect", name: "Collect", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "Collect+", name: "Collect+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[],
        });

        // ---- Uncommon: Wreath of Flame ---- (cost 1, gain 5 Vigor; +3 magic upgrade)
        insert(cards, CardDef {
            id: "WreathOfFlame", name: "Wreath of Flame", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["vigor"],
        });
        insert(cards, CardDef {
            id: "WreathOfFlame+", name: "Wreath of Flame+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 8, exhaust: false, enter_stance: None,
            effects: &["vigor"],
        });

        insert(cards, CardDef {
            id: "Conclude", name: "Conclude", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["end_turn"],
        });
        insert(cards, CardDef {
            id: "Conclude+", name: "Conclude+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 16, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["end_turn"],
        });
        insert(cards, CardDef {
            id: "TalkToTheHand", name: "Talk to the Hand", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 5, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["apply_block_return"],
        });
        insert(cards, CardDef {
            id: "TalkToTheHand+", name: "Talk to the Hand+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["apply_block_return"],
        });
        insert(cards, CardDef {
            id: "Pray", name: "Pray", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["mantra"],
        });
        insert(cards, CardDef {
            id: "Pray+", name: "Pray+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["mantra"],
        });
        insert(cards, CardDef {
            id: "Worship", name: "Worship", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["mantra"],
        });
        insert(cards, CardDef {
            id: "Worship+", name: "Worship+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["mantra", "retain"],
        });

        // ---- Power Cards ----
        insert(cards, CardDef {
            id: "Adaptation", name: "Rushdown", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["on_wrath_draw"],
        });
        insert(cards, CardDef {
            id: "Adaptation+", name: "Rushdown+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["on_wrath_draw"],
        });
        insert(cards, CardDef {
            id: "MentalFortress", name: "Mental Fortress", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["on_stance_change_block"],
        });
        insert(cards, CardDef {
            id: "MentalFortress+", name: "Mental Fortress+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 6, exhaust: false, enter_stance: None,
            effects: &["on_stance_change_block"],
        });

        // ---- Rare Watcher Cards ----
        insert(cards, CardDef {
            id: "Ragnarok", name: "Ragnarok", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 3, base_damage: 5, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["damage_random_x_times"],
        });
        insert(cards, CardDef {
            id: "Ragnarok+", name: "Ragnarok+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 3, base_damage: 6, base_block: -1,
            base_magic: 6, exhaust: false, enter_stance: None,
            effects: &["damage_random_x_times"],
        });

        // ---- Rare: Alpha ---- (cost 1, skill, exhaust, shuffle Beta into draw; upgrade: innate)
        insert(cards, CardDef {
            id: "Alpha", name: "Alpha", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["add_beta_to_draw"],
        });
        insert(cards, CardDef {
            id: "Alpha+", name: "Alpha+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["add_beta_to_draw", "innate"],
        });

        // ---- Rare: Blasphemy ---- (cost 1, skill, exhaust, enter Divinity, die next turn; upgrade: retain)
        insert(cards, CardDef {
            id: "Blasphemy", name: "Blasphemy", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Divinity"),
            effects: &["die_next_turn"],
        });
        insert(cards, CardDef {
            id: "Blasphemy+", name: "Blasphemy+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Divinity"),
            effects: &["die_next_turn", "retain"],
        });

        // ---- Rare: Brilliance ---- (cost 1, 12 dmg + mantra gained this combat; +4 dmg upgrade)
        insert(cards, CardDef {
            id: "Brilliance", name: "Brilliance", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["damage_plus_mantra"],
        });
        insert(cards, CardDef {
            id: "Brilliance+", name: "Brilliance+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 16, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["damage_plus_mantra"],
        });

        // ---- Rare: Conjure Blade ---- (cost X, skill, exhaust, create Expunger with X hits; upgrade: X+1 hits)
        insert(cards, CardDef {
            id: "ConjureBlade", name: "Conjure Blade", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["conjure_blade"], // TODO: full X-cost + Expunger creation
        });
        insert(cards, CardDef {
            id: "ConjureBlade+", name: "Conjure Blade+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["conjure_blade"],
        });

        // ---- Rare: Deva Form ---- (cost 3, power, ethereal, gain 1 energy each turn (stacks); upgrade: no ethereal)
        insert(cards, CardDef {
            id: "DevaForm", name: "Deva Form", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["deva_form", "ethereal"],
        });
        insert(cards, CardDef {
            id: "DevaForm+", name: "Deva Form+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["deva_form"],
        });

        // ---- Rare: Devotion ---- (cost 1, power, gain 2 Mantra at start of each turn; +1 magic upgrade)
        insert(cards, CardDef {
            id: "Devotion", name: "Devotion", card_type: CardType::Power,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["devotion"],
        });
        insert(cards, CardDef {
            id: "Devotion+", name: "Devotion+", card_type: CardType::Power,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["devotion"],
        });

        // ---- Rare: Establishment ---- (cost 1, power, retained cards cost 1 less; upgrade: innate)
        insert(cards, CardDef {
            id: "Establishment", name: "Establishment", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["establishment"],
        });
        insert(cards, CardDef {
            id: "Establishment+", name: "Establishment+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["establishment", "innate"],
        });

        // ---- Rare (listed): Fasting ---- (Java: Uncommon, cost 2, power, +3 str/dex, -1 energy; +1 magic upgrade)
        insert(cards, CardDef {
            id: "Fasting", name: "Fasting", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["fasting"],
        });
        insert(cards, CardDef {
            id: "Fasting+", name: "Fasting+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["fasting"],
        });

        // ---- Rare: Judgement ---- (cost 1, skill, if enemy HP <= 30, kill it; +10 magic upgrade)
        insert(cards, CardDef {
            id: "Judgement", name: "Judgement", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 30, exhaust: false, enter_stance: None,
            effects: &["judgement"],
        });
        insert(cards, CardDef {
            id: "Judgement+", name: "Judgement+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 40, exhaust: false, enter_stance: None,
            effects: &["judgement"],
        });

        // ---- Rare: Lesson Learned ---- (cost 2, 10 dmg, exhaust, if kill upgrade a random card; +3 dmg upgrade)
        insert(cards, CardDef {
            id: "LessonLearned", name: "Lesson Learned", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["lesson_learned"],
        });
        insert(cards, CardDef {
            id: "LessonLearned+", name: "Lesson Learned+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 13, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["lesson_learned"],
        });

        // ---- Rare: Master Reality ---- (cost 1, power, created cards are upgraded; upgrade: cost 0)
        insert(cards, CardDef {
            id: "MasterReality", name: "Master Reality", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["master_reality"],
        });
        insert(cards, CardDef {
            id: "MasterReality+", name: "Master Reality+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["master_reality"],
        });

        // ---- Rare: Omniscience ---- (cost 4, skill, exhaust, choose card from draw pile play it twice; upgrade: cost 3)
        // TODO: Full effect requires choosing a card from draw pile and playing it twice
        insert(cards, CardDef {
            id: "Omniscience", name: "Omniscience", card_type: CardType::Skill,
            target: CardTarget::None, cost: 4, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["omniscience"],
        });
        insert(cards, CardDef {
            id: "Omniscience+", name: "Omniscience+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["omniscience"],
        });

        // ---- Rare: Scrawl ---- (cost 1, skill, exhaust, draw until you have 10 cards; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Scrawl", name: "Scrawl", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["draw_to_ten"],
        });
        insert(cards, CardDef {
            id: "Scrawl+", name: "Scrawl+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["draw_to_ten"],
        });

        // ---- Rare: Spirit Shield ---- (cost 2, skill, gain 3 block per card in hand; +1 magic upgrade)
        insert(cards, CardDef {
            id: "SpiritShield", name: "Spirit Shield", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["block_per_card_in_hand"],
        });
        insert(cards, CardDef {
            id: "SpiritShield+", name: "Spirit Shield+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["block_per_card_in_hand"],
        });

        // ---- Rare: Vault ---- (cost 3, skill, exhaust, skip enemy turn, end turn; upgrade: cost 2)
        insert(cards, CardDef {
            id: "Vault", name: "Vault", card_type: CardType::Skill,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["skip_enemy_turn", "end_turn"],
        });
        insert(cards, CardDef {
            id: "Vault+", name: "Vault+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["skip_enemy_turn", "end_turn"],
        });

        // ---- Rare: Wish ---- (cost 3, skill, exhaust, choose: +3 str, or 25 gold, or 6 block; upgrade: +1/+5/+2)
        // TODO: Full effect requires ChooseOne UI (BecomeAlmighty, FameAndFortune, LiveForever)
        insert(cards, CardDef {
            id: "Wish", name: "Wish", card_type: CardType::Skill,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["wish"],
        });
        insert(cards, CardDef {
            id: "Wish+", name: "Wish+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: true, enter_stance: None,
            effects: &["wish"],
        });

        // ---- Rare: Deus Ex Machina ---- (cost -2 (unplayable), skill, exhaust, on draw: add 2 Miracles to hand; +1 magic upgrade)
        insert(cards, CardDef {
            id: "DeusExMachina", name: "Deus Ex Machina", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "DeusExMachina+", name: "Deus Ex Machina+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None, effects: &[],
        });

        // ---- Rare: Discipline ---- (cost 2, power, deprecated; upgrade: cost 1)
        insert(cards, CardDef {
            id: "Discipline", name: "Discipline", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "Discipline+", name: "Discipline+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        });

        // ---- Rare: Unraveling ---- (cost 2, skill, exhaust, play all cards in hand for free; upgrade: cost 1)
        insert(cards, CardDef {
            id: "Unraveling", name: "Unraveling", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[],
        });
        insert(cards, CardDef {
            id: "Unraveling+", name: "Unraveling+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[],
        });

        // ---- Special Cards ----
        insert(cards, CardDef {
            id: "Miracle", name: "Miracle", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["gain_energy"],
        });
        insert(cards, CardDef {
            id: "Miracle+", name: "Miracle+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["gain_energy"],
        });
        insert(cards, CardDef {
            id: "Smite", name: "Smite", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"],
        });
        insert(cards, CardDef {
            id: "Smite+", name: "Smite+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 16, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"],
        });

        // ---- Special Generated Cards ----
        // Beta (from Alpha chain): cost 2, skill, exhaust, add Omega to draw
        insert(cards, CardDef {
            id: "Beta", name: "Beta", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["add_omega_to_draw"],
        });
        insert(cards, CardDef {
            id: "Beta+", name: "Beta+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["add_omega_to_draw"],
        });
        // Omega (from Beta chain): cost 3, power, deal 50 dmg at end of turn
        insert(cards, CardDef {
            id: "Omega", name: "Omega", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 50, exhaust: false, enter_stance: None,
            effects: &["omega"],
        });
        insert(cards, CardDef {
            id: "Omega+", name: "Omega+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 60, exhaust: false, enter_stance: None,
            effects: &["omega"],
        });
        // Through Violence (from Reach Heaven): cost 0, 20 dmg, retain
        insert(cards, CardDef {
            id: "ThroughViolence", name: "Through Violence", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 20, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"],
        });
        insert(cards, CardDef {
            id: "ThroughViolence+", name: "Through Violence+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 30, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"],
        });
        // Safety (from Deceive Reality): cost 1, 12 block, retain, exhaust
        insert(cards, CardDef {
            id: "Safety", name: "Safety", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 12,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"],
        });
        insert(cards, CardDef {
            id: "Safety+", name: "Safety+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 16,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"],
        });
        // Insight (from Evaluate / Study): cost 0, draw 2, retain, exhaust
        insert(cards, CardDef {
            id: "Insight", name: "Insight", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["draw", "retain"],
        });
        insert(cards, CardDef {
            id: "Insight+", name: "Insight+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["draw", "retain"],
        });
        // Expunger (from Conjure Blade): cost 1, deal 9 dmg X times
        insert(cards, CardDef {
            id: "Expunger", name: "Expunger", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["multi_hit"],
        });
        insert(cards, CardDef {
            id: "Expunger+", name: "Expunger+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["multi_hit"],
        });

}

fn insert(map: &mut HashMap<&'static str, CardDef>, card: CardDef) {
    map.insert(card.id, card);
}
