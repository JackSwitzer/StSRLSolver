use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Setup ---- (cost 1, put card from hand on top of draw at 0 cost; upgrade: cost 0)
    insert(cards, CardDef {
                id: "Setup", name: "Setup", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["setup"], effect_data: &[
                    E::ChooseCards {
                        source: P::Hand,
                        filter: CardFilter::All,
                        action: ChoiceAction::PutOnTopAtCostZero,
                        min_picks: A::Fixed(1),
                        max_picks: A::Fixed(1),
                        post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                    },
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Setup+", name: "Setup+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["setup"], effect_data: &[
                    E::ChooseCards {
                        source: P::Hand,
                        filter: CardFilter::All,
                        action: ChoiceAction::PutOnTopAtCostZero,
                        min_picks: A::Fixed(1),
                        max_picks: A::Fixed(1),
                        post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                    },
                ], complex_hook: None,
            });
}
