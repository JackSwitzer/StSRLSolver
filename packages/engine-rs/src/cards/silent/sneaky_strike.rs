use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Common: Sneaky Strike ---- (cost 2, 12 dmg, refund 2 energy if discarded; +4 dmg)
    insert(cards, CardDef {
                id: "Sneaky Strike", name: "Sneaky Strike", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["refund_energy_on_discard"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Sneaky Strike+", name: "Sneaky Strike+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 16, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["refund_energy_on_discard"], effect_data: &[], complex_hook: None,
            });
}
