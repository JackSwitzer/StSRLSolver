use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Force Field: 4 cost, 12 block, costs 1 less per power played
    insert(cards, CardDef {
        id: "Force Field", name: "Force Field", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 4, base_damage: -1, base_block: 12,
        base_magic: -1, exhaust: false, enter_stance: None,
        effects: &["reduce_cost_per_power"],
        effect_data: &[E::Simple(SE::GainBlock(A::Block))],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Force Field+", name: "Force Field+", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 4, base_damage: -1, base_block: 16,
        base_magic: -1, exhaust: false, enter_stance: None,
        effects: &["reduce_cost_per_power"],
        effect_data: &[E::Simple(SE::GainBlock(A::Block))],
        complex_hook: None,
    });
}
