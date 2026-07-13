use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: reference/extracted/methods/card/Rebound.java queues damage before
    // applying one ReboundPower stack. ReboundPower.java makes the next
    // non-Power card go to the top of draw.
    insert(cards, CardDef {
                id: "Rebound", name: "Rebound", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Simple(SE::AddStatus(T::Player, sid::REBOUND, A::Fixed(1))),
                ],
                complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Rebound+", name: "Rebound+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Simple(SE::AddStatus(T::Player, sid::REBOUND, A::Fixed(1))),
                ],
                complex_hook: None,
            });
}
