use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, CardFilter, ChoiceAction, Effect, Pile as P};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Omniscience ---- (cost 4, skill, exhaust, choose card from draw pile play it twice; upgrade: cost 3)
    insert(cards, CardDef {
                id: "Omniscience", name: "Omniscience", card_type: CardType::Skill,
                target: CardTarget::None, cost: 4, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["omniscience"], effect_data: &[Effect::ChooseCards {
                    source: P::Draw,
                    filter: CardFilter::All,
                    action: ChoiceAction::PlayForFree,
                    min_picks: A::Fixed(1),
                    max_picks: A::Fixed(1),
                    post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                }],
                complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Omniscience+", name: "Omniscience+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["omniscience"], effect_data: &[Effect::ChooseCards {
                    source: P::Draw,
                    filter: CardFilter::All,
                    action: ChoiceAction::PlayForFree,
                    min_picks: A::Fixed(1),
                    max_picks: A::Fixed(1),
                    post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                }],
                complex_hook: None,
            });
}
