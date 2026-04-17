use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Common: Anger ---- (cost 0, 6 dmg, add copy to discard; +2 dmg)
    insert(cards, CardDef {
                id: "Anger", name: "Anger", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::CopyThisCardTo(P::Discard)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Anger+", name: "Anger+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 8, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::CopyThisCardTo(P::Discard)),
                ], complex_hook: None,
            });
}
