// Java references:
// /tmp/sts-decompiled/com/megacrit/cardcrawl/cards/purple/{Alpha.java,BattleHymn.java,Blasphemy.java,BowlingBash.java,Brilliance.java,CarveReality.java,Collect.java,Conclude.java,ConjureBlade.java,Consecrate.java,Crescendo.java,CrushJoints.java,CutThroughFate.java,DeceiveReality.java,Defend_Watcher.java,DeusExMachina.java,DevaForm.java,Devotion.java,Discipline.java,EmptyBody.java,EmptyFist.java,EmptyMind.java,Eruption.java,Establishment.java,Evaluate.java,Fasting.java,FearNoEvil.java,FlurryOfBlows.java,FlyingSleeves.java,FollowUp.java,ForeignInfluence.java,Foresight.java,Halt.java,Indignation.java,InnerPeace.java,Judgement.java,JustLucky.java,LessonLearned.java,LikeWater.java,MasterReality.java,Meditate.java,MentalFortress.java,Nirvana.java,Omniscience.java,Perseverance.java,Pray.java,PressurePoints.java,Prostrate.java,Protect.java,Ragnarok.java,ReachHeaven.java,Rushdown.java,Sanctity.java,SandsOfTime.java,SashWhip.java,Scrawl.java,SignatureMove.java,SimmeringFury.java,SpiritShield.java,Strike_Purple.java,Study.java,Swivel.java,TalkToTheHand.java,Tantrum.java,ThirdEye.java,Tranquility.java,Unraveling.java,Vault.java,Vigilance.java,Wallop.java,WaveOfTheHand.java,Weave.java,WheelKick.java,WindmillStrike.java,Wish.java,Worship.java,WreathOfFlame.java}

#[cfg(test)]
mod watcher_card_java_parity_tests {
    use crate::cards::{CardRegistry, CardTarget, CardType};
    use crate::status_ids::sid;
    use crate::engine::{CombatEngine, CombatPhase};
    use crate::actions::Action;
    use crate::state::Stance;
    use crate::tests::support::*;

    fn reg() -> &'static CardRegistry {
        crate::cards::global_registry()
    }

    fn assert_card(
        id: &str,
        name: &str,
        cost: i32,
        damage: i32,
        block: i32,
        magic: i32,
        card_type: CardType,
        target: CardTarget,
        exhaust: bool,
        enter_stance: Option<&str>,
        effects: &[&str],
    ) {
        let registry = reg();
        let card = match registry.get(id) {
            Some(card) => card,
            None => panic!("missing Rust registry entry for Java card {id}"),
        };
        assert_eq!(card.id, id, "{id} id");
        assert_eq!(card.name, name, "{id} name");
        assert_eq!(card.cost, cost, "{id} cost");
        assert_eq!(card.base_damage, damage, "{id} damage");
        assert_eq!(card.base_block, block, "{id} block");
        assert_eq!(card.base_magic, magic, "{id} magic");
        assert_eq!(card.card_type, card_type, "{id} type");
        assert_eq!(card.target, target, "{id} target");
        assert_eq!(card.exhaust, exhaust, "{id} exhaust");
        assert_eq!(card.enter_stance, enter_stance, "{id} stance");
        assert_card_markers_eq(card, effects, id);
    }

    fn one_enemy_engine(enemy_id: &str, hp: i32, dmg: i32) -> CombatEngine {
        let mut engine = engine_without_start(
            vec![],
            vec![enemy(enemy_id, hp, hp, 1, dmg, 1)],
            3,
        );
        force_player_turn(&mut engine);
        engine
    }

    fn two_enemy_engine(
        a: (&str, i32, i32),
        b: (&str, i32, i32),
    ) -> CombatEngine {
        let mut engine = engine_without_start(
            vec![],
            vec![
                enemy(a.0, a.1, a.1, 1, a.2, 1),
                enemy(b.0, b.1, b.1, 1, b.2, 1),
            ],
            3,
        );
        force_player_turn(&mut engine);
        engine
    }

    macro_rules! watcher_test {
        (
            $name:ident,
            base = ($base_id:expr, $base_name:expr, $base_cost:expr, $base_damage:expr, $base_block:expr, $base_magic:expr, $base_type:expr, $base_target:expr, $base_exhaust:expr, $base_stance:expr, [$($base_eff:expr),*]),
            plus = ($plus_id:expr, $plus_name:expr, $plus_cost:expr, $plus_damage:expr, $plus_block:expr, $plus_magic:expr, $plus_type:expr, $plus_target:expr, $plus_exhaust:expr, $plus_stance:expr, [$($plus_eff:expr),*]),
            $body:block
        ) => {
            #[test]
            fn $name() {
                assert_card(
                    $base_id,
                    $base_name,
                    $base_cost,
                    $base_damage,
                    $base_block,
                    $base_magic,
                    $base_type,
                    $base_target,
                    $base_exhaust,
                    $base_stance,
                    &[$($base_eff),*],
                );
                assert_card(
                    $plus_id,
                    $plus_name,
                    $plus_cost,
                    $plus_damage,
                    $plus_block,
                    $plus_magic,
                    $plus_type,
                    $plus_target,
                    $plus_exhaust,
                    $plus_stance,
                    &[$($plus_eff),*],
                );
                $body
            }
        };
    }

    // Basic stance and starter cards.
    watcher_test!(
        strike_p_java_parity,
        base = ("Strike_P", "Strike", 1, 6, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, []),
        plus = ("Strike_P+", "Strike+", 1, 9, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, []),
        {}
    );
    watcher_test!(
        defend_p_java_parity,
        base = ("Defend_P", "Defend", 1, -1, 5, -1, CardType::Skill, CardTarget::SelfTarget, false, None, []),
        plus = ("Defend_P+", "Defend+", 1, -1, 8, -1, CardType::Skill, CardTarget::SelfTarget, false, None, []),
        {}
    );
    watcher_test!(
        eruption_java_parity,
        base = ("Eruption", "Eruption", 2, 9, -1, -1, CardType::Attack, CardTarget::Enemy, false, Some("Wrath"), []),
        plus = ("Eruption+", "Eruption+", 1, 9, -1, -1, CardType::Attack, CardTarget::Enemy, false, Some("Wrath"), []),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "Eruption");
            set_stance(&mut engine, Stance::Wrath);
            play_on_enemy(&mut engine, "Eruption", 0);
            assert_eq!(engine.state.enemies[0].entity.hp, 32);
        }
    );
    watcher_test!(
        vigilance_java_parity,
        base = ("Vigilance", "Vigilance", 2, -1, 8, -1, CardType::Skill, CardTarget::SelfTarget, false, Some("Calm"), []),
        plus = ("Vigilance+", "Vigilance+", 2, -1, 12, -1, CardType::Skill, CardTarget::SelfTarget, false, Some("Calm"), []),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "Vigilance");
            play_self(&mut engine, "Vigilance");
            assert_eq!(engine.state.player.block, 8);
            assert_eq!(engine.state.stance, Stance::Calm);
        }
    );

    // Common cards.
    watcher_test!(
        bowling_bash_java_parity,
        base = ("BowlingBash", "Bowling Bash", 1, 7, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["damage_per_enemy"]),
        plus = ("BowlingBash+", "Bowling Bash+", 1, 10, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["damage_per_enemy"]),
        {
            let mut engine = two_enemy_engine(("JawWorm", 50, 0), ("Cultist", 50, 0));
            ensure_in_hand(&mut engine, "BowlingBash");
            play_on_enemy(&mut engine, "BowlingBash", 0);
            assert_eq!(engine.state.enemies[0].entity.hp, 36);
        }
    );
    watcher_test!(
        crush_joints_java_parity,
        base = ("CrushJoints", "Crush Joints", 1, 8, -1, 1, CardType::Attack, CardTarget::Enemy, false, None, ["vuln_if_last_skill"]),
        plus = ("CrushJoints+", "Crush Joints+", 1, 10, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, ["vuln_if_last_skill"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "Defend_P");
            ensure_in_hand(&mut engine, "CrushJoints");
            play_self(&mut engine, "Defend_P");
            play_on_enemy(&mut engine, "CrushJoints", 0);
            assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 1);
        }
    );
    watcher_test!(
        cut_through_fate_java_parity,
        base = ("CutThroughFate", "Cut Through Fate", 1, 7, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, ["scry", "draw"]),
        plus = ("CutThroughFate+", "Cut Through Fate+", 1, 9, -1, 3, CardType::Attack, CardTarget::Enemy, false, None, ["scry", "draw"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.draw_pile = make_deck(&["Strike_P", "Defend_P", "Worship"]);
            ensure_in_hand(&mut engine, "CutThroughFate");
            let hand_before = engine.state.hand.len();
            play_on_enemy(&mut engine, "CutThroughFate", 0);
            assert_eq!(engine.state.hand.len(), hand_before + 1);
        }
    );
    watcher_test!(
        empty_body_java_parity,
        base = ("EmptyBody", "Empty Body", 1, -1, 7, -1, CardType::Skill, CardTarget::SelfTarget, false, Some("Neutral"), ["exit_stance"]),
        plus = ("EmptyBody+", "Empty Body+", 1, -1, 10, -1, CardType::Skill, CardTarget::SelfTarget, false, Some("Neutral"), ["exit_stance"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            set_stance(&mut engine, Stance::Wrath);
            ensure_in_hand(&mut engine, "EmptyBody");
            play_self(&mut engine, "EmptyBody");
            assert_eq!(engine.state.player.block, 7);
            assert_eq!(engine.state.stance, Stance::Neutral);
        }
    );
    watcher_test!(
        flurry_of_blows_java_parity,
        base = ("FlurryOfBlows", "Flurry of Blows", 0, 4, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, []),
        plus = ("FlurryOfBlows+", "Flurry of Blows+", 0, 6, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, []),
        {
            let mut engine = one_enemy_engine("JawWorm", 40, 0);
            ensure_in_hand(&mut engine, "FlurryOfBlows");
            let energy_before = engine.state.energy;
            play_on_enemy(&mut engine, "FlurryOfBlows", 0);
            assert_eq!(engine.state.energy, energy_before);
            assert_eq!(engine.state.enemies[0].entity.hp, 36);
        }
    );
    watcher_test!(
        flying_sleeves_java_parity,
        base = ("FlyingSleeves", "Flying Sleeves", 1, 4, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, ["multi_hit", "retain"]),
        plus = ("FlyingSleeves+", "Flying Sleeves+", 1, 6, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, ["multi_hit", "retain"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 60, 0);
            ensure_in_hand(&mut engine, "FlyingSleeves");
            play_on_enemy(&mut engine, "FlyingSleeves", 0);
            assert_eq!(engine.state.enemies[0].entity.hp, 52);
        }
    );
    watcher_test!(
        follow_up_java_parity,
        base = ("FollowUp", "Follow-Up", 1, 7, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["energy_if_last_attack"]),
        plus = ("FollowUp+", "Follow-Up+", 1, 11, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["energy_if_last_attack"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "Strike_P");
            ensure_in_hand(&mut engine, "FollowUp");
            play_on_enemy(&mut engine, "Strike_P", 0);
            let energy_before = engine.state.energy;
            play_on_enemy(&mut engine, "FollowUp", 0);
            assert_eq!(engine.state.energy, energy_before);
        }
    );
    watcher_test!(
        halt_java_parity,
        base = ("Halt", "Halt", 0, -1, 3, 9, CardType::Skill, CardTarget::SelfTarget, false, None, ["extra_block_in_wrath"]),
        plus = ("Halt+", "Halt+", 0, -1, 4, 14, CardType::Skill, CardTarget::SelfTarget, false, None, ["extra_block_in_wrath"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            set_stance(&mut engine, Stance::Wrath);
            ensure_in_hand(&mut engine, "Halt");
            play_self(&mut engine, "Halt");
            assert_eq!(engine.state.player.block, 12);
        }
    );
    watcher_test!(
        prostrate_java_parity,
        base = ("Prostrate", "Prostrate", 0, -1, 4, 2, CardType::Skill, CardTarget::SelfTarget, false, None, ["mantra"]),
        plus = ("Prostrate+", "Prostrate+", 0, -1, 4, 3, CardType::Skill, CardTarget::SelfTarget, false, None, ["mantra"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "Prostrate");
            play_self(&mut engine, "Prostrate");
            assert_eq!(engine.state.player.block, 4);
            assert_eq!(engine.state.mantra, 2);
        }
    );
    watcher_test!(
        tantrum_java_parity,
        base = ("Tantrum", "Tantrum", 1, 3, -1, 3, CardType::Attack, CardTarget::Enemy, false, Some("Wrath"), ["multi_hit", "shuffle_self_into_draw"]),
        plus = ("Tantrum+", "Tantrum+", 1, 3, -1, 4, CardType::Attack, CardTarget::Enemy, false, Some("Wrath"), ["multi_hit", "shuffle_self_into_draw"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 60, 0);
            ensure_in_hand(&mut engine, "Tantrum");
            play_on_enemy(&mut engine, "Tantrum", 0);
            assert_eq!(engine.state.stance, Stance::Wrath);
            assert!(engine.state.draw_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Tantrum"));
        }
    );
    watcher_test!(
        consecrate_java_parity,
        base = ("Consecrate", "Consecrate", 0, 5, -1, -1, CardType::Attack, CardTarget::AllEnemy, false, None, []),
        plus = ("Consecrate+", "Consecrate+", 0, 8, -1, -1, CardType::Attack, CardTarget::AllEnemy, false, None, []),
        {
            let mut engine = two_enemy_engine(("JawWorm", 20, 0), ("Cultist", 20, 0));
            ensure_in_hand(&mut engine, "Consecrate");
            play_on_enemy(&mut engine, "Consecrate", 0);
            assert_eq!(engine.state.enemies[0].entity.hp, 15);
            assert_eq!(engine.state.enemies[1].entity.hp, 15);
        }
    );
    watcher_test!(
        crescendo_java_parity,
        base = ("Crescendo", "Crescendo", 1, -1, -1, -1, CardType::Skill, CardTarget::SelfTarget, true, Some("Wrath"), ["retain"]),
        plus = ("Crescendo+", "Crescendo+", 0, -1, -1, -1, CardType::Skill, CardTarget::SelfTarget, true, Some("Wrath"), ["retain"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "Crescendo");
            play_self(&mut engine, "Crescendo");
            assert_eq!(engine.state.stance, Stance::Wrath);
            assert!(engine.state.exhaust_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Crescendo"));
        }
    );
    watcher_test!(
        just_lucky_java_parity,
        base = ("JustLucky", "Just Lucky", 0, 3, 2, 1, CardType::Attack, CardTarget::Enemy, false, None, ["scry"]),
        plus = ("JustLucky+", "Just Lucky+", 0, 4, 3, 2, CardType::Attack, CardTarget::Enemy, false, None, ["scry"]),
        {}
    );
    watcher_test!(
        pressure_points_java_parity,
        base = ("PathToVictory", "Pressure Points", 1, -1, -1, 8, CardType::Skill, CardTarget::Enemy, false, None, ["pressure_points"]),
        plus = ("PathToVictory+", "Pressure Points+", 1, -1, -1, 11, CardType::Skill, CardTarget::Enemy, false, None, ["pressure_points"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 40, 0);
            ensure_in_hand(&mut engine, "PathToVictory");
            play_on_enemy(&mut engine, "PathToVictory", 0);
            assert_eq!(engine.state.enemies[0].entity.status(sid::MARK), 8);
            assert_eq!(engine.state.enemies[0].entity.hp, 32);
        }
    );
    watcher_test!(
        protect_java_parity_watcher,
        base = ("Protect", "Protect", 2, -1, 12, -1, CardType::Skill, CardTarget::SelfTarget, false, None, ["retain"]),
        plus = ("Protect+", "Protect+", 2, -1, 16, -1, CardType::Skill, CardTarget::SelfTarget, false, None, ["retain"]),
        {}
    );
    watcher_test!(
        sash_whip_java_parity,
        base = ("SashWhip", "Sash Whip", 1, 8, -1, 1, CardType::Attack, CardTarget::Enemy, false, None, ["weak_if_last_attack"]),
        plus = ("SashWhip+", "Sash Whip+", 1, 10, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, ["weak_if_last_attack"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "Strike_P");
            ensure_in_hand(&mut engine, "SashWhip");
            play_on_enemy(&mut engine, "Strike_P", 0);
            play_on_enemy(&mut engine, "SashWhip", 0);
            assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 1);
        }
    );
    watcher_test!(
        third_eye_java_parity,
        base = ("ThirdEye", "Third Eye", 1, -1, 7, 3, CardType::Skill, CardTarget::SelfTarget, false, None, ["scry"]),
        plus = ("ThirdEye+", "Third Eye+", 1, -1, 9, 5, CardType::Skill, CardTarget::SelfTarget, false, None, ["scry"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.draw_pile = make_deck(&["Strike_P", "Defend_P", "Worship"]);
            ensure_in_hand(&mut engine, "ThirdEye");
            play_self(&mut engine, "ThirdEye");
            assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
            // ThirdEye scries 3, so up to 3 cards revealed; select all for discard.
            let num_options = engine.choice.as_ref().unwrap().options.len();
            for i in 0..num_options {
                engine.execute_action(&Action::Choose(i));
            }
            engine.execute_action(&Action::ConfirmSelection);
            assert_eq!(engine.phase, CombatPhase::PlayerTurn);
            assert_eq!(engine.state.player.block, 7);
            assert_eq!(engine.state.discard_pile.len(), 4);
            assert!(engine.state.discard_pile.iter().any(|card| engine.card_registry.card_name(card.def_id) == "ThirdEye"));
        }
    );

    // Uncommon cards and powers.
    watcher_test!(
        battle_hymn_java_parity,
        base = ("BattleHymn", "Battle Hymn", 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, []),
        plus = ("BattleHymn+", "Battle Hymn+", 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, ["innate"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "BattleHymn");
            play_self(&mut engine, "BattleHymn");
            end_turn(&mut engine);
            assert_eq!(hand_count(&engine, "Smite"), 1);
        }
    );
    watcher_test!(
        carve_reality_java_parity,
        base = ("CarveReality", "Carve Reality", 1, 6, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["add_smite_to_hand"]),
        plus = ("CarveReality+", "Carve Reality+", 1, 10, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["add_smite_to_hand"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "CarveReality");
            play_on_enemy(&mut engine, "CarveReality", 0);
            assert_eq!(hand_count(&engine, "Smite"), 1);
        }
    );
    watcher_test!(
        deceive_reality_java_parity,
        base = ("DeceiveReality", "Deceive Reality", 1, -1, 4, -1, CardType::Skill, CardTarget::SelfTarget, false, None, ["add_safety_to_hand"]),
        plus = ("DeceiveReality+", "Deceive Reality+", 1, -1, 7, -1, CardType::Skill, CardTarget::SelfTarget, false, None, ["add_safety_to_hand"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "DeceiveReality");
            play_self(&mut engine, "DeceiveReality");
            assert_eq!(hand_count(&engine, "Safety"), 1);
        }
    );
    watcher_test!(
        empty_mind_java_parity,
        base = ("EmptyMind", "Empty Mind", 1, -1, -1, 2, CardType::Skill, CardTarget::SelfTarget, false, Some("Neutral"), ["draw", "exit_stance"]),
        plus = ("EmptyMind+", "Empty Mind+", 1, -1, -1, 3, CardType::Skill, CardTarget::SelfTarget, false, Some("Neutral"), ["draw", "exit_stance"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            set_stance(&mut engine, Stance::Calm);
            engine.state.draw_pile = make_deck(&["Strike_P", "Defend_P", "Worship"]);
            ensure_in_hand(&mut engine, "EmptyMind");
            let hand_before = engine.state.hand.len();
            play_self(&mut engine, "EmptyMind");
            assert_eq!(engine.state.stance, Stance::Neutral);
            assert_eq!(engine.state.hand.len(), hand_before + 1);
        }
    );
    watcher_test!(
        fear_no_evil_java_parity,
        base = ("FearNoEvil", "Fear No Evil", 1, 8, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["calm_if_enemy_attacking"]),
        plus = ("FearNoEvil+", "Fear No Evil+", 1, 11, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["calm_if_enemy_attacking"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 12);
            ensure_in_hand(&mut engine, "FearNoEvil");
            play_on_enemy(&mut engine, "FearNoEvil", 0);
            assert_eq!(engine.state.stance, Stance::Calm);
        }
    );
    watcher_test!(
        foreign_influence_java_parity,
        base = ("ForeignInfluence", "Foreign Influence", 0, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, ["foreign_influence"]),
        plus = ("ForeignInfluence+", "Foreign Influence+", 0, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, ["foreign_influence"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "ForeignInfluence");
            play_self(&mut engine, "ForeignInfluence");
            assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
            engine.execute_action(&Action::Choose(0)); // pick first option
            assert_eq!(engine.phase, CombatPhase::PlayerTurn);
            assert_eq!(engine.state.hand.len(), 1); // 1 discovered card added
            assert!(engine.state.exhaust_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "ForeignInfluence"));
        }
    );
    watcher_test!(
        inner_peace_java_parity,
        base = ("InnerPeace", "Inner Peace", 1, -1, -1, 3, CardType::Skill, CardTarget::SelfTarget, false, None, ["if_calm_draw_else_calm"]),
        plus = ("InnerPeace+", "Inner Peace+", 1, -1, -1, 4, CardType::Skill, CardTarget::SelfTarget, false, None, ["if_calm_draw_else_calm"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            set_stance(&mut engine, Stance::Calm);
            engine.state.draw_pile = make_deck(&["Strike_P", "Defend_P", "Worship", "Protect"]);
            ensure_in_hand(&mut engine, "InnerPeace");
            let hand_before = engine.state.hand.len();
            play_self(&mut engine, "InnerPeace");
            assert_eq!(engine.state.hand.len(), hand_before + 2);
        }
    );
    watcher_test!(
        like_water_java_parity,
        base = ("LikeWater", "Like Water", 1, -1, -1, 5, CardType::Power, CardTarget::None, false, None, []),
        plus = ("LikeWater+", "Like Water+", 1, -1, -1, 7, CardType::Power, CardTarget::None, false, None, []),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "LikeWater");
            play_self(&mut engine, "LikeWater");
            assert_eq!(engine.state.player.status(sid::LIKE_WATER), 5);
        }
    );
    watcher_test!(
        meditate_java_parity,
        base = ("Meditate", "Meditate", 1, -1, -1, 1, CardType::Skill, CardTarget::None, false, Some("Calm"), ["meditate", "end_turn"]),
        plus = ("Meditate+", "Meditate+", 1, -1, -1, 2, CardType::Skill, CardTarget::None, false, Some("Calm"), ["meditate", "end_turn"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.discard_pile = make_deck(&["Strike_P", "Defend_P"]);
            ensure_in_hand(&mut engine, "Meditate");
            play_self(&mut engine, "Meditate");
            // Meditate now presents a choice to pick from discard
            assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
            engine.execute_action(&Action::Choose(0)); // select first card
            engine.execute_action(&Action::ConfirmSelection);
            assert!(engine.state.hand.iter().any(|c| { let n = engine.card_registry.card_name(c.def_id); n == "Defend_P" || n == "Strike_P" }));
        }
    );
    watcher_test!(
        nirvana_java_parity,
        base = ("Nirvana", "Nirvana", 1, -1, -1, 3, CardType::Power, CardTarget::SelfTarget, false, None, ["on_scry_block"]),
        plus = ("Nirvana+", "Nirvana+", 1, -1, -1, 4, CardType::Power, CardTarget::SelfTarget, false, None, ["on_scry_block"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.draw_pile = make_deck(&["Strike_P", "Defend_P", "Worship"]);
            ensure_in_hand(&mut engine, "Nirvana");
            play_self(&mut engine, "Nirvana");
            let block_before = engine.state.player.block;
            ensure_in_hand(&mut engine, "ThirdEye");
            play_self(&mut engine, "ThirdEye");
            assert_eq!(engine.state.player.block, block_before + 7); // ThirdEye base_block=7 preamble
        }
    );
    watcher_test!(
        perseverance_java_parity,
        base = ("Perseverance", "Perseverance", 1, -1, 5, 2, CardType::Skill, CardTarget::SelfTarget, false, None, ["retain", "grow_block_on_retain"]),
        plus = ("Perseverance+", "Perseverance+", 1, -1, 7, 3, CardType::Skill, CardTarget::SelfTarget, false, None, ["retain", "grow_block_on_retain"]),
        {}
    );
    watcher_test!(
        pray_java_parity,
        base = ("Pray", "Pray", 1, -1, -1, 3, CardType::Skill, CardTarget::SelfTarget, false, None, ["mantra"]),
        plus = ("Pray+", "Pray+", 1, -1, -1, 4, CardType::Skill, CardTarget::SelfTarget, false, None, ["mantra"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "Pray");
            play_self(&mut engine, "Pray");
            assert_eq!(engine.state.mantra, 3);
        }
    );
    watcher_test!(
        protect_java_parity_coverage,
        base = ("Protect", "Protect", 2, -1, 12, -1, CardType::Skill, CardTarget::SelfTarget, false, None, ["retain"]),
        plus = ("Protect+", "Protect+", 2, -1, 16, -1, CardType::Skill, CardTarget::SelfTarget, false, None, ["retain"]),
        {}
    );
    watcher_test!(
        ragnarok_java_parity,
        base = ("Ragnarok", "Ragnarok", 3, 5, -1, 5, CardType::Attack, CardTarget::AllEnemy, false, None, []),
        plus = ("Ragnarok+", "Ragnarok+", 3, 6, -1, 6, CardType::Attack, CardTarget::AllEnemy, false, None, []),
        {}
    );
    watcher_test!(
        reach_heaven_java_parity,
        base = ("ReachHeaven", "Reach Heaven", 2, 10, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["add_through_violence_to_draw"]),
        plus = ("ReachHeaven+", "Reach Heaven+", 2, 15, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["add_through_violence_to_draw"]),
        {}
    );
    watcher_test!(
        rushdown_java_parity,
        base = ("Adaptation", "Rushdown", 1, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false, None, []),
        plus = ("Adaptation+", "Rushdown+", 0, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false, None, []),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.draw_pile = make_deck(&["Strike_P", "Defend_P", "Worship"]);
            ensure_in_hand(&mut engine, "Adaptation");
            ensure_in_hand(&mut engine, "Eruption");
            play_self(&mut engine, "Adaptation");
            let hand_before = engine.state.hand.len();
            play_on_enemy(&mut engine, "Eruption", 0);
            assert_eq!(engine.state.hand.len(), hand_before + 1); // Eruption played (-1) + Rushdown draws 2 on Wrath entry
        }
    );
    watcher_test!(
        scrawl_java_parity,
        base = ("Scrawl", "Scrawl", 1, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, ["draw_to_ten"]),
        plus = ("Scrawl+", "Scrawl+", 0, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, ["draw_to_ten"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.hand = make_deck(&["Scrawl", "Strike_P", "Defend_P"]);
            engine.state.draw_pile = make_deck(&["Strike_P", "Strike_P", "Strike_P", "Strike_P", "Strike_P", "Strike_P", "Strike_P", "Strike_P", "Strike_P"]);
            ensure_in_hand(&mut engine, "Scrawl");
            play_self(&mut engine, "Scrawl");
            assert_eq!(engine.state.hand.len(), 10);
        }
    );
    watcher_test!(
        spirit_shield_java_parity,
        base = ("SpiritShield", "Spirit Shield", 2, -1, -1, 3, CardType::Skill, CardTarget::SelfTarget, false, None, ["block_per_card_in_hand"]),
        plus = ("SpiritShield+", "Spirit Shield+", 2, -1, -1, 4, CardType::Skill, CardTarget::SelfTarget, false, None, ["block_per_card_in_hand"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.hand = make_deck(&["SpiritShield", "Strike_P", "Defend_P", "Worship", "Protect", "Prostrate"]);
            play_self(&mut engine, "SpiritShield");
            assert_eq!(engine.state.player.block, 15);
        }
    );
    watcher_test!(
        study_java_parity,
        base = ("Study", "Study", 2, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, []),
        plus = ("Study+", "Study+", 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, []),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.draw_pile = make_deck_n("Strike_P", 5);
            ensure_in_hand(&mut engine, "Study");
            play_self(&mut engine, "Study");
            end_turn(&mut engine);
            let total_insights = hand_prefix_count(&engine, "Insight")
                + draw_prefix_count(&engine, "Insight")
                + discard_prefix_count(&engine, "Insight");
            assert_eq!(total_insights, 1); // Study adds exactly 1 Insight at end of turn
        }
    );
    watcher_test!(
        swivel_java_parity,
        base = ("Swivel", "Swivel", 2, -1, 8, -1, CardType::Skill, CardTarget::SelfTarget, false, None, ["next_attack_free"]),
        plus = ("Swivel+", "Swivel+", 2, -1, 11, -1, CardType::Skill, CardTarget::SelfTarget, false, None, ["next_attack_free"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "Swivel");
            ensure_in_hand(&mut engine, "Strike_P");
            play_self(&mut engine, "Swivel");
            let energy_before = engine.state.energy;
            play_on_enemy(&mut engine, "Strike_P", 0);
            assert_eq!(engine.state.energy, energy_before);
        }
    );
    watcher_test!(
        talk_to_the_hand_java_parity,
        base = ("TalkToTheHand", "Talk to the Hand", 1, 5, -1, 2, CardType::Attack, CardTarget::Enemy, true, None, ["apply_block_return"]),
        plus = ("TalkToTheHand+", "Talk to the Hand+", 1, 7, -1, 3, CardType::Attack, CardTarget::Enemy, true, None, ["apply_block_return"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "TalkToTheHand");
            ensure_in_hand(&mut engine, "Strike_P");
            play_on_enemy(&mut engine, "TalkToTheHand", 0);
            play_on_enemy(&mut engine, "Strike_P", 0);
            assert_eq!(engine.state.player.block, 2); // BlockReturn=2 triggered by Strike hit
        }
    );
    watcher_test!(
        vault_java_parity,
        base = ("Vault", "Vault", 3, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, ["skip_enemy_turn", "end_turn"]),
        plus = ("Vault+", "Vault+", 2, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, ["skip_enemy_turn", "end_turn"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 9);
            ensure_in_hand(&mut engine, "Vault");
            let turn_before = engine.state.turn;
            play_self(&mut engine, "Vault");
            assert_eq!(engine.state.turn, turn_before + 1);
        }
    );
    watcher_test!(
        wallop_java_parity,
        base = ("Wallop", "Wallop", 2, 9, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["block_from_damage"]),
        plus = ("Wallop+", "Wallop+", 2, 12, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["block_from_damage"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 40, 0);
            engine.state.enemies[0].entity.block = 5;
            ensure_in_hand(&mut engine, "Wallop");
            play_on_enemy(&mut engine, "Wallop", 0);
            assert_eq!(engine.state.player.block, 4);
        }
    );
    watcher_test!(
        weave_java_parity,
        base = ("Weave", "Weave", 0, 4, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["return_on_scry"]),
        plus = ("Weave+", "Weave+", 0, 6, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["return_on_scry"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            // Need cards in draw pile so Scry has something to reveal
            engine.state.draw_pile = make_deck(&["Strike_P", "Defend_P", "Worship"]);
            ensure_in_hand(&mut engine, "Weave");
            ensure_in_hand(&mut engine, "ThirdEye");
            play_on_enemy(&mut engine, "Weave", 0);
            play_self(&mut engine, "ThirdEye");
            // ThirdEye triggers Scry which now needs choice resolution
            assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
            engine.execute_action(&Action::ConfirmSelection); // keep all, don't discard
            // Weave should return from discard to hand on Scry
            assert!(engine.state.hand.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Weave"));
        }
    );
    watcher_test!(
        wreath_of_flame_java_parity,
        base = ("WreathOfFlame", "Wreath of Flame", 1, -1, -1, 5, CardType::Skill, CardTarget::SelfTarget, false, None, ["vigor"]),
        plus = ("WreathOfFlame+", "Wreath of Flame+", 1, -1, -1, 8, CardType::Skill, CardTarget::SelfTarget, false, None, ["vigor"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "WreathOfFlame");
            play_self(&mut engine, "WreathOfFlame");
            assert_eq!(engine.state.player.status(sid::VIGOR), 5);
        }
    );

    // Rare cards and watcher-specific mechanics.
    watcher_test!(
        brilliance_java_parity,
        base = ("Brilliance", "Brilliance", 1, 12, -1, 0, CardType::Attack, CardTarget::Enemy, false, None, ["damage_plus_mantra"]),
        plus = ("Brilliance+", "Brilliance+", 1, 16, -1, 0, CardType::Attack, CardTarget::Enemy, false, None, ["damage_plus_mantra"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 60, 0);
            ensure_in_hand(&mut engine, "Pray");
            ensure_in_hand(&mut engine, "Brilliance");
            play_self(&mut engine, "Pray");
            play_on_enemy(&mut engine, "Brilliance", 0);
            assert_eq!(engine.state.enemies[0].entity.hp, 45);
        }
    );
    watcher_test!(
        conclude_java_parity,
        base = ("Conclude", "Conclude", 1, 12, -1, -1, CardType::Attack, CardTarget::AllEnemy, false, None, ["end_turn"]),
        plus = ("Conclude+", "Conclude+", 1, 16, -1, -1, CardType::Attack, CardTarget::AllEnemy, false, None, ["end_turn"]),
        {
            let mut engine = two_enemy_engine(("JawWorm", 50, 0), ("Cultist", 50, 0));
            ensure_in_hand(&mut engine, "Conclude");
            let turn_before = engine.state.turn;
            play_on_enemy(&mut engine, "Conclude", 0);
            assert_eq!(engine.state.turn, turn_before + 1);
            assert_eq!(engine.state.enemies[0].entity.hp, 38);
            assert_eq!(engine.state.enemies[1].entity.hp, 38);
        }
    );
    watcher_test!(
        devotion_java_parity,
        base = ("Devotion", "Devotion", 1, -1, -1, 2, CardType::Power, CardTarget::None, false, None, []),
        plus = ("Devotion+", "Devotion+", 1, -1, -1, 3, CardType::Power, CardTarget::None, false, None, []),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "Devotion");
            play_self(&mut engine, "Devotion");
            end_turn(&mut engine);
            assert_eq!(engine.state.mantra, 2);
        }
    );
    watcher_test!(
        deva_form_java_parity,
        base = ("DevaForm", "Deva Form", 3, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, ["ethereal"]),
        plus = ("DevaForm+", "Deva Form+", 3, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, []),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "DevaForm");
            play_self(&mut engine, "DevaForm");
            end_turn(&mut engine);
            assert_eq!(engine.state.energy, 4);
        }
    );
    watcher_test!(
        evaluate_java_parity,
        base = ("Evaluate", "Evaluate", 1, -1, 6, -1, CardType::Skill, CardTarget::SelfTarget, false, None, ["insight_to_draw"]),
        plus = ("Evaluate+", "Evaluate+", 1, -1, 10, -1, CardType::Skill, CardTarget::SelfTarget, false, None, ["insight_to_draw"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "Evaluate");
            play_self(&mut engine, "Evaluate");
            assert_eq!(draw_prefix_count(&engine, "Insight"), 1);
        }
    );
    watcher_test!(
        fasting_java_parity,
        base = ("Fasting2", "Fasting", 2, -1, -1, 3, CardType::Power, CardTarget::SelfTarget, false, None, []),
        plus = ("Fasting2+", "Fasting+", 2, -1, -1, 4, CardType::Power, CardTarget::SelfTarget, false, None, []),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "Fasting2");
            play_self(&mut engine, "Fasting2");
            assert_eq!(engine.state.player.strength(), 3);
            assert_eq!(engine.state.player.dexterity(), 3);
            assert_eq!(engine.state.max_energy, 2);
        }
    );
    watcher_test!(
        judgement_java_parity,
        base = ("Judgement", "Judgement", 1, -1, -1, 30, CardType::Skill, CardTarget::Enemy, false, None, ["judgement"]),
        plus = ("Judgement+", "Judgement+", 1, -1, -1, 40, CardType::Skill, CardTarget::Enemy, false, None, ["judgement"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 30, 0);
            ensure_in_hand(&mut engine, "Judgement");
            play_on_enemy(&mut engine, "Judgement", 0);
            assert!(engine.state.enemies[0].entity.is_dead());
        }
    );
    watcher_test!(
        lesson_learned_java_parity,
        base = ("LessonLearned", "Lesson Learned", 2, 10, -1, -1, CardType::Attack, CardTarget::Enemy, true, None, ["lesson_learned"]),
        plus = ("LessonLearned+", "Lesson Learned+", 2, 13, -1, -1, CardType::Attack, CardTarget::Enemy, true, None, ["lesson_learned"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 10, 0);
            engine.state.draw_pile = make_deck(&["Evaluate"]);
            ensure_in_hand(&mut engine, "LessonLearned");
            play_on_enemy(&mut engine, "LessonLearned", 0);
            assert!(engine.state.draw_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Evaluate+"));
        }
    );
    watcher_test!(
        master_reality_java_parity,
        base = ("MasterReality", "Master Reality", 1, -1, -1, -1, CardType::Power, CardTarget::SelfTarget, false, None, []),
        plus = ("MasterReality+", "Master Reality+", 0, -1, -1, -1, CardType::Power, CardTarget::SelfTarget, false, None, []),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "MasterReality");
            ensure_in_hand(&mut engine, "Evaluate");
            play_self(&mut engine, "MasterReality");
            play_self(&mut engine, "Evaluate");
            assert!(engine.state.draw_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Insight+"));
        }
    );
    watcher_test!(
        mediate_java_parity,
        base = ("Meditate", "Meditate", 1, -1, -1, 1, CardType::Skill, CardTarget::None, false, Some("Calm"), ["meditate", "end_turn"]),
        plus = ("Meditate+", "Meditate+", 1, -1, -1, 2, CardType::Skill, CardTarget::None, false, Some("Calm"), ["meditate", "end_turn"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.discard_pile = make_deck(&["Strike_P", "Defend_P"]);
            ensure_in_hand(&mut engine, "Meditate");
            play_self(&mut engine, "Meditate");
            // Meditate now presents a choice to pick from discard
            assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
            engine.execute_action(&Action::Choose(0));
            engine.execute_action(&Action::ConfirmSelection);
            assert!(engine.state.hand.iter().any(|c| { let n = engine.card_registry.card_name(c.def_id); n == "Strike_P" || n == "Defend_P" }));
        }
    );
    watcher_test!(
        mental_fortress_java_parity,
        base = ("MentalFortress", "Mental Fortress", 1, -1, -1, 4, CardType::Power, CardTarget::SelfTarget, false, None, []),
        plus = ("MentalFortress+", "Mental Fortress+", 1, -1, -1, 6, CardType::Power, CardTarget::SelfTarget, false, None, []),
        {
            let mut engine = one_enemy_engine("JawWorm", 100, 0);
            ensure_in_hand(&mut engine, "MentalFortress");
            ensure_in_hand(&mut engine, "Eruption");
            play_self(&mut engine, "MentalFortress");
            play_on_enemy(&mut engine, "Eruption", 0);
            assert_eq!(engine.state.player.block, 4);
        }
    );
    watcher_test!(
        press_points_java_parity,
        base = ("PathToVictory", "Pressure Points", 1, -1, -1, 8, CardType::Skill, CardTarget::Enemy, false, None, ["pressure_points"]),
        plus = ("PathToVictory+", "Pressure Points+", 1, -1, -1, 11, CardType::Skill, CardTarget::Enemy, false, None, ["pressure_points"]),
        {}
    );
    watcher_test!(
        rushdown_alias_java_parity,
        base = ("Adaptation", "Rushdown", 1, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false, None, []),
        plus = ("Adaptation+", "Rushdown+", 0, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false, None, []),
        {}
    );
    watcher_test!(
        sanctity_java_parity,
        base = ("Sanctity", "Sanctity", 1, -1, 6, 2, CardType::Skill, CardTarget::SelfTarget, false, None, []),
        plus = ("Sanctity+", "Sanctity+", 1, -1, 9, 2, CardType::Skill, CardTarget::SelfTarget, false, None, []),
        {}
    );
    watcher_test!(
        signature_move_java_parity,
        base = ("SignatureMove", "Signature Move", 2, 30, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["only_attack_in_hand"]),
        plus = ("SignatureMove+", "Signature Move+", 2, 40, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["only_attack_in_hand"]),
        {}
    );
    watcher_test!(
        spirit_shield_more_java_parity,
        base = ("SpiritShield", "Spirit Shield", 2, -1, -1, 3, CardType::Skill, CardTarget::SelfTarget, false, None, ["block_per_card_in_hand"]),
        plus = ("SpiritShield+", "Spirit Shield+", 2, -1, -1, 4, CardType::Skill, CardTarget::SelfTarget, false, None, ["block_per_card_in_hand"]),
        {}
    );
    watcher_test!(
        trancendental_java_parity,
        base = ("ClearTheMind", "Tranquility", 1, -1, -1, -1, CardType::Skill, CardTarget::SelfTarget, true, Some("Calm"), ["retain"]),
        plus = ("ClearTheMind+", "Tranquility+", 0, -1, -1, -1, CardType::Skill, CardTarget::SelfTarget, true, Some("Calm"), ["retain"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "ClearTheMind");
            play_self(&mut engine, "ClearTheMind");
            assert_eq!(engine.state.stance, Stance::Calm);
            assert!(engine.state.exhaust_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "ClearTheMind"));
        }
    );
    watcher_test!(
        wish_java_parity,
        base = ("Wish", "Wish", 3, 3, 6, 25, CardType::Skill, CardTarget::None, true, None, ["wish"]),
        plus = ("Wish+", "Wish+", 3, 4, 8, 30, CardType::Skill, CardTarget::None, true, None, ["wish"]),
        {}
    );
    watcher_test!(
        worship_java_parity,
        base = ("Worship", "Worship", 2, -1, -1, 5, CardType::Skill, CardTarget::SelfTarget, false, None, ["mantra"]),
        plus = ("Worship+", "Worship+", 2, -1, -1, 5, CardType::Skill, CardTarget::SelfTarget, false, None, ["mantra", "retain"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "Worship");
            play_self(&mut engine, "Worship");
            assert_eq!(engine.state.mantra, 5);
        }
    );

    // Missing Java cards or unsupported parity gaps.
    watcher_test!(
        collect_java_parity,
        base = ("Collect", "Collect", -1, -1, -1, -1, CardType::Skill, CardTarget::SelfTarget, true, None, []),
        plus = ("Collect+", "Collect+", -1, -1, -1, -1, CardType::Skill, CardTarget::SelfTarget, true, None, []),
        {}
    );
    watcher_test!(
        deus_ex_machina_java_parity,
        base = ("DeusExMachina", "Deus Ex Machina", -2, -1, -1, 2, CardType::Skill, CardTarget::SelfTarget, true, None, ["unplayable", "deus_ex_machina"]),
        plus = ("DeusExMachina+", "Deus Ex Machina+", -2, -1, -1, 3, CardType::Skill, CardTarget::SelfTarget, true, None, ["unplayable", "deus_ex_machina"]),
        {}
    );
    watcher_test!(
        foresight_java_parity,
        base = ("Wireheading", "Foresight", 1, -1, -1, 3, CardType::Power, CardTarget::None, false, None, []),
        plus = ("Wireheading+", "Foresight+", 1, -1, -1, 4, CardType::Power, CardTarget::None, false, None, []),
        {}
    );
    watcher_test!(
        simmering_fury_java_parity,
        base = ("Vengeance", "Simmering Fury", 1, -1, -1, 2, CardType::Skill, CardTarget::None, false, None, []),
        plus = ("Vengeance+", "Simmering Fury+", 1, -1, -1, 3, CardType::Skill, CardTarget::None, false, None, []),
        {}
    );
}
