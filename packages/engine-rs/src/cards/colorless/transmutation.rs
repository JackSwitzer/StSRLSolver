use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource, Effect, GeneratedCardPool, GeneratedCostRule};

static TRANSMUTATION: [Effect; 1] = [Effect::GenerateRandomCardsToHand {
    pool: GeneratedCardPool::Colorless,
    count: AmountSource::XCost,
    cost_rule: GeneratedCostRule::ZeroThisTurn,
}];

static TRANSMUTATION_PLUS: [Effect; 1] = [Effect::GenerateRandomCardsToHand {
    pool: GeneratedCardPool::Colorless,
    count: AmountSource::XCost,
    cost_rule: GeneratedCostRule::ZeroThisTurnAndUpgradeGenerated,
}];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
        id: "Transmutation",
        name: "Transmutation",
        card_type: CardType::Skill,
        target: CardTarget::SelfTarget,
        cost: -1,
        base_damage: -1,
        base_block: -1,
        base_magic: -1,
        exhaust: true,
        enter_stance: None,
        effects: &[],
        effect_data: &TRANSMUTATION,
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Transmutation+",
        name: "Transmutation+",
        card_type: CardType::Skill,
        target: CardTarget::SelfTarget,
        cost: -1,
        base_damage: -1,
        base_block: -1,
        base_magic: -1,
        exhaust: true,
        enter_stance: None,
        effects: &[],
        effect_data: &TRANSMUTATION_PLUS,
        complex_hook: None,
    });
}
