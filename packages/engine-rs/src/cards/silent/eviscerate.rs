use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Eviscerate ---- (cost 3, 7 dmg x3, -1 cost per discard; +1 dmg)
    insert(cards, CardDef {
                id: "Eviscerate", name: "Eviscerate", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 3, base_damage: 7, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[E::ExtraHits(A::Magic)], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Eviscerate+", name: "Eviscerate+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 3, base_damage: 8, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[E::ExtraHits(A::Magic)], complex_hook: None,
            });
}
