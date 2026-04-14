use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Rare: Exhume ---- (cost 1, exhaust, put card from exhaust pile into hand; upgrade: cost 0)
    insert(cards, CardDef {
                id: "Exhume", name: "Exhume", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["exhume"], effect_data: &[
                    E::ChooseCards {
                        source: P::Exhaust,
                        filter: crate::effects::declarative::CardFilter::All,
                        action: crate::effects::declarative::ChoiceAction::MoveToHand,
                        min_picks: A::Fixed(1),
                        max_picks: A::Fixed(1),
                        post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                    },
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Exhume+", name: "Exhume+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["exhume"], effect_data: &[
                    E::ChooseCards {
                        source: P::Exhaust,
                        filter: crate::effects::declarative::CardFilter::All,
                        action: crate::effects::declarative::ChoiceAction::MoveToHand,
                        min_picks: A::Fixed(1),
                        max_picks: A::Fixed(1),
                        post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                    },
                ], complex_hook: None,
            });
}
