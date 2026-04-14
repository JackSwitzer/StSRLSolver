use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "EmptyBody", name: "Empty Body", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["exit_stance"], effect_data: &[
                    E::Simple(SE::GainBlock(A::Block)),
                    E::Simple(SE::ChangeStance(Stance::Neutral)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "EmptyBody+", name: "Empty Body+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 10,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["exit_stance"], effect_data: &[
                    E::Simple(SE::GainBlock(A::Block)),
                    E::Simple(SE::ChangeStance(Stance::Neutral)),
                ], complex_hook: None,
            });
}
