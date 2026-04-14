use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Rare: Impervious ---- (cost 2, 30 block, exhaust; +10 block)
    insert(cards, CardDef {
                id: "Impervious", name: "Impervious", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 30,
                base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[E::Simple(SE::GainBlock(A::Block))], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Impervious+", name: "Impervious+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 40,
                base_magic: -1, exhaust: true, enter_stance: None, effects: &[], effect_data: &[E::Simple(SE::GainBlock(A::Block))], complex_hook: None,
            });
}
