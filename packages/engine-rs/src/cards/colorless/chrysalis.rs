use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Chrysalis: 2 cost, shuffle 3 random upgraded Skills into draw pile, exhaust
    insert(cards, CardDef {
                id: "Chrysalis", name: "Chrysalis", card_type: CardType::Skill,
                target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["add_random_skills_to_draw"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_chrysalis),
            });
    insert(cards, CardDef {
                id: "Chrysalis+", name: "Chrysalis+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 5, exhaust: true, enter_stance: None,
                effects: &["add_random_skills_to_draw"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_chrysalis),
            });
}
