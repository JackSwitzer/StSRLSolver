use crate::cards::prelude::*;

// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Collect.java
// Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/CollectAction.java
//   X-cost applies CollectPower for X turns (+1 when upgraded, +2 Chemical X).
//   ApplyPowerAction stacks with an existing CollectPower.
// Java: decompiled/java-src/com/megacrit/cardcrawl/powers/CollectPower.java
//   onEnergyRecharge(): adds ONE upgraded Miracle, then decrements/removes the power.
pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Uncommon: Collect ---- (cost X, skill, exhaust, gain X Miracles next turn; upgrade: X+1)
    insert(
        cards,
        CardDef {
            id: "Collect",
            name: "Collect",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: -1,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::SelfEntity,
                sid::COLLECT_MIRACLES,
                A::XCostPlus(0),
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Collect+",
            name: "Collect+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: -1,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::SelfEntity,
                sid::COLLECT_MIRACLES,
                A::XCostPlus(1),
            ))],
            complex_hook: None,
        },
    );
}
