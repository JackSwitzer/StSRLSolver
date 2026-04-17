use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Secret Technique: 0 cost, choose a Skill from draw pile, put it in hand
    insert(cards, CardDef {
        id: "Secret Technique", name: "Secret Technique", card_type: CardType::Skill,
        target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
        base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
            E::ChooseCards {
                source: P::Draw,
                filter: CardFilter::Skills,
                action: ChoiceAction::MoveToHand,
                min_picks: A::Fixed(1),
                max_picks: A::Fixed(1),
                post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
            },
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Secret Technique+", name: "Secret Technique+", card_type: CardType::Skill,
        target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::ChooseCards {
                source: P::Draw,
                filter: CardFilter::Skills,
                action: ChoiceAction::MoveToHand,
                min_picks: A::Fixed(1),
                max_picks: A::Fixed(1),
                post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
            },
        ], complex_hook: None,
    });
}
