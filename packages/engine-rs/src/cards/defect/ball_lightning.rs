use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Defect Common Cards ----
    // Source: cards/blue/BallLightning.java costs 1, queues 7 damage then
    // channels one Lightning, and upgrades only its damage by 3.
    insert(
        cards,
        CardDef {
            id: "Ball Lightning",
            name: "Ball Lightning",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 7,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::ChannelOrb(OrbType::Lightning, A::Fixed(1)))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Ball Lightning+",
            name: "Ball Lightning+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 10,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::ChannelOrb(OrbType::Lightning, A::Fixed(1)))],
            complex_hook: None,
        },
    );
}
