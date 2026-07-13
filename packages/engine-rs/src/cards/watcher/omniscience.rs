use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, CardFilter, ChoiceAction, Effect, Pile as P};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // OmniscienceAction removes one selected draw-pile card, forces the
        // original to exhaust, then queues it and a purge-on-use stat-equivalent
        // copy for autoplay with random targets. The upgrade only lowers cost.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Omniscience.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/OmniscienceAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/NewQueueCardAction.java
    insert(cards, CardDef {
                id: "Omniscience", name: "Omniscience", card_type: CardType::Skill,
                target: CardTarget::None, cost: 4, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effect_data: &[Effect::ChooseCards {
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
                effect_data: &[Effect::ChooseCards {
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
