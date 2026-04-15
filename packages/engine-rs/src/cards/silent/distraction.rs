use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource, Effect, GeneratedCardPool, GeneratedCostRule};

static DISTRACTION: [Effect; 1] = [Effect::GenerateRandomCardsToHand {
    pool: GeneratedCardPool::Skill,
    count: AmountSource::Fixed(1),
    cost_rule: GeneratedCostRule::ZeroThisTurn,
}];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
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
