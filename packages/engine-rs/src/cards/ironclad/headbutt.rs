use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Ironclad Common: Headbutt ----
    // cost 1, 9 dmg, put card from discard on top of draw; upgrade: +3 dmg
    insert(cards, CardDef {
        id: "Headbutt",
        name: "Headbutt",
        card_type: CardType::Attack,
        target: CardTarget::Enemy,
        cost: 1,
        base_damage: 9,
        base_block: -1,
        base_magic: -1,
        exhaust: false,
        enter_stance: None,
        effects: &[],
        effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
        complex_hook: Some(crate::effects::hooks_complex::hook_discard_to_top_of_draw),
    });
    insert(cards, CardDef {
        id: "Headbutt+",
        name: "Headbutt+",
        card_type: CardType::Attack,
        target: CardTarget::Enemy,
        cost: 1,
        base_damage: 12,
        base_block: -1,
        base_magic: -1,
        exhaust: false,
        enter_stance: None,
        effects: &[],
        effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
        complex_hook: Some(crate::effects::hooks_complex::hook_discard_to_top_of_draw),
    });
}

#[cfg(test)]
#[path = "../../tests/test_zone_batch_java_wave2.rs"]
mod test_zone_batch_java_wave2;
