use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare (listed): Fasting ---- (Java: Uncommon, cost 2, power, +3 str/dex, -1 energy; +1 magic upgrade)
    insert(cards, CardDef {
                id: "Fasting", name: "Fasting", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["fasting"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Fasting+", name: "Fasting+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effects: &["fasting"], effect_data: &[], complex_hook: None,
            });
}
