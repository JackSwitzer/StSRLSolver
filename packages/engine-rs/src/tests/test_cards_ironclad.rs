#[cfg(test)]
mod ironclad_card_java_parity_tests {
    // Java references:
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/cards/red/*.java

    use crate::actions::Action;
    use crate::status_ids::sid;
    use crate::tests::support::{
        combat_state_with, ensure_in_hand, engine_with, engine_with_enemies, force_player_turn,
        make_deck, make_deck_n, play_card, play_on_enemy, play_self, TEST_SEED, enemy,
        discard_prefix_count, exhaust_prefix_count,
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
    card_pair_test!(armaments, "Armaments", "Armaments+", 1, -1, 5, -1, 1, -1, 5, -1, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(body_slam, "Body Slam", "Body Slam+", 1, 0, -1, -1, 0, 0, -1, -1, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(clash, "Clash", "Clash+", 0, 14, -1, -1, 0, 18, -1, -1, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(cleave, "Cleave", "Cleave+", 1, 8, -1, -1, 1, 11, -1, -1, CardType::Attack, CardTarget::AllEnemy, false);
    card_pair_test!(clothesline, "Clothesline", "Clothesline+", 2, 12, -1, 2, 2, 14, -1, 3, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(flex, "Flex", "Flex+", 0, -1, -1, 2, 0, -1, -1, 4, CardType::Skill, CardTarget::SelfTarget, false);
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
    card_pair_test!(carnage, "Carnage", "Carnage+", 2, 20, -1, -1, 2, 28, -1, -1, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(combust, "Combust", "Combust+", 1, -1, -1, 5, 1, -1, -1, 7, CardType::Power, CardTarget::SelfTarget, false);
    card_pair_test!(dark_embrace, "Dark Embrace", "Dark Embrace+", 2, -1, -1, 1, 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false);
    card_pair_test!(disarm, "Disarm", "Disarm+", 1, -1, -1, 2, 1, -1, -1, 3, CardType::Skill, CardTarget::Enemy, true);
    card_pair_test!(dropkick, "Dropkick", "Dropkick+", 1, 5, -1, -1, 1, 8, -1, -1, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(dual_wield, "Dual Wield", "Dual Wield+", 1, -1, -1, 1, 1, -1, -1, 2, CardType::Skill, CardTarget::None, false);
    card_pair_test!(entrench, "Entrench", "Entrench+", 2, -1, -1, -1, 1, -1, -1, -1, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(evolve, "Evolve", "Evolve+", 1, -1, -1, 1, 1, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false);
    card_pair_test!(feel_no_pain, "Feel No Pain", "Feel No Pain+", 1, -1, -1, 3, 1, -1, -1, 4, CardType::Power, CardTarget::SelfTarget, false);
    card_pair_test!(fire_breathing, "Fire Breathing", "Fire Breathing+", 1, -1, -1, 6, 1, -1, -1, 10, CardType::Power, CardTarget::SelfTarget, false);
    card_pair_test!(flame_barrier, "Flame Barrier", "Flame Barrier+", 2, -1, 12, 4, 2, -1, 16, 6, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(ghostly_armor, "Ghostly Armor", "Ghostly Armor+", 1, -1, 10, -1, 1, -1, 13, -1, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(hemokinesis, "Hemokinesis", "Hemokinesis+", 1, 15, -1, 2, 1, 20, -1, 2, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(infernal_blade, "Infernal Blade", "Infernal Blade+", 1, -1, -1, -1, 0, -1, -1, -1, CardType::Skill, CardTarget::None, true);
    card_pair_test!(inflame, "Inflame", "Inflame+", 1, -1, -1, 2, 1, -1, -1, 3, CardType::Power, CardTarget::SelfTarget, false);
    card_pair_test!(intimidate, "Intimidate", "Intimidate+", 0, -1, -1, 1, 0, -1, -1, 2, CardType::Skill, CardTarget::AllEnemy, true);
    card_pair_test!(metallicize, "Metallicize", "Metallicize+", 1, -1, -1, 3, 1, -1, -1, 4, CardType::Power, CardTarget::SelfTarget, false);
    card_pair_test!(power_through, "Power Through", "Power Through+", 1, -1, 15, -1, 1, -1, 20, -1, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(pummel, "Pummel", "Pummel+", 1, 2, -1, 4, 1, 2, -1, 5, CardType::Attack, CardTarget::Enemy, true);
    card_pair_test!(rage, "Rage", "Rage+", 0, -1, -1, 3, 0, -1, -1, 5, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(rampage, "Rampage", "Rampage+", 1, 8, -1, 5, 1, 8, -1, 8, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(reckless_charge, "Reckless Charge", "Reckless Charge+", 0, 7, -1, -1, 0, 10, -1, -1, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(rupture, "Rupture", "Rupture+", 1, -1, -1, 1, 1, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false);
    card_pair_test!(searing_blow, "Searing Blow", "Searing Blow+", 2, 12, -1, -1, 2, 16, -1, -1, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(second_wind, "Second Wind", "Second Wind+", 1, -1, 5, -1, 1, -1, 7, -1, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(seeing_red, "Seeing Red", "Seeing Red+", 1, -1, -1, 2, 0, -1, -1, 2, CardType::Skill, CardTarget::None, true);
    card_pair_test!(sentinel, "Sentinel", "Sentinel+", 1, -1, 5, 2, 1, -1, 8, 3, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(sever_soul, "Sever Soul", "Sever Soul+", 2, 16, -1, -1, 2, 22, -1, -1, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(shockwave, "Shockwave", "Shockwave+", 2, -1, -1, 3, 2, -1, -1, 5, CardType::Skill, CardTarget::AllEnemy, true);
    card_pair_test!(spot_weakness, "Spot Weakness", "Spot Weakness+", 1, -1, -1, 3, 1, -1, -1, 4, CardType::Skill, CardTarget::Enemy, false);
    card_pair_test!(uppercut, "Uppercut", "Uppercut+", 2, 13, -1, 1, 2, 13, -1, 2, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(whirlwind, "Whirlwind", "Whirlwind+", -1, 5, -1, -1, -1, 8, -1, -1, CardType::Attack, CardTarget::AllEnemy, false);

    card_pair_test!(barricade, "Barricade", "Barricade+", 3, -1, -1, -1, 2, -1, -1, -1, CardType::Power, CardTarget::SelfTarget, false);
    card_pair_test!(berserk, "Berserk", "Berserk+", 0, -1, -1, 2, 0, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false);
    card_pair_test!(bludgeon, "Bludgeon", "Bludgeon+", 3, 32, -1, -1, 3, 42, -1, -1, CardType::Attack, CardTarget::Enemy, false);
    card_pair_test!(brutality, "Brutality", "Brutality+", 0, -1, -1, 1, 0, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false);
    card_pair_test!(corruption, "Corruption", "Corruption+", 3, -1, -1, -1, 2, -1, -1, -1, CardType::Power, CardTarget::SelfTarget, false);
    card_pair_test!(demon_form, "Demon Form", "Demon Form+", 3, -1, -1, 2, 3, -1, -1, 3, CardType::Power, CardTarget::None, false);
    card_pair_test!(double_tap, "Double Tap", "Double Tap+", 1, -1, -1, 1, 1, -1, -1, 2, CardType::Skill, CardTarget::SelfTarget, false);
    card_pair_test!(exhume, "Exhume", "Exhume+", 1, -1, -1, -1, 0, -1, -1, -1, CardType::Skill, CardTarget::None, true);
    card_pair_test!(feed, "Feed", "Feed+", 1, 10, -1, 3, 1, 12, -1, 4, CardType::Attack, CardTarget::Enemy, true);
    card_pair_test!(fiend_fire, "Fiend Fire", "Fiend Fire+", 2, 7, -1, -1, 2, 10, -1, -1, CardType::Attack, CardTarget::Enemy, true);
    card_pair_test!(immolate, "Immolate", "Immolate+", 2, 21, -1, -1, 2, 28, -1, -1, CardType::Attack, CardTarget::AllEnemy, false);
    card_pair_test!(impervious, "Impervious", "Impervious+", 2, -1, 30, -1, 2, -1, 40, -1, CardType::Skill, CardTarget::SelfTarget, true);
    card_pair_test!(juggernaut, "Juggernaut", "Juggernaut+", 2, -1, -1, 5, 2, -1, -1, 7, CardType::Power, CardTarget::SelfTarget, false);
    card_pair_test!(limit_break, "Limit Break", "Limit Break+", 1, -1, -1, -1, 1, -1, -1, -1, CardType::Skill, CardTarget::SelfTarget, true, false);
    card_pair_test!(offering, "Offering", "Offering+", 0, -1, -1, 3, 0, -1, -1, 5, CardType::Skill, CardTarget::SelfTarget, true);
    card_pair_test!(reaper, "Reaper", "Reaper+", 2, 4, -1, -1, 2, 5, -1, -1, CardType::Attack, CardTarget::AllEnemy, false);

    // ------------------------------------------------------------------
    // Deep behavior checks for cards that are already wired through Rust.
    // ------------------------------------------------------------------

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
            &["Clash", "Defend_P"],
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
    fn iron_wave_damage_and_block() {
        let mut e = engine_for(&["Iron Wave"], &[], &[], vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 3);
        let hp = e.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut e, "Iron Wave", 0));
        assert_eq!(e.state.enemies[0].entity.hp, hp - 5);
        assert_eq!(e.state.player.block, 5);
    }

    #[test]
    fn pommel_strike_draws_one() {
        let mut e = engine_for(&["Pommel Strike"], &["Strike_P"], &[], vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 3);
        let hand = e.state.hand.len();
        assert!(play_on_enemy(&mut e, "Pommel Strike", 0));
        assert_eq!(e.state.hand.len(), hand);
    }

    #[test]
    fn shrug_it_off_blocks_and_draws() {
        let mut e = engine_for(&["Shrug It Off"], &["Strike_P"], &[], vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 3);
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
        let mut e = engine_for(&["Warcry"], &["Strike_P"], &[], vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 3);
        let hand = e.state.hand.len();
        assert!(play_self(&mut e, "Warcry"));
        assert_eq!(e.state.hand.len(), hand);
        assert_eq!(exhaust_prefix_count(&e, "Warcry"), 1);
    }

    #[test]
    fn battle_trance_draws_three() {
        let mut e = engine_for(
            &["Battle Trance"],
            &["Strike_P", "Strike_P", "Strike_P"],
            &[],
            vec![enemy("JawWorm", 50, 50, 1, 0, 1)],
            3,
        );
        assert!(play_self(&mut e, "Battle Trance"));
        assert_eq!(e.state.hand.len(), 3);
    }

    #[test]
    fn seeing_red_grants_energy() {
        let mut e = engine_for(&["Seeing Red"], &[], &[], vec![enemy("JawWorm", 50, 50, 1, 0, 1)], 3);
        let energy = e.state.energy;
        assert!(play_self(&mut e, "Seeing Red"));
        assert_eq!(e.state.energy, energy + 1);
        assert_eq!(exhaust_prefix_count(&e, "Seeing Red"), 1);
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
}
