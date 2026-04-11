use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Distraction ---- (cost 1, add random skill to hand at 0 cost, exhaust; upgrade: cost 0)
    insert(cards, CardDef {
                id: "Distraction", name: "Distraction", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["random_skill_to_hand"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Distraction+", name: "Distraction+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["random_skill_to_hand"], effect_data: &[], complex_hook: None,
            });
}
