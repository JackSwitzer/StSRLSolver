use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, Effect as E, GeneratedCardPool, GeneratedCostRule};

static METAMORPHOSIS: [E; 1] = [E::GenerateRandomCardsToDraw {
    pool: GeneratedCardPool::Attack,
    count: A::Magic,
    cost_rule: GeneratedCostRule::ZeroIfPositiveThisTurn,
}];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
        id: "Metamorphosis",
        name: "Metamorphosis",
        card_type: CardType::Skill,
        target: CardTarget::None,
        cost: 2,
        base_damage: -1,
        base_block: -1,
        base_magic: 3,
        exhaust: true,
        enter_stance: None,
        effects: &["random_attacks_to_draw"],
        effect_data: &METAMORPHOSIS,
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Metamorphosis+",
        name: "Metamorphosis+",
        card_type: CardType::Skill,
        target: CardTarget::None,
        cost: 2,
        base_damage: -1,
        base_block: -1,
        base_magic: 5,
        exhaust: true,
        enter_stance: None,
        effects: &["random_attacks_to_draw"],
        effect_data: &METAMORPHOSIS,
        complex_hook: None,
    });
}
