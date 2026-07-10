use crate::cards::prelude::*;

// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Rushdown.java
//   ctor: ID "Adaptation", cost 1, baseMagicNumber 2, POWER targeting SELF.
//   use(): applies RushdownPower for magicNumber (2) stacks.
//   upgrade(): upgradeBaseCost(0) only; the power amount stays 2.
pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Power Cards ----
    insert(cards, CardDef {
        id: "Adaptation", name: "Rushdown", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::RUSHDOWN, A::Magic)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Adaptation+", name: "Rushdown+", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
        base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::RUSHDOWN, A::Magic)),
        ], complex_hook: None,
    });
}
