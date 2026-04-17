use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Halt", name: "Halt", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 3,
                base_magic: 9, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Conditional(Cond::InStance(Stance::Wrath), &[E::Simple(SE::GainBlock(A::Magic))], &[]),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Halt+", name: "Halt+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 4,
                base_magic: 14, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Conditional(Cond::InStance(Stance::Wrath), &[E::Simple(SE::GainBlock(A::Magic))], &[]),
                ], complex_hook: None,
            });
}
