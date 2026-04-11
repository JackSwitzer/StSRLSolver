use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Impulse: 1 cost, trigger all orb passives, exhaust (upgrade: no exhaust)
    insert(cards, CardDef {
                id: "Impulse", name: "Impulse", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["trigger_all_passives"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Impulse+", name: "Impulse+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["trigger_all_passives"], effect_data: &[], complex_hook: None,
            });
}
