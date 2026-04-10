use std::collections::HashMap;
use super::{CardDef, CardType, CardTarget};
use crate::effects::declarative::{Effect as E, SimpleEffect as SE, Target as T, AmountSource as A, Pile as P, BoolFlag as BF};
use crate::status_ids::sid;

pub fn register_silent(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Basic: Strike_G ----
        insert(cards, CardDef {
            id: "Strike_G", name: "Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Strike_G+", name: "Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        // ---- Silent Basic: Defend_G ----
        insert(cards, CardDef {
            id: "Defend_G", name: "Defend", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Defend_G+", name: "Defend+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        // ---- Silent Basic: Neutralize ---- (cost 0, 3 dmg, 1 weak; +1/+1)
        insert(cards, CardDef {
            id: "Neutralize", name: "Neutralize", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["weak"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Neutralize+", name: "Neutralize+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["weak"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
            ], complex_hook: None,
        });
        // ---- Silent Basic: Survivor ---- (cost 1, 8 block, discard 1; +3 block)
        insert(cards, CardDef {
            id: "Survivor", name: "Survivor", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["discard"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Survivor+", name: "Survivor+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 11,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["discard"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Common: Acrobatics ---- (cost 1, draw 3, discard 1; +1 draw)
        insert(cards, CardDef {
            id: "Acrobatics", name: "Acrobatics", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["draw", "discard"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Acrobatics+", name: "Acrobatics+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["draw", "discard"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Common: Backflip ---- (cost 1, 5 block, draw 2; +3 block)
        insert(cards, CardDef {
            id: "Backflip", name: "Backflip", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Backflip+", name: "Backflip+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
            ], complex_hook: None,
        });

        // ---- Silent Common: Bane ---- (cost 1, 7 dmg, double if poisoned; +3 dmg)
        insert(cards, CardDef {
            id: "Bane", name: "Bane", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["double_if_poisoned"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Bane+", name: "Bane+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["double_if_poisoned"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Common: Blade Dance ---- (cost 1, add 3 Shivs to hand; +1)
        insert(cards, CardDef {
            id: "Blade Dance", name: "Blade Dance", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["add_shivs"], effect_data: &[
                E::Simple(SE::AddCard("Shiv", P::Hand, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Blade Dance+", name: "Blade Dance+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["add_shivs"], effect_data: &[
                E::Simple(SE::AddCard("Shiv", P::Hand, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Silent Common: Cloak and Dagger ---- (cost 1, 6 block, add 1 Shiv to hand; +1 shiv)
        insert(cards, CardDef {
            id: "Cloak and Dagger", name: "Cloak and Dagger", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 6,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["add_shivs"], effect_data: &[
                E::Simple(SE::AddCard("Shiv", P::Hand, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Cloak and Dagger+", name: "Cloak and Dagger+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 6,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["add_shivs"], effect_data: &[
                E::Simple(SE::AddCard("Shiv", P::Hand, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Silent Common: Dagger Spray ---- (cost 1, 4 dmg x2 AoE; +2 dmg)
        insert(cards, CardDef {
            id: "Dagger Spray", name: "Dagger Spray", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 4, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["multi_hit"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Dagger Spray+", name: "Dagger Spray+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["multi_hit"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Common: Dagger Throw ---- (cost 1, 9 dmg, draw 1, discard 1; +3 dmg)
        insert(cards, CardDef {
            id: "Dagger Throw", name: "Dagger Throw", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["draw", "discard"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Dagger Throw+", name: "Dagger Throw+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["draw", "discard"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Common: Deadly Poison ---- (cost 1, 5 poison; +2)
        insert(cards, CardDef {
            id: "Deadly Poison", name: "Deadly Poison", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["poison"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::POISON, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Deadly Poison+", name: "Deadly Poison+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 7, exhaust: false, enter_stance: None,
            effects: &["poison"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::POISON, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Silent Common: Deflect ---- (cost 0, 4 block; +3)
        insert(cards, CardDef {
            id: "Deflect", name: "Deflect", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 4,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Deflect+", name: "Deflect+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 7,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Common: Dodge and Roll ---- (cost 1, 4 block, next turn 4 block; +2/+2)
        insert(cards, CardDef {
            id: "Dodge and Roll", name: "Dodge and Roll", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 4,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["next_turn_block"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Dodge and Roll+", name: "Dodge and Roll+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 6,
            base_magic: 6, exhaust: false, enter_stance: None,
            effects: &["next_turn_block"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Common: Flying Knee ---- (cost 1, 8 dmg, +1 energy next turn; +3 dmg)
        insert(cards, CardDef {
            id: "Flying Knee", name: "Flying Knee", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["next_turn_energy"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Flying Knee+", name: "Flying Knee+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["next_turn_energy"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Common: Outmaneuver ---- (cost 1, +2 energy next turn; +1 energy)
        insert(cards, CardDef {
            id: "Outmaneuver", name: "Outmaneuver", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["next_turn_energy"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Outmaneuver+", name: "Outmaneuver+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["next_turn_energy"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Common: Piercing Wail ---- (cost 1, -6 str to all enemies this turn, exhaust; +2 magic)
        insert(cards, CardDef {
            id: "Piercing Wail", name: "Piercing Wail", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 6, exhaust: true, enter_stance: None,
            effects: &["reduce_strength_all_temp"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Piercing Wail+", name: "Piercing Wail+", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 8, exhaust: true, enter_stance: None,
            effects: &["reduce_strength_all_temp"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Common: Poisoned Stab ---- (cost 1, 6 dmg, 3 poison; +1/+1)
        insert(cards, CardDef {
            id: "Poisoned Stab", name: "Poisoned Stab", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["poison"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::POISON, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Poisoned Stab+", name: "Poisoned Stab+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["poison"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::POISON, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Silent Common: Prepared ---- (cost 0, draw 1, discard 1; upgrade: draw 2 discard 2)
        insert(cards, CardDef {
            id: "Prepared", name: "Prepared", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["draw", "discard"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Prepared+", name: "Prepared+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw", "discard"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Common: Quick Slash ---- (cost 1, 8 dmg, draw 1; +4 dmg)
        insert(cards, CardDef {
            id: "Quick Slash", name: "Quick Slash", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Quick Slash+", name: "Quick Slash+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["draw"], effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
            ], complex_hook: None,
        });

        // ---- Silent Common: Slice ---- (cost 0, 6 dmg; +3 dmg)
        insert(cards, CardDef {
            id: "Slice", name: "Slice", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Slice+", name: "Slice+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 9, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Common: Sneaky Strike ---- (cost 2, 12 dmg, refund 2 energy if discarded; +4 dmg)
        insert(cards, CardDef {
            id: "Sneaky Strike", name: "Sneaky Strike", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["refund_energy_on_discard"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Sneaky Strike+", name: "Sneaky Strike+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 16, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["refund_energy_on_discard"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Common: Sucker Punch ---- (cost 1, 7 dmg, 1 weak; +2/+1)
        insert(cards, CardDef {
            id: "Sucker Punch", name: "Sucker Punch", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["weak"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Sucker Punch+", name: "Sucker Punch+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["weak"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Silent Uncommon: Accuracy ---- (cost 1, power, Shivs +4 dmg; +2)
        insert(cards, CardDef {
            id: "Accuracy", name: "Accuracy", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["accuracy"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Accuracy+", name: "Accuracy+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 6, exhaust: false, enter_stance: None,
            effects: &["accuracy"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: All-Out Attack ---- (cost 1, 10 AoE dmg, discard random; +4 dmg)
        insert(cards, CardDef {
            id: "All-Out Attack", name: "All-Out Attack", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["discard_random"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "All-Out Attack+", name: "All-Out Attack+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 14, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["discard_random"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Backstab ---- (cost 0, 11 dmg, innate, exhaust; +4 dmg)
        insert(cards, CardDef {
            id: "Backstab", name: "Backstab", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 11, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["innate"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Backstab+", name: "Backstab+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 15, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["innate"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Blur ---- (cost 1, 5 block, block not removed next turn; +3 block)
        insert(cards, CardDef {
            id: "Blur", name: "Blur", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["retain_block"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Blur+", name: "Blur+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["retain_block"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Bouncing Flask ---- (cost 2, 3 poison x3 to random; +1 hit)
        insert(cards, CardDef {
            id: "Bouncing Flask", name: "Bouncing Flask", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["poison_random_multi"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Bouncing Flask+", name: "Bouncing Flask+", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: false, enter_stance: None,
            effects: &["poison_random_multi"], effect_data: &[], complex_hook: None,  // 4 bounces (upgraded from 3)
        });

        // ---- Silent Uncommon: Calculated Gamble ---- (cost 0, discard hand draw that many, exhaust; upgrade: no exhaust)
        insert(cards, CardDef {
            id: "Calculated Gamble", name: "Calculated Gamble", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["calculated_gamble"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Calculated Gamble+", name: "Calculated Gamble+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["calculated_gamble"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Caltrops ---- (cost 1, power, deal 3 dmg when attacked; +2)
        insert(cards, CardDef {
            id: "Caltrops", name: "Caltrops", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["thorns"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Caltrops+", name: "Caltrops+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["thorns"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Catalyst ---- (cost 1, double poison on enemy, exhaust; upgrade: triple)
        insert(cards, CardDef {
            id: "Catalyst", name: "Catalyst", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["catalyst_double"], effect_data: &[
                E::Simple(SE::MultiplyStatus(T::SelectedEnemy, sid::POISON, 2)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Catalyst+", name: "Catalyst+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["catalyst_triple"], effect_data: &[
                E::Simple(SE::MultiplyStatus(T::SelectedEnemy, sid::POISON, 3)),
            ], complex_hook: None,
        });

        // ---- Silent Uncommon: Choke ---- (cost 2, 12 dmg, deal 3 dmg per card played this turn; +2 magic)
        insert(cards, CardDef {
            id: "Choke", name: "Choke", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["choke"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::CONSTRICTED, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Choke+", name: "Choke+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["choke"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::CONSTRICTED, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Silent Uncommon: Concentrate ---- (cost 0, discard 3, gain 2 energy; -1 discard)
        insert(cards, CardDef {
            id: "Concentrate", name: "Concentrate", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["discard_gain_energy"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Concentrate+", name: "Concentrate+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["discard_gain_energy"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Crippling Cloud (CripplingPoison) ---- (cost 2, 4 poison + 2 weak to all; +3/+1)
        insert(cards, CardDef {
            id: "Crippling Cloud", name: "Crippling Cloud", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 4, exhaust: true, enter_stance: None,
            effects: &["poison_all", "weak_all"], effect_data: &[
                E::Simple(SE::AddStatus(T::AllEnemies, sid::POISON, A::Magic)),
                E::Simple(SE::AddStatus(T::AllEnemies, sid::WEAKENED, A::Fixed(2))),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Crippling Cloud+", name: "Crippling Cloud+", card_type: CardType::Skill,
            target: CardTarget::AllEnemy, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 7, exhaust: true, enter_stance: None,
            effects: &["poison_all", "weak_all"], effect_data: &[
                E::Simple(SE::AddStatus(T::AllEnemies, sid::POISON, A::Magic)),
                E::Simple(SE::AddStatus(T::AllEnemies, sid::WEAKENED, A::Fixed(3))),
            ], complex_hook: None,
        });

        // ---- Silent Uncommon: Dash ---- (cost 2, 10 dmg + 10 block; +3/+3)
        insert(cards, CardDef {
            id: "Dash", name: "Dash", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: 10,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Dash+", name: "Dash+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 13, base_block: 13,
            base_magic: -1, exhaust: false, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Distraction ---- (cost 1, add random skill to hand at 0 cost, exhaust; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Distraction", name: "Distraction", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["random_skill_to_hand"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Distraction+", name: "Distraction+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["random_skill_to_hand"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Endless Agony ---- (cost 0, 4 dmg, exhaust, copy to hand on draw; +2 dmg)
        insert(cards, CardDef {
            id: "Endless Agony", name: "Endless Agony", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["copy_on_draw"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Endless Agony+", name: "Endless Agony+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["copy_on_draw"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Envenom ---- (cost 2, power, apply 1 poison on attack dmg; upgrade: cost 1)
        insert(cards, CardDef {
            id: "Envenom", name: "Envenom", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["envenom"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Envenom+", name: "Envenom+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["envenom"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Escape Plan ---- (cost 0, draw 1, if skill gain 3 block; +2 block)
        insert(cards, CardDef {
            id: "Escape Plan", name: "Escape Plan", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 3,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["draw", "block_if_skill"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Escape Plan+", name: "Escape Plan+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 5,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["draw", "block_if_skill"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Eviscerate ---- (cost 3, 7 dmg x3, -1 cost per discard; +1 dmg)
        insert(cards, CardDef {
            id: "Eviscerate", name: "Eviscerate", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 3, base_damage: 7, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["multi_hit", "cost_reduce_on_discard"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Eviscerate+", name: "Eviscerate+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 3, base_damage: 8, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["multi_hit", "cost_reduce_on_discard"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Expertise ---- (cost 1, draw to 6 cards; upgrade: draw to 7)
        insert(cards, CardDef {
            id: "Expertise", name: "Expertise", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 6, exhaust: false, enter_stance: None,
            effects: &["draw_to_n"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Expertise+", name: "Expertise+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 7, exhaust: false, enter_stance: None,
            effects: &["draw_to_n"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Finisher ---- (cost 1, 6 dmg per attack played this turn; +2 dmg)
        insert(cards, CardDef {
            id: "Finisher", name: "Finisher", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["finisher"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Finisher+", name: "Finisher+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["finisher"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Flechettes ---- (cost 1, 4 dmg per skill in hand; +2 dmg)
        insert(cards, CardDef {
            id: "Flechettes", name: "Flechettes", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 4, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["flechettes"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Flechettes+", name: "Flechettes+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["flechettes"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Footwork ---- (cost 1, power, +2 dex; +1)
        insert(cards, CardDef {
            id: "Footwork", name: "Footwork", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["gain_dexterity"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Footwork+", name: "Footwork+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["gain_dexterity"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Heel Hook ---- (cost 1, 5 dmg, if weak gain 1 energy + draw 1; +3 dmg)
        insert(cards, CardDef {
            id: "Heel Hook", name: "Heel Hook", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 5, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["if_weak_energy_draw"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Heel Hook+", name: "Heel Hook+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["if_weak_energy_draw"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Infinite Blades ---- (cost 1, power, add Shiv to hand at turn start; upgrade: cost 0)  [Note: ID is actually "Infinite Blades" not "InfiniteBlades"]
        insert(cards, CardDef {
            id: "Infinite Blades", name: "Infinite Blades", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["infinite_blades"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Infinite Blades+", name: "Infinite Blades+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["infinite_blades", "innate"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Leg Sweep ---- (cost 2, 2 weak, 11 block; +1/+3)
        insert(cards, CardDef {
            id: "Leg Sweep", name: "Leg Sweep", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 2, base_damage: -1, base_block: 11,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["weak"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Leg Sweep+", name: "Leg Sweep+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 2, base_damage: -1, base_block: 14,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["weak"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Silent Uncommon: Masterful Stab ---- (cost 0, 12 dmg, costs 1 more per HP lost; +4 dmg)
        insert(cards, CardDef {
            id: "Masterful Stab", name: "Masterful Stab", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 12, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["cost_increase_on_hp_loss"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Masterful Stab+", name: "Masterful Stab+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 16, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["cost_increase_on_hp_loss"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Noxious Fumes ---- (cost 1, power, 2 poison to all at turn start; +1)
        insert(cards, CardDef {
            id: "Noxious Fumes", name: "Noxious Fumes", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["noxious_fumes"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Noxious Fumes+", name: "Noxious Fumes+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["noxious_fumes"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Predator ---- (cost 2, 15 dmg, draw 2 next turn; +5 dmg)
        insert(cards, CardDef {
            id: "Predator", name: "Predator", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 15, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw_next_turn"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Predator+", name: "Predator+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 20, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["draw_next_turn"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Reflex ---- (cost -2, unplayable, draw 2 on discard; +1)
        insert(cards, CardDef {
            id: "Reflex", name: "Reflex", card_type: CardType::Skill,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["unplayable", "draw_on_discard"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Reflex+", name: "Reflex+", card_type: CardType::Skill,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["unplayable", "draw_on_discard"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Riddle with Holes ---- (cost 2, 3 dmg x5; +1 dmg)
        insert(cards, CardDef {
            id: "Riddle with Holes", name: "Riddle with Holes", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 3, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["multi_hit"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Riddle with Holes+", name: "Riddle with Holes+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 2, base_damage: 4, base_block: -1,
            base_magic: 5, exhaust: false, enter_stance: None,
            effects: &["multi_hit"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Setup ---- (cost 1, put card from hand on top of draw at 0 cost; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Setup", name: "Setup", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["setup"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Setup+", name: "Setup+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["setup"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Skewer ---- (cost X, 7 dmg x X times; +3 dmg)
        insert(cards, CardDef {
            id: "Skewer", name: "Skewer", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: -1, base_damage: 7, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["x_cost"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Skewer+", name: "Skewer+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: -1, base_damage: 10, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["x_cost"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Tactician ---- (cost -2, unplayable, gain 1 energy on discard; +1)
        insert(cards, CardDef {
            id: "Tactician", name: "Tactician", card_type: CardType::Skill,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["unplayable", "energy_on_discard"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Tactician+", name: "Tactician+", card_type: CardType::Skill,
            target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["unplayable", "energy_on_discard"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Uncommon: Terror ---- (cost 1, 99 vuln, exhaust; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Terror", name: "Terror", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 99, exhaust: true, enter_stance: None,
            effects: &["vulnerable"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Terror+", name: "Terror+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 99, exhaust: true, enter_stance: None,
            effects: &["vulnerable"], effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Silent Uncommon: Well-Laid Plans ---- (cost 1, power, retain 1 card/turn; +1)
        insert(cards, CardDef {
            id: "Well-Laid Plans", name: "Well-Laid Plans", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["well_laid_plans"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Well-Laid Plans+", name: "Well-Laid Plans+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["well_laid_plans"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Rare: A Thousand Cuts ---- (cost 2, power, deal 1 dmg per card played; +1)
        insert(cards, CardDef {
            id: "A Thousand Cuts", name: "A Thousand Cuts", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["thousand_cuts"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "A Thousand Cuts+", name: "A Thousand Cuts+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["thousand_cuts"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Rare: Adrenaline ---- (cost 0, gain 1 energy, draw 2, exhaust; +1 draw)
        insert(cards, CardDef {
            id: "Adrenaline", name: "Adrenaline", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: true, enter_stance: None,
            effects: &["gain_energy_1", "draw"], effect_data: &[
                E::Simple(SE::GainEnergy(A::Fixed(1))),
                E::Simple(SE::DrawCards(A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Adrenaline+", name: "Adrenaline+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["gain_energy_1", "draw"], effect_data: &[
                E::Simple(SE::GainEnergy(A::Fixed(1))),
                E::Simple(SE::DrawCards(A::Magic)),
            ], complex_hook: None,
        });

        // ---- Silent Rare: After Image ---- (cost 1, power, 1 block per card played; upgrade: cost 0)  [Note: ID is "After Image"]
        insert(cards, CardDef {
            id: "After Image", name: "After Image", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["after_image"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "After Image+", name: "After Image+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["after_image"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Rare: Alchemize ---- (cost 1, gain random potion, exhaust; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Alchemize", name: "Alchemize", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["alchemize"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Alchemize+", name: "Alchemize+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None,
            effects: &["alchemize"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Rare: Bullet Time ---- (cost 3, cards cost 0 this turn, no more draw; upgrade: cost 2)
        insert(cards, CardDef {
            id: "Bullet Time", name: "Bullet Time", card_type: CardType::Skill,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["bullet_time"], effect_data: &[
                E::Simple(SE::SetFlag(BF::BulletTime)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Bullet Time+", name: "Bullet Time+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["bullet_time"], effect_data: &[
                E::Simple(SE::SetFlag(BF::BulletTime)),
            ], complex_hook: None,
        });

        // ---- Silent Rare: Burst ---- (cost 1, next skill played twice; upgrade: next 2 skills)
        insert(cards, CardDef {
            id: "Burst", name: "Burst", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["burst"], effect_data: &[
                E::Simple(SE::SetStatus(T::Player, sid::BURST, A::Magic)),
            ], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Burst+", name: "Burst+", card_type: CardType::Skill,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["burst"], effect_data: &[
                E::Simple(SE::SetStatus(T::Player, sid::BURST, A::Magic)),
            ], complex_hook: None,
        });

        // ---- Silent Rare: Corpse Explosion ---- (cost 2, 6 poison, on death deal dmg = max HP to all; +3 poison)
        insert(cards, CardDef {
            id: "Corpse Explosion", name: "Corpse Explosion", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 6, exhaust: false, enter_stance: None,
            effects: &["corpse_explosion"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Corpse Explosion+", name: "Corpse Explosion+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 9, exhaust: false, enter_stance: None,
            effects: &["corpse_explosion"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Rare: Die Die Die ---- (cost 1, 13 AoE dmg, exhaust; +4 dmg)
        insert(cards, CardDef {
            id: "Die Die Die", name: "Die Die Die", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 13, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Die Die Die+", name: "Die Die Die+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 1, base_damage: 17, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Rare: Doppelganger ---- (cost X, gain X energy + draw X next turn; upgrade: +1/+1)
        insert(cards, CardDef {
            id: "Doppelganger", name: "Doppelganger", card_type: CardType::Skill,
            target: CardTarget::None, cost: -1, base_damage: -1, base_block: -1,
            base_magic: 0, exhaust: true, enter_stance: None,
            effects: &["x_cost", "doppelganger"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Doppelganger+", name: "Doppelganger+", card_type: CardType::Skill,
            target: CardTarget::None, cost: -1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["x_cost", "doppelganger"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Rare: Glass Knife ---- (cost 1, 8 dmg x2, -2 dmg each play; +2 dmg)
        insert(cards, CardDef {
            id: "Glass Knife", name: "Glass Knife", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["multi_hit", "glass_knife"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Glass Knife+", name: "Glass Knife+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["multi_hit", "glass_knife"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Rare: Grand Finale ---- (cost 0, 50 dmg AoE, only if draw pile empty; +10 dmg)
        insert(cards, CardDef {
            id: "Grand Finale", name: "Grand Finale", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 0, base_damage: 50, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["only_empty_draw"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Grand Finale+", name: "Grand Finale+", card_type: CardType::Attack,
            target: CardTarget::AllEnemy, cost: 0, base_damage: 60, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["only_empty_draw"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Rare: Malaise ---- (cost X, -X str + X weak to enemy, exhaust; +1/+1)
        insert(cards, CardDef {
            id: "Malaise", name: "Malaise", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: -1, base_damage: -1, base_block: -1,
            base_magic: 0, exhaust: true, enter_stance: None,
            effects: &["x_cost", "malaise"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Malaise+", name: "Malaise+", card_type: CardType::Skill,
            target: CardTarget::Enemy, cost: -1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: true, enter_stance: None,
            effects: &["x_cost", "malaise"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Rare: Nightmare ---- (cost 3, choose card in hand, add 3 copies next turn, exhaust; upgrade: cost 2)
        insert(cards, CardDef {
            id: "Nightmare", name: "Nightmare", card_type: CardType::Skill,
            target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["nightmare"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Nightmare+", name: "Nightmare+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: true, enter_stance: None,
            effects: &["nightmare"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Rare: Phantasmal Killer ---- (cost 1, double damage next turn, ethereal; upgrade: no ethereal)
        insert(cards, CardDef {
            id: "Phantasmal Killer", name: "Phantasmal Killer", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["phantasmal_killer", "ethereal"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Phantasmal Killer+", name: "Phantasmal Killer+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["phantasmal_killer"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Rare: Storm of Steel ---- (cost 1, discard hand, add Shiv per card; upgrade: Shiv+)
        insert(cards, CardDef {
            id: "Storm of Steel", name: "Storm of Steel", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["storm_of_steel"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Storm of Steel+", name: "Storm of Steel+", card_type: CardType::Skill,
            target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["storm_of_steel"], effect_data: &[], complex_hook: None,  // handler checks card name for Shiv vs Shiv+
        });

        // ---- Silent Rare: Tools of the Trade ---- (cost 1, power, draw 1 + discard 1 at turn start; upgrade: cost 0)
        insert(cards, CardDef {
            id: "Tools of the Trade", name: "Tools of the Trade", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["tools_of_the_trade"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Tools of the Trade+", name: "Tools of the Trade+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
            base_magic: 1, exhaust: false, enter_stance: None,
            effects: &["tools_of_the_trade"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Rare: Unload ---- (cost 1, 14 dmg, discard all non-attacks; +4 dmg)
        insert(cards, CardDef {
            id: "Unload", name: "Unload", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 14, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["discard_non_attacks"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Unload+", name: "Unload+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 1, base_damage: 18, base_block: -1,
            base_magic: -1, exhaust: false, enter_stance: None,
            effects: &["discard_non_attacks"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Rare: Wraith Form ---- (cost 3, power, +2 intangible, -1 dex/turn; +1 intangible)
        insert(cards, CardDef {
            id: "Wraith Form", name: "Wraith Form", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 2, exhaust: false, enter_stance: None,
            effects: &["wraith_form"], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Wraith Form+", name: "Wraith Form+", card_type: CardType::Power,
            target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
            base_magic: 3, exhaust: false, enter_stance: None,
            effects: &["wraith_form"], effect_data: &[], complex_hook: None,
        });

        // ---- Silent Special: Shiv ---- (cost 0, 4 dmg, exhaust; +2 dmg)
        insert(cards, CardDef {
            id: "Shiv", name: "Shiv", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
        insert(cards, CardDef {
            id: "Shiv+", name: "Shiv+", card_type: CardType::Attack,
            target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
            base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[], complex_hook: None,
        });
}

fn insert(map: &mut HashMap<&'static str, CardDef>, card: CardDef) {
    map.insert(card.id, card);
}
