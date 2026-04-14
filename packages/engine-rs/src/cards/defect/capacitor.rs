use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Capacitor: 1 cost, power, gain 2 orb slots
    insert(cards, CardDef {
                id: "Capacitor", name: "Capacitor", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &[], effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::ORB_SLOTS, A::Magic))], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Capacitor+", name: "Capacitor+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &[], effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::ORB_SLOTS, A::Magic))], complex_hook: None,
            });
}
