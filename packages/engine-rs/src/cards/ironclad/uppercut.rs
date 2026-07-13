use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // The NORMAL hit resolves before separate Weak and Vulnerable applications;
    // upgrading raises both debuffs from one turn to two without changing damage.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Uppercut.java
    insert(cards, CardDef {
                id: "Uppercut", name: "Uppercut", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 13, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Uppercut+", name: "Uppercut+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 13, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::VULNERABLE, A::Magic)),
                ], complex_hook: None,
            });
}
