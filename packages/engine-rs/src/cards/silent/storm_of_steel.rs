use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Rare: Storm of Steel ---- (cost 1, discard hand, add Shiv per card; upgrade: Shiv+)
    insert(cards, CardDef {
                id: "Storm of Steel", name: "Storm of Steel", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["storm_of_steel"], effect_data: &[
                    E::ForEachInPile {
                        pile: P::Hand,
                        filter: CardFilter::All,
                        action: BulkAction::Discard,
                    },
                    E::Simple(SE::AddCard("Shiv", P::Hand, A::HandSizeAtPlay)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Storm of Steel+", name: "Storm of Steel+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["storm_of_steel"], effect_data: &[
                    E::ForEachInPile {
                        pile: P::Hand,
                        filter: CardFilter::All,
                        action: BulkAction::Discard,
                    },
                    E::Simple(SE::AddCard("Shiv+", P::Hand, A::HandSizeAtPlay)),
                ], complex_hook: None,
            });
}
