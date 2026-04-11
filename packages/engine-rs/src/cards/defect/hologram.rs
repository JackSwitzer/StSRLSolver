use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Hologram: 1 cost, 3 block, put card from discard into hand, exhaust (upgrade: no exhaust)
    insert(cards, CardDef {
                id: "Hologram", name: "Hologram", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 3,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["return_from_discard"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Hologram+", name: "Hologram+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["return_from_discard"], effect_data: &[], complex_hook: None,
            });
}
