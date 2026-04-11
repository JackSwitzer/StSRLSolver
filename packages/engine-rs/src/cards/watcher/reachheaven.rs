use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Reach Heaven ---- (cost 2, 10 dmg, shuffle Through Violence into draw; +5 dmg upgrade)
    insert(cards, CardDef {
                id: "ReachHeaven", name: "Reach Heaven", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 10, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["add_through_violence_to_draw"], effect_data: &[
                    E::Simple(SE::AddCard("ThroughViolence", P::Draw, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "ReachHeaven+", name: "Reach Heaven+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 15, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["add_through_violence_to_draw"], effect_data: &[
                    E::Simple(SE::AddCard("ThroughViolence", P::Draw, A::Fixed(1))),
                ], complex_hook: None,
            });
}
