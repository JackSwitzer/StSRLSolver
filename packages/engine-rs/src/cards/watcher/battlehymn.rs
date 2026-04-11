use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Battle Hymn ---- (cost 1, power, add Smite to hand each turn; upgrade: innate)
    insert(cards, CardDef {
                id: "BattleHymn", name: "Battle Hymn", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["battle_hymn"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "BattleHymn+", name: "Battle Hymn+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["battle_hymn", "innate"], effect_data: &[], complex_hook: None,
            });
}
