use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Trip: 0 cost, apply 2 Vulnerable (upgrade: target all)
    insert(cards, CardDef {
                id: "Trip", name: "Trip", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["apply_vulnerable"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Trip+", name: "Trip+", card_type: CardType::Skill,
                target: CardTarget::AllEnemy, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["apply_vulnerable"], effect_data: &[], complex_hook: None,
            });
}
