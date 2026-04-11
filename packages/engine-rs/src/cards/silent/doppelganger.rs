use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Rare: Doppelganger ---- (cost X, gain X energy + draw X next turn; upgrade: +1/+1)
    insert(cards, CardDef {
                id: "Doppelganger", name: "Doppelganger", card_type: CardType::Skill,
                target: CardTarget::None, cost: -1, base_damage: -1, base_block: -1,
                base_magic: 0, exhaust: true, enter_stance: None,
                effects: &["x_cost", "doppelganger"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Doppelganger+", name: "Doppelganger+", card_type: CardType::Skill,
                target: CardTarget::None, cost: -1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effects: &["x_cost", "doppelganger"], effect_data: &[], complex_hook: None,
            });
}
