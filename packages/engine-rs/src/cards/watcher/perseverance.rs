use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Perseverance ---- (cost 1, 5 block, retain, block grows by 2 each retain; +2 block +1 magic upgrade)
    insert(cards, CardDef {
                id: "Perseverance", name: "Perseverance", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["retain", "grow_block_on_retain"], effect_data: &[
                    E::Simple(SE::GainBlock(A::Block)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Perseverance+", name: "Perseverance+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["retain", "grow_block_on_retain"], effect_data: &[
                    E::Simple(SE::GainBlock(A::Block)),
                ], complex_hook: None,
            });
}
