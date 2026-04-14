use crate::cards::prelude::*;
use crate::effects::declarative::{BulkAction, Effect, CardFilter, Pile as P};

static ENLIGHTENMENT_PLUS: [Effect; 1] = [Effect::ForEachInPile {
    pile: P::Hand,
    filter: CardFilter::All,
    action: BulkAction::SetCost(1),
}];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Enlightenment: 0 cost, reduce cost of all cards in hand to 1 (this turn, upgrade: permanent)
    insert(cards, CardDef {
                id: "Enlightenment", name: "Enlightenment", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["enlightenment_this_turn"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_enlightenment),
            });
    insert(cards, CardDef {
                id: "Enlightenment+", name: "Enlightenment+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["enlightenment_permanent"], effect_data: &ENLIGHTENMENT_PLUS, complex_hook: None,
            });
}
