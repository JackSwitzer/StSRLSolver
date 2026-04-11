use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Heel Hook ---- (cost 1, 5 dmg, if weak gain 1 energy + draw 1; +3 dmg)
    insert(cards, CardDef {
                id: "Heel Hook", name: "Heel Hook", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 5, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["if_weak_energy_draw"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Heel Hook+", name: "Heel Hook+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["if_weak_energy_draw"], effect_data: &[], complex_hook: None,
            });
}
