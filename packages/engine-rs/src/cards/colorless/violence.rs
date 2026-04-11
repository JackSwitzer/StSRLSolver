use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Violence: 0 cost, put 3 random Attacks from draw pile into hand, exhaust
    insert(cards, CardDef {
                id: "Violence", name: "Violence", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["draw_attacks_from_draw"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_draw_attacks_from_draw),
            });
    insert(cards, CardDef {
                id: "Violence+", name: "Violence+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: true, enter_stance: None,
                effects: &["draw_attacks_from_draw"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_draw_attacks_from_draw),
            });
}
