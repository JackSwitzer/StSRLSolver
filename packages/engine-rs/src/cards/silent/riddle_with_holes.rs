use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Riddle with Holes ---- (cost 2, 3 dmg x5; +1 dmg)
    insert(cards, CardDef {
                id: "Riddle with Holes", name: "Riddle with Holes", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 3, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effects: &["multi_hit"], effect_data: &[E::ExtraHits(A::Magic)], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Riddle with Holes+", name: "Riddle with Holes+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 4, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effects: &["multi_hit"], effect_data: &[E::ExtraHits(A::Magic)], complex_hook: None,
            });
}
