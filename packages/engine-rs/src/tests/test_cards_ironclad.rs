#[cfg(test)]
mod ironclad_card_java_parity_tests {
    // Java references:
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/cards/red/*.java

    use crate::actions::Action;
    use crate::status_ids::sid;
    use crate::tests::support::{
        combat_state_with, ensure_in_hand, engine_with, engine_with_enemies, force_player_turn,
        make_deck, make_deck_n, play_card, play_on_enemy, play_self, TEST_SEED, enemy,
        enemy_no_intent, end_turn,
        discard_prefix_count, exhaust_prefix_count, hand_count,
    };
    use crate::cards::{CardDef, CardRegistry, CardTarget, CardType};
    use crate::engine::CombatEngine;

    fn reg() -> &'static CardRegistry {
        crate::cards::global_registry()
    }

    fn card(id: &str) -> CardDef {
        reg().get(id).unwrap().clone()
    }

    fn assert_card(
        id: &str,
        cost: i32,
        damage: i32,
        block: i32,
        magic: i32,
        card_type: CardType,
        target: CardTarget,
        exhaust: bool,
    ) {
        let c = card(id);
        assert_eq!(c.cost, cost, "{id} cost");
        assert_eq!(c.base_damage, damage, "{id} damage");
        assert_eq!(c.base_block, block, "{id} block");
        assert_eq!(c.base_magic, magic, "{id} magic");
        assert_eq!(c.card_type, card_type, "{id} type");
        assert_eq!(c.target, target, "{id} target");
        assert_eq!(c.exhaust, exhaust, "{id} exhaust");
    }

    macro_rules! card_pair_test {
        ($name:ident, $id:literal, $up:literal,
         $bc:expr, $bd:expr, $bb:expr, $bm:expr,
         $uc:expr, $ud:expr, $ub:expr, $um:expr,
         $ty:expr, $target:expr, $exhaust:expr) => {
            mod $name {
                use super::*;

                #[test]
                fn base() {
                    assert_card(
                        $id, $bc, $bd, $bb, $bm, $ty, $target, $exhaust,
                    );
                }

                #[test]
                fn upgraded() {
                    assert_card(
                        $up, $uc, $ud, $ub, $um, $ty, $target, $exhaust,
                    );
                }
            }
        };
        ($name:ident, $id:literal, $up:literal,
         $bc:expr, $bd:expr, $bb:expr, $bm:expr,
         $uc:expr, $ud:expr, $ub:expr, $um:expr,
         $ty:expr, $target:expr, $base_exhaust:expr, $up_exhaust:expr) => {
            mod $name {
                use super::*;

                #[test]
                fn base() {
                    assert_card(
                        $id, $bc, $bd, $bb, $bm, $ty, $target, $base_exhaust,
                    );
                }

                #[test]
                fn upgraded() {
                    assert_card(
                        $up, $uc, $ud, $ub, $um, $ty, $target, $up_exhaust,
                    );
                }
            }
        };
    }

    fn engine_for(
        hand: &[&str],
        draw: &[&str],
        discard: &[&str],
        enemies: Vec<crate::state::EnemyCombatState>,
        energy: i32,
    ) -> CombatEngine {
        let mut state = combat_state_with(
            make_deck(draw),
            enemies,
            energy,
        );
        state.hand = make_deck(hand);
        state.discard_pile = make_deck(discard);
        let mut engine = CombatEngine::new(state, TEST_SEED);
        force_player_turn(&mut engine);
        engine.state.turn = 1;
        engine
    }

    // ------------------------------------------------------------------
    // Base/upgrade parity table
    // ------------------------------------------------------------------

    card_pair_test!(bash, "Bash", "Bash+", 2, 8, -1, 2, 2, 10, -1, 3, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(anger, "Anger", "Anger+", 0, 6, -1, -1, 0, 8, -1, -1, CardType::Attack, CardTarget::Enemy, false);

    #[test]
    fn anger_variants_damage_then_add_a_stat_equivalent_discard_copy() {
        // Anger.java deals 6 (8 upgraded), then adds one
        // makeStatEquivalentCopy to discard. The ordinary played card also
        // reaches discard, while a purge-on-use original disappears; the copy
        // preserves costs, misc, upgrade/free/bottle state but not purge,
        // retain, or exhaust-on-use flags.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Anger.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/AbstractCard.java
        for (card_id, damage) in [("Anger", 6), ("Anger+", 8)] {
            let mut engine = engine_for(
                &[card_id],
                &[],
                &[],
                vec![enemy("JawWorm", 50, 50, 1, 0, 1)],
                0,
            );
            assert!(play_on_enemy(&mut engine, card_id, 0));
            assert_eq!(engine.state.enemies[0].entity.hp, 50 - damage);
            assert_eq!(discard_prefix_count(&engine, card_id), 2);
        }

        let mut dynamic = engine_for(
            &[],
            &[],
            &[],
            vec![enemy("JawWorm", 50, 50, 1, 0, 1)],
            0,
        );
        let mut original = dynamic.card_registry.make_card("Anger+");
        original.cost = 2;
        original.base_cost = 2;
        original.misc = 7;
        original.flags |= crate::combat_types::CardInstance::FLAG_FREE
            | crate::combat_types::CardInstance::FLAG_INNATE
            | crate::combat_types::CardInstance::FLAG_RETAINED
            | crate::combat_types::CardInstance::FLAG_PURGE
            | crate::combat_types::CardInstance::FLAG_EXHAUST_ON_USE;
        dynamic.state.hand = vec![original];

        assert!(play_on_enemy(&mut dynamic, "Anger+", 0));
        assert_eq!(dynamic.state.enemies[0].entity.hp, 42);
        assert_eq!(dynamic.state.discard_pile.len(), 1);
        let copy = dynamic.state.discard_pile[0];
        assert_eq!((copy.cost, copy.base_cost, copy.misc), (2, 2, 7));
        assert!(copy.is_upgraded());
        assert!(copy.is_free());
        assert_ne!(copy.flags & crate::combat_types::CardInstance::FLAG_INNATE, 0);
        assert_eq!(copy.flags & crate::combat_types::CardInstance::FLAG_RETAINED, 0);
        assert_eq!(copy.flags & crate::combat_types::CardInstance::FLAG_PURGE, 0);
        assert_eq!(copy.flags & crate::combat_types::CardInstance::FLAG_EXHAUST_ON_USE, 0);
    }
    card_pair_test!(armaments, "Armaments", "Armaments+", 1, -1, 5, -1, 1, -1, 5, -1, CardType::Skill, CardTarget::SelfTarget, false);

    #[test]
    fn armaments_base_auto_or_selects_only_can_upgrade_cards_and_plus_upgrades_all() {
        // Armaments.java grants 5 Block before ArmamentsAction. Base skips
        // cards whose canUpgrade is false, does nothing with zero candidates,
        // auto-upgrades one candidate, and opens a mandatory choice for many;
        // Armaments+ upgrades every eligible hand card without a choice.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Armaments.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ArmamentsAction.java
        let mut none = engine_for(
            &["Armaments", "AscendersBane", "Strike+"], &[], &[],
            vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 1,
        );
        assert!(play_self(&mut none, "Armaments"));
        assert_eq!(none.phase, crate::engine::CombatPhase::PlayerTurn);
        assert_eq!(none.state.player.block, 5);
        assert!(none.state.hand.iter().any(|card| {
            none.card_registry.card_name(card.def_id) == "AscendersBane"
                && !card.is_upgraded()
        }));

        let mut one = engine_for(
            &["Armaments", "AscendersBane", "Strike"], &[], &[],
            vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 1,
        );
        assert!(play_self(&mut one, "Armaments"));
        assert_eq!(one.phase, crate::engine::CombatPhase::PlayerTurn);
        assert!(one.state.hand.iter().any(|card| {
            one.card_registry.card_name(card.def_id) == "Strike+"
        }));

        let mut many = engine_for(
            &["Armaments", "Strike", "AscendersBane", "Defend"], &[], &[],
            vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 1,
        );
        assert!(play_self(&mut many, "Armaments"));
        assert_eq!(many.phase, crate::engine::CombatPhase::AwaitingChoice);
        assert_eq!(many.state.player.block, 5);
        let choice = many.choice.as_ref().expect("Armaments choice");
        assert_eq!(choice.options.len(), 2);
        assert!(choice.options.iter().all(|option| {
            matches!(option, crate::engine::ChoiceOption::HandCard(index)
                if many.card_registry.card_name(many.state.hand[*index].def_id) != "AscendersBane")
        }));
        let defend_choice = choice.options.iter().position(|option| {
            matches!(option, crate::engine::ChoiceOption::HandCard(index)
                if many.card_registry.card_name(many.state.hand[*index].def_id) == "Defend")
        }).expect("Defend choice");
        many.execute_action(&Action::Choose(defend_choice));
        assert!(many.state.hand.iter().any(|card| {
            many.card_registry.card_name(card.def_id) == "Defend+"
        }));
        assert!(many.state.hand.iter().any(|card| {
            many.card_registry.card_name(card.def_id) == "Strike"
        }));

        let mut plus = engine_for(
            &["Armaments+", "Strike", "AscendersBane", "Defend", "Strike+"], &[], &[],
            vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 1,
        );
        assert!(play_self(&mut plus, "Armaments+"));
        assert_eq!(plus.phase, crate::engine::CombatPhase::PlayerTurn);
        assert_eq!(plus.state.player.block, 5);
        assert!(plus.state.hand.iter().any(|card| {
            plus.card_registry.card_name(card.def_id) == "Defend+"
        }));
        assert_eq!(plus.state.hand.iter().filter(|card| {
            plus.card_registry.card_name(card.def_id) == "Strike+"
        }).count(), 2);
        assert!(plus.state.hand.iter().any(|card| {
            plus.card_registry.card_name(card.def_id) == "AscendersBane"
                && !card.is_upgraded()
        }));
    }
    card_pair_test!(body_slam, "Body Slam", "Body Slam+", 1, 0, -1, -1, 0, 0, -1, -1, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(clash, "Clash", "Clash+", 0, 14, -1, -1, 0, 18, -1, -1, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(cleave, "Cleave", "Cleave+", 1, 8, -1, -1, 1, 11, -1, -1, CardType::Attack, CardTarget::AllEnemy, false);
    card_pair_test!(clothesline, "Clothesline", "Clothesline+", 2, 12, -1, 2, 2, 14, -1, 3, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(flex, "Flex", "Flex+", 0, -1, -1, 2, 0, -1, -1, 4, CardType::Skill, CardTarget::SelfTarget, false);

    #[test]
    fn flex_uses_artifact_blockable_lose_strength_power() {
        // Flex.java applies StrengthPower before LoseStrengthPower of the same
        // amount. LoseStrengthPower is a DEBUFF and removes that Strength at
        // end of turn; Artifact consumes itself on only the second application,
        // leaving the Strength gain permanent.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Flex.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/LoseStrengthPower.java
        let mut ordinary = engine_for(
            &["Flex+"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
            0,
        );
        assert!(play_self(&mut ordinary, "Flex+"));
        assert_eq!(ordinary.state.player.strength(), 4);
        assert_eq!(ordinary.state.player.status(sid::LOSE_STRENGTH), 4);
        end_turn(&mut ordinary);
        assert_eq!(ordinary.state.player.strength(), 0);
        assert_eq!(ordinary.state.player.status(sid::LOSE_STRENGTH), 0);

        let mut artifact = engine_for(
            &["Flex+"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
            0,
        );
        artifact.state.player.set_status(sid::ARTIFACT, 1);
        assert!(play_self(&mut artifact, "Flex+"));
        assert_eq!(artifact.state.player.strength(), 4);
        assert_eq!(artifact.state.player.status(sid::LOSE_STRENGTH), 0);
        assert_eq!(artifact.state.player.status(sid::ARTIFACT), 0);
        end_turn(&mut artifact);
        assert_eq!(artifact.state.player.strength(), 4);
    }
    card_pair_test!(havoc, "Havoc", "Havoc+", 1, -1, -1, -1, 0, -1, -1, -1, CardType::Skill, CardTarget::None, false);
    card_pair_test!(headbutt, "Headbutt", "Headbutt+", 1, 9, -1, -1, 1, 12, -1, -1, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(heavy_blade, "Heavy Blade", "Heavy Blade+", 2, 14, -1, 3, 2, 14, -1, 5, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(iron_wave, "Iron Wave", "Iron Wave+", 1, 5, 5, -1, 1, 7, 7, -1, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(perfected_strike, "Perfected Strike", "Perfected Strike+", 2, 6, -1, 2, 2, 6, -1, 3, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(pommel_strike, "Pommel Strike", "Pommel Strike+", 1, 9, -1, 1, 1, 10, -1, 2, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(shrug_it_off, "Shrug It Off", "Shrug It Off+", 1, -1, 8, -1, 1, -1, 11, -1, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(sword_boomerang, "Sword Boomerang", "Sword Boomerang+", 1, 3, -1, 3, 1, 3, -1, 4, CardType::Attack, CardTarget::AllEnemy, false);
    card_pair_test!(thunderclap, "Thunderclap", "Thunderclap+", 1, 4, -1, 1, 1, 7, -1, 1, CardType::Attack, CardTarget::AllEnemy, false);
    card_pair_test!(true_grit, "True Grit", "True Grit+", 1, -1, 7, -1, 1, -1, 9, -1, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(twin_strike, "Twin Strike", "Twin Strike+", 1, 5, -1, 2, 1, 7, -1, 2, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(warcry, "Warcry", "Warcry+", 0, -1, -1, 1, 0, -1, -1, 2, CardType::Skill, CardTarget::SelfTarget, true);
    card_pair_test!(wild_strike, "Wild Strike", "Wild Strike+", 1, 12, -1, -1, 1, 17, -1, -1, CardType::Attack, CardTarget::Enemy, false);

    card_pair_test!(battle_trance, "Battle Trance", "Battle Trance+", 0, -1, -1, 3, 0, -1, -1, 4, CardType::Skill, CardTarget::None, false);
    card_pair_test!(blood_for_blood, "Blood for Blood", "Blood for Blood+", 4, 18, -1, -1, 3, 22, -1, -1, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(bloodletting, "Bloodletting", "Bloodletting+", 0, -1, -1, 2, 0, -1, -1, 3, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(burning_pact, "Burning Pact", "Burning Pact+", 1, -1, -1, 2, 1, -1, -1, 3, CardType::Skill, CardTarget::None, false);

    #[test]
    fn burning_pact_auto_resolves_small_hands_then_draws_and_choices_large_hands() {
        // BurningPact.java queues ExhaustAction(1, false) before its 2-card
        // draw (3 upgraded). ExhaustAction immediately finishes on an empty
        // hand, auto-exhausts the whole hand when size <= amount, and opens a
        // mandatory selection only when more than one card remains.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/BurningPact.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ExhaustAction.java
        let mut empty = engine_for(
            &["Burning Pact+"],
            &["Strike", "Defend", "Bash"],
            &[],
            vec![enemy_no_intent("Dummy", 50, 50)],
            1,
        );
        assert!(play_self(&mut empty, "Burning Pact+"));
        assert_eq!(empty.phase, crate::engine::CombatPhase::PlayerTurn);
        assert_eq!(empty.state.hand.len(), 3);
        assert_eq!(empty.state.energy, 0);

        let mut singleton = engine_for(
            &["Burning Pact", "Strike"],
            &["Defend", "Bash"],
            &[],
            vec![enemy_no_intent("Dummy", 50, 50)],
            1,
        );
        assert!(play_self(&mut singleton, "Burning Pact"));
        assert_eq!(singleton.phase, crate::engine::CombatPhase::PlayerTurn);
        assert_eq!(exhaust_prefix_count(&singleton, "Strike"), 1);
        assert_eq!(singleton.state.hand.len(), 2);

        let mut choice = engine_for(
            &["Burning Pact+", "Strike", "Defend"],
            &["Anger", "Cleave", "Bash"],
            &[],
            vec![enemy_no_intent("Dummy", 50, 50)],
            1,
        );
        assert!(play_self(&mut choice, "Burning Pact+"));
        assert_eq!(choice.phase, crate::engine::CombatPhase::AwaitingChoice);
        assert_eq!(choice.state.hand.len(), 2, "draw waits for exhaust selection");
        let defend_option = choice.choice.as_ref().unwrap().options.iter().position(|option| {
            matches!(option, crate::engine::ChoiceOption::HandCard(index)
                if choice.card_registry.card_name(choice.state.hand[*index].def_id) == "Defend")
        }).expect("Defend exhaust option");
        choice.execute_action(&Action::Choose(defend_option));
        assert_eq!(choice.phase, crate::engine::CombatPhase::PlayerTurn);
        assert_eq!(exhaust_prefix_count(&choice, "Defend"), 1);
        assert_eq!(choice.state.hand.len(), 4);
    }
    card_pair_test!(carnage, "Carnage", "Carnage+", 2, 20, -1, -1, 2, 28, -1, -1, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(combust, "Combust", "Combust+", 1, -1, -1, 5, 1, -1, -1, 7, CardType::Power, CardTarget::SelfTarget, false);
    card_pair_test!(dark_embrace, "Dark Embrace", "Dark Embrace+", 2, -1, -1, 1, 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false);

    #[test]
    fn dark_embrace_stacks_draws_per_exhaust_and_stops_when_monsters_are_dead() {
        // Source: DarkEmbrace.java applies one stack and changes only cost on
        // upgrade. DarkEmbracePower.java draws `amount` on each exhaust unless
        // areMonstersBasicallyDead() is true.
        let mut active = engine_for(
            &["Dark Embrace+", "Dark Embrace+", "Seeing Red+"],
            &["Strike", "Defend"],
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
            2,
        );
        assert!(play_self(&mut active, "Dark Embrace+"));
        assert!(play_self(&mut active, "Dark Embrace+"));
        assert_eq!(active.state.energy, 0);
        assert_eq!(active.state.player.status(sid::DARK_EMBRACE), 2);

        assert!(play_self(&mut active, "Seeing Red+"));
        assert_eq!(active.state.hand.len(), 2);
        assert_eq!(active.state.draw_pile.len(), 0);

        let mut defeated = engine_for(
            &[],
            &["Strike"],
            &[],
            vec![enemy_no_intent("JawWorm", 0, 40)],
            0,
        );
        defeated.state.player.set_status(sid::DARK_EMBRACE, 1);
        defeated.trigger_on_exhaust();
        assert!(defeated.state.hand.is_empty());
        assert_eq!(defeated.state.draw_pile.len(), 1);
    }
    card_pair_test!(disarm, "Disarm", "Disarm+", 1, -1, -1, 2, 1, -1, -1, 3, CardType::Skill, CardTarget::Enemy, true);

    #[test]
    fn disarm_permanently_reduces_strength_unless_artifact_blocks_it() {
        // Disarm.java applies StrengthPower(-2), upgraded to -3, and queues no
        // restoration power. Negative StrengthPower is blocked by Artifact.
        let mut base = engine_for(
            &["Disarm"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
            1,
        );
        base.state.enemies[0].entity.set_status(sid::STRENGTH, 5);

        assert!(play_on_enemy(&mut base, "Disarm", 0));
        assert_eq!(base.state.enemies[0].entity.status(sid::STRENGTH), 3);
        assert_eq!(base.state.enemies[0].entity.status(sid::TEMP_STRENGTH_LOSS), 0);
        assert_eq!(exhaust_prefix_count(&base, "Disarm"), 1);
        base.execute_action(&Action::EndTurn);
        assert_eq!(base.state.enemies[0].entity.status(sid::STRENGTH), 3);

        let mut blocked = engine_for(
            &["Disarm+"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
            1,
        );
        blocked.state.enemies[0].entity.set_status(sid::STRENGTH, 5);
        blocked.state.enemies[0].entity.set_status(sid::ARTIFACT, 1);

        assert!(play_on_enemy(&mut blocked, "Disarm+", 0));
        assert_eq!(blocked.state.enemies[0].entity.status(sid::ARTIFACT), 0);
        assert_eq!(blocked.state.enemies[0].entity.status(sid::STRENGTH), 5);
        assert_eq!(blocked.state.enemies[0].entity.status(sid::TEMP_STRENGTH_LOSS), 0);
    }
    card_pair_test!(dropkick, "Dropkick", "Dropkick+", 1, 5, -1, -1, 1, 8, -1, -1, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(dual_wield, "Dual Wield", "Dual Wield+", 1, -1, -1, 1, 1, -1, -1, 2, CardType::Skill, CardTarget::None, false);
    card_pair_test!(entrench, "Entrench", "Entrench+", 2, -1, -1, -1, 1, -1, -1, -1, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(evolve, "Evolve", "Evolve+", 1, -1, -1, 1, 1, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false);

    #[test]
    fn evolve_draws_its_power_amount_after_each_status_draw() {
        // EvolvePower.onCardDraw checks for CardType.STATUS and queues a
        // DrawCardAction for its stacked amount. Evolve+ installs amount two.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/EvolvePower.java
        let mut upgraded = engine_for(
            &["Evolve+"],
            &["Defend", "Strike", "Wound"],
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
            1,
        );
        assert!(play_self(&mut upgraded, "Evolve+"));
        assert_eq!(upgraded.state.player.status(sid::EVOLVE), 2);

        upgraded.draw_cards(1);
        assert_eq!(upgraded.state.hand.len(), 3);
        assert!(upgraded.state.draw_pile.is_empty());

        let mut non_status = engine_for(
            &["Evolve"],
            &["Wound", "Strike"],
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
            1,
        );
        assert!(play_self(&mut non_status, "Evolve"));
        non_status.draw_cards(1);
        assert_eq!(non_status.state.hand.len(), 1);
        assert_eq!(non_status.state.draw_pile.len(), 1);
        assert_eq!(
            non_status.card_registry.card_name(non_status.state.hand[0].def_id),
            "Strike"
        );
    }
    card_pair_test!(feel_no_pain, "Feel No Pain", "Feel No Pain+", 1, -1, -1, 3, 1, -1, -1, 4, CardType::Power, CardTarget::SelfTarget, false);

    #[test]
    fn feel_no_pain_installs_and_gains_raw_block_on_each_exhaust() {
        // FeelNoPainPower.onExhaust queues GainBlockAction(amount). Dexterity
        // and Frail alter True Grit's card Block, but not this power action.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/FeelNoPain.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/FeelNoPainPower.java
        let mut engine = engine_for(
            &["Feel No Pain+", "True Grit", "Defend"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
            2,
        );
        engine.state.player.set_status(sid::DEXTERITY, 5);
        engine.state.player.set_status(sid::FRAIL, 1);

        assert!(play_self(&mut engine, "Feel No Pain+"));
        assert_eq!(engine.state.player.status(sid::FEEL_NO_PAIN), 4);
        assert!(play_self(&mut engine, "True Grit"));

        // True Grit: floor((7 + 5) * 0.75) = 9; Feel No Pain adds raw 4.
        assert_eq!(engine.state.player.block, 13);
        assert_eq!(exhaust_prefix_count(&engine, "Defend"), 1);
    }
    card_pair_test!(fire_breathing, "Fire Breathing", "Fire Breathing+", 1, -1, -1, 6, 1, -1, -1, 10, CardType::Power, CardTarget::SelfTarget, false);

    #[test]
    fn fire_breathing_status_and_curse_draws_deal_thorns_damage_to_all_enemies() {
        // FireBreathingPower.onCardDraw triggers only for Status and Curse
        // cards, using a pure damage matrix and DamageType.THORNS. Flight and
        // Slow therefore neither alter the damage nor consume Flight stacks.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/FireBreathingPower.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/FlightPower.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/SlowPower.java
        let mut engine = engine_for(
            &["Fire Breathing+"],
            &[],
            &[],
            vec![
                enemy_no_intent("Byrd", 60, 60),
                enemy_no_intent("GiantHead", 60, 60),
            ],
            1,
        );
        assert!(play_self(&mut engine, "Fire Breathing+"));
        assert_eq!(engine.state.player.status(sid::FIRE_BREATHING), 10);
        engine.state.enemies[0].entity.set_status(sid::FLIGHT, 3);
        engine.state.enemies[1].entity.set_status(sid::SLOW, 3);

        engine.state.draw_pile = make_deck(&["Wound"]);
        engine.draw_cards(1);
        assert_eq!(engine.state.enemies[0].entity.hp, 50);
        assert_eq!(engine.state.enemies[1].entity.hp, 50);
        assert_eq!(engine.state.enemies[0].entity.status(sid::FLIGHT), 3);

        engine.state.draw_pile = make_deck(&["Doubt"]);
        engine.draw_cards(1);
        assert_eq!(engine.state.enemies[0].entity.hp, 40);
        assert_eq!(engine.state.enemies[1].entity.hp, 40);
        assert_eq!(engine.state.enemies[0].entity.status(sid::FLIGHT), 3);

        engine.state.draw_pile = make_deck(&["Strike"]);
        engine.draw_cards(1);
        assert_eq!(engine.state.enemies[0].entity.hp, 40);
        assert_eq!(engine.state.enemies[1].entity.hp, 40);
    }
    card_pair_test!(flame_barrier, "Flame Barrier", "Flame Barrier+", 2, -1, 12, 4, 2, -1, 16, 6, CardType::Skill, CardTarget::SelfTarget, false);

    #[test]
    fn flame_barrier_plus_retaliates_per_fully_blocked_hit_then_expires() {
        // FlameBarrier.java grants 16 Block and FlameBarrierPower(6) when
        // upgraded. FlameBarrierPower.onAttacked has no positive-damage guard,
        // so each of two fully blocked NORMAL hits returns 6 THORNS damage;
        // atStartOfTurn then removes the power.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/FlameBarrier.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/FlameBarrierPower.java
        let mut engine = engine_for(
            &["Flame Barrier+"],
            &[],
            &[],
            vec![enemy("Cultist", 60, 60, 1, 1, 2)],
            2,
        );

        assert!(play_self(&mut engine, "Flame Barrier+"));
        assert_eq!(engine.state.player.block, 16);
        assert_eq!(engine.state.player.status(sid::FLAME_BARRIER), 6);
        let player_hp = engine.state.player.hp;

        end_turn(&mut engine);

        assert_eq!(engine.state.player.hp, player_hp);
        assert_eq!(engine.state.enemies[0].entity.hp, 48);
        assert_eq!(engine.state.player.status(sid::FLAME_BARRIER), 0);
    }
    card_pair_test!(ghostly_armor, "Ghostly Armor", "Ghostly Armor+", 1, -1, 10, -1, 1, -1, 13, -1, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(hemokinesis, "Hemokinesis", "Hemokinesis+", 1, 15, -1, 2, 1, 20, -1, 2, CardType::Attack, CardTarget::Enemy, false);

    #[test]
    fn hemokinesis_loses_two_hp_before_its_damage_action() {
        // Hemokinesis.java queues LoseHPAction before DamageAction. The HP loss
        // makes RupturePower add Strength to the top of the action queue, so
        // the same Hemokinesis hit gets that Strength. Upgrade is damage-only.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Hemokinesis.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/RupturePower.java
        for (card, printed_damage) in [("Hemokinesis", 15), ("Hemokinesis+", 20)] {
            let mut engine = engine_for(
                &[card],
                &[],
                &[],
                vec![enemy_no_intent("JawWorm", 50, 50)],
                3,
            );
            engine.state.player.set_status(sid::RUPTURE, 1);
            let player_hp_before = engine.state.player.hp;

            assert!(play_on_enemy(&mut engine, card, 0));

            assert_eq!(engine.state.player.hp, player_hp_before - 2);
            assert_eq!(engine.state.player.status(sid::STRENGTH), 1);
            assert_eq!(engine.state.enemies[0].entity.hp, 50 - printed_damage - 1);
        }
    }
    card_pair_test!(infernal_blade, "Infernal Blade", "Infernal Blade+", 1, -1, -1, -1, 0, -1, -1, -1, CardType::Skill, CardTarget::None, true);
    card_pair_test!(inflame, "Inflame", "Inflame+", 1, -1, -1, 2, 1, -1, -1, 3, CardType::Power, CardTarget::SelfTarget, false);
    card_pair_test!(intimidate, "Intimidate", "Intimidate+", 0, -1, -1, 1, 0, -1, -1, 2, CardType::Skill, CardTarget::AllEnemy, true);

    #[test]
    fn intimidate_source_applies_one_or_two_weak_to_each_enemy_and_exhausts() {
        // Intimidate.java queues one WeakPower application for every monster.
        // Base magicNumber is 1 and upgradeMagicNumber(1) raises it to 2;
        // ApplyPowerAction lets each target's Artifact block independently.
        let mut engine = engine_for(
            &["Intimidate", "Intimidate+"],
            &[],
            &[],
            vec![
                enemy_no_intent("JawWorm", 40, 40),
                enemy_no_intent("Cultist", 40, 40),
            ],
            0,
        );
        engine.state.enemies[1].entity.set_status(sid::ARTIFACT, 1);

        assert!(play_self(&mut engine, "Intimidate"));
        assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 1);
        assert_eq!(engine.state.enemies[1].entity.status(sid::WEAKENED), 0);
        assert_eq!(engine.state.enemies[1].entity.status(sid::ARTIFACT), 0);

        assert!(play_self(&mut engine, "Intimidate+"));
        assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 3);
        assert_eq!(engine.state.enemies[1].entity.status(sid::WEAKENED), 2);
        assert_eq!(engine.state.energy, 0);
        assert_eq!(exhaust_prefix_count(&engine, "Intimidate"), 2);
    }
    card_pair_test!(metallicize, "Metallicize", "Metallicize+", 1, -1, -1, 3, 1, -1, -1, 4, CardType::Power, CardTarget::SelfTarget, false);

    #[test]
    fn metallicize_source_gains_fixed_block_before_end_turn_hand_cards() {
        // MetallicizePower.atEndOfTurnPreEndTurnCards queues fixed GainBlock,
        // unaffected by Dexterity/Frail. Four Block absorbs Burn+'s four damage
        // first, leaving Buffer to prevent the following enemy attack.
        let mut engine = engine_for(
            &["Metallicize+", "Burn+"],
            &[],
            &[],
            vec![enemy("JawWorm", 40, 40, 1, 5, 1)],
            3,
        );
        engine.state.player.set_status(sid::BUFFER, 1);
        engine.state.player.set_status(sid::DEXTERITY, -99);
        engine.state.player.set_status(sid::FRAIL, 1);
        let hp_before = engine.state.player.hp;

        assert!(play_self(&mut engine, "Metallicize+"));
        assert_eq!(engine.state.player.status(sid::METALLICIZE), 4);
        end_turn(&mut engine);

        assert_eq!(engine.state.player.hp, hp_before);
        assert_eq!(engine.state.player.status(sid::BUFFER), 0);
        assert_eq!(engine.state.player.status(sid::METALLICIZE), 4);
    }
    card_pair_test!(power_through, "Power Through", "Power Through+", 1, -1, 15, -1, 1, -1, 20, -1, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(pummel, "Pummel", "Pummel+", 1, 2, -1, 4, 1, 2, -1, 5, CardType::Attack, CardTarget::Enemy, true);
    card_pair_test!(rage, "Rage", "Rage+", 0, -1, -1, 3, 0, -1, -1, 5, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(rampage, "Rampage", "Rampage+", 1, 8, -1, 5, 1, 8, -1, 8, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(reckless_charge, "Reckless Charge", "Reckless Charge+", 0, 7, -1, -1, 0, 10, -1, -1, CardType::Attack, CardTarget::Enemy, false);

    #[test]
    fn reckless_charge_inserts_dazed_with_card_random_without_shuffling_after_nonlethal_damage() {
        // RecklessCharge.java queues DamageAction before
        // MakeTempCardInDrawPileAction(Dazed, 1, true, true). randomSpot=true
        // delegates to CardGroup.addToRandomSpot: a nonempty pile consumes one
        // cardRandomRng tick and inserts below the top while preserving the
        // relative order of every existing card. A combat-ending DamageAction
        // clears the later MakeTempCard action entirely.
        // Sources: reference/extracted/methods/card/RecklessCharge.java;
        // decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInDrawPileAction.java;
        // decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java; and
        // decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java.
        let mut engine = engine_for(
            &["Reckless Charge+"],
            &["Strike", "Defend", "Bash"],
            &[],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            0,
        );
        let card_random_before = engine.card_random_rng.counter;
        let shuffle_before = engine.rng.counter;

        assert!(play_on_enemy(&mut engine, "Reckless Charge+", 0));

        assert_eq!(engine.state.enemies[0].entity.hp, 30);
        assert_eq!(engine.card_random_rng.counter, card_random_before + 1);
        assert_eq!(engine.rng.counter, shuffle_before);
        let existing: Vec<_> = engine
            .state
            .draw_pile
            .iter()
            .filter_map(|card| {
                let id = engine.card_registry.card_name(card.def_id);
                (id != "Dazed").then_some(id)
            })
            .collect();
        assert_eq!(existing, vec!["Strike", "Defend", "Bash"]);
        assert_eq!(
            engine
                .card_registry
                .card_name(engine.state.draw_pile.last().expect("draw top").def_id),
            "Bash"
        );

        let mut empty = engine_for(
            &["Reckless Charge"],
            &[],
            &[],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            0,
        );
        let card_random_before = empty.card_random_rng.counter;
        assert!(play_on_enemy(&mut empty, "Reckless Charge", 0));
        assert_eq!(empty.state.enemies[0].entity.hp, 33);
        assert_eq!(empty.card_random_rng.counter, card_random_before);
        assert_eq!(empty.state.draw_pile.len(), 1);
        assert_eq!(
            empty.card_registry.card_name(empty.state.draw_pile[0].def_id),
            "Dazed"
        );

        let mut lethal = engine_for(
            &["Reckless Charge+"],
            &["Strike"],
            &[],
            vec![enemy("JawWorm", 10, 10, 1, 0, 1)],
            0,
        );
        let card_random_before = lethal.card_random_rng.counter;
        assert!(play_on_enemy(&mut lethal, "Reckless Charge+", 0));
        assert_eq!(lethal.card_random_rng.counter, card_random_before);
        assert!(lethal.state.draw_pile.iter().all(|card| {
            lethal.card_registry.card_name(card.def_id) != "Dazed"
        }));
    }
    card_pair_test!(rupture, "Rupture", "Rupture+", 1, -1, -1, 1, 1, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false);

    #[test]
    fn rupture_triggers_only_for_positive_hp_loss_owned_by_the_player() {
        // Rupture.java applies magic 1 (2 upgraded) as RupturePower. Its
        // wasHPLost hook requires both positive HP loss and DamageInfo.owner ==
        // player: Bloodletting's LoseHPAction(p, p, 3) qualifies, while an
        // enemy's ordinary DamageAction does not.
        // Java: reference/extracted/methods/card/Rupture.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/RupturePower.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Bloodletting.java
        let mut self_owned = engine_for(
            &["Rupture+", "Bloodletting"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
            1,
        );
        let hp_before = self_owned.state.player.hp;

        assert!(play_self(&mut self_owned, "Rupture+"));
        assert!(play_self(&mut self_owned, "Bloodletting"));
        assert_eq!(self_owned.state.player.hp, hp_before - 3);
        assert_eq!(self_owned.state.player.status(sid::STRENGTH), 2);

        let mut enemy_owned = engine_for(
            &["Rupture+"],
            &[],
            &[],
            vec![enemy("Cultist", 40, 40, 1, 5, 1)],
            1,
        );
        let hp_before = enemy_owned.state.player.hp;

        assert!(play_self(&mut enemy_owned, "Rupture+"));
        end_turn(&mut enemy_owned);
        assert_eq!(enemy_owned.state.player.hp, hp_before - 5);
        assert_eq!(enemy_owned.state.player.status(sid::STRENGTH), 0);
    }

    card_pair_test!(searing_blow, "Searing Blow", "Searing Blow+", 2, 12, -1, -1, 2, 16, -1, -1, CardType::Attack, CardTarget::Enemy, false);

    #[test]
    fn searing_blow_repeated_upgrades_preserve_counter_and_damage_curve() {
        // SearingBlow.upgrade adds 4 + the previous timesUpgraded, increments
        // that counter, and canUpgrade always returns true. Three upgrades from
        // 12 therefore produce 16, then 21, then 27 damage.
        // Java: reference/extracted/methods/card/SearingBlow.java
        let registry = reg();
        let mut searing = registry.make_card("Searing Blow");
        assert_eq!(searing.misc, 0);
        for expected_level in 1..=3 {
            assert!(registry.can_upgrade_card(&searing));
            registry.upgrade_card(&mut searing);
            assert_eq!(searing.misc, expected_level);
            assert_eq!(registry.card_name(searing.def_id), "Searing Blow+");
        }
        assert!(registry.can_upgrade_card(&searing));

        let mut engine = engine_for(
            &[],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 50, 50)],
            2,
        );
        engine.state.hand.push(searing);

        assert!(play_on_enemy(&mut engine, "Searing Blow+", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 23);
    }

    card_pair_test!(second_wind, "Second Wind", "Second Wind+", 1, -1, 5, -1, 1, -1, 7, -1, CardType::Skill, CardTarget::SelfTarget, false);

    #[test]
    fn second_wind_plus_modifies_and_gains_block_once_per_non_attack() {
        // BlockPerNonAttackAction snapshots the two remaining non-Attacks,
        // exhausts them, and queues two GainBlockAction(this.block) calls.
        // Second Wind+ has block 7, so two Dexterity makes each event 9; two
        // Juggernaut procs each select through cardRandomRng even with one enemy.
        // Java: reference/extracted/methods/card/SecondWind.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/
        // BlockPerNonAttackAction.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/JuggernautPower.java
        let mut engine = engine_for(
            &["Second Wind+", "Defend", "Battle Trance", "Strike"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 50, 50)],
            1,
        );
        engine.state.player.set_status(sid::DEXTERITY, 2);
        engine.state.player.set_status(sid::JUGGERNAUT, 5);
        let card_random_before = engine.card_random_rng.counter;

        assert!(play_self(&mut engine, "Second Wind+"));

        assert_eq!(engine.state.player.block, 18);
        assert_eq!(engine.state.enemies[0].entity.hp, 40);
        assert_eq!(engine.card_random_rng.counter, card_random_before + 2);
        assert_eq!(exhaust_prefix_count(&engine, "Defend"), 1);
        assert_eq!(exhaust_prefix_count(&engine, "Battle Trance"), 1);
        assert_eq!(hand_count(&engine, "Strike"), 1);
    }

    card_pair_test!(seeing_red, "Seeing Red", "Seeing Red+", 1, -1, -1, 2, 0, -1, -1, 2, CardType::Skill, CardTarget::None, true);
    card_pair_test!(sentinel, "Sentinel", "Sentinel+", 1, -1, 5, 2, 1, -1, 8, 3, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(sever_soul, "Sever Soul", "Sever Soul+", 2, 16, -1, -1, 2, 22, -1, -1, CardType::Attack, CardTarget::Enemy, false);

    #[test]
    fn sever_soul_exhausts_all_non_attacks_before_lethal_damage() {
        // SeverSoul.java queues ExhaustAllNonAttackAction before DamageAction.
        // Even when the 16-damage hit ends combat, Sentinel+ has already
        // exhausted and granted three energy.
        let mut engine = engine_for(
            &["Sever Soul", "Sentinel+", "Defend", "Strike"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 16, 16)],
            2,
        );

        assert!(play_on_enemy(&mut engine, "Sever Soul", 0));

        assert!(engine.state.player_won);
        assert_eq!(engine.state.enemies[0].entity.hp, 0);
        assert_eq!(engine.state.energy, 3);
        assert_eq!(exhaust_prefix_count(&engine, "Sentinel"), 1);
        assert_eq!(exhaust_prefix_count(&engine, "Defend"), 1);
        assert_eq!(hand_count(&engine, "Strike"), 1);
    }

    #[test]
    fn sever_soul_plus_deals_twenty_two_after_exhausting_every_non_attack() {
        let mut engine = engine_for(
            &["Sever Soul+", "Strike", "Defend", "Wound", "Inflame"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 50, 50)],
            2,
        );

        assert!(play_on_enemy(&mut engine, "Sever Soul+", 0));

        assert_eq!(engine.state.enemies[0].entity.hp, 28);
        assert_eq!(hand_count(&engine, "Strike"), 1);
        assert_eq!(exhaust_prefix_count(&engine, "Defend"), 1);
        assert_eq!(exhaust_prefix_count(&engine, "Wound"), 1);
        assert_eq!(exhaust_prefix_count(&engine, "Inflame"), 1);
        assert_eq!(discard_prefix_count(&engine, "Sever Soul"), 1);
    }
    card_pair_test!(shockwave, "Shockwave", "Shockwave+", 2, -1, -1, 3, 2, -1, -1, 5, CardType::Skill, CardTarget::AllEnemy, true);

    #[test]
    fn shockwave_applies_weak_then_vulnerable_to_every_monster_and_exhausts() {
        // Shockwave.java queues Weak before Vulnerable for each monster. With
        // one Artifact, the first monster therefore blocks Weak, consumes its
        // Artifact, and still receives Vulnerable; an unprotected monster gets
        // both debuffs. The base card applies three stacks and exhausts.
        let mut protected = enemy_no_intent("Sentry", 40, 40);
        protected.entity.set_status(sid::ARTIFACT, 1);
        let mut engine = engine_for(
            &["Shockwave"],
            &[],
            &[],
            vec![protected, enemy_no_intent("JawWorm", 40, 40)],
            2,
        );

        assert!(play_self(&mut engine, "Shockwave"));

        assert_eq!(engine.state.enemies[0].entity.status(sid::ARTIFACT), 0);
        assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 0);
        assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 3);
        assert_eq!(engine.state.enemies[1].entity.status(sid::WEAKENED), 3);
        assert_eq!(engine.state.enemies[1].entity.status(sid::VULNERABLE), 3);
        assert_eq!(engine.state.energy, 0);
        assert_eq!(exhaust_prefix_count(&engine, "Shockwave"), 1);
    }

    card_pair_test!(spot_weakness, "Spot Weakness", "Spot Weakness+", 1, -1, -1, 3, 1, -1, -1, 4, CardType::Skill, CardTarget::Enemy, false);
    card_pair_test!(uppercut, "Uppercut", "Uppercut+", 2, 13, -1, 1, 2, 13, -1, 2, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(whirlwind, "Whirlwind", "Whirlwind+", -1, 5, -1, -1, -1, 8, -1, -1, CardType::Attack, CardTarget::AllEnemy, false);

    card_pair_test!(barricade, "Barricade", "Barricade+", 3, -1, -1, -1, 2, -1, -1, -1, CardType::Power, CardTarget::SelfTarget, false);
    card_pair_test!(berserk, "Berserk", "Berserk+", 0, -1, -1, 2, 0, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false);
    card_pair_test!(bludgeon, "Bludgeon", "Bludgeon+", 3, 32, -1, -1, 3, 42, -1, -1, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(brutality, "Brutality", "Brutality+", 0, -1, -1, 1, 0, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false);

    #[test]
    fn brutality_applies_one_post_draw_hp_loss_stack_and_upgrade_is_innate_only() {
        // Brutality.java applies exactly one stack for zero energy; its upgrade
        // changes only isInnate. BrutalityPower.atStartOfTurnPostDraw queues one
        // draw before LoseHPAction(amount), whose HP_LOSS damage can be stopped
        // by Buffer without touching block.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Brutality.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/BrutalityPower.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/LoseHPAction.java
        let base = card("Brutality");
        let plus = card("Brutality+");
        assert!(!base.runtime_traits().innate);
        assert!(plus.runtime_traits().innate);
        assert_eq!(
            crate::powers::defs::DEF_BRUTALITY.triggers[0].trigger,
            crate::effects::trigger::Trigger::TurnStartPostDraw,
        );

        for (card_id, buffered) in [("Brutality", false), ("Brutality+", true)] {
            let mut engine = engine_for(
                &[card_id],
                &["Strike", "Defend", "Bash", "Inflame", "Anger", "Cleave"],
                &[],
                vec![enemy_no_intent("Dummy", 60, 60)],
                0,
            );
            if buffered {
                engine.state.player.set_status(sid::BUFFER, 1);
                engine.state.player.block = 7;
            }
            let hp_before = engine.state.player.hp;

            assert!(play_self(&mut engine, card_id));
            assert_eq!(engine.state.player.status(sid::BRUTALITY), 1);
            assert_eq!(engine.state.energy, 0);
            engine.execute_action(&Action::EndTurn);

            assert_eq!(engine.state.turn, 2, "{card_id}");
            assert_eq!(engine.state.hand.len(), 6, "{card_id}");
            if buffered {
                assert_eq!(engine.state.player.hp, hp_before, "{card_id}");
                assert_eq!(engine.state.player.status(sid::BUFFER), 0, "{card_id}");
                assert_eq!(engine.state.player.block, 0, "old block clears before draw");
            } else {
                assert_eq!(engine.state.player.hp, hp_before - 1, "{card_id}");
                assert_eq!(engine.state.player.status(sid::HP_LOSS_THIS_COMBAT), 1);
            }
        }
    }
    card_pair_test!(corruption, "Corruption", "Corruption+", 3, -1, -1, -1, 2, -1, -1, -1, CardType::Power, CardTarget::SelfTarget, false);

    #[test]
    fn corruption_plus_makes_skills_free_and_exhausts_only_skills() {
        // Sources: Corruption.java upgrades only cost 3 -> 2;
        // ApplyPowerAction.java reduces Skills already in hand/draw pile;
        // CorruptionPower.java marks used Skills, but not Attacks, to Exhaust.
        let mut engine = engine_for(
            &["Corruption+", "Defend", "Strike"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
            3,
        );

        assert!(play_self(&mut engine, "Corruption+"));
        assert_eq!(engine.state.energy, 1);
        assert_eq!(engine.state.player.status(sid::CORRUPTION), 1);

        assert!(play_self(&mut engine, "Defend"));
        assert_eq!(engine.state.energy, 1);
        assert_eq!(engine.state.player.block, 5);
        assert_eq!(exhaust_prefix_count(&engine, "Defend"), 1);

        assert!(play_on_enemy(&mut engine, "Strike", 0));
        assert_eq!(engine.state.energy, 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 34);
        assert_eq!(discard_prefix_count(&engine, "Strike"), 1);
        assert_eq!(exhaust_prefix_count(&engine, "Strike"), 0);
    }

    card_pair_test!(demon_form, "Demon Form", "Demon Form+", 3, -1, -1, 2, 3, -1, -1, 3, CardType::Power, CardTarget::None, false);
    card_pair_test!(double_tap, "Double Tap", "Double Tap+", 1, -1, -1, 1, 1, -1, -1, 2, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(exhume, "Exhume", "Exhume+", 1, -1, -1, -1, 0, -1, -1, -1, CardType::Skill, CardTarget::None, true);

    #[test]
    fn exhume_excludes_itself_and_uses_java_singleton_selection_rules() {
        // ExhumeAction immediately returns a lone non-Exhume, but removes all
        // Exhumes before a multi-card grid choice and no-ops if none remain.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ExhumeAction.java
        let mut singleton = engine_for(
            &["Exhume"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
            1,
        );
        singleton.state.exhaust_pile = make_deck(&["Strike"]);
        assert!(singleton.get_legal_actions().contains(&Action::PlayCard {
            card_idx: 0,
            target_idx: -1,
        }));
        assert!(play_self(&mut singleton, "Exhume"));
        assert_eq!(singleton.phase, crate::engine::CombatPhase::PlayerTurn);
        assert_eq!(singleton.state.hand.len(), 1);
        assert_eq!(
            singleton.card_registry.card_name(singleton.state.hand[0].def_id),
            "Strike"
        );
        assert_eq!(exhaust_prefix_count(&singleton, "Exhume"), 1);

        let mut only_exhumes = engine_for(
            &["Exhume+"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
            0,
        );
        only_exhumes.state.exhaust_pile = make_deck(&["Exhume", "Exhume+"]);
        assert!(only_exhumes.get_legal_actions().contains(&Action::PlayCard {
            card_idx: 0,
            target_idx: -1,
        }));
        assert!(play_self(&mut only_exhumes, "Exhume+"));
        assert_eq!(only_exhumes.phase, crate::engine::CombatPhase::PlayerTurn);
        assert!(only_exhumes.state.hand.is_empty());
        assert_eq!(exhaust_prefix_count(&only_exhumes, "Exhume"), 3);

        let mut filtered_choice = engine_for(
            &["Exhume"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 40, 40)],
            1,
        );
        filtered_choice.state.exhaust_pile = make_deck(&["Exhume+", "Strike"]);
        assert!(play_self(&mut filtered_choice, "Exhume"));
        assert_eq!(filtered_choice.phase, crate::engine::CombatPhase::AwaitingChoice);
        let choice = filtered_choice.choice.as_ref().expect("Exhume choice");
        assert_eq!(choice.options.len(), 1);
        assert!(matches!(
            choice.options[0],
            crate::engine::ChoiceOption::ExhaustCard(index)
                if filtered_choice.card_registry.card_name(
                    filtered_choice.state.exhaust_pile[index].def_id
                ) == "Strike"
        ));
        filtered_choice.execute_action(&Action::Choose(0));
        assert_eq!(filtered_choice.phase, crate::engine::CombatPhase::PlayerTurn);
        assert_eq!(filtered_choice.state.hand.len(), 1);
        assert_eq!(
            filtered_choice.card_registry.card_name(filtered_choice.state.hand[0].def_id),
            "Strike"
        );
    }
    card_pair_test!(feed, "Feed", "Feed+", 1, 10, -1, 3, 1, 12, -1, 4, CardType::Attack, CardTarget::Enemy, true);
    card_pair_test!(fiend_fire, "Fiend Fire", "Fiend Fire+", 2, 7, -1, -1, 2, 10, -1, -1, CardType::Attack, CardTarget::Enemy, true);
    card_pair_test!(immolate, "Immolate", "Immolate+", 2, 21, -1, -1, 2, 28, -1, -1, CardType::Attack, CardTarget::AllEnemy, false);

    #[test]
    fn immolate_aoe_adds_burn_only_when_combat_continues() {
        // Immolate.java queues 21 AoE (28 upgraded) then one Burn. When the
        // AoE is final lethal, DamageAllEnemiesAction clears the later
        // MakeTempCardInDiscardAction before it resolves.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Immolate.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAllEnemiesAction.java
        for (card, damage) in [("Immolate", 21), ("Immolate+", 28)] {
            let mut engine = engine_for(
                &[card],
                &[],
                &[],
                vec![
                    enemy_no_intent("JawWorm", 50, 50),
                    enemy_no_intent("Cultist", 50, 50),
                ],
                2,
            );

            assert!(play_self(&mut engine, card));

            assert_eq!(engine.state.enemies[0].entity.hp, 50 - damage);
            assert_eq!(engine.state.enemies[1].entity.hp, 50 - damage);
            assert_eq!(discard_prefix_count(&engine, "Burn"), 1);
        }

        let mut lethal = engine_for(
            &["Immolate"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 21, 21)],
            2,
        );
        assert!(play_self(&mut lethal, "Immolate"));
        assert!(lethal.state.is_victory());
        assert_eq!(discard_prefix_count(&lethal, "Burn"), 0);
    }
    card_pair_test!(impervious, "Impervious", "Impervious+", 2, -1, 30, -1, 2, -1, 40, -1, CardType::Skill, CardTarget::SelfTarget, true);
    card_pair_test!(juggernaut, "Juggernaut", "Juggernaut+", 2, -1, -1, 5, 2, -1, -1, 7, CardType::Power, CardTarget::SelfTarget, false);

    #[test]
    fn juggernaut_source_uses_card_random_thorns_on_each_positive_block_gain() {
        // JuggernautPower.onGainedBlock queues one DamageRandomEnemyAction when
        // blockAmount > 0. It rolls a living target with cardRandomRng and deals
        // 5 THORNS damage (7 upgraded), which does not consume Flight or trigger
        // Malleable. Zero Block queues nothing and consumes no RNG.
        // Java: powers/JuggernautPower.java and actions/common/DamageRandomEnemyAction.java.
        for (card_id, damage) in [("Juggernaut", 5), ("Juggernaut+", 7)] {
            let mut engine = engine_for(
                &[card_id, "Defend"],
                &[],
                &[],
                vec![
                    enemy_no_intent("JawWorm", 40, 40),
                    enemy_no_intent("Cultist", 40, 40),
                ],
                3,
            );
            for enemy in &mut engine.state.enemies {
                enemy.entity.set_status(sid::FLIGHT, 2);
                enemy.entity.set_status(sid::MALLEABLE, 1);
            }

            assert!(play_self(&mut engine, card_id));
            assert_eq!(engine.state.player.status(sid::JUGGERNAUT), damage);
            let card_random_before_zero = engine.card_random_rng.counter;
            engine.gain_block_player(0);
            assert_eq!(engine.card_random_rng.counter, card_random_before_zero);

            let mut oracle = engine.card_random_rng.clone();
            let expected_target = oracle.random(1) as usize;
            let general_before = engine.rng.counter;
            assert!(play_self(&mut engine, "Defend"));

            for (idx, enemy) in engine.state.enemies.iter().enumerate() {
                assert_eq!(enemy.entity.hp, 40 - i32::from(idx == expected_target) * damage);
                assert_eq!(enemy.entity.status(sid::FLIGHT), 2);
                assert_eq!(enemy.entity.status(sid::MALLEABLE), 1);
            }
            assert_eq!(engine.state.player.block, 5);
            assert_eq!(engine.card_random_rng.counter, oracle.counter);
            assert_eq!(engine.rng.counter, general_before);
        }
    }
    card_pair_test!(limit_break, "Limit Break", "Limit Break+", 1, -1, -1, -1, 1, -1, -1, -1, CardType::Skill, CardTarget::SelfTarget, true, false);
    card_pair_test!(offering, "Offering", "Offering+", 0, -1, -1, 3, 0, -1, -1, 5, CardType::Skill, CardTarget::SelfTarget, true);
    // Reaper.java sets exhaust=true in the constructor; upgradeDamage(1) does
    // not remove it, so both the 4- and 5-damage variants Exhaust.
    card_pair_test!(reaper, "Reaper", "Reaper+", 2, 4, -1, -1, 2, 5, -1, -1, CardType::Attack, CardTarget::AllEnemy, true);

    // ------------------------------------------------------------------
    // Deep behavior checks for cards that are already wired through Rust.
    // ------------------------------------------------------------------

    #[test]
    fn double_tap_stacks_and_replays_one_attack_per_stack() {
        // DoubleTap.java applies one DoubleTapPower per base copy, and
        // DoubleTapPower.onUseCard decrements one amount after queuing one
        // purge-on-use copy of each Attack. Two stacks cover two Attacks.
        // Java: reference/extracted/methods/card/DoubleTap.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/DoubleTapPower.java
        let mut engine = engine_for(
            &["Double Tap", "Double Tap", "Strike", "Strike"],
            &[],
            &[],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            4,
        );

        assert!(play_self(&mut engine, "Double Tap"));
        assert!(play_self(&mut engine, "Double Tap"));
        assert_eq!(engine.state.player.status(sid::DOUBLE_TAP), 2);

        assert!(play_on_enemy(&mut engine, "Strike", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 28);
        assert_eq!(engine.state.player.status(sid::DOUBLE_TAP), 1);

        assert!(play_on_enemy(&mut engine, "Strike", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 16);
        assert_eq!(engine.state.player.status(sid::DOUBLE_TAP), 0);
        assert_eq!(engine.state.energy, 0);
    }

    #[test]
    fn dropkick_refunds_and_draws_when_its_target_was_vulnerable() {
        // DropkickAction.java checks Vulnerable before queuing its 5 damage,
        // then queues GainEnergyAction(1) and DrawCardAction(1). The benefit
        // therefore still occurs when the damage kills that target.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DropkickAction.java
        let mut vulnerable = engine_for(
            &["Dropkick"],
            &["Defend"],
            &[],
            vec![
                enemy("JawWorm", 5, 5, 1, 0, 1),
                enemy("Cultist", 20, 20, 1, 0, 1),
            ],
            1,
        );
        vulnerable.state.enemies[0].entity.set_status(sid::VULNERABLE, 1);

        assert!(play_on_enemy(&mut vulnerable, "Dropkick", 0));
        assert_eq!(vulnerable.state.enemies[0].entity.hp, 0);
        assert_eq!(vulnerable.state.energy, 1);
        assert!(vulnerable.state.hand.iter().any(|card| {
            vulnerable.card_registry.card_name(card.def_id) == "Defend"
        }));

        let mut ordinary = engine_for(
            &["Dropkick"],
            &["Defend"],
            &[],
            vec![enemy("JawWorm", 20, 20, 1, 0, 1)],
            1,
        );
        assert!(play_on_enemy(&mut ordinary, "Dropkick", 0));
        assert_eq!(ordinary.state.enemies[0].entity.hp, 15);
        assert_eq!(ordinary.state.energy, 0);
        assert!(ordinary.state.hand.is_empty());
    }

    #[test]
    fn entrench_doubles_current_block_without_dexterity_or_frail_modifiers() {
        // DoubleYourBlockAction.java passes currentBlock straight to addBlock;
        // it does not calculate a card Block amount, so +2 Dexterity and Frail
        // leave 13 + 13 = 26 for both card variants.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/DoubleYourBlockAction.java
        for (card_id, energy) in [("Entrench", 2), ("Entrench+", 1)] {
            let mut engine = engine_for(
                &[card_id],
                &[],
                &[],
                vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
                energy,
            );
            engine.state.player.block = 13;
            engine.state.player.set_status(sid::DEXTERITY, 2);
            engine.state.player.set_status(sid::FRAIL, 1);

            assert!(play_self(&mut engine, card_id));
            assert_eq!(engine.state.player.block, 26);
            assert_eq!(engine.state.energy, 0);
        }
    }

    #[test]
    fn bash_applies_vulnerable() {
        let mut e = engine_with(make_deck_n("Bash", 5), 50, 0);
        ensure_in_hand(&mut e, "Bash");
        let hp = e.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut e, "Bash", 0));
        assert_eq!(e.state.enemies[0].entity.hp, hp - 8);
        assert_eq!(e.state.enemies[0].entity.status(sid::VULNERABLE), 2);
    }

    #[test]
    fn bash_plus_deals_damage_before_applying_its_vulnerable() {
        // Source: Bash.java queues DamageAction before ApplyPowerAction;
        // upgradeDamage(2) and upgradeMagicNumber(1) produce 10 damage and 3 Vulnerable.
        let mut engine = engine_for(
            &["Bash+"],
            &[],
            &[],
            vec![enemy("JawWorm", 40, 40, 1, 0, 1)],
            3,
        );

        assert!(play_on_enemy(&mut engine, "Bash+", 0));

        assert_eq!(engine.state.enemies[0].entity.hp, 30);
        assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 3);
        assert_eq!(engine.state.energy, 1);
    }

    #[test]
    fn body_slam_uses_current_block() {
        let mut e = engine_for(&["Body Slam"], &[], &[], vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 3);
        e.state.player.block = 13;
        let hp = e.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut e, "Body Slam", 0));
        assert_eq!(e.state.enemies[0].entity.hp, hp - 13);
    }

    #[test]
    fn clash_requires_only_attacks() {
        let e = engine_for(
            &["Clash", "Defend"],
            &[],
            &[],
            vec![enemy("JawWorm", 50, 50, 1, 0, 1)],
            3,
        );
        let clash_idx = e.state.hand.iter().position(|card| e.card_registry.card_name(card.def_id) == "Clash").expect("Clash should be in hand");
        assert!(
            !e.get_legal_actions().iter().any(|action| matches!(
                action,
                Action::PlayCard { card_idx, target_idx }
                    if *card_idx == clash_idx && *target_idx == 0
            ))
        );
    }

    #[test]
    fn cleave_hits_all_enemies() {
        let mut e = engine_with_enemies(
            make_deck_n("Cleave", 5),
            vec![
                enemy("JawWorm", 40, 40, 1, 0, 1),
                enemy("Cultist", 40, 40, 1, 0, 1),
            ],
            3,
        );
        ensure_in_hand(&mut e, "Cleave");
        let hp0 = e.state.enemies[0].entity.hp;
        let hp1 = e.state.enemies[1].entity.hp;
        assert!(play_on_enemy(&mut e, "Cleave", 0));
        assert_eq!(e.state.enemies[0].entity.hp, hp0 - 8);
        assert_eq!(e.state.enemies[1].entity.hp, hp1 - 8);
    }

    #[test]
    fn clothesline_applies_weak() {
        let mut e = engine_for(&["Clothesline"], &[], &[], vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 3);
        let hp = e.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut e, "Clothesline", 0));
        assert_eq!(e.state.enemies[0].entity.hp, hp - 12);
        assert_eq!(e.state.enemies[0].entity.status(sid::WEAKENED), 2);
    }

    #[test]
    fn clothesline_plus_damages_before_artifact_blocks_weak() {
        // Source: Clothesline.java queues DamageAction before ApplyPowerAction;
        // upgradeDamage(2) and upgradeMagicNumber(1) produce 14 damage and 3 Weak.
        let mut e = engine_for(&["Clothesline+"], &[], &[], vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 3);
        e.state.enemies[0].entity.set_status(sid::ARTIFACT, 1);

        assert!(play_on_enemy(&mut e, "Clothesline+", 0));

        assert_eq!(e.state.enemies[0].entity.hp, 36);
        assert_eq!(e.state.enemies[0].entity.status(sid::ARTIFACT), 0);
        assert_eq!(e.state.enemies[0].entity.status(sid::WEAKENED), 0);
    }

    #[test]
    fn iron_wave_source_blocks_before_damage_and_sharp_hide_retaliation() {
        // IronWave.java queues GainBlockAction before DamageAction. UseCardAction
        // then queues SharpHidePower's THORNS retaliation after the card's own
        // actions, so Iron Wave's newly gained block absorbs that retaliation.
        // Upgrade adds exactly two damage and two block.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/IronWave.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/SharpHidePower.java
        for (card_id, amount, hp_loss, block_left) in [
            ("Iron Wave", 5, 1, 0),
            ("Iron Wave+", 7, 0, 1),
        ] {
            let mut engine = engine_for(
                &[card_id],
                &[],
                &[],
                vec![enemy_no_intent("JawWorm", 50, 50)],
                3,
            );
            engine.state.enemies[0].entity.set_status(sid::SHARP_HIDE, 6);
            let player_hp = engine.state.player.hp;

            assert!(play_on_enemy(&mut engine, card_id, 0));

            assert_eq!(engine.state.enemies[0].entity.hp, 50 - amount);
            assert_eq!(engine.state.player.hp, player_hp - hp_loss);
            assert_eq!(engine.state.player.block, block_left);
            assert_eq!(engine.state.energy, 2);
        }
    }

    #[test]
    fn pommel_strike_draws_one() {
        let mut e = engine_for(&["Pommel Strike"], &["Strike"], &[], vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 3);
        let hand = e.state.hand.len();
        assert!(play_on_enemy(&mut e, "Pommel Strike", 0));
        assert_eq!(e.state.hand.len(), hand);
    }

    #[test]
    fn shrug_it_off_blocks_and_draws() {
        let mut e = engine_for(&["Shrug It Off"], &["Strike"], &[], vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 3);
        let hand = e.state.hand.len();
        assert!(play_self(&mut e, "Shrug It Off"));
        assert_eq!(e.state.player.block, 8);
        assert_eq!(e.state.hand.len(), hand);
    }

    #[test]
    fn sword_boomerang_hits_three_times_with_one_enemy() {
        let mut e = engine_for(&["Sword Boomerang"], &[], &[], vec![enemy("JawWorm", 60, 60, 1, 0, 1)], 3);
        let hp = e.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut e, "Sword Boomerang", 0));
        assert_eq!(e.state.enemies[0].entity.hp, hp - 9);
    }

    #[test]
    fn thunderclap_applies_vulnerable_all() {
        let mut e = engine_for(
            &["Thunderclap"],
            &[],
            &[],
            vec![
                enemy("JawWorm", 40, 40, 1, 0, 1),
                enemy("Cultist", 40, 40, 1, 0, 1),
            ],
            3,
        );
        assert!(play_card(&mut e, "Thunderclap", 0));
        assert_eq!(e.state.enemies[0].entity.status(sid::VULNERABLE), 1);
        assert_eq!(e.state.enemies[1].entity.status(sid::VULNERABLE), 1);
        assert_eq!(e.state.enemies[0].entity.hp, 36);
        assert_eq!(e.state.enemies[1].entity.hp, 36);
    }

    #[test]
    fn twin_strike_hits_twice() {
        let mut e = engine_for(&["Twin Strike"], &[], &[], vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 3);
        let hp = e.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut e, "Twin Strike", 0));
        assert_eq!(e.state.enemies[0].entity.hp, hp - 10);
    }

    #[test]
    fn warcry_draws_and_exhausts_itself() {
        let mut e = engine_for(&["Warcry"], &["Strike"], &[], vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 3);
        assert!(play_self(&mut e, "Warcry"));
        assert_eq!(e.phase, crate::engine::CombatPhase::AwaitingChoice);
        e.execute_action(&Action::Choose(0));
        assert_eq!(e.state.hand.len(), 0);
        assert_eq!(e.state.draw_pile.len(), 1);
        assert_eq!(exhaust_prefix_count(&e, "Warcry"), 1);
    }

    #[test]
    fn battle_trance_draws_three() {
        let mut e = engine_for(
            &["Battle Trance"],
            &["Strike", "Strike", "Strike"],
            &[],
            vec![enemy("JawWorm", 50, 50, 1, 0, 1)],
            3,
        );
        assert!(play_self(&mut e, "Battle Trance"));
        assert_eq!(e.state.hand.len(), 3);
    }

    #[test]
    fn seeing_red_grants_energy() {
        // SeeingRed.java spends the base cost of one, then queues
        // GainEnergyAction(2), for a net gain of one, and always Exhausts.
        let mut e = engine_for(&["Seeing Red"], &[], &[], vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 3);
        let energy = e.state.energy;
        assert!(play_self(&mut e, "Seeing Red"));
        assert_eq!(e.state.energy, energy + 1);
        assert_eq!(exhaust_prefix_count(&e, "Seeing Red"), 1);
    }

    #[test]
    fn seeing_red_upgrade_gains_two_energy_for_free_and_still_exhausts() {
        let mut e = engine_for(
            &["Seeing Red+"],
            &[],
            &[],
            vec![enemy("JawWorm", 50, 50, 1, 0, 1)],
            0,
        );

        assert!(play_self(&mut e, "Seeing Red+"));
        assert_eq!(e.state.energy, 2);
        assert_eq!(exhaust_prefix_count(&e, "Seeing Red"), 1);
        assert!(e.state.discard_pile.is_empty());
    }

    #[test]
    fn carnage_exhausts_on_end_turn() {
        let mut e = engine_for(&["Carnage"], &[], &[], vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 3);
        assert!(play_on_enemy(&mut e, "Carnage", 0));
        let hp = e.state.enemies[0].entity.hp;
        assert_eq!(hp, 30);
        assert_eq!(discard_prefix_count(&e, "Carnage"), 1);
    }

    #[test]
    fn bludgeon_deals_exact_damage() {
        let mut e = engine_for(&["Bludgeon"], &[], &[], vec![enemy("JawWorm", 60, 60, 1, 0, 1)], 3);
        let hp = e.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut e, "Bludgeon", 0));
        assert_eq!(e.state.enemies[0].entity.hp, hp - 32);
    }

    #[test]
    fn limit_break_doubles_strength() {
        let mut e = engine_for(&["Limit Break"], &[], &[], vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 3);
        e.state.player.set_status(sid::STRENGTH, 3);
        assert!(play_self(&mut e, "Limit Break"));
        assert_eq!(e.state.player.strength(), 6);
    }

    #[test]
    fn limit_break_source_doubles_signed_strength_caps_and_upgrade_does_not_exhaust() {
        // LimitBreakAction reads the existing StrengthPower amount and applies
        // another StrengthPower with that signed amount. StrengthPower.stackPower
        // clamps to ±999. Limit Break's upgrade changes only exhaust=false.
        // Java: actions/unique/LimitBreakAction.java and powers/StrengthPower.java.
        let mut negative = engine_for(
            &["Limit Break+"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 50, 50)],
            1,
        );
        negative.state.player.set_status(sid::STRENGTH, -2);
        assert!(play_self(&mut negative, "Limit Break+"));
        assert_eq!(negative.state.player.status(sid::STRENGTH), -4);
        assert_eq!(discard_prefix_count(&negative, "Limit Break+"), 1);
        assert_eq!(exhaust_prefix_count(&negative, "Limit Break+"), 0);

        let mut capped = engine_for(
            &["Limit Break"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 50, 50)],
            1,
        );
        capped.state.player.set_status(sid::STRENGTH, 600);
        assert!(play_self(&mut capped, "Limit Break"));
        assert_eq!(capped.state.player.status(sid::STRENGTH), 999);
        assert_eq!(exhaust_prefix_count(&capped, "Limit Break"), 1);
    }

    #[test]
    fn feed_increases_max_hp_on_kill() {
        let mut e = engine_for(&["Feed"], &[], &[], vec![enemy("JawWorm", 10, 10, 1, 0, 1)], 3);
        let max_hp = e.state.player.max_hp;
        assert!(play_on_enemy(&mut e, "Feed", 0));
        assert_eq!(e.state.enemies[0].entity.hp, 0);
        assert_eq!(e.state.player.max_hp, max_hp + 3);
        assert_eq!(e.state.player.hp, 83);
    }

    #[test]
    fn reaper_heals_for_unblocked_damage() {
        let mut e = engine_for(
            &["Reaper"],
            &[],
            &[],
            vec![
                enemy("JawWorm", 20, 20, 1, 0, 1),
                enemy("Cultist", 20, 20, 1, 0, 1),
            ],
            3,
        );
        e.state.player.hp = 50;
        assert!(play_card(&mut e, "Reaper", 0));
        assert_eq!(e.state.player.hp, 58);
    }

    #[test]
    fn impervious_grants_block_and_exhausts() {
        let mut e = engine_for(&["Impervious"], &[], &[], vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 3);
        assert!(play_self(&mut e, "Impervious"));
        assert_eq!(e.state.player.block, 30);
        assert_eq!(exhaust_prefix_count(&e, "Impervious"), 1);
    }

    #[test]
    fn impervious_upgrade_changes_only_block_and_still_exhausts() {
        // Impervious.java upgrades block by exactly 10; its 2 cost and Exhaust
        // flag are unchanged.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Impervious.java
        let mut engine = engine_for(
            &["Impervious+"],
            &[],
            &[],
            vec![enemy_no_intent("JawWorm", 50, 50)],
            3,
        );

        assert!(play_self(&mut engine, "Impervious+"));

        assert_eq!(engine.state.energy, 1);
        assert_eq!(engine.state.player.block, 40);
        assert_eq!(exhaust_prefix_count(&engine, "Impervious"), 1);
        assert_eq!(discard_prefix_count(&engine, "Impervious"), 0);
    }
}
