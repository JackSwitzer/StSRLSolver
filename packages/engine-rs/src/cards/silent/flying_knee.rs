use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Common: Flying Knee ---- (cost 1, 8 dmg, +1 energy next turn; +3 dmg)
    insert(cards, CardDef {
                id: "Flying Knee", name: "Flying Knee", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["next_turn_energy"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Flying Knee+", name: "Flying Knee+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["next_turn_energy"], effect_data: &[], complex_hook: None,
            });
}
