use crate::cards::prelude::*;

// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Judgement.java
// Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/JudgementAction.java
// Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/InstantKillAction.java
// At or below the cutoff, InstantKillAction bypasses ordinary damage and sets
// the target's HP directly to zero.
pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Judgement ---- (cost 1, skill, if enemy HP <= 30, kill it; +10 magic upgrade)
    insert(cards, CardDef {
                id: "Judgement", name: "Judgement", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 30, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::Judgement(A::Magic))], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Judgement+", name: "Judgement+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 40, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::Judgement(A::Magic))], complex_hook: None,
            });
}
