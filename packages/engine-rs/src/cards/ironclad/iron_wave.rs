use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // IronWave.java queues GainBlockAction before DamageAction: 5 Block and
    // 5 damage for one energy; upgradeDamage(2) and upgradeBlock(2).
    insert(cards, CardDef {
                id: "Iron Wave", name: "Iron Wave", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 5, base_block: 5,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::GainBlock(A::Block)), E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Iron Wave+", name: "Iron Wave+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: 7,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::GainBlock(A::Block)), E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
            });
}
