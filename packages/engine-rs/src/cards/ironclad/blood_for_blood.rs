use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Sources: cards/red/BloodForBlood.java costs 4, deals 18 damage, and
    // tookDamage() reduces cost once per positive damage event. Its upgrade
    // lowers the current/base cost by 1 and adds 4 damage.
    insert(
        cards,
        CardDef {
            id: "Blood for Blood",
            name: "Blood for Blood",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 4,
            base_damage: 18,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Blood for Blood+",
            name: "Blood for Blood+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 3,
            base_damage: 22,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
            complex_hook: None,
        },
    );
}

#[cfg(test)]
#[path = "../../tests/test_card_runtime_ironclad_wave4.rs"]
mod test_card_runtime_ironclad_wave4;
