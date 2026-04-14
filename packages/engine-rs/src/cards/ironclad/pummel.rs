use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Pummel ---- (cost 1, 2 dmg x4, exhaust; +1 hit)
    insert(cards, CardDef {
                id: "Pummel", name: "Pummel", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 2, base_block: -1,
                base_magic: 4, exhaust: true, enter_stance: None,
                effects: &["multi_hit"], effect_data: &[
                    E::ExtraHits(A::Magic),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Pummel+", name: "Pummel+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 2, base_block: -1,
                base_magic: 5, exhaust: true, enter_stance: None,
                effects: &["multi_hit"], effect_data: &[
                    E::ExtraHits(A::Magic),
                ], complex_hook: None,
            });
}
