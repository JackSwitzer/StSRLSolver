use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/Catalyst.java
        // selects DoublePoisonAction or TriplePoisonAction and always Exhausts.
    insert(cards, CardDef {
                id: "Catalyst", name: "Catalyst", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::MultiplyStatus(T::SelectedEnemy, sid::POISON, 2)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Catalyst+", name: "Catalyst+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::MultiplyStatus(T::SelectedEnemy, sid::POISON, 3)),
                ], complex_hook: None,
            });
}
