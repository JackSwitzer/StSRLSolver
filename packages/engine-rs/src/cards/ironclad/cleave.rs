use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // DamageAllEnemiesAction uses one 8-damage matrix hit per enemy; upgrade adds 3.
    // Source: reference/extracted/methods/card/Cleave.java
    // ---- Ironclad Common: Cleave ---- (cost 1, 8 dmg AoE; +3 dmg)
    insert(
        cards,
        CardDef {
            id: "Cleave",
            name: "Cleave",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 1,
            base_damage: 8,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Cleave+",
            name: "Cleave+",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 1,
            base_damage: 11,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))],
            complex_hook: None,
        },
    );
}
