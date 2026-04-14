use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Hologram", name: "Hologram", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 3,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["return_from_discard"], effect_data: &[
                    E::ChooseCards {
                        source: P::Discard, filter: CardFilter::All, action: ChoiceAction::MoveToHand,
                        min_picks: A::Fixed(1), max_picks: A::Fixed(1),
                        post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                    },
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Hologram+", name: "Hologram+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["return_from_discard"], effect_data: &[
                    E::ChooseCards {
                        source: P::Discard, filter: CardFilter::All, action: ChoiceAction::MoveToHand,
                        min_picks: A::Fixed(1), max_picks: A::Fixed(1),
                        post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                    },
                ], complex_hook: None,
            });
}
