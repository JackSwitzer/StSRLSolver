use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Blizzard: 1 cost, dmg = 2 * frost channeled this combat, AoE
    insert(cards, CardDef {
                id: "Blizzard", name: "Blizzard", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 0, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["damage_per_frost_channeled"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Blizzard+", name: "Blizzard+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 0, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["damage_per_frost_channeled"], effect_data: &[], complex_hook: None,
            });
}
