use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Sever Soul ---- (cost 2, 16 dmg, exhaust all non-attacks in hand; +6 dmg)
    insert(cards, CardDef {
                id: "Sever Soul", name: "Sever Soul", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 16, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ForEachInPile {
                        pile: P::Hand,
                        filter: crate::effects::declarative::CardFilter::NonAttacks,
                        action: crate::effects::declarative::BulkAction::Exhaust,
                    },
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Sever Soul+", name: "Sever Soul+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 22, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ForEachInPile {
                        pile: P::Hand,
                        filter: crate::effects::declarative::CardFilter::NonAttacks,
                        action: crate::effects::declarative::BulkAction::Exhaust,
                    },
                ], complex_hook: None,
            });
}
