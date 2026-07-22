use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: reference/extracted/methods/card/RecklessCharge.java. After its
    // damage, MakeTempCardInDrawPileAction inserts one Dazed at a
    // cardRandomRng-selected non-top index; it does not shuffle the pile.
    insert(
        cards,
        CardDef {
            id: "Reckless Charge",
            name: "Reckless Charge",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 0,
            base_damage: 7,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddCardToRandomDrawSpot("Dazed", A::Fixed(1)))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Reckless Charge+",
            name: "Reckless Charge+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 0,
            base_damage: 10,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddCardToRandomDrawSpot("Dazed", A::Fixed(1)))],
            complex_hook: None,
        },
    );
}
