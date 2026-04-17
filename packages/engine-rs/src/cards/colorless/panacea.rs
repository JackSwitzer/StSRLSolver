use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Panacea: 0 cost, gain 1 Artifact, exhaust
    insert(cards, CardDef {
                id: "Panacea", name: "Panacea", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ARTIFACT, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Panacea+", name: "Panacea+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ARTIFACT, A::Magic)),
                ], complex_hook: None,
            });
}
