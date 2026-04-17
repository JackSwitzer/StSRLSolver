use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Equilibrium (Java ID: Undo): 2 cost, 13 block, retain hand this turn
    insert(cards, CardDef {
                id: "Undo", name: "Equilibrium", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 13,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::SetFlag(BF::RetainHand)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Undo+", name: "Equilibrium+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 16,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::SetFlag(BF::RetainHand)),
                ], complex_hook: None,
            });
}
