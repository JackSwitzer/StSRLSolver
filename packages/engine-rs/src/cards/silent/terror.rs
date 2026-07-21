use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Terror.java applies 99 Vulnerable and exhausts; upgradeBaseCost is the
        // only upgrade change, reducing cost from 1 to 0.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/Terror.java
    insert(cards, CardDef {
                id: "Terror", name: "Terror", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 99, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Terror+", name: "Terror+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 99, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic)),
                ], complex_hook: None,
            });
}
