use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource, CardFilter, ChoiceAction, Effect, Pile};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    static NIGHTMARE_EFFECTS: [Effect; 1] = [Effect::ChooseCards {
        source: Pile::Hand,
        filter: CardFilter::All,
        action: ChoiceAction::StoreCardForNextTurnCopies,
        min_picks: AmountSource::Fixed(1),
        max_picks: AmountSource::Fixed(1),
        post_choice_draw: AmountSource::Fixed(0),
    }];

    insert(cards, CardDef {
                id: "Nightmare", name: "Nightmare", card_type: CardType::Skill,
                target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["nightmare"], effect_data: &NIGHTMARE_EFFECTS, complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Nightmare+", name: "Nightmare+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["nightmare"], effect_data: &NIGHTMARE_EFFECTS, complex_hook: None,
            });
}
