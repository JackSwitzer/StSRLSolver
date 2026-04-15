use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Common: Consecrate ---- (cost 0, 5 dmg AoE, +3 upgrade)
    insert(cards, CardDef {
        id: "Consecrate", name: "Consecrate", card_type: CardType::Attack,
        target: CardTarget::AllEnemy, cost: 0, base_damage: 5, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Consecrate+", name: "Consecrate+", card_type: CardType::Attack,
        target: CardTarget::AllEnemy, cost: 0, base_damage: 8, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))], complex_hook: None,
    });
}
