use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Doppelganger.java has no magic number. DoppelgangerAction.java computes
    // X from energyOnUse, adds 2 for Chemical X, then adds 1 when upgraded and
    // applies matching Energized and DrawCardNextTurn powers when positive.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DoppelgangerAction.java
    insert(cards, CardDef {
                id: "Doppelganger", name: "Doppelganger", card_type: CardType::Skill,
                target: CardTarget::None, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::DOPPELGANGER_ENERGY, A::XCost)),
                    E::Simple(SE::AddStatus(T::Player, sid::DOPPELGANGER_DRAW, A::XCost)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Doppelganger+", name: "Doppelganger+", card_type: CardType::Skill,
                target: CardTarget::None, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::DOPPELGANGER_ENERGY, A::XCostPlus(1))),
                    E::Simple(SE::AddStatus(T::Player, sid::DOPPELGANGER_DRAW, A::XCostPlus(1))),
                ], complex_hook: None,
            });
}
