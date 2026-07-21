use crate::cards::prelude::*;
use crate::effects::declarative::{BulkAction, Effect, CardFilter, Pile as P};

static ENLIGHTENMENT_PLUS: [Effect; 1] = [Effect::ForEachInPile {
    pile: P::Hand,
    filter: CardFilter::All,
    action: BulkAction::SetCost(1),
}];
static ENLIGHTENMENT_THIS_TURN: [Effect; 1] = [Effect::ForEachInPile {
    pile: P::Hand,
    filter: CardFilter::All,
    action: BulkAction::SetCostForTurn(1),
}];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // EnlightenmentAction.java reduces only current costs above 1 for the turn;
    // upgraded, it also permanently reduces only base costs above 1.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java
    insert(cards, CardDef {
                id: "Enlightenment", name: "Enlightenment", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &ENLIGHTENMENT_THIS_TURN, complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Enlightenment+", name: "Enlightenment+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &ENLIGHTENMENT_PLUS, complex_hook: None,
            });
}
