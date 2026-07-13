use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Neutralize.java queues one NORMAL damage hit followed by Weak: base is
    // zero cost, 3 damage and 1 Weak; upgrade adds 1 to damage and Weak.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/Neutralize.java
    insert(cards, CardDef {
                id: "Neutralize", name: "Neutralize", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Neutralize+", name: "Neutralize+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                ], complex_hook: None,
            });
}
