use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
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
}
