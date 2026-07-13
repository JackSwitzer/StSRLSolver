use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: cards/green/Concentrate.java targets SELF, queues a mandatory
    // discard of 3 before gaining 2 energy, and upgrades the discard count to 2.
    insert(cards, CardDef {
        id: "Concentrate", name: "Concentrate", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
        base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[E::ChooseCards {
            source: P::Hand,
            filter: CardFilter::All,
            action: ChoiceAction::DiscardForEffect,
            min_picks: A::Magic,
            max_picks: A::Magic,
            post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
        }],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Concentrate+", name: "Concentrate+", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
        base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[E::ChooseCards {
            source: P::Hand,
            filter: CardFilter::All,
            action: ChoiceAction::DiscardForEffect,
            min_picks: A::Magic,
            max_picks: A::Magic,
            post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
        }],
        complex_hook: None,
    });
}
