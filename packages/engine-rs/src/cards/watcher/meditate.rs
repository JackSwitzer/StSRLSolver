use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Meditate ---- (cost 1, put 1 card from discard into hand + retain it, enter Calm, end turn; +1 magic upgrade)
    insert(cards, CardDef {
                id: "Meditate", name: "Meditate", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: Some("Calm"),
                effects: &["meditate", "end_turn"], effect_data: &[
                    E::ChooseCards {
                        source: P::Discard, filter: CardFilter::All, action: ChoiceAction::MoveToHand,
                        min_picks: A::Fixed(1), max_picks: A::Magic,
                        post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                    },
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Meditate+", name: "Meditate+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: Some("Calm"),
                effects: &["meditate", "end_turn"], effect_data: &[
                    E::ChooseCards {
                        source: P::Discard, filter: CardFilter::All, action: ChoiceAction::MoveToHand,
                        min_picks: A::Fixed(1), max_picks: A::Magic,
                        post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                    },
                ], complex_hook: None,
            });
}
