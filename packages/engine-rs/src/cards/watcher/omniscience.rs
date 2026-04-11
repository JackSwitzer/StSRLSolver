use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Omniscience ---- (cost 4, skill, exhaust, choose card from draw pile play it twice; upgrade: cost 3)
        // complex_hook presents draw pile as choices, plays chosen card for free,
        // then adds a cost-0 copy to hand (MCTS approximation of "play it twice").
    insert(cards, CardDef {
                id: "Omniscience", name: "Omniscience", card_type: CardType::Skill,
                target: CardTarget::None, cost: 4, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["omniscience"], effect_data: &[],
                complex_hook: Some(crate::effects::hooks_complex::hook_omniscience),
            });
    insert(cards, CardDef {
                id: "Omniscience+", name: "Omniscience+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["omniscience"], effect_data: &[],
                complex_hook: Some(crate::effects::hooks_complex::hook_omniscience),
            });
}
