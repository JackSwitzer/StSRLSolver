use std::collections::HashMap;
use super::{CardDef, CardType, CardTarget};
use crate::effects::declarative::{Effect as E, SimpleEffect as SE, Target as T, AmountSource as A, Pile as P, Condition as Cond, BoolFlag as BF};
use crate::status_ids::sid;
use crate::state::Stance;

pub fn register_watcher(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Watcher Basic Cards ----
        insert(cards, CardDef {
            id: "Strike_P", name: "Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Strike_P+", name: "Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Defend_P", name: "Defend", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Defend_P+", name: "Defend+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Eruption", name: "Eruption", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: Some("Wrath"), effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Eruption+", name: "Eruption+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: Some("Wrath"), effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Vigilance", name: "Vigilance", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: Some("Calm"), effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Vigilance+", name: "Vigilance+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 12,
            base_magic: -1, exhaust: false, enter_stance: Some("Calm"), effects: &[], effect_data: &[], complex_hook: None,
        });

        // ---- Common Watcher Cards ----
        insert(cards, CardDef {
            id: "BowlingBash", name: "Bowling Bash", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_per_enemy"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "BowlingBash+", name: "Bowling Bash+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["damage_per_enemy"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "CrushJoints", name: "Crush Joints", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["vuln_if_last_skill"], effect_data: &[
                E::Conditional(Cond::LastCardType(CardType::Skill), &[E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic))], &[]),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "CrushJoints+", name: "Crush Joints+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["vuln_if_last_skill"], effect_data: &[
                E::Conditional(Cond::LastCardType(CardType::Skill), &[E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic))], &[]),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "CutThroughFate", name: "Cut Through Fate", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["scry", "draw"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
                E::Simple(SE::Scry(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "CutThroughFate+", name: "Cut Through Fate+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["scry", "draw"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
                E::Simple(SE::Scry(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "EmptyBody", name: "Empty Body", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: -1, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["exit_stance"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "EmptyBody+", name: "Empty Body+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 11,
            base_magic: -1, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["exit_stance"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Flurry", name: "Flurry of Blows", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Flurry+", name: "Flurry of Blows+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "FlyingSleeves", name: "Flying Sleeves", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 4, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["multi_hit"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "FlyingSleeves+", name: "Flying Sleeves+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["multi_hit"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "FollowUp", name: "Follow-Up", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["energy_if_last_attack"], effect_data: &[
                E::Conditional(Cond::LastCardType(CardType::Attack), &[E::Simple(SE::GainEnergy(A::Fixed(1)))], &[]),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "FollowUp+", name: "Follow-Up+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["energy_if_last_attack"], effect_data: &[
                E::Conditional(Cond::LastCardType(CardType::Attack), &[E::Simple(SE::GainEnergy(A::Fixed(1)))], &[]),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Halt", name: "Halt", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 3,
            base_magic: 9, exhaust: false, enter_stance: None,
            effects: &["extra_block_in_wrath"], effect_data: &[
                E::Conditional(Cond::InStance(Stance::Wrath), &[E::Simple(SE::GainBlock(A::Magic))], &[]),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Halt+", name: "Halt+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 4,
            base_magic: 14, exhaust: false, enter_stance: None,
            effects: &["extra_block_in_wrath"], effect_data: &[
                E::Conditional(Cond::InStance(Stance::Wrath), &[E::Simple(SE::GainBlock(A::Magic))], &[]),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Prostrate", name: "Prostrate", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 4,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["mantra"], effect_data: &[
                E::Simple(SE::GainMantra(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Prostrate+", name: "Prostrate+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 4,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["mantra"], effect_data: &[
                E::Simple(SE::GainMantra(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Tantrum", name: "Tantrum", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 3, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: Some("Wrath"),
            effects: &["multi_hit", "shuffle_self_into_draw"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Tantrum+", name: "Tantrum+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 3, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: Some("Wrath"),
            effects: &["multi_hit", "shuffle_self_into_draw"], effect_data: &[], complex_hook: None,
        });

        // ---- Common: Consecrate ---- (cost 0, 5 dmg AoE, +3 upgrade)
        insert(cards, CardDef {
            id: "Consecrate", name: "Consecrate", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 0, base_damage: 5, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Consecrate+", name: "Consecrate+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 0, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });

        // ---- Common: Crescendo ---- (cost 1, enter Wrath, exhaust, retain; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Crescendo", name: "Crescendo", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Wrath"),
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Crescendo+", name: "Crescendo+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Wrath"),
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });

        // ---- Common: Empty Fist ---- (cost 1, 9 dmg, exit stance; +5 upgrade)
        insert(cards, CardDef {
            id: "EmptyFist", name: "Empty Fist", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["exit_stance"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "EmptyFist+", name: "Empty Fist+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 14, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["exit_stance"], effect_data: &[], complex_hook: None,
        });

        // ---- Common: Evaluate ---- (cost 1, 6 block, add Insight to draw; +4 block upgrade)
        insert(cards, CardDef {
            id: "Evaluate", name: "Evaluate", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 6,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["insight_to_draw"], effect_data: &[
                E::Simple(SE::AddCard("Insight", P::Draw, A::Fixed(1))),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Evaluate+", name: "Evaluate+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 10,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["insight_to_draw"], effect_data: &[
                E::Simple(SE::AddCard("Insight", P::Draw, A::Fixed(1))),
            ], complex_hook: None,
        });

        // ---- Common: Just Lucky ---- (cost 0, 3 dmg, 2 block, scry 1; +1/+1/+1 upgrade)
        insert(cards, CardDef {
            id: "JustLucky", name: "Just Lucky", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: 2,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["scry"], effect_data: &[
                E::Simple(SE::Scry(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "JustLucky+", name: "Just Lucky+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: 3,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["scry"], effect_data: &[
                E::Simple(SE::Scry(A::Magic)),
            ], complex_hook: None,
        });

        // ---- Common: Pressure Points ---- (cost 1, skill, apply 8 Mark, trigger; +3 upgrade)
        // Java ID: PathToVictory, run.rs uses PressurePoints
        insert(cards, CardDef {
            id: "PressurePoints", name: "Pressure Points", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 8, exhaust: false, enter_stance: None,
            effects: &["pressure_points"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "PressurePoints+", name: "Pressure Points+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 11, exhaust: false, enter_stance: None,
            effects: &["pressure_points"], effect_data: &[], complex_hook: None,
        });

        // ---- Common: Protect ---- (cost 2, 12 block, retain; +4 upgrade)
        insert(cards, CardDef {
            id: "Protect", name: "Protect", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 12,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Protect+", name: "Protect+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 16,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });

        // ---- Common: Sash Whip ---- (cost 1, 8 dmg, weak 1 if last attack; +2 dmg +1 magic upgrade)
        insert(cards, CardDef {
            id: "SashWhip", name: "Sash Whip", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["weak_if_last_attack"], effect_data: &[
                E::Conditional(Cond::LastCardType(CardType::Attack), &[E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic))], &[]),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "SashWhip+", name: "Sash Whip+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["weak_if_last_attack"], effect_data: &[
                E::Conditional(Cond::LastCardType(CardType::Attack), &[E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic))], &[]),
            ], complex_hook: None,
        });

        // ---- Common: Tranquility ---- (cost 1, enter Calm, exhaust, retain; upgrade: cost 0)
        // Java ID: ClearTheMind, run.rs uses Tranquility
        insert(cards, CardDef {
            id: "Tranquility", name: "Tranquility", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Calm"),
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Tranquility+", name: "Tranquility+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Calm"),
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });

        // ---- Common Watcher Cards (continued) ----
        insert(cards, CardDef {
            id: "ThirdEye", name: "Third Eye", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["scry"], effect_data: &[
                E::Simple(SE::Scry(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "ThirdEye+", name: "Third Eye+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 9,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["scry"], effect_data: &[
                E::Simple(SE::Scry(A::Magic)),
            ], complex_hook: None,
        });

        // ---- Uncommon Watcher Cards ----
        insert(cards, CardDef {
            id: "InnerPeace", name: "Inner Peace", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["if_calm_draw_else_calm"], effect_data: &[
                E::Conditional(Cond::InStance(Stance::Calm), &[E::Simple(SE::DrawCards(A::Magic))], &[E::Simple(SE::ChangeStance(Stance::Calm))]),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "InnerPeace+", name: "Inner Peace+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["if_calm_draw_else_calm"], effect_data: &[
                E::Conditional(Cond::InStance(Stance::Calm), &[E::Simple(SE::DrawCards(A::Magic))], &[E::Simple(SE::ChangeStance(Stance::Calm))]),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "WheelKick", name: "Wheel Kick", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 15, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "WheelKick+", name: "Wheel Kick+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 20, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
            ], complex_hook: None,
        });
        // ---- Uncommon: Battle Hymn ---- (cost 1, power, add Smite to hand each turn; upgrade: innate)
        insert(cards, CardDef {
            id: "BattleHymn", name: "Battle Hymn", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["battle_hymn"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "BattleHymn+", name: "Battle Hymn+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["battle_hymn", "innate"], effect_data: &[], complex_hook: None,
        });

        // ---- Uncommon: Carve Reality ---- (cost 1, 6 dmg, add Smite to hand; +4 dmg upgrade)
        insert(cards, CardDef {
            id: "CarveReality", name: "Carve Reality", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_smite_to_hand"], effect_data: &[
                E::Simple(SE::AddCard("Smite", P::Hand, A::Fixed(1))),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "CarveReality+", name: "Carve Reality+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_smite_to_hand"], effect_data: &[
                E::Simple(SE::AddCard("Smite", P::Hand, A::Fixed(1))),
            ], complex_hook: None,
        });

        // ---- Uncommon: Deceive Reality ---- (cost 1, 4 block, add Safety to hand; +3 block upgrade)
        insert(cards, CardDef {
            id: "DeceiveReality", name: "Deceive Reality", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 4,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_safety_to_hand"], effect_data: &[
                E::Simple(SE::AddCard("Safety", P::Hand, A::Fixed(1))),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "DeceiveReality+", name: "Deceive Reality+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_safety_to_hand"], effect_data: &[
                E::Simple(SE::AddCard("Safety", P::Hand, A::Fixed(1))),
            ], complex_hook: None,
        });

        // ---- Uncommon: Empty Mind ---- (cost 1, draw 2, exit stance; +1 draw upgrade)
        insert(cards, CardDef {
            id: "EmptyMind", name: "Empty Mind", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["draw", "exit_stance"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "EmptyMind+", name: "Empty Mind+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: Some("Neutral"),
            effects: &["draw", "exit_stance"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
            ], complex_hook: None,
        });

        // ---- Uncommon: Fear No Evil ---- (cost 1, 8 dmg, enter Calm if enemy attacking; +3 dmg upgrade)
        insert(cards, CardDef {
            id: "FearNoEvil", name: "Fear No Evil", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["calm_if_enemy_attacking"], effect_data: &[
                E::Conditional(Cond::EnemyAttacking, &[E::Simple(SE::ChangeStance(Stance::Calm))], &[]),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "FearNoEvil+", name: "Fear No Evil+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["calm_if_enemy_attacking"], effect_data: &[
                E::Conditional(Cond::EnemyAttacking, &[E::Simple(SE::ChangeStance(Stance::Calm))], &[]),
            ], complex_hook: None,
        });

        // ---- Uncommon: Foreign Influence ---- (cost 0, skill, exhaust, choose attack from other class; upgrade: upgraded choices)
        insert(cards, CardDef {
            id: "ForeignInfluence", name: "Foreign Influence", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["foreign_influence"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "ForeignInfluence+", name: "Foreign Influence+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["foreign_influence"], effect_data: &[], complex_hook: None,
        });

        // ---- Uncommon: Indignation ---- (cost 1, if in Wrath apply 3 vuln to all, else enter Wrath; +2 magic upgrade)
        insert(cards, CardDef {
            id: "Indignation", name: "Indignation", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["indignation"], effect_data: &[
                E::Conditional(Cond::InStance(Stance::Wrath), &[E::Simple(SE::AddStatus(T::AllEnemies, sid::VULNERABLE, A::Magic))], &[E::Simple(SE::ChangeStance(Stance::Wrath))]),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Indignation+", name: "Indignation+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["indignation"], effect_data: &[
                E::Conditional(Cond::InStance(Stance::Wrath), &[E::Simple(SE::AddStatus(T::AllEnemies, sid::VULNERABLE, A::Magic))], &[E::Simple(SE::ChangeStance(Stance::Wrath))]),
            ], complex_hook: None,
        });

        // ---- Uncommon: Like Water ---- (cost 1, power, if in Calm at end of turn gain 5 block; +2 magic upgrade)
        insert(cards, CardDef {
            id: "LikeWater", name: "Like Water", card_type: CardType::Power,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["like_water"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "LikeWater+", name: "Like Water+", card_type: CardType::Power,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 7, exhaust: false, enter_stance: None,
            effects: &["like_water"], effect_data: &[], complex_hook: None,
        });

        // ---- Uncommon: Meditate ---- (cost 1, put 1 card from discard into hand + retain it, enter Calm, end turn; +1 magic upgrade)
        insert(cards, CardDef {
            id: "Meditate", name: "Meditate", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: Some("Calm"),
            effects: &["meditate", "end_turn"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Meditate+", name: "Meditate+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: Some("Calm"),
            effects: &["meditate", "end_turn"], effect_data: &[], complex_hook: None,
        });

        // ---- Uncommon: Nirvana ---- (cost 1, power, gain 3 block whenever you Scry; +1 magic upgrade)
        insert(cards, CardDef {
            id: "Nirvana", name: "Nirvana", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["on_scry_block"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Nirvana+", name: "Nirvana+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["on_scry_block"], effect_data: &[], complex_hook: None,
        });

        // ---- Uncommon: Perseverance ---- (cost 1, 5 block, retain, block grows by 2 each retain; +2 block +1 magic upgrade)
        insert(cards, CardDef {
            id: "Perseverance", name: "Perseverance", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["retain", "grow_block_on_retain"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Perseverance+", name: "Perseverance+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["retain", "grow_block_on_retain"], effect_data: &[], complex_hook: None,
        });

        // ---- Uncommon: Reach Heaven ---- (cost 2, 10 dmg, shuffle Through Violence into draw; +5 dmg upgrade)
        insert(cards, CardDef {
            id: "ReachHeaven", name: "Reach Heaven", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_through_violence_to_draw"], effect_data: &[
                E::Simple(SE::AddCard("ThroughViolence", P::Draw, A::Fixed(1))),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "ReachHeaven+", name: "Reach Heaven+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 15, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["add_through_violence_to_draw"], effect_data: &[
                E::Simple(SE::AddCard("ThroughViolence", P::Draw, A::Fixed(1))),
            ], complex_hook: None,
        });

        // ---- Uncommon: Sands of Time ---- (cost 4, 20 dmg, retain, cost -1 each retain; +6 dmg upgrade)
        insert(cards, CardDef {
            id: "SandsOfTime", name: "Sands of Time", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 4, base_damage: 20, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["retain", "reduce_cost_on_retain"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "SandsOfTime+", name: "Sands of Time+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 4, base_damage: 26, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["retain", "reduce_cost_on_retain"], effect_data: &[], complex_hook: None,
        });

        // ---- Uncommon: Signature Move ---- (cost 2, 30 dmg, only playable if no other attacks in hand; +10 dmg upgrade)
        insert(cards, CardDef {
            id: "SignatureMove", name: "Signature Move", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 30, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["only_attack_in_hand"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "SignatureMove+", name: "Signature Move+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 40, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["only_attack_in_hand"], effect_data: &[], complex_hook: None,
        });

        // ---- Uncommon: Study ---- (cost 2, power, add Insight to draw at end of turn; upgrade: cost 1)
        insert(cards, CardDef {
            id: "Study", name: "Study", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["study"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Study+", name: "Study+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["study"], effect_data: &[], complex_hook: None,
        });

        // ---- Uncommon: Swivel ---- (cost 2, 8 block, next attack costs 0; +3 block upgrade)
        insert(cards, CardDef {
            id: "Swivel", name: "Swivel", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["next_attack_free"], effect_data: &[
                E::Simple(SE::SetFlag(BF::NextAttackFree)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Swivel+", name: "Swivel+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 11,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["next_attack_free"], effect_data: &[
                E::Simple(SE::SetFlag(BF::NextAttackFree)),
            ], complex_hook: None,
        });

        // ---- Uncommon: Wallop ---- (cost 2, 9 dmg, gain block equal to unblocked damage; +3 dmg upgrade)
        insert(cards, CardDef {
            id: "Wallop", name: "Wallop", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_from_damage"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Wallop+", name: "Wallop+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["block_from_damage"], effect_data: &[], complex_hook: None,
        });

        // ---- Uncommon: Wave of the Hand ---- (cost 1, skill, whenever you gain block this turn apply 1 Weak; +1 magic upgrade)
        insert(cards, CardDef {
            id: "WaveOfTheHand", name: "Wave of the Hand", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["wave_of_the_hand"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::WAVE_OF_THE_HAND, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "WaveOfTheHand+", name: "Wave of the Hand+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["wave_of_the_hand"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::WAVE_OF_THE_HAND, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Uncommon: Weave ---- (cost 0, 4 dmg, returns to hand on Scry; +2 dmg upgrade)
        insert(cards, CardDef {
            id: "Weave", name: "Weave", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["return_on_scry"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Weave+", name: "Weave+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["return_on_scry"], effect_data: &[], complex_hook: None,
        });

        // ---- Uncommon: Windmill Strike ---- (cost 2, 7 dmg, retain, +4 dmg each retain; +3 dmg +1 magic upgrade)
        insert(cards, CardDef {
            id: "WindmillStrike", name: "Windmill Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 7, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["retain", "grow_damage_on_retain"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "WindmillStrike+", name: "Windmill Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["retain", "grow_damage_on_retain"], effect_data: &[], complex_hook: None,
        });

        // ---- Uncommon: Sanctity ---- (cost 1, 6 block, draw 2 if last card played was Skill; +3 block upgrade)
        insert(cards, CardDef {
            id: "Sanctity", name: "Sanctity", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 6,
            base_magic: 2, exhaust: false, enter_stance: None, effects: &[], effect_data: &[
                E::Conditional(Cond::LastCardType(CardType::Skill), &[E::Simple(SE::DrawCards(A::Magic))], &[]),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Sanctity+", name: "Sanctity+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 9,
            base_magic: 2, exhaust: false, enter_stance: None, effects: &[], effect_data: &[
                E::Conditional(Cond::LastCardType(CardType::Skill), &[E::Simple(SE::DrawCards(A::Magic))], &[]),
            ], complex_hook: None,
        });

        // ---- Uncommon: Simmering Fury ---- (Java ID: Vengeance, cost 1, next turn enter Wrath + draw 2; +1 magic upgrade)
        insert(cards, CardDef {
            id: "Vengeance", name: "Simmering Fury", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Vengeance+", name: "Simmering Fury+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });

        // ---- Uncommon: Foresight ---- (Java ID: Wireheading, cost 1, power, scry 3 at start of turn; +1 magic upgrade)
        insert(cards, CardDef {
            id: "Wireheading", name: "Foresight", card_type: CardType::Power,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Wireheading+", name: "Foresight+", card_type: CardType::Power,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });

        // ---- Uncommon: Collect ---- (cost X, skill, exhaust, gain X Miracles next turn; upgrade: X+1)
        insert(cards, CardDef {
            id: "Collect", name: "Collect", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Collect+", name: "Collect+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });

        // ---- Uncommon: Wreath of Flame ---- (cost 1, gain 5 Vigor; +3 magic upgrade)
        insert(cards, CardDef {
            id: "WreathOfFlame", name: "Wreath of Flame", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["vigor"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::VIGOR, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "WreathOfFlame+", name: "Wreath of Flame+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 8, exhaust: false, enter_stance: None,
            effects: &["vigor"], effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::VIGOR, A::Magic)),
            ], complex_hook: None,
        });

        insert(cards, CardDef {
            id: "Conclude", name: "Conclude", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["end_turn"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Conclude+", name: "Conclude+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 16, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["end_turn"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "TalkToTheHand", name: "Talk to the Hand", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 5, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["apply_block_return"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::BLOCK_RETURN, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "TalkToTheHand+", name: "Talk to the Hand+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["apply_block_return"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::BLOCK_RETURN, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Pray", name: "Pray", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["mantra"], effect_data: &[
                E::Simple(SE::GainMantra(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Pray+", name: "Pray+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["mantra"], effect_data: &[
                E::Simple(SE::GainMantra(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Worship", name: "Worship", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["mantra"], effect_data: &[
                E::Simple(SE::GainMantra(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Worship+", name: "Worship+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["mantra", "retain"], effect_data: &[
                E::Simple(SE::GainMantra(A::Magic)),
            ], complex_hook: None,
        });

        // ---- Power Cards ----
        insert(cards, CardDef {
            id: "Adaptation", name: "Rushdown", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["on_wrath_draw"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Adaptation+", name: "Rushdown+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["on_wrath_draw"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "MentalFortress", name: "Mental Fortress", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["on_stance_change_block"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "MentalFortress+", name: "Mental Fortress+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 6, exhaust: false, enter_stance: None,
            effects: &["on_stance_change_block"], effect_data: &[], complex_hook: None,
        });

        // ---- Rare Watcher Cards ----
        insert(cards, CardDef {
            id: "Ragnarok", name: "Ragnarok", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 3, base_damage: 5, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["damage_random_x_times"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Ragnarok+", name: "Ragnarok+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 3, base_damage: 6, base_block: -1,
            base_magic: 6, exhaust: false, enter_stance: None,
            effects: &["damage_random_x_times"], effect_data: &[], complex_hook: None,
        });

        // ---- Rare: Alpha ---- (cost 1, skill, exhaust, shuffle Beta into draw; upgrade: innate)
        insert(cards, CardDef {
            id: "Alpha", name: "Alpha", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["add_beta_to_draw"], effect_data: &[
                E::Simple(SE::AddCard("Beta", P::Draw, A::Fixed(1))),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Alpha+", name: "Alpha+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["add_beta_to_draw", "innate"], effect_data: &[
                E::Simple(SE::AddCard("Beta", P::Draw, A::Fixed(1))),
            ], complex_hook: None,
        });

        // ---- Rare: Blasphemy ---- (cost 1, skill, exhaust, enter Divinity, die next turn; upgrade: retain)
        insert(cards, CardDef {
            id: "Blasphemy", name: "Blasphemy", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Divinity"),
            effects: &["die_next_turn"], effect_data: &[
                E::Simple(SE::SetFlag(BF::Blasphemy)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Blasphemy+", name: "Blasphemy+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: Some("Divinity"),
            effects: &["die_next_turn", "retain"], effect_data: &[
                E::Simple(SE::SetFlag(BF::Blasphemy)),
            ], complex_hook: None,
        });

        // ---- Rare: Brilliance ---- (cost 1, 12 dmg + mantra gained this combat; +4 dmg upgrade)
        insert(cards, CardDef {
            id: "Brilliance", name: "Brilliance", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["damage_plus_mantra"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Brilliance+", name: "Brilliance+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 16, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["damage_plus_mantra"], effect_data: &[], complex_hook: None,
        });

        // ---- Rare: Conjure Blade ---- (cost X, skill, exhaust, create Expunger with X hits; upgrade: X+1 hits)
        insert(cards, CardDef {
            id: "ConjureBlade", name: "Conjure Blade", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["conjure_blade"], effect_data: &[], complex_hook: None, // TODO: full X-cost + Expunger creation
        });
        insert(cards, CardDef {
            id: "ConjureBlade+", name: "Conjure Blade+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["conjure_blade"], effect_data: &[], complex_hook: None,
        });

        // ---- Rare: Deva Form ---- (cost 3, power, ethereal, gain 1 energy each turn (stacks); upgrade: no ethereal)
        insert(cards, CardDef {
            id: "DevaForm", name: "Deva Form", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["deva_form", "ethereal"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "DevaForm+", name: "Deva Form+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["deva_form"], effect_data: &[], complex_hook: None,
        });

        // ---- Rare: Devotion ---- (cost 1, power, gain 2 Mantra at start of each turn; +1 magic upgrade)
        insert(cards, CardDef {
            id: "Devotion", name: "Devotion", card_type: CardType::Power,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["devotion"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Devotion+", name: "Devotion+", card_type: CardType::Power,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["devotion"], effect_data: &[], complex_hook: None,
        });

        // ---- Rare: Establishment ---- (cost 1, power, retained cards cost 1 less; upgrade: innate)
        insert(cards, CardDef {
            id: "Establishment", name: "Establishment", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["establishment"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Establishment+", name: "Establishment+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["establishment", "innate"], effect_data: &[], complex_hook: None,
        });

        // ---- Rare (listed): Fasting ---- (Java: Uncommon, cost 2, power, +3 str/dex, -1 energy; +1 magic upgrade)
        insert(cards, CardDef {
            id: "Fasting", name: "Fasting", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["fasting"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Fasting+", name: "Fasting+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["fasting"], effect_data: &[], complex_hook: None,
        });

        // ---- Rare: Judgement ---- (cost 1, skill, if enemy HP <= 30, kill it; +10 magic upgrade)
        insert(cards, CardDef {
            id: "Judgement", name: "Judgement", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 30, exhaust: false, enter_stance: None,
            effects: &["judgement"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Judgement+", name: "Judgement+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 40, exhaust: false, enter_stance: None,
            effects: &["judgement"], effect_data: &[], complex_hook: None,
        });

        // ---- Rare: Lesson Learned ---- (cost 2, 10 dmg, exhaust, if kill upgrade a random card; +3 dmg upgrade)
        insert(cards, CardDef {
            id: "LessonLearned", name: "Lesson Learned", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["lesson_learned"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "LessonLearned+", name: "Lesson Learned+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 13, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["lesson_learned"], effect_data: &[], complex_hook: None,
        });

        // ---- Rare: Master Reality ---- (cost 1, power, created cards are upgraded; upgrade: cost 0)
        insert(cards, CardDef {
            id: "MasterReality", name: "Master Reality", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["master_reality"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "MasterReality+", name: "Master Reality+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["master_reality"], effect_data: &[], complex_hook: None,
        });

        // ---- Rare: Omniscience ---- (cost 4, skill, exhaust, choose card from draw pile play it twice; upgrade: cost 3)
        // TODO: Full effect requires choosing a card from draw pile and playing it twice
        insert(cards, CardDef {
            id: "Omniscience", name: "Omniscience", card_type: CardType::Skill,
            target: CardTarget::None, cost: 4, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["omniscience"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Omniscience+", name: "Omniscience+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["omniscience"], effect_data: &[], complex_hook: None,
        });

        // ---- Rare: Scrawl ---- (cost 1, skill, exhaust, draw until you have 10 cards; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Scrawl", name: "Scrawl", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["draw_to_ten"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Scrawl+", name: "Scrawl+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["draw_to_ten"], effect_data: &[], complex_hook: None,
        });

        // ---- Rare: Spirit Shield ---- (cost 2, skill, gain 3 block per card in hand; +1 magic upgrade)
        insert(cards, CardDef {
            id: "SpiritShield", name: "Spirit Shield", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["block_per_card_in_hand"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "SpiritShield+", name: "Spirit Shield+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["block_per_card_in_hand"], effect_data: &[], complex_hook: None,
        });

        // ---- Rare: Vault ---- (cost 3, skill, exhaust, skip enemy turn, end turn; upgrade: cost 2)
        insert(cards, CardDef {
            id: "Vault", name: "Vault", card_type: CardType::Skill,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["skip_enemy_turn", "end_turn"], effect_data: &[
                E::Simple(SE::SetFlag(BF::SkipEnemyTurn)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Vault+", name: "Vault+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["skip_enemy_turn", "end_turn"], effect_data: &[
                E::Simple(SE::SetFlag(BF::SkipEnemyTurn)),
            ], complex_hook: None,
        });

        // ---- Rare: Wish ---- (cost 3, skill, exhaust, choose: +3 str, or 25 gold, or 6 block; upgrade: +1/+5/+2)
        // TODO: Full effect requires ChooseOne UI (BecomeAlmighty, FameAndFortune, LiveForever)
        insert(cards, CardDef {
            id: "Wish", name: "Wish", card_type: CardType::Skill,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["wish"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Wish+", name: "Wish+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: true, enter_stance: None,
            effects: &["wish"], effect_data: &[], complex_hook: None,
        });

        // ---- Rare: Deus Ex Machina ---- (cost -2 (unplayable), skill, exhaust, on draw: add 2 Miracles to hand; +1 magic upgrade)
        insert(cards, CardDef {
            id: "DeusExMachina", name: "Deus Ex Machina", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "DeusExMachina+", name: "Deus Ex Machina+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });

        // ---- Rare: Discipline ---- (cost 2, power, deprecated; upgrade: cost 1)
        insert(cards, CardDef {
            id: "Discipline", name: "Discipline", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Discipline+", name: "Discipline+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });

        // ---- Rare: Unraveling ---- (cost 2, skill, exhaust, play all cards in hand for free; upgrade: cost 1)
        insert(cards, CardDef {
            id: "Unraveling", name: "Unraveling", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Unraveling+", name: "Unraveling+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });

        // ---- Special Cards ----
        insert(cards, CardDef {
            id: "Miracle", name: "Miracle", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["gain_energy"], effect_data: &[
                E::Simple(SE::GainEnergy(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Miracle+", name: "Miracle+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["gain_energy"], effect_data: &[
                E::Simple(SE::GainEnergy(A::Magic)),
            ], complex_hook: None,
        });
        // Holy Water: 0 cost, 5 block, retain, exhaust (from HolyWater relic)
        insert(cards, CardDef {
            id: "HolyWater", name: "HolyWater", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "HolyWater+", name: "HolyWater+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Smite", name: "Smite", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Smite+", name: "Smite+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 16, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });

        // ---- Special Generated Cards ----
        // Beta (from Alpha chain): cost 2, skill, exhaust, add Omega to draw
        insert(cards, CardDef {
            id: "Beta", name: "Beta", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["add_omega_to_draw"], effect_data: &[
                E::Simple(SE::AddCard("Omega", P::Draw, A::Fixed(1))),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Beta+", name: "Beta+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["add_omega_to_draw"], effect_data: &[
                E::Simple(SE::AddCard("Omega", P::Draw, A::Fixed(1))),
            ], complex_hook: None,
        });
        // Omega (from Beta chain): cost 3, power, deal 50 dmg at end of turn
        insert(cards, CardDef {
            id: "Omega", name: "Omega", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 50, exhaust: false, enter_stance: None,
            effects: &["omega"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Omega+", name: "Omega+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 60, exhaust: false, enter_stance: None,
            effects: &["omega"], effect_data: &[], complex_hook: None,
        });
        // Through Violence (from Reach Heaven): cost 0, 20 dmg, retain
        insert(cards, CardDef {
            id: "ThroughViolence", name: "Through Violence", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 20, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "ThroughViolence+", name: "Through Violence+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 30, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });
        // Safety (from Deceive Reality): cost 1, 12 block, retain, exhaust
        insert(cards, CardDef {
            id: "Safety", name: "Safety", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 12,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Safety+", name: "Safety+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 16,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["retain"], effect_data: &[], complex_hook: None,
        });
        // Insight (from Evaluate / Study): cost 0, draw 2, retain, exhaust
        insert(cards, CardDef {
            id: "Insight", name: "Insight", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["draw", "retain"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Insight+", name: "Insight+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["draw", "retain"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
            ], complex_hook: None,
        });
        // Expunger (from Conjure Blade): cost 1, deal 9 dmg X times
        insert(cards, CardDef {
            id: "Expunger", name: "Expunger", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["multi_hit"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Expunger+", name: "Expunger+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 0, exhaust: false, enter_stance: None,
            effects: &["multi_hit"], effect_data: &[], complex_hook: None,
        });

}

fn insert(map: &mut HashMap<&'static str, CardDef>, card: CardDef) {
    map.insert(card.id, card);
}
