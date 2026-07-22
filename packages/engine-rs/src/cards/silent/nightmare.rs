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

    // Nightmare.java's class/display name is Nightmare, but its canonical card
    // ID is "Night Terror". NightmareAction stores three copies for next turn;
    // upgrade changes only cost 3 -> 2.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/Nightmare.java
    insert(
        cards,
        CardDef {
            id: "Night Terror",
            name: "Nightmare",
            card_type: CardType::Skill,
            target: CardTarget::None,
            cost: 3,
            base_damage: -1,
            base_block: -1,
            base_magic: 3,
            exhaust: true,
            enter_stance: None,
            effect_data: &NIGHTMARE_EFFECTS,
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Night Terror+",
            name: "Nightmare+",
            card_type: CardType::Skill,
            target: CardTarget::None,
            cost: 2,
            base_damage: -1,
            base_block: -1,
            base_magic: 3,
            exhaust: true,
            enter_stance: None,
            effect_data: &NIGHTMARE_EFFECTS,
            complex_hook: None,
        },
    );
}
