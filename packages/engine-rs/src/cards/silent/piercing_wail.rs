use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Common: Piercing Wail ---- (cost 1, -6 str to all enemies this turn, exhaust; +2 magic)
    insert(cards, CardDef {
                id: "Piercing Wail", name: "Piercing Wail", card_type: CardType::Skill,
                target: CardTarget::AllEnemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 6, exhaust: true, enter_stance: None,
                effects: &["reduce_strength_all_temp"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Piercing Wail+", name: "Piercing Wail+", card_type: CardType::Skill,
                target: CardTarget::AllEnemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 8, exhaust: true, enter_stance: None,
                effects: &["reduce_strength_all_temp"], effect_data: &[], complex_hook: None,
            });
}
