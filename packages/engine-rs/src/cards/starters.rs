//! Shared starter basics plus canonical per-class aliases.
//!
//! Java ships 4 class-specific copies (`Strike_R`/`Strike_G`/`Strike_B`/
//! `Strike_P`, same for `Defend`) that are byte-identical except for the
//! `id` string and class color. The generic `Strike`/`Defend` pair remains
//! for existing Watcher run state, while canonical Defend IDs are registered
//! so source/trace-facing paths can preserve the real card identity.

use crate::cards::prelude::*;

// Preserve the Defect wave-6 test hook originally attached to strike_b.rs.
// The test file itself is unchanged; the hook just needs to mount somewhere
// so `cargo test` discovers it.
#[cfg(test)]
#[path = "../tests/test_card_runtime_defect_wave6.rs"]
mod test_card_runtime_defect_wave6;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Watcher ground truth: Strike_Purple.java uses cost 1, base damage 6,
    // upgradeDamage(3), and an ordinary single-target DamageAction. The
    // runtime intentionally shares this identical behavior under Strike.
    // decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Strike_Purple.java
    // Watcher ground truth: Defend_Watcher.java uses cost 1, base block 5,
    // and upgradeBlock(3). The runtime intentionally shares this identical
    // behavior under the unified Defend/Defend+ starter definitions.
    // decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Defend_Watcher.java
    insert(
        cards,
        CardDef {
            id: "Strike",
            name: "Strike",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 6,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Strike+",
            name: "Strike+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 9,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Defend",
            name: "Defend",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 5,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::GainBlock(A::Block))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Defend+",
            name: "Defend+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 8,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::GainBlock(A::Block))],
            complex_hook: None,
        },
    );

    // Defend_Blue.java, Defend_Green.java, Defend_Red.java, and
    // Defend_Watcher.java all use cost 1, base Block 5, and upgradeBlock(3)
    // outside the game's debug-only 50-Block branch.
    // Java: reference/extracted/methods/card/Defend_Blue.java
    // Java: reference/extracted/methods/card/Defend_Green.java
    // Java: reference/extracted/methods/card/Defend_Red.java
    // Java: reference/extracted/methods/card/Defend_Watcher.java
    for (base_id, upgraded_id) in [
        ("Defend_B", "Defend_B+"),
        ("Defend_G", "Defend_G+"),
        ("Defend_R", "Defend_R+"),
        ("Defend_P", "Defend_P+"),
    ] {
        insert(
            cards,
            CardDef {
                id: base_id,
                name: "Defend",
                card_type: CardType::Skill,
                target: CardTarget::SelfTarget,
                cost: 1,
                base_damage: -1,
                base_block: 5,
                base_magic: -1,
                exhaust: false,
                enter_stance: None,
                effect_data: &[E::Simple(SE::GainBlock(A::Block))],
                complex_hook: None,
            },
        );
        insert(
            cards,
            CardDef {
                id: upgraded_id,
                name: "Defend+",
                card_type: CardType::Skill,
                target: CardTarget::SelfTarget,
                cost: 1,
                base_damage: -1,
                base_block: 8,
                base_magic: -1,
                exhaust: false,
                enter_stance: None,
                effect_data: &[E::Simple(SE::GainBlock(A::Block))],
                complex_hook: None,
            },
        );
    }
}
