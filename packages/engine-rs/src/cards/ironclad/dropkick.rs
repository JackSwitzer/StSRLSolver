use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // DropkickAction.java checks Vulnerable, queues damage first, then gains 1
    // energy and draws 1. The upgrade changes only damage from 5 to 8.
    // Java: reference/extracted/methods/card/Dropkick.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DropkickAction.java
    insert(cards, CardDef {
                id: "Dropkick", name: "Dropkick", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 5, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Conditional(
                        crate::effects::declarative::Condition::EnemyHasStatus(sid::VULNERABLE),
                        &[E::Simple(SE::GainEnergy(A::Fixed(1))), E::Simple(SE::DrawCards(A::Fixed(1)))],
                        &[],
                    ),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Dropkick+", name: "Dropkick+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Conditional(
                        crate::effects::declarative::Condition::EnemyHasStatus(sid::VULNERABLE),
                        &[E::Simple(SE::GainEnergy(A::Fixed(1))), E::Simple(SE::DrawCards(A::Fixed(1)))],
                        &[],
                    ),
                ], complex_hook: None,
            });
}
