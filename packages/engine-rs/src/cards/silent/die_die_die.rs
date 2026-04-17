use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Silent Rare: Die Die Die ---- (cost 1, 13 AoE dmg, exhaust; +4 dmg)
    insert(cards, CardDef {
        id: "Die Die Die", name: "Die Die Die", card_type: CardType::Attack,
        target: CardTarget::AllEnemy, cost: 1, base_damage: 13, base_block: -1,
        base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Die Die Die+", name: "Die Die Die+", card_type: CardType::Attack,
        target: CardTarget::AllEnemy, cost: 1, base_damage: 17, base_block: -1,
        base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))], complex_hook: None,
    });
}
