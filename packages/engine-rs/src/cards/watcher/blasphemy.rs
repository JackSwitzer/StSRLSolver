use crate::cards::prelude::*;

// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Blasphemy.java
//   use(): enters Divinity, then applies EndTurnDeathPower.
// Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/EndTurnDeathPower.java
//   atStartOfTurn(): LoseHPAction(owner, owner, 99999), then removes the power.
//   upgrade(): makes Blasphemy self-retaining; cost/effect/exhaust stay unchanged.
pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Blasphemy ---- (cost 1, skill, exhaust, enter Divinity, die next turn; upgrade: retain)
    insert(cards, CardDef {
                id: "Blasphemy", name: "Blasphemy", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: Some("Divinity"),
                effect_data: &[
                    E::Simple(SE::SetFlag(BF::Blasphemy)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Blasphemy+", name: "Blasphemy+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: Some("Divinity"),
                effect_data: &[
                    E::Simple(SE::SetFlag(BF::Blasphemy)),
                ], complex_hook: None,
            });
}
