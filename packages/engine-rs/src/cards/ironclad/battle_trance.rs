use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Sources: cards/red/BattleTrance.java costs 0, queues draw 3 before
    // applying NoDrawPower, and upgrades only the draw by 1.
    insert(
        cards,
        CardDef {
            id: "Battle Trance",
            name: "Battle Trance",
            card_type: CardType::Skill,
            target: CardTarget::None,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 3,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
                E::Simple(SE::AddStatus(T::Player, sid::NO_DRAW, A::Fixed(1))),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Battle Trance+",
            name: "Battle Trance+",
            card_type: CardType::Skill,
            target: CardTarget::None,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 4,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
                E::Simple(SE::AddStatus(T::Player, sid::NO_DRAW, A::Fixed(1))),
            ],
            complex_hook: None,
        },
    );
}
