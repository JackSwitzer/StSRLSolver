use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Concentrate ---- (cost 0, discard 3, gain 2 energy; -1 discard)
    insert(cards, CardDef {
                id: "Concentrate", name: "Concentrate", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["discard_gain_energy"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Concentrate+", name: "Concentrate+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["discard_gain_energy"], effect_data: &[], complex_hook: None,
            });
}
