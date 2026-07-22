use crate::cards::prelude::*;
use crate::effects::declarative::{Effect, GeneratedCardPool, GeneratedCostRule};

static DISCOVERY: [Effect; 1] = [Effect::GenerateDiscoveryChoice {
    pool: GeneratedCardPool::WatcherAny,
    option_count: 3,
    preview_cost_rule: GeneratedCostRule::Base,
    selected_cost_rule: GeneratedCostRule::ZeroThisTurn,
}];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Discovery.java queues the no-argument DiscoveryAction, which offers
    // three current-character cards and makes the selected copy cost 0 this
    // turn. Upgrading removes Exhaust and changes nothing else.
    // Java: reference/extracted/methods/card/Discovery.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DiscoveryAction.java
    insert(
        cards,
        CardDef {
            id: "Discovery",
            name: "Discovery",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &DISCOVERY,
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Discovery+",
            name: "Discovery+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &DISCOVERY,
            complex_hook: None,
        },
    );
}
