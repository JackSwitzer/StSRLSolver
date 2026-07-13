use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Ironclad Uncommon: Ghostly Armor ----
    // Ghostly Armor gains 10 block and is Ethereal; upgrading adds 3 block
    // and deliberately leaves Ethereal in place.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/GhostlyArmor.java
    insert(cards, CardDef {
        id: "Ghostly Armor",
        name: "Ghostly Armor",
        card_type: CardType::Skill,
        target: CardTarget::SelfTarget,
        cost: 1,
        base_damage: -1,
        base_block: 10,
        base_magic: -1,
        exhaust: false,
        enter_stance: None,
                effect_data: &[E::Simple(SE::GainBlock(A::Block))],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Ghostly Armor+",
        name: "Ghostly Armor+",
        card_type: CardType::Skill,
        target: CardTarget::SelfTarget,
        cost: 1,
        base_damage: -1,
        base_block: 13,
        base_magic: -1,
        exhaust: false,
        enter_stance: None,
                effect_data: &[E::Simple(SE::GainBlock(A::Block))],
        complex_hook: None,
    });
}
