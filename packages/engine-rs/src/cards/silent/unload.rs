use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Damage resolves before UnloadAction snapshots the hand and queues a
    // manual DiscardSpecificCardAction for every non-Attack.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/Unload.java
    // and actions/unique/UnloadAction.java.
    insert(cards, CardDef {
                id: "Unload", name: "Unload", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 14, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ForEachInPile {
                        pile: P::Hand,
                        filter: CardFilter::NonAttacks,
                        action: BulkAction::Discard,
                    },
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Unload+", name: "Unload+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 18, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ForEachInPile {
                        pile: P::Hand,
                        filter: CardFilter::NonAttacks,
                        action: BulkAction::Discard,
                    },
                ], complex_hook: None,
            });
}
