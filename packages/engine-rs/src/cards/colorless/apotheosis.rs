use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Colorless Rare ----
        // Apotheosis: 2 cost, upgrade all cards in deck, exhaust (upgrade: cost 1)
    insert(cards, CardDef {
                id: "Apotheosis", name: "Apotheosis", card_type: CardType::Skill,
                target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["upgrade_all_cards"], effect_data: &[
                    E::ForEachInPile {
                        pile: P::Hand,
                        filter: CardFilter::All,
                        action: BulkAction::Upgrade,
                    },
                    E::ForEachInPile {
                        pile: P::Draw,
                        filter: CardFilter::All,
                        action: BulkAction::Upgrade,
                    },
                    E::ForEachInPile {
                        pile: P::Discard,
                        filter: CardFilter::All,
                        action: BulkAction::Upgrade,
                    },
                    E::ForEachInPile {
                        pile: P::Exhaust,
                        filter: CardFilter::All,
                        action: BulkAction::Upgrade,
                    },
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Apotheosis+", name: "Apotheosis+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["upgrade_all_cards"], effect_data: &[
                    E::ForEachInPile {
                        pile: P::Hand,
                        filter: CardFilter::All,
                        action: BulkAction::Upgrade,
                    },
                    E::ForEachInPile {
                        pile: P::Draw,
                        filter: CardFilter::All,
                        action: BulkAction::Upgrade,
                    },
                    E::ForEachInPile {
                        pile: P::Discard,
                        filter: CardFilter::All,
                        action: BulkAction::Upgrade,
                    },
                    E::ForEachInPile {
                        pile: P::Exhaust,
                        filter: CardFilter::All,
                        action: BulkAction::Upgrade,
                    },
                ], complex_hook: None,
            });
}
