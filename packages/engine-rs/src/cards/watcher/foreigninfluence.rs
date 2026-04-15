use crate::cards::prelude::*;
use crate::effects::declarative::{Effect as E, GeneratedCardPool, GeneratedCostRule};

static FOREIGN_INFLUENCE: [E; 1] = [E::GenerateDiscoveryChoice {
    pool: GeneratedCardPool::AnyColorAttackRarityWeighted,
    option_count: 3,
    preview_cost_rule: GeneratedCostRule::Base,
    selected_cost_rule: GeneratedCostRule::Base,
}];

static FOREIGN_INFLUENCE_PLUS: [E; 1] = [E::GenerateDiscoveryChoice {
    pool: GeneratedCardPool::AnyColorAttackRarityWeighted,
    option_count: 3,
    preview_cost_rule: GeneratedCostRule::Base,
    selected_cost_rule: GeneratedCostRule::ZeroThisTurn,
}];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Foreign Influence ---- (cost 0, skill, exhaust, choose attack from any class; upgrade: chosen card costs 0)
    insert(cards, CardDef {
                id: "ForeignInfluence", name: "Foreign Influence", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &FOREIGN_INFLUENCE, complex_hook: None,
            });
    insert(cards, CardDef {
                id: "ForeignInfluence+", name: "Foreign Influence+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &FOREIGN_INFLUENCE_PLUS, complex_hook: None,
            });
}
