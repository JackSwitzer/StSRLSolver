use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Ironclad Rare: Bludgeon ---- (cost 3, 32 dmg; +10 dmg)
    insert(cards, CardDef {
        id: "Bludgeon",
        name: "Bludgeon",
        card_type: CardType::Attack,
        target: CardTarget::Enemy,
        cost: 3,
        base_damage: 32,
        base_block: -1,
        base_magic: -1,
        exhaust: false,
        enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Bludgeon+",
        name: "Bludgeon+",
        card_type: CardType::Attack,
        target: CardTarget::Enemy,
        cost: 3,
        base_damage: 42,
        base_block: -1,
        base_magic: -1,
        exhaust: false,
        enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
        complex_hook: None,
    });
}

#[cfg(test)]
#[path = "../../tests/test_card_runtime_ironclad_wave6.rs"]
mod test_card_runtime_ironclad_wave6;
