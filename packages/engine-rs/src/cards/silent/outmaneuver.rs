use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Outmaneuver.java applies EnergizedPower(2), or 3 when upgraded, for one
    // energy. EnergizedPower stacks, grants its amount on the next energy
    // recharge, then removes itself.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/Outmaneuver.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/EnergizedPower.java
    insert(cards, CardDef {
                id: "Outmaneuver", name: "Outmaneuver", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ENERGIZED, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Outmaneuver+", name: "Outmaneuver+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ENERGIZED, A::Magic)),
                ], complex_hook: None,
            });
}
