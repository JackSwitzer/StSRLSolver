use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, ChoiceAction, CardFilter, Effect, Pile as P};

static FORETHOUGHT_BASE: [Effect; 1] = [Effect::ChooseCards {
    source: P::Hand,
    filter: CardFilter::All,
    action: ChoiceAction::PutOnBottomAtCostZero,
    min_picks: A::Fixed(1),
    max_picks: A::Fixed(1),
    post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
}];

static FORETHOUGHT_PLUS: [Effect; 1] = [Effect::ChooseCards {
    source: P::Hand,
    filter: CardFilter::All,
    action: ChoiceAction::PutOnBottomAtCostZero,
    min_picks: A::Fixed(0),
    max_picks: A::Fixed(99),
    post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
}];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Forethought: 0 cost, put card from hand to bottom of draw pile at 0 cost
    insert(cards, CardDef {
                id: "Forethought", name: "Forethought", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &FORETHOUGHT_BASE, complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Forethought+", name: "Forethought+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &FORETHOUGHT_PLUS, complex_hook: None,
            });
}
