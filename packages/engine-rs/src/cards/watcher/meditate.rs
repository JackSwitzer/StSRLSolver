use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // MeditateAction is mandatory when a choice is needed, automatically
        // moves the whole pile when pile size <= magic, and marks returned cards
        // retained before the queued Calm change and end-turn action resolve.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Meditate.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/MeditateAction.java
    insert(cards, CardDef {
                id: "Meditate", name: "Meditate", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: Some("Calm"),
                effect_data: &[
                    E::ChooseCards {
                        source: P::Discard, filter: CardFilter::All, action: ChoiceAction::MoveToHand,
                        min_picks: A::Fixed(0), max_picks: A::Magic,
                        post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                    },
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Meditate+", name: "Meditate+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: Some("Calm"),
                effect_data: &[
                    E::ChooseCards {
                        source: P::Discard, filter: CardFilter::All, action: ChoiceAction::MoveToHand,
                        min_picks: A::Fixed(0), max_picks: A::Magic,
                        post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                    },
                ], complex_hook: None,
            });
}
