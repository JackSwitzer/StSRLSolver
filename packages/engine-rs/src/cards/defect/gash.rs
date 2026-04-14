use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Claw (Java ID: Gash): 0 cost, 3 dmg, all Claw dmg +2 for rest of combat.
    insert(cards, CardDef {
        id: "Gash", name: "Claw", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: -1,
        base_magic: 2, exhaust: false, enter_stance: None,
        effects: &["claw_scaling"], effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::CLAW_BONUS, A::Magic)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Gash+", name: "Claw+", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 0, base_damage: 5, base_block: -1,
        base_magic: 2, exhaust: false, enter_stance: None,
        effects: &["claw_scaling"], effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::CLAW_BONUS, A::Magic)),
        ], complex_hook: None,
    });
}
