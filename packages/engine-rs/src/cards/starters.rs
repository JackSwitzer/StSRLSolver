//! Shared starter basics plus canonical per-class aliases.
//!
//! Java ships 4 class-specific copies (`Strike_R`/`Strike_G`/`Strike_B`/
//! `Strike_P`, same for `Defend`) that are byte-identical except for the
//! `id` string and class color. The generic `Strike`/`Defend` pair remains for
//! isolated combat fixtures; run state uses canonical class-specific IDs.

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
    // Isolated combat fixtures may share this identical behavior under Strike.
    // decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Strike_Purple.java
    // Watcher ground truth: Defend_Watcher.java uses cost 1, base block 5,
    // and upgradeBlock(3). The runtime intentionally shares this identical
    // behavior under generic Defend/Defend+ fixture definitions.
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

    // The four Java starter Strikes have distinct card IDs/colors but share
    // cost 1, base damage 6, upgradeDamage(3), STRIKE/STARTER_STRIKE tags,
    // and one single-target DamageAction outside debug mode. Preserve their
    // canonical IDs for source/trace-facing state alongside generic Strike.
    // Java: reference/extracted/methods/card/Strike_Blue.java
    // Java: reference/extracted/methods/card/Strike_Green.java
    // Java: reference/extracted/methods/card/Strike_Red.java
    // Java: reference/extracted/methods/card/Strike_Purple.java
    for (base_id, upgraded_id) in [
        ("Strike_B", "Strike_B+"),
        ("Strike_G", "Strike_G+"),
        ("Strike_R", "Strike_R+"),
        ("Strike_P", "Strike_P+"),
    ] {
        insert(
            cards,
            CardDef {
                id: base_id,
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
                id: upgraded_id,
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
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::support::{
        enemy_no_intent, engine_without_start, force_player_turn, play_on_enemy,
    };

    fn assert_canonical_strike(base_id: &str, upgraded_id: &str) {
        let registry = crate::cards::global_registry();
        let base = registry.get(base_id).expect("canonical base Strike");
        let upgraded = registry
            .get(upgraded_id)
            .expect("canonical upgraded Strike");
        assert_eq!((base.cost, base.base_damage), (1, 6));
        assert_eq!((upgraded.cost, upgraded.base_damage), (1, 9));
        assert_eq!(
            base.effect_data,
            &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
        );
        assert!(registry.is_strike(registry.card_id(base_id)));

        let mut upgraded_instance = registry.make_card(base_id);
        registry.upgrade_card(&mut upgraded_instance);
        assert_eq!(registry.card_name(upgraded_instance.def_id), upgraded_id);

        let mut engine = engine_without_start(
            vec![registry.make_card(base_id), upgraded_instance],
            vec![enemy_no_intent("JawWorm", 30, 30)],
            2,
        );
        force_player_turn(&mut engine);
        engine.state.hand = vec![registry.make_card(base_id), upgraded_instance];
        assert!(play_on_enemy(&mut engine, base_id, 0));
        assert!(play_on_enemy(&mut engine, upgraded_id, 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 15);
    }

    // Java: reference/extracted/methods/card/Strike_Blue.java
    #[test]
    fn strike_blue_preserves_canonical_identity_and_source_damage() {
        assert_canonical_strike("Strike_B", "Strike_B+");
    }

    // Java: reference/extracted/methods/card/Strike_Green.java
    #[test]
    fn strike_green_preserves_canonical_identity_and_source_damage() {
        assert_canonical_strike("Strike_G", "Strike_G+");
    }

    // Java: reference/extracted/methods/card/Strike_Red.java
    #[test]
    fn strike_red_preserves_canonical_identity_and_source_damage() {
        assert_canonical_strike("Strike_R", "Strike_R+");
    }
}
