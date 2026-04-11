use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Ritual Dagger: 1 cost, dmg from misc, gain 3 per kill, exhaust
    insert(cards, CardDef {
                id: "RitualDagger", name: "Ritual Dagger", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["ritual_dagger"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "RitualDagger+", name: "Ritual Dagger+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
                base_magic: 5, exhaust: true, enter_stance: None,
                effects: &["ritual_dagger"], effect_data: &[], complex_hook: None,
            });
}
