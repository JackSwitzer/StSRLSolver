use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Mind Blast: 2 cost, dmg = draw pile size, innate (upgrade: cost 1)
    insert(cards, CardDef {
                id: "Mind Blast", name: "Mind Blast", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 0, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["damage_from_draw_pile", "innate"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_mind_blast),
            });
    insert(cards, CardDef {
                id: "Mind Blast+", name: "Mind Blast+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 0, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["damage_from_draw_pile", "innate"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_mind_blast),
            });
}
