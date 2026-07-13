use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // LockOn.java deals 8 damage before applying 2 Lock-On for one energy;
    // upgradeDamage(3) and upgradeMagicNumber(1). LockOnPower is a turn-based
    // debuff consumed one stack at end of round.
    insert(cards, CardDef {
                id: "Lockon", name: "Lock-On", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::LOCK_ON, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Lockon+", name: "Lock-On+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::LOCK_ON, A::Magic)),
                ], complex_hook: None,
            });
}
