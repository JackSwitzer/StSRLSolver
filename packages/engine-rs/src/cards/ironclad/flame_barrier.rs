use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // FlameBarrier.java grants 12 Block and FlameBarrierPower(4);
    // upgrading adds 4 Block and 2 to the power amount.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/FlameBarrier.java
    insert(
        cards,
        CardDef {
            id: "Flame Barrier",
            name: "Flame Barrier",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: 12,
            base_magic: 4,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::FLAME_BARRIER,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Flame Barrier+",
            name: "Flame Barrier+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: 16,
            base_magic: 6,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::FLAME_BARRIER,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
}
