use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Purity: 0 cost, exhaust up to N cards from hand, exhaust
    insert(cards, CardDef {
        id: "Purity", name: "Purity", card_type: CardType::Skill,
        target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
        base_magic: 3, exhaust: true, enter_stance: None,
        effects: &["exhaust_from_hand"], effect_data: &[
            E::ChooseCards {
                source: P::Hand,
                filter: CardFilter::All,
                action: ChoiceAction::Exhaust,
                min_picks: A::Fixed(0),
                max_picks: A::Magic,
                post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
            },
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Purity+", name: "Purity+", card_type: CardType::Skill,
        target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
        base_magic: 5, exhaust: true, enter_stance: None,
        effects: &["exhaust_from_hand"], effect_data: &[
            E::ChooseCards {
                source: P::Hand,
                filter: CardFilter::All,
                action: ChoiceAction::Exhaust,
                min_picks: A::Fixed(0),
                max_picks: A::Magic,
                post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
            },
        ], complex_hook: None,
    });
}
