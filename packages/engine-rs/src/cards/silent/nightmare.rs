use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Rare: Nightmare ---- (cost 3, choose card in hand, add 3 copies next turn, exhaust; upgrade: cost 2)
    insert(cards, CardDef {
                id: "Nightmare", name: "Nightmare", card_type: CardType::Skill,
                target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["nightmare"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Nightmare+", name: "Nightmare+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["nightmare"], effect_data: &[], complex_hook: None,
            });
}
