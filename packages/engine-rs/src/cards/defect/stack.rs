use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Stack: 1 cost, block = discard pile size (upgrade: +3)
    insert(cards, CardDef {
                id: "Stack", name: "Stack", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 0,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainBlock(A::DiscardPileSize)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Stack+", name: "Stack+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 3,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainBlock(A::DiscardPileSize)),
                ], complex_hook: None,
            });
}
