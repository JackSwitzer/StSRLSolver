use crate::cards::prelude::*;
use crate::effects::declarative::{Effect, GeneratedCardPool, GeneratedCostRule};

static DISCOVERY: [Effect; 1] = [Effect::GenerateDiscoveryChoice {
    pool: GeneratedCardPool::Colorless,
    option_count: 3,
    preview_cost_rule: GeneratedCostRule::Base,
    selected_cost_rule: GeneratedCostRule::Base,
}];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Discovery: 1 cost, choose 1 of 3 cards to add to hand, exhaust (upgrade: no exhaust)
    insert(cards, CardDef {
                id: "Discovery", name: "Discovery", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["discovery"], effect_data: &DISCOVERY, complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Discovery+", name: "Discovery+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["discovery"], effect_data: &DISCOVERY, complex_hook: None,
            });
}
