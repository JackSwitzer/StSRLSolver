use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Secret Technique: 0 cost, choose Skill from draw pile, put in hand, exhaust (upgrade: no exhaust)
    insert(cards, CardDef {
                id: "Secret Technique", name: "Secret Technique", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["search_skill"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_search_skill),
            });
    insert(cards, CardDef {
                id: "Secret Technique+", name: "Secret Technique+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["search_skill"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_search_skill),
            });
}
