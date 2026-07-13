use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource, Effect, GeneratedCardPool, GeneratedCostRule};

static DISTRACTION: [Effect; 1] = [Effect::GenerateRandomCardsToHand {
    pool: GeneratedCardPool::Skill,
    count: AmountSource::Fixed(1),
    cost_rule: GeneratedCostRule::ZeroThisTurn,
}];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Distraction.java consumes one cardRandom selection from the current
    // character's Skill pool, makes that copy free this turn, and exhausts.
    // Upgrading changes only the base cost from 1 to 0.
    // Java: reference/extracted/methods/card/Distraction.java
    insert(cards, CardDef {
        id: "Distraction",
        name: "Distraction",
        card_type: CardType::Skill,
        target: CardTarget::None,
        cost: 1,
        base_damage: -1,
        base_block: -1,
        base_magic: -1,
        exhaust: true,
        enter_stance: None,
                effect_data: &DISTRACTION,
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Distraction+",
        name: "Distraction+",
        card_type: CardType::Skill,
        target: CardTarget::None,
        cost: 0,
        base_damage: -1,
        base_block: -1,
        base_magic: -1,
        exhaust: true,
        enter_stance: None,
                effect_data: &DISTRACTION,
        complex_hook: None,
    });
}
