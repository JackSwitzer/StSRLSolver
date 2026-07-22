use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: Discipline.java costs 2 (1 upgraded) and installs
    // DEPRECATEDDisciplinePower. The power stores unspent end-turn energy and
    // draws that many cards at the next turn start.
    // decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Discipline.java
    // decompiled/java-src/com/megacrit/cardcrawl/powers/deprecated/DEPRECATEDDisciplinePower.java
    insert(
        cards,
        CardDef {
            id: "Discipline",
            name: "Discipline",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::SetStatus(
                T::Player,
                sid::DISCIPLINE,
                A::Fixed(1),
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Discipline+",
            name: "Discipline+",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::SetStatus(
                T::Player,
                sid::DISCIPLINE,
                A::Fixed(1),
            ))],
            complex_hook: None,
        },
    );
}
