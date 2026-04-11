use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Infernal Blade ---- (cost 1, exhaust, add random attack to hand at cost 0; upgrade: cost 0)
    insert(cards, CardDef {
                id: "Infernal Blade", name: "Infernal Blade", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["random_attack_to_hand"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Infernal Blade+", name: "Infernal Blade+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["random_attack_to_hand"], effect_data: &[], complex_hook: None,
            });
}
