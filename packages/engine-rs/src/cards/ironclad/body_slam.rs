use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: cards/red/BodySlam.java uses current block as base damage,
    // then applies normal card damage modifiers; upgrading only costs 1 -> 0.
    insert(
        cards,
        CardDef {
            id: "Body Slam",
            name: "Body Slam",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 0,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::PlayerBlock))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Body Slam+",
            name: "Body Slam+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 0,
            base_damage: 0,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::PlayerBlock))],
            complex_hook: None,
        },
    );
}
