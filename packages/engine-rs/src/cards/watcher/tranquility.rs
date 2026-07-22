use crate::cards::prelude::*;

// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Tranquility.java
//   ctor: ID "ClearTheMind", cost 1 SKILL targeting SELF, exhaust + selfRetain.
//   use(): changes stance to Calm.
//   upgrade(): upgradeBaseCost(0) only.
pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Common: Tranquility ---- (cost 1, enter Calm, exhaust, retain; upgrade: cost 0)
    // Java ID: ClearTheMind, run.rs uses Tranquility
    insert(
        cards,
        CardDef {
            id: "ClearTheMind",
            name: "Tranquility",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: true,
            enter_stance: Some("Calm"),
            effect_data: &[E::Simple(SE::ChangeStance(Stance::Calm))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "ClearTheMind+",
            name: "Tranquility+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: true,
            enter_stance: Some("Calm"),
            effect_data: &[E::Simple(SE::ChangeStance(Stance::Calm))],
            complex_hook: None,
        },
    );
}
