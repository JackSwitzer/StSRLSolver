use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource, Effect, GeneratedCardPool, GeneratedCostRule};

static INFERNAL_BLADE: [Effect; 1] = [Effect::GenerateRandomCardsToHand {
    pool: GeneratedCardPool::Attack,
    count: AmountSource::Fixed(1),
    cost_rule: GeneratedCostRule::ZeroThisTurn,
}];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
        id: "Infernal Blade",
        name: "Infernal Blade",
        card_type: CardType::Skill,
        target: CardTarget::None,
        cost: 1,
        base_damage: -1,
        base_block: -1,
        base_magic: -1,
        exhaust: true,
        enter_stance: None,
        effects: &["random_attack_to_hand"],
        effect_data: &INFERNAL_BLADE,
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Infernal Blade+",
        name: "Infernal Blade+",
        card_type: CardType::Skill,
        target: CardTarget::None,
        cost: 0,
        base_damage: -1,
        base_block: -1,
        base_magic: -1,
        exhaust: true,
        enter_stance: None,
        effects: &["random_attack_to_hand"],
        effect_data: &INFERNAL_BLADE,
        complex_hook: None,
    });
}
