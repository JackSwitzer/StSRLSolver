use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Common: Armaments ---- (cost 1, 5 block, upgrade 1 card in hand; upgrade: all cards)
    insert(cards, CardDef {
                id: "Armaments", name: "Armaments", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["upgrade_one_card"], effect_data: &[
                    E::ChooseCards {
                        source: P::Hand,
                        filter: crate::effects::declarative::CardFilter::Upgradeable,
                        action: crate::effects::declarative::ChoiceAction::Upgrade,
                        min_picks: A::Fixed(1),
                        max_picks: A::Fixed(1),
                        post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                    },
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Armaments+", name: "Armaments+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["upgrade_all_cards"], effect_data: &[
                    E::ForEachInPile {
                        pile: P::Hand,
                        filter: crate::effects::declarative::CardFilter::Upgradeable,
                        action: crate::effects::declarative::BulkAction::Upgrade,
                    },
                ], complex_hook: None,
            });
}
