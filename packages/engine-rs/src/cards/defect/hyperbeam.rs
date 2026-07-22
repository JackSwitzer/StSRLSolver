use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Hyperbeam queues its 26-damage AoE before applying FocusPower(-3).
    // FocusPower classifies a negative amount as a DEBUFF, so Artifact can
    // block it; a final lethal AoE clears the queued ApplyPowerAction. Upgrade
    // adds 8 damage and leaves the Focus loss unchanged.
    // Java: reference/extracted/methods/card/Hyperbeam.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/FocusPower.java
    insert(
        cards,
        CardDef {
            id: "Hyperbeam",
            name: "Hyperbeam",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 2,
            base_damage: 26,
            base_block: -1,
            base_magic: 3,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DealDamage(T::AllEnemies, A::Damage)),
                E::Simple(SE::AddStatus(T::Player, sid::FOCUS, A::Fixed(-3))),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Hyperbeam+",
            name: "Hyperbeam+",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 2,
            base_damage: 34,
            base_block: -1,
            base_magic: 3,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DealDamage(T::AllEnemies, A::Damage)),
                E::Simple(SE::AddStatus(T::Player, sid::FOCUS, A::Fixed(-3))),
            ],
            complex_hook: None,
        },
    );
}
