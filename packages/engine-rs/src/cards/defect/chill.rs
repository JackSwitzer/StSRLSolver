use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Chill: 0 cost, channel 1 Frost per enemy, exhaust (upgrade: innate)
    insert(cards, CardDef {
                id: "Chill", name: "Chill", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effects: &["channel_frost_per_enemy"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Chill+", name: "Chill+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effects: &["channel_frost_per_enemy", "innate"], effect_data: &[], complex_hook: None,
            });
}
