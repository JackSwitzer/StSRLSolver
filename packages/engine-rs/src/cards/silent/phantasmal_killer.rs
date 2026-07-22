use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: cards/green/PhantasmalKiller.java applies PhantasmalPower now;
    // that power grants DoubleDamagePower at the start of a later turn.
    insert(
        cards,
        CardDef {
            id: "Phantasmal Killer",
            name: "Phantasmal Killer",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::PHANTASMAL,
                A::Fixed(1),
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Phantasmal Killer+",
            name: "Phantasmal Killer+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::PHANTASMAL,
                A::Fixed(1),
            ))],
            complex_hook: None,
        },
    );
}
