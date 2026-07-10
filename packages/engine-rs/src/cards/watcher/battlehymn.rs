use crate::cards::prelude::*;

// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/BattleHymn.java
//   ctor: cost 1 POWER targeting SELF with magicNumber 1.
//   use(): applies BattleHymnPower for magicNumber (1) stack.
//   upgrade(): makes the card innate; cost and power amount stay unchanged.
pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Uncommon: Battle Hymn ---- (cost 1, power, add Smite to hand each turn; upgrade: innate)
    insert(cards, CardDef {
        id: "BattleHymn", name: "Battle Hymn", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::BATTLE_HYMN, A::Magic)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "BattleHymn+", name: "Battle Hymn+", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::BATTLE_HYMN, A::Magic)),
        ], complex_hook: None,
    });
}
