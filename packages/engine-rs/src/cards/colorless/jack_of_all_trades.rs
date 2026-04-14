use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource, Effect, GeneratedCardPool, GeneratedCostRule};

static JACK_OF_ALL_TRADES: [Effect; 1] = [Effect::GenerateRandomCardsToHand {
    pool: GeneratedCardPool::Colorless,
    count: AmountSource::Fixed(1),
    cost_rule: GeneratedCostRule::Base,
}];

static JACK_OF_ALL_TRADES_PLUS: [Effect; 1] = [Effect::GenerateRandomCardsToHand {
    pool: GeneratedCardPool::Colorless,
    count: AmountSource::Fixed(2),
    cost_rule: GeneratedCostRule::Base,
}];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
        id: "Jack Of All Trades",
        name: "Jack Of All Trades",
        card_type: CardType::Skill,
        target: CardTarget::None,
        cost: 0,
        base_damage: -1,
        base_block: -1,
        base_magic: 1,
        exhaust: true,
        enter_stance: None,
        effects: &["add_random_colorless"],
        effect_data: &JACK_OF_ALL_TRADES,
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Jack Of All Trades+",
        name: "Jack Of All Trades+",
        card_type: CardType::Skill,
        target: CardTarget::None,
        cost: 0,
        base_damage: -1,
        base_block: -1,
        base_magic: 2,
        exhaust: true,
        enter_stance: None,
        effects: &["add_random_colorless"],
        effect_data: &JACK_OF_ALL_TRADES_PLUS,
        complex_hook: None,
    });
}
