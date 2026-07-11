// Java references:
// /tmp/sts-decompiled/com/megacrit/cardcrawl/cards/purple/{Alpha.java,BattleHymn.java,Blasphemy.java,BowlingBash.java,Brilliance.java,CarveReality.java,Collect.java,Conclude.java,ConjureBlade.java,Consecrate.java,Crescendo.java,CrushJoints.java,CutThroughFate.java,DeceiveReality.java,Defend_Watcher.java,DeusExMachina.java,DevaForm.java,Devotion.java,Discipline.java,EmptyBody.java,EmptyFist.java,EmptyMind.java,Eruption.java,Establishment.java,Evaluate.java,Fasting.java,FearNoEvil.java,FlurryOfBlows.java,FlyingSleeves.java,FollowUp.java,ForeignInfluence.java,Foresight.java,Halt.java,Indignation.java,InnerPeace.java,Judgement.java,JustLucky.java,LessonLearned.java,LikeWater.java,MasterReality.java,Meditate.java,MentalFortress.java,Nirvana.java,Omniscience.java,Perseverance.java,Pray.java,PressurePoints.java,Prostrate.java,Protect.java,Ragnarok.java,ReachHeaven.java,Rushdown.java,Sanctity.java,SandsOfTime.java,SashWhip.java,Scrawl.java,SignatureMove.java,SimmeringFury.java,SpiritShield.java,Strike_Purple.java,Study.java,Swivel.java,TalkToTheHand.java,Tantrum.java,ThirdEye.java,Tranquility.java,Unraveling.java,Vault.java,Vigilance.java,Wallop.java,WaveOfTheHand.java,Weave.java,WheelKick.java,WindmillStrike.java,Wish.java,Worship.java,WreathOfFlame.java}

#[cfg(test)]
mod watcher_card_java_parity_tests {
    use crate::cards::{CardRegistry, CardTarget, CardType};
    use crate::status_ids::sid;
    use crate::engine::{ChoiceOption, CombatEngine, CombatPhase};
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
        base = ("Strike", "Strike", 1, 6, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, []),
        plus = ("Strike+", "Strike+", 1, 9, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, []),
        {}
    );
    // Source-derived (verify card/Strike_P): the non-debug Watcher Strike is a
    // one-cost, six-damage single-target attack and upgradeDamage(3) makes nine.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Strike_Purple.java
    #[test]
    fn strike_p_source_uses_six_damage_and_plus_three_upgrade() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.hand = make_deck(&["Strike", "Strike+"]);
        engine.state.energy = 2;
        assert!(play_on_enemy(&mut engine, "Strike", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 44);
        assert!(play_on_enemy(&mut engine, "Strike+", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 35);
    }
    watcher_test!(
        defend_p_java_parity,
        base = ("Defend", "Defend", 1, -1, 5, -1, CardType::Skill, CardTarget::SelfTarget, false, None, []),
        plus = ("Defend+", "Defend+", 1, -1, 8, -1, CardType::Skill, CardTarget::SelfTarget, false, None, []),
        {}
    );
    // Source-derived (verify card/Defend_P): Defend_Watcher.java has cost 1,
    // block 5, and upgradeBlock(3). Non-debug use queues GainBlockAction(block).
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Defend_Watcher.java
    #[test]
    fn defend_p_source_grants_five_block_and_upgrade_grants_eight() {
        let mut base = one_enemy_engine("JawWorm", 50, 0);
        ensure_in_hand(&mut base, "Defend");
        play_self(&mut base, "Defend");
        assert_eq!(base.state.player.block, 5);

        let mut plus = one_enemy_engine("JawWorm", 50, 0);
        ensure_in_hand(&mut plus, "Defend+");
        play_self(&mut plus, "Defend+");
        assert_eq!(plus.state.player.block, 8);
    }
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
    // Source-derived (verify card/Eruption): Eruption.java use() queues
    // DamageAction BEFORE ChangeStanceAction("Wrath"), so from Neutral the hit
    // is the unmodified 9 (not Wrath-doubled), and the player ends in Wrath.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Eruption.java
    #[test]
    fn eruption_source_order_damage_resolves_before_wrath() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        ensure_in_hand(&mut engine, "Eruption");
        assert_eq!(engine.state.stance, Stance::Neutral);
        play_on_enemy(&mut engine, "Eruption", 0);
        // 50 - 9 = 41; a damage-after-stance bug would give 50 - 18 = 32.
        assert_eq!(engine.state.enemies[0].entity.hp, 41);
        assert_eq!(engine.state.stance, Stance::Wrath);
    }

    // Source-derived (verify card/Eruption): upgrade() only calls
    // upgradeBaseCost(1); baseDamage stays 9 for both versions.
    #[test]
    fn eruption_upgrade_only_reduces_cost() {
        let base = reg().get("Eruption").expect("Eruption registered");
        let plus = reg().get("Eruption+").expect("Eruption+ registered");
        assert_eq!((base.cost, base.base_damage), (2, 9));
        assert_eq!((plus.cost, plus.base_damage), (1, 9));
    }

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
    // Source-derived (verify card/Vigilance): GainBlockAction resolves before
    // ChangeStanceAction; the upgrade adds four block and does not change cost.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Vigilance.java
    #[test]
    fn vigilance_source_gains_modified_twelve_block_then_enters_calm() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        set_stance(&mut engine, Stance::Wrath);
        engine.state.hand = make_deck(&["Vigilance+"]);
        engine.state.player.set_status(sid::DEXTERITY, 2);
        assert!(play_self(&mut engine, "Vigilance+"));
        assert_eq!(engine.state.player.block, 14);
        assert_eq!(engine.state.stance, Stance::Calm);
        assert_eq!(engine.state.energy, 1);
    }

    // Source-derived (verify card/Alpha): Alpha.java exhausts and queues one
    // stat-equivalent Beta into the draw pile. upgrade() changes only isInnate.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Alpha.java
    #[test]
    fn alpha_source_adds_one_beta_exhausts_and_upgrade_is_innate_only() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.draw_pile.clear();
        engine.state.discard_pile.clear();
        ensure_in_hand(&mut engine, "Alpha");
        let energy_before = engine.state.energy;
        play_self(&mut engine, "Alpha");

        assert_eq!(draw_prefix_count(&engine, "Beta"), 1);
        assert_eq!(engine.state.energy, energy_before - 1);
        assert_eq!(
            engine
                .state
                .exhaust_pile
                .iter()
                .filter(|card| engine.card_registry.card_name(card.def_id) == "Alpha")
                .count(),
            1
        );

        let base = reg().get("Alpha").expect("Alpha registered");
        let plus = reg().get("Alpha+").expect("Alpha+ registered");
        assert_eq!((base.cost, base.exhaust), (1, true));
        assert_eq!((plus.cost, plus.exhaust), (1, true));
        assert!(!base.runtime_traits().innate);
        assert!(plus.runtime_traits().innate);
    }

    // Source-derived (verify card/Beta): Beta.java exhausts and queues one
    // stat-equivalent Omega into the draw pile; upgrade() only changes cost 2 -> 1.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/tempCards/Beta.java
    #[test]
    fn beta_source_adds_one_omega_exhausts_and_upgrade_only_reduces_cost() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.draw_pile.clear();
        engine.state.discard_pile.clear();
        ensure_in_hand(&mut engine, "Beta");
        let energy_before = engine.state.energy;
        play_self(&mut engine, "Beta");

        assert_eq!(draw_prefix_count(&engine, "Omega"), 1);
        assert_eq!(engine.state.energy, energy_before - 2);
        assert_eq!(exhaust_prefix_count(&engine, "Beta"), 1);

        let base = reg().get("Beta").expect("Beta registered");
        let plus = reg().get("Beta+").expect("Beta+ registered");
        assert_eq!((base.cost, base.exhaust), (2, true));
        assert_eq!((plus.cost, plus.exhaust), (1, true));
    }

    // Source-derived (verify card/Blasphemy): Blasphemy.java enters Divinity,
    // applies EndTurnDeathPower, and exhausts. upgrade() changes only selfRetain.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Blasphemy.java
    #[test]
    fn blasphemy_source_enters_divinity_sets_delayed_death_and_upgrade_retains() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        ensure_in_hand(&mut engine, "Blasphemy");
        let energy_before = engine.state.energy;
        play_self(&mut engine, "Blasphemy");

        assert_eq!(engine.state.stance, Stance::Divinity);
        assert_eq!(engine.state.energy, energy_before - 1 + 3);
        assert!(engine.state.blasphemy_active);
        assert_eq!(exhaust_prefix_count(&engine, "Blasphemy"), 1);

        let base = reg().get("Blasphemy").expect("Blasphemy registered");
        let plus = reg().get("Blasphemy+").expect("Blasphemy+ registered");
        assert!(!base.runtime_traits().retain);
        assert!(plus.runtime_traits().retain);
        assert_eq!((base.cost, plus.cost), (1, 1));
    }

    // Source-derived (verify card/Blasphemy): EndTurnDeathPower.java uses
    // LoseHPAction(99999), and AbstractPlayer.damage() caps HP_LOSS to 1 under
    // Intangible before Buffer can reduce it to 0.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/EndTurnDeathPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
    #[test]
    fn blasphemy_delayed_hp_loss_respects_intangible_and_buffer() {
        let mut intangible = one_enemy_engine("JawWorm", 50, 0);
        ensure_in_hand(&mut intangible, "Blasphemy");
        play_self(&mut intangible, "Blasphemy");
        intangible.state.player.set_status(sid::INTANGIBLE, 2);
        let hp_before = intangible.state.player.hp;
        end_turn(&mut intangible);
        assert_eq!(intangible.state.player.hp, hp_before - 1);
        assert!(!intangible.state.combat_over);

        let mut buffer = one_enemy_engine("JawWorm", 50, 0);
        ensure_in_hand(&mut buffer, "Blasphemy");
        play_self(&mut buffer, "Blasphemy");
        buffer.state.player.set_status(sid::BUFFER, 1);
        let hp_before = buffer.state.player.hp;
        end_turn(&mut buffer);
        assert_eq!(buffer.state.player.hp, hp_before);
        assert_eq!(buffer.state.player.status(sid::BUFFER), 0);
        assert!(!buffer.state.combat_over);
    }

    // Source-derived (verify card/Blasphemy): the delayed LoseHPAction follows
    // AbstractPlayer.damage(), where Mark of the Bloom prevents Fairy/Lizard revival.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
    #[test]
    fn blasphemy_mark_of_bloom_blocks_fairy_revival() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.potions[0] = "FairyPotion".to_string();
        engine.state.player.set_status(sid::HAS_MARK_OF_BLOOM, 1);
        ensure_in_hand(&mut engine, "Blasphemy");
        play_self(&mut engine, "Blasphemy");
        end_turn(&mut engine);

        assert_eq!(engine.state.player.hp, 0);
        assert!(engine.state.combat_over);
        assert_eq!(engine.state.potions[0], "FairyPotion");
    }

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

    // Source-derived (verify card/BowlingBash): BowlingBash.java queues one
    // hit on the selected target per monster that is not dead or escaped.
    // With two living monsters and one dead monster, base Bowling Bash deals 2 * 7.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/BowlingBash.java
    #[test]
    fn bowling_bash_source_counts_only_living_non_escaped_monsters() {
        let mut dead = enemy("Cultist", 1, 50, 1, 0, 1);
        dead.entity.hp = 0;
        let mut engine = engine_without_start(
            vec![],
            vec![
                enemy("JawWorm", 50, 50, 1, 0, 1),
                enemy("Cultist", 50, 50, 1, 0, 1),
                dead,
            ],
            3,
        );
        force_player_turn(&mut engine);
        ensure_in_hand(&mut engine, "BowlingBash");
        play_on_enemy(&mut engine, "BowlingBash", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 36);

        let base = reg().get("BowlingBash").expect("BowlingBash registered");
        let plus = reg().get("BowlingBash+").expect("BowlingBash+ registered");
        assert_eq!((base.cost, base.base_damage), (1, 7));
        assert_eq!((plus.cost, plus.base_damage), (1, 10));
    }
    // Source-derived (verify card/CrushJoints): CrushJointsAction.java inspects
    // the card at cardsPlayedThisCombat[size - 2], so only a preceding Skill
    // applies Vulnerable. CrushJoints.java upgrades damage 8 -> 10 and magic 1 -> 2.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/CrushJoints.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/CrushJointsAction.java
    watcher_test!(
        crush_joints_java_parity,
        base = ("CrushJoints", "Crush Joints", 1, 8, -1, 1, CardType::Attack, CardTarget::Enemy, false, None, ["vuln_if_last_skill"]),
        plus = ("CrushJoints+", "Crush Joints+", 1, 10, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, ["vuln_if_last_skill"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "Defend");
            ensure_in_hand(&mut engine, "CrushJoints");
            play_self(&mut engine, "Defend");
            play_on_enemy(&mut engine, "CrushJoints", 0);
            assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 1);

            let mut after_attack = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut after_attack, "FlurryOfBlows");
            ensure_in_hand(&mut after_attack, "CrushJoints");
            play_on_enemy(&mut after_attack, "FlurryOfBlows", 0);
            play_on_enemy(&mut after_attack, "CrushJoints", 0);
            assert_eq!(after_attack.state.enemies[0].entity.status(sid::VULNERABLE), 0);
        }
    );
    // Source-derived (verify card/CutThroughFate): CutThroughFate.java queues
    // damage, Scry(magicNumber), then exactly one draw. The draw therefore occurs
    // only after the Scry selection resolves, and the upgrade changes Scry 2 -> 3.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/CutThroughFate.java
    watcher_test!(
        cut_through_fate_java_parity,
        base = ("CutThroughFate", "Cut Through Fate", 1, 7, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, ["scry", "draw"]),
        plus = ("CutThroughFate+", "Cut Through Fate+", 1, 9, -1, 3, CardType::Attack, CardTarget::Enemy, false, None, ["scry", "draw"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.draw_pile = make_deck(&["Strike", "Defend", "Worship"]);
            ensure_in_hand(&mut engine, "CutThroughFate");
            play_on_enemy(&mut engine, "CutThroughFate", 0);
            assert_eq!(engine.state.enemies[0].entity.hp, 43);
            assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
            assert!(engine.state.hand.is_empty(), "draw must wait for Scry");
            assert_eq!(engine.choice.as_ref().unwrap().options.len(), 2);
            engine.execute_action(&Action::Choose(1));
            engine.execute_action(&Action::ConfirmSelection);
            assert_eq!(engine.phase, CombatPhase::PlayerTurn);
            assert_eq!(engine.state.hand.len(), 1, "Cut Through Fate draws exactly one");
            assert_eq!(hand_count(&engine, "Defend"), 1);
            assert_eq!(discard_prefix_count(&engine, "Worship"), 1);
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
    // Source-derived (verify card/EmptyBody): GainBlockAction(block) precedes
    // ChangeStanceAction(Neutral); upgradeBlock(3) changes 7 -> 10.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/EmptyBody.java
    #[test]
    fn empty_body_source_blocks_then_exits_calm() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        set_stance(&mut engine, Stance::Calm);
        ensure_in_hand(&mut engine, "EmptyBody+");
        play_self(&mut engine, "EmptyBody+");
        assert_eq!(engine.state.player.block, 10);
        assert_eq!(engine.state.stance, Stance::Neutral);
        assert_eq!(engine.state.energy, 4);
    }
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
    // Source-derived (verify card/FlurryOfBlows): stance changes queue
    // DiscardToHandAction, which checks hand size before removing the card.
    // A full hand therefore leaves Flurry in discard rather than deleting it.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/FlurryOfBlows.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/DiscardToHandAction.java
    #[test]
    fn flurry_of_blows_source_stays_in_discard_when_hand_is_full() {
        let mut engine = one_enemy_engine("JawWorm", 40, 0);
        engine.state.hand = make_deck(&["Strike"; 10]);
        engine.state.discard_pile = make_deck(&["FlurryOfBlows+"]);
        engine.change_stance(Stance::Wrath);
        assert_eq!(engine.state.hand.len(), 10);
        assert_eq!(discard_prefix_count(&engine, "FlurryOfBlows+"), 1);
    }
    watcher_test!(
        flying_sleeves_java_parity,
        base = ("FlyingSleeves", "Flying Sleeves", 1, 4, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["multi_hit", "retain"]),
        plus = ("FlyingSleeves+", "Flying Sleeves+", 1, 6, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["multi_hit", "retain"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 60, 0);
            ensure_in_hand(&mut engine, "FlyingSleeves");
            play_on_enemy(&mut engine, "FlyingSleeves", 0);
            assert_eq!(engine.state.enemies[0].entity.hp, 52);
        }
    );
    // Source-derived (verify card/FlyingSleeves): use() queues two independent
    // DamageActions and the constructor sets selfRetain. No magicNumber exists.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/FlyingSleeves.java
    #[test]
    fn flying_sleeves_source_has_fixed_two_hits_and_self_retain() {
        let mut attack = one_enemy_engine("JawWorm", 40, 0);
        ensure_in_hand(&mut attack, "FlyingSleeves+");
        play_on_enemy(&mut attack, "FlyingSleeves+", 0);
        assert_eq!(attack.state.enemies[0].entity.hp, 28);

        let mut retained = one_enemy_engine("JawWorm", 40, 0);
        ensure_in_hand(&mut retained, "FlyingSleeves");
        end_turn(&mut retained);
        assert_eq!(hand_count(&retained, "FlyingSleeves"), 1);
        assert_eq!(reg().get("FlyingSleeves").unwrap().base_magic, -1);
    }
    watcher_test!(
        follow_up_java_parity,
        base = ("FollowUp", "Follow-Up", 1, 7, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["energy_if_last_attack"]),
        plus = ("FollowUp+", "Follow-Up+", 1, 11, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["energy_if_last_attack"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "Strike");
            ensure_in_hand(&mut engine, "FollowUp");
            play_on_enemy(&mut engine, "Strike", 0);
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
    // Source-derived (verify card/FollowUp): FollowUpAction checks the card at
    // cardsPlayedThisCombat[size - 2], so only an immediately preceding Attack
    // refunds the one energy Follow-Up spends.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/FollowUp.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/FollowUpAction.java
    #[test]
    fn follow_up_source_checks_the_immediately_previous_played_card() {
        let mut after_attack = one_enemy_engine("JawWorm", 50, 0);
        after_attack.state.hand = make_deck(&["Strike", "FollowUp+"]);
        play_on_enemy(&mut after_attack, "Strike", 0);
        let energy_after_attack = after_attack.state.energy;
        play_on_enemy(&mut after_attack, "FollowUp+", 0);
        assert_eq!(after_attack.state.energy, energy_after_attack);
        assert_eq!(after_attack.state.enemies[0].entity.hp, 33);

        let mut after_skill = one_enemy_engine("JawWorm", 50, 0);
        after_skill.state.hand = make_deck(&["Defend", "FollowUp"]);
        play_self(&mut after_skill, "Defend");
        let energy_after_skill = after_skill.state.energy;
        play_on_enemy(&mut after_skill, "FollowUp", 0);
        assert_eq!(after_skill.state.energy, energy_after_skill - 1);
        assert_eq!(after_skill.state.enemies[0].entity.hp, 43);
    }

    // Source-derived (verify card/Halt): Halt.applyPowers calculates the normal
    // and Wrath components separately. Dexterity therefore applies to both;
    // Frail floors both before HaltAction adds them together.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Halt.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/HaltAction.java
    #[test]
    fn halt_source_modifies_both_wrath_block_components_independently() {
        let mut dexterity = one_enemy_engine("JawWorm", 50, 0);
        set_stance(&mut dexterity, Stance::Wrath);
        dexterity.state.player.set_status(sid::DEXTERITY, 2);
        ensure_in_hand(&mut dexterity, "Halt+");
        play_self(&mut dexterity, "Halt+");
        assert_eq!(dexterity.state.player.block, 22); // (4 + 2) + (14 + 2)

        let mut frail = one_enemy_engine("JawWorm", 50, 0);
        set_stance(&mut frail, Stance::Wrath);
        frail.state.player.set_status(sid::FRAIL, 1);
        ensure_in_hand(&mut frail, "Halt");
        play_self(&mut frail, "Halt");
        assert_eq!(frail.state.player.block, 8); // floor(3*.75) + floor(9*.75)
    }

    // Source-derived (verify card/ForeignInfluence): every candidate attempt
    // consumes cardRandomRng.random(99), cardRandomRng.randomLong() inside the
    // library shuffle, and cardRng.random(bucket_size - 1). Duplicate IDs retry
    // all three calls. CardTags.HEALING attacks are excluded.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/ForeignInfluenceAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/CardLibrary.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
    #[test]
    fn foreign_influence_source_uses_split_rng_ticks_and_unique_non_healing_attacks() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        ensure_in_hand(&mut engine, "ForeignInfluence");
        let card_random_before = engine.card_random_rng.counter;
        let card_before = engine.rng.counter;
        play_self(&mut engine, "ForeignInfluence");

        let choice = engine.choice.as_ref().expect("Foreign Influence choice");
        assert_eq!(choice.options.len(), 3);
        let mut ids = Vec::new();
        for option in &choice.options {
            let ChoiceOption::GeneratedCard(card) = option else {
                panic!("Foreign Influence must offer generated cards");
            };
            let def = engine.card_registry.card_def_by_id(card.def_id);
            assert_eq!(def.card_type, CardType::Attack);
            assert!(!matches!(def.id, "Feed" | "Reaper" | "LessonLearned"));
            assert!(!ids.contains(&def.id));
            ids.push(def.id);
        }

        let card_attempts = engine.rng.counter - card_before;
        let card_random_ticks = engine.card_random_rng.counter - card_random_before;
        assert!(card_attempts >= 3);
        assert_eq!(card_random_ticks, card_attempts * 2);
    }

    // ForeignInfluenceAction sends the selected copy to discard if the hand is
    // full when the choice resolves. The upgrade sets costForTurn(0), leaving
    // the selected card's permanent base cost intact.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/ForeignInfluenceAction.java
    #[test]
    fn foreign_influence_source_spills_selected_zero_this_turn_copy_to_discard() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        ensure_in_hand(&mut engine, "ForeignInfluence+");
        play_self(&mut engine, "ForeignInfluence+");
        let choice = engine.choice.as_ref().expect("Foreign Influence+ choice");
        let selected = choice
            .options
            .iter()
            .position(|option| match option {
                ChoiceOption::GeneratedCard(card) => {
                    engine.card_registry.card_def_by_id(card.def_id).cost > 0
                }
                _ => false,
            })
            .expect("seeded choices should contain a positive-cost attack");
        let selected_card = match choice.options[selected] {
            ChoiceOption::GeneratedCard(card) => card,
            _ => unreachable!(),
        };
        while engine.state.hand.len() < 10 {
            engine.state.hand.push(engine.card_registry.make_card("Defend"));
        }

        engine.execute_action(&Action::Choose(selected));
        assert_eq!(engine.state.hand.len(), 10);
        let generated = engine
            .state
            .discard_pile
            .iter()
            .find(|card| card.def_id == selected_card.def_id)
            .expect("full-hand selection should spill to discard");
        assert_eq!(generated.cost, 0);
        assert_eq!(
            generated.base_cost as i32,
            engine.card_registry.card_def_by_id(generated.def_id).cost
        );
    }
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
    // Source-derived (verify card/Tantrum): three (four upgraded) three-damage
    // actions resolve before Wrath. UseCardAction then moveToDeck(..., true)
    // inserts Tantrum with one cardRandomRng tick without shuffling existing cards.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Tantrum.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
    #[test]
    fn tantrum_source_randomly_inserts_without_shuffling_after_pre_wrath_hits() {
        let mut engine = one_enemy_engine("JawWorm", 100, 0);
        engine.state.draw_pile = make_deck(&["Strike", "Defend", "Worship"]);
        engine.state.hand = make_deck(&["Tantrum+"]);
        let card_random_before = engine.card_random_rng.counter;
        let shuffle_before = engine.rng.counter;
        assert!(play_on_enemy(&mut engine, "Tantrum+", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 88);
        assert_eq!(engine.state.stance, Stance::Wrath);
        assert_eq!(engine.card_random_rng.counter, card_random_before + 1);
        assert_eq!(engine.rng.counter, shuffle_before);
        let existing: Vec<_> = engine
            .state
            .draw_pile
            .iter()
            .filter_map(|card| {
                let name = engine.card_registry.card_name(card.def_id);
                (name != "Tantrum+").then_some(name)
            })
            .collect();
        assert_eq!(existing, vec!["Strike", "Defend", "Worship"]);
    }
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

    // Source-derived (verify card/Consecrate): Consecrate.java is a zero-cost
    // 5-damage AoE; upgrade() adds 3 damage and changes nothing else.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Consecrate.java
    #[test]
    fn consecrate_source_hits_every_enemy_for_zero_energy() {
        let mut engine = two_enemy_engine(("JawWorm", 30, 0), ("Cultist", 30, 0));
        ensure_in_hand(&mut engine, "Consecrate");
        let energy_before = engine.state.energy;
        play_on_enemy(&mut engine, "Consecrate", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 25);
        assert_eq!(engine.state.enemies[1].entity.hp, 25);
        assert_eq!(engine.state.energy, energy_before);

        let base = reg().get("Consecrate").expect("Consecrate registered");
        let plus = reg().get("Consecrate+").expect("Consecrate+ registered");
        assert_eq!((base.cost, base.base_damage), (0, 5));
        assert_eq!((plus.cost, plus.base_damage), (0, 8));
    }
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

    // Source-derived (verify card/Crescendo): Crescendo.java is self-retaining,
    // enters Wrath, and exhausts; upgrade() only changes cost 1 -> 0.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Crescendo.java
    #[test]
    fn crescendo_source_retains_unplayed_and_plus_enters_wrath_for_zero() {
        let mut retained = one_enemy_engine("JawWorm", 50, 0);
        retained.state.hand = make_deck(&["Crescendo"]);
        retained.state.draw_pile.clear();
        retained.state.discard_pile.clear();
        end_turn(&mut retained);
        assert_eq!(hand_count(&retained, "Crescendo"), 1);

        let mut played = one_enemy_engine("JawWorm", 50, 0);
        ensure_in_hand(&mut played, "Crescendo+");
        let energy_before = played.state.energy;
        play_self(&mut played, "Crescendo+");
        assert_eq!(played.state.stance, Stance::Wrath);
        assert_eq!(played.state.energy, energy_before);
        assert_eq!(exhaust_prefix_count(&played, "Crescendo+"), 1);

        let base = reg().get("Crescendo").expect("Crescendo registered");
        let plus = reg().get("Crescendo+").expect("Crescendo+ registered");
        assert_eq!((base.cost, plus.cost), (1, 0));
        assert!(base.runtime_traits().retain);
        assert!(plus.runtime_traits().retain);
    }
    watcher_test!(
        just_lucky_java_parity,
        base = ("JustLucky", "Just Lucky", 0, 3, 2, 1, CardType::Attack, CardTarget::Enemy, false, None, ["scry"]),
        plus = ("JustLucky+", "Just Lucky+", 0, 4, 3, 2, CardType::Attack, CardTarget::Enemy, false, None, ["scry"]),
        {}
    );
    // Source-derived (verify card/JustLucky): Java queues ScryAction before its
    // GainBlockAction and DamageAction. An interactive Scry therefore pauses
    // both later actions until the selection resolves.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/JustLucky.java
    #[test]
    fn just_lucky_source_defers_block_and_damage_until_after_scry() {
        let mut engine = one_enemy_engine("JawWorm", 40, 0);
        engine.state.draw_pile = make_deck(&["Strike"]);
        ensure_in_hand(&mut engine, "JustLucky");
        play_on_enemy(&mut engine, "JustLucky", 0);

        assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
        assert_eq!(engine.state.player.block, 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 40);
        engine.execute_action(&Action::ConfirmSelection);
        assert_eq!(engine.state.player.block, 2);
        assert_eq!(engine.state.enemies[0].entity.hp, 37);
    }
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
            ensure_in_hand(&mut engine, "Strike");
            ensure_in_hand(&mut engine, "SashWhip");
            play_on_enemy(&mut engine, "Strike", 0);
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
            engine.state.draw_pile = make_deck(&["Strike", "Defend", "Worship"]);
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
    // Source-derived (verify card/ThirdEye): GainBlockAction is queued before
    // ScryAction, so upgraded block (9 plus Dexterity) is already present while
    // the five-card Scry selection is awaiting input.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ThirdEye.java
    #[test]
    fn third_eye_source_gains_block_before_opening_scry_choice() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.draw_pile = make_deck(&[
            "Strike", "Defend", "Worship", "Protect", "Prostrate",
        ]);
        engine.state.hand = make_deck(&["ThirdEye+"]);
        engine.state.player.set_status(sid::DEXTERITY, 2);
        assert!(play_self(&mut engine, "ThirdEye+"));
        assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
        assert_eq!(engine.state.player.block, 11);
        assert_eq!(engine.choice.as_ref().expect("Scry choice").options.len(), 5);
    }

    // Source-derived (verify card/Unraveling): the leftover class exists, but
    // retail CardLibrary.addPurpleCards() neither imports nor registers it.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Unraveling.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/CardLibrary.java
    #[test]
    fn unraveling_source_is_not_in_the_retail_card_library() {
        assert!(reg().get("Unraveling").is_none());
        assert!(reg().get("Unraveling+").is_none());
    }

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

    // Source-derived (verify card/BattleHymn): BattleHymn.java applies one
    // BattleHymnPower stack, and upgrade() changes only isInnate. The installed
    // power therefore creates one Smite at the following turn start.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/BattleHymn.java
    #[test]
    fn battle_hymn_source_applies_one_stack_and_upgrade_is_innate_only() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        ensure_in_hand(&mut engine, "BattleHymn");
        play_self(&mut engine, "BattleHymn");
        assert_eq!(engine.state.player.status(sid::BATTLE_HYMN), 1);
        end_turn(&mut engine);
        assert_eq!(hand_count(&engine, "Smite"), 1);

        let base = reg().get("BattleHymn").expect("BattleHymn registered");
        let plus = reg().get("BattleHymn+").expect("BattleHymn+ registered");
        assert_eq!((base.cost, base.base_magic), (1, 1));
        assert_eq!((plus.cost, plus.base_magic), (1, 1));
        assert!(!base.runtime_traits().innate);
        assert!(plus.runtime_traits().innate);
    }
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

    // Source-derived (verify card/CarveReality): CarveReality.java deals 6
    // damage, then creates one Smite in hand; upgrade() changes damage to 10.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/CarveReality.java
    #[test]
    fn carve_reality_source_deals_damage_then_adds_one_smite() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        ensure_in_hand(&mut engine, "CarveReality");
        play_on_enemy(&mut engine, "CarveReality", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 44);
        assert_eq!(hand_count(&engine, "Smite"), 1);

        let base = reg().get("CarveReality").expect("CarveReality registered");
        let plus = reg().get("CarveReality+").expect("CarveReality+ registered");
        assert_eq!((base.cost, base.base_damage), (1, 6));
        assert_eq!((plus.cost, plus.base_damage), (1, 10));
    }
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
    // Source-derived (verify card/DeceiveReality): DeceiveReality.java queues
    // GainBlockAction(block), then creates one stat-equivalent Safety. Its only
    // upgrade is +3 block. MakeTempCardInHandAction honors Master Reality.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/DeceiveReality.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInHandAction.java
    #[test]
    fn deceive_reality_source_gains_block_then_creates_one_safety() {
        let mut base = one_enemy_engine("JawWorm", 50, 0);
        base.state.player.set_status(sid::MASTER_REALITY, 1);
        ensure_in_hand(&mut base, "DeceiveReality");
        play_self(&mut base, "DeceiveReality");
        assert_eq!(base.state.player.block, 4);
        assert_eq!(hand_count(&base, "Safety+"), 1);

        let plus = reg().get("DeceiveReality+").expect("DeceiveReality+ registered");
        assert_eq!((plus.cost, plus.base_block), (1, 7));
    }
    watcher_test!(
        empty_mind_java_parity,
        base = ("EmptyMind", "Empty Mind", 1, -1, -1, 2, CardType::Skill, CardTarget::SelfTarget, false, Some("Neutral"), ["draw", "exit_stance"]),
        plus = ("EmptyMind+", "Empty Mind+", 1, -1, -1, 3, CardType::Skill, CardTarget::SelfTarget, false, Some("Neutral"), ["draw", "exit_stance"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            set_stance(&mut engine, Stance::Calm);
            engine.state.draw_pile = make_deck(&["Strike", "Defend", "Worship"]);
            ensure_in_hand(&mut engine, "EmptyMind");
            let hand_before = engine.state.hand.len();
            play_self(&mut engine, "EmptyMind");
            assert_eq!(engine.state.stance, Stance::Neutral);
            assert_eq!(engine.state.hand.len(), hand_before + 1);
        }
    );
    // Source-derived (verify card/EmptyMind): DrawCardAction(magicNumber)
    // precedes ChangeStanceAction(Neutral); upgradeMagicNumber(1) changes 2 -> 3.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/EmptyMind.java
    #[test]
    fn empty_mind_source_draws_then_exits_calm() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        set_stance(&mut engine, Stance::Calm);
        engine.state.draw_pile = make_deck(&["Strike", "Defend"]);
        ensure_in_hand(&mut engine, "EmptyMind");
        play_self(&mut engine, "EmptyMind");
        assert_eq!(engine.state.hand.len(), 2);
        assert_eq!(engine.state.draw_pile.len(), 0);
        assert_eq!(engine.state.stance, Stance::Neutral);
        assert_eq!(engine.state.energy, 4);
    }
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
    // Source-derived (verify card/FearNoEvil): FearNoEvilAction recognizes only
    // attacking intent variants, deals damage, then enters Calm. Upgrade is +3 damage.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/FearNoEvil.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/FearNoEvilAction.java
    #[test]
    fn fear_no_evil_source_enters_calm_only_for_attacking_intents() {
        let mut attacking = one_enemy_engine("JawWorm", 40, 8);
        ensure_in_hand(&mut attacking, "FearNoEvil+");
        play_on_enemy(&mut attacking, "FearNoEvil+", 0);
        assert_eq!(attacking.state.enemies[0].entity.hp, 29);
        assert_eq!(attacking.state.stance, Stance::Calm);

        let mut passive = one_enemy_engine("JawWorm", 40, 0);
        ensure_in_hand(&mut passive, "FearNoEvil");
        play_on_enemy(&mut passive, "FearNoEvil", 0);
        assert_eq!(passive.state.enemies[0].entity.hp, 32);
        assert_eq!(passive.state.stance, Stance::Neutral);
    }
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
            engine.state.draw_pile = make_deck(&["Strike", "Defend", "Worship", "Protect"]);
            ensure_in_hand(&mut engine, "InnerPeace");
            let hand_before = engine.state.hand.len();
            play_self(&mut engine, "InnerPeace");
            assert_eq!(engine.state.hand.len(), hand_before + 2);
        }
    );
    // Source-derived (verify card/Indignation): IndignationAction branches on
    // current stance. In Wrath it applies Vulnerable to all monsters through
    // ApplyPowerAction (so Artifact can block it); otherwise it only enters
    // Wrath.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Indignation.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/IndignationAction.java
    #[test]
    fn indignation_source_branches_between_wrath_and_artifact_aware_vulnerable() {
        let mut entering = two_enemy_engine(("JawWorm", 50, 0), ("Cultist", 50, 0));
        ensure_in_hand(&mut entering, "Indignation");
        play_self(&mut entering, "Indignation");
        assert_eq!(entering.state.stance, Stance::Wrath);
        assert_eq!(entering.state.enemies[0].entity.status(sid::VULNERABLE), 0);
        assert_eq!(entering.state.enemies[1].entity.status(sid::VULNERABLE), 0);

        let mut debuffing = two_enemy_engine(("JawWorm", 50, 0), ("Cultist", 50, 0));
        set_stance(&mut debuffing, Stance::Wrath);
        debuffing.state.enemies[0].entity.set_status(sid::ARTIFACT, 1);
        ensure_in_hand(&mut debuffing, "Indignation+");
        play_self(&mut debuffing, "Indignation+");
        assert_eq!(debuffing.state.enemies[0].entity.status(sid::ARTIFACT), 0);
        assert_eq!(debuffing.state.enemies[0].entity.status(sid::VULNERABLE), 0);
        assert_eq!(debuffing.state.enemies[1].entity.status(sid::VULNERABLE), 5);
    }

    // Source-derived (verify card/InnerPeace): InnerPeaceAction draws only when
    // already in Calm. Its other branch enters Calm without drawing, while the
    // Calm branch never exits the stance or activates Violet Lotus.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/InnerPeace.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/InnerPeaceAction.java
    #[test]
    fn inner_peace_source_draws_only_when_already_calm() {
        let mut entering = one_enemy_engine("JawWorm", 50, 0);
        entering.state.draw_pile = make_deck(&["Strike", "Defend", "Worship", "Protect"]);
        ensure_in_hand(&mut entering, "InnerPeace+");
        play_self(&mut entering, "InnerPeace+");
        assert_eq!(entering.state.stance, Stance::Calm);
        assert!(entering.state.hand.is_empty());
        assert_eq!(entering.state.draw_pile.len(), 4);

        let mut drawing = one_enemy_engine("JawWorm", 50, 0);
        set_stance(&mut drawing, Stance::Calm);
        drawing.state.relics.push("Violet Lotus".to_string());
        drawing.state.draw_pile = make_deck(&["Strike", "Defend", "Worship", "Protect"]);
        ensure_in_hand(&mut drawing, "InnerPeace+");
        play_self(&mut drawing, "InnerPeace+");
        assert_eq!(drawing.state.stance, Stance::Calm);
        assert_eq!(drawing.state.hand.len(), 4);
        assert_eq!(drawing.state.energy, 2);
    }
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
    // Source-derived (verify card/LikeWater): LikeWaterPower stacks to a 999
    // cap and its end-turn GainBlockAction uses the raw power amount. Card block
    // modifiers such as Dexterity and Frail do not alter it.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/LikeWater.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/LikeWaterPower.java
    #[test]
    fn like_water_source_caps_stacks_and_grants_raw_block_in_calm() {
        let mut dexterity = one_enemy_engine("JawWorm", 60, 12);
        set_stance(&mut dexterity, Stance::Calm);
        dexterity.state.player.set_status(sid::DEXTERITY, 5);
        ensure_in_hand(&mut dexterity, "LikeWater");
        play_self(&mut dexterity, "LikeWater");
        end_turn(&mut dexterity);
        assert_eq!(dexterity.state.player.hp, 73); // 12 incoming - raw 5 block

        let mut frail = one_enemy_engine("JawWorm", 60, 12);
        set_stance(&mut frail, Stance::Calm);
        frail.state.player.set_status(sid::FRAIL, 1);
        frail.state.player.set_status(sid::LIKE_WATER, 995);
        ensure_in_hand(&mut frail, "LikeWater+");
        play_self(&mut frail, "LikeWater+");
        assert_eq!(frail.state.player.status(sid::LIKE_WATER), 999);
    }
    watcher_test!(
        meditate_java_parity,
        base = ("Meditate", "Meditate", 1, -1, -1, 1, CardType::Skill, CardTarget::None, false, Some("Calm"), ["meditate", "end_turn"]),
        plus = ("Meditate+", "Meditate+", 1, -1, -1, 2, CardType::Skill, CardTarget::None, false, Some("Calm"), ["meditate", "end_turn"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.discard_pile = make_deck(&["Strike", "Defend"]);
            ensure_in_hand(&mut engine, "Meditate");
            play_self(&mut engine, "Meditate");
            // Meditate now presents a choice to pick from discard
            assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
            engine.execute_action(&Action::Choose(0)); // select first card
            engine.execute_action(&Action::ConfirmSelection);
            assert!(engine.state.hand.iter().any(|c| { let n = engine.card_registry.card_name(c.def_id); n == "Defend" || n == "Strike" }));
        }
    );
    watcher_test!(
        nirvana_java_parity,
        base = ("Nirvana", "Nirvana", 1, -1, -1, 3, CardType::Power, CardTarget::SelfTarget, false, None, ["on_scry_block"]),
        plus = ("Nirvana+", "Nirvana+", 1, -1, -1, 4, CardType::Power, CardTarget::SelfTarget, false, None, ["on_scry_block"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.draw_pile = make_deck(&["Strike", "Defend", "Worship"]);
            ensure_in_hand(&mut engine, "Nirvana");
            play_self(&mut engine, "Nirvana");
            let block_before = engine.state.player.block;
            ensure_in_hand(&mut engine, "ThirdEye");
            play_self(&mut engine, "ThirdEye");
            // ScryAction calls NirvanaPower.onScry before opening grid select.
            // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/ScryAction.java
            assert_eq!(engine.state.player.block, block_before + 10); // Third Eye 7 + Nirvana 3
        }
    );
    // Source-derived (verify card/Nirvana): onScry fires before the draw-pile
    // empty check and queues power-owned raw block immediately.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Nirvana.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/NirvanaPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/ScryAction.java
    #[test]
    fn nirvana_source_triggers_immediately_and_on_empty_draw() {
        let mut with_cards = one_enemy_engine("JawWorm", 50, 0);
        with_cards.state.player.set_status(sid::NIRVANA, 3);
        with_cards.state.player.set_status(sid::DEXTERITY, 5);
        with_cards.state.player.set_status(sid::FRAIL, 1);
        with_cards.state.draw_pile = make_deck(&["Strike"]);
        with_cards.do_scry(1);
        assert_eq!(with_cards.state.player.block, 3);
        assert_eq!(with_cards.phase, CombatPhase::AwaitingChoice);

        let mut empty = one_enemy_engine("JawWorm", 50, 0);
        empty.state.player.set_status(sid::NIRVANA, 4);
        empty.do_scry(3);
        assert_eq!(empty.state.player.block, 4);
        assert_eq!(empty.phase, CombatPhase::PlayerTurn);
    }
    watcher_test!(
        perseverance_java_parity,
        base = ("Perseverance", "Perseverance", 1, -1, 5, 2, CardType::Skill, CardTarget::SelfTarget, false, None, ["retain", "grow_block_on_retain"]),
        plus = ("Perseverance+", "Perseverance+", 1, -1, 7, 3, CardType::Skill, CardTarget::SelfTarget, false, None, ["retain", "grow_block_on_retain"]),
        {}
    );
    // Source-derived (verify card/Perseverance): onRetained calls upgradeBlock
    // on that AbstractCard instance. A freshly drawn second copy must not
    // inherit the first copy's accumulated block.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Perseverance.java
    #[test]
    fn perseverance_source_growth_is_per_card_instance() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.hand = make_deck(&["Perseverance"]);
        engine.state.draw_pile = make_deck(&["Perseverance"]);
        end_turn(&mut engine);

        let mut growth: Vec<_> = engine
            .state
            .hand
            .iter()
            .filter(|card| engine.card_registry.card_name(card.def_id) == "Perseverance")
            .map(|card| card.misc)
            .collect();
        growth.sort_unstable();
        assert_eq!(growth, vec![-1, 7]);

        assert!(play_self(&mut engine, "Perseverance"));
        assert!(play_self(&mut engine, "Perseverance"));
        assert_eq!(engine.state.player.block, 12); // retained 7 + fresh 5
    }

    // Source-derived (verify card/Omniscience): the temporary choice is sorted
    // by name, rarity descending, then Status-last. The selected original is
    // exhausted, its stat-equivalent copy purges, and both random-target
    // autoplays consume cardRandomRng even with one living enemy.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Omniscience.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/OmniscienceAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/NewQueueCardAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    #[test]
    fn omniscience_source_sorts_then_autoplays_twice_with_exhaust_and_purge() {
        let mut engine = one_enemy_engine("JawWorm", 40, 0);
        engine.state.energy = 4;
        engine.state.hand = make_deck(&["Omniscience"]);
        engine.state.draw_pile = make_deck(&["Strike", "Burn", "Perseverance", "Ragnarok"]);
        assert!(play_self(&mut engine, "Omniscience"));

        let choice = engine.choice.as_ref().expect("Omniscience draw-pile choice");
        let names: Vec<_> = choice
            .options
            .iter()
            .map(|option| match option {
                ChoiceOption::DrawCard(idx) => engine
                    .card_registry
                    .card_name(engine.state.draw_pile[*idx].def_id),
                other => panic!("unexpected Omniscience option {other:?}"),
            })
            .collect();
        assert_eq!(names, vec!["Ragnarok", "Perseverance", "Strike", "Burn"]);

        let card_random_before = engine.card_random_rng.counter;
        engine.execute_action(&Action::Choose(2));
        assert_eq!(engine.state.enemies[0].entity.hp, 28);
        assert_eq!(engine.card_random_rng.counter, card_random_before + 2);
        assert_eq!(exhaust_prefix_count(&engine, "Strike"), 1);
        assert_eq!(discard_prefix_count(&engine, "Strike"), 0);
        assert_eq!(hand_prefix_count(&engine, "Strike"), 0);
        assert_eq!(exhaust_prefix_count(&engine, "Omniscience"), 1);
    }

    #[test]
    fn omniscience_source_nested_choice_finishes_before_the_queued_copy() {
        let mut engine = one_enemy_engine("JawWorm", 40, 0);
        engine.state.energy = 4;
        engine.state.hand = make_deck(&["Omniscience"]);
        engine.state.draw_pile = make_deck(&["Strike", "Omniscience+"]);

        assert!(play_self(&mut engine, "Omniscience"));
        engine.execute_action(&Action::Choose(0)); // sorted rare Omniscience+
        assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
        assert_eq!(engine.choice.as_ref().expect("nested choice").options.len(), 1);

        engine.execute_action(&Action::Choose(0)); // Strike from nested Omniscience
        assert_eq!(engine.phase, CombatPhase::PlayerTurn);
        assert_eq!(engine.state.enemies[0].entity.hp, 28);
        assert_eq!(exhaust_prefix_count(&engine, "Omniscience"), 2);
        assert_eq!(exhaust_prefix_count(&engine, "Strike"), 1);
        assert!(engine.omniscience_autoplay.is_empty());
        assert!(engine.runtime_played_card.is_none());
    }
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
    // Source-derived (verify card/Pray): randomSpot inserts one Insight using
    // cardRandomRng while preserving the relative order of the existing draw
    // pile; an empty pile needs no RNG call.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Pray.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInDrawPileAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
    #[test]
    fn pray_source_uses_one_card_random_insertion_without_shuffling() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.draw_pile = make_deck(&["Strike", "Defend", "Worship"]);
        ensure_in_hand(&mut engine, "Pray");
        let card_random_before = engine.card_random_rng.counter;
        let card_before = engine.rng.counter;
        assert!(play_self(&mut engine, "Pray"));
        assert_eq!(engine.state.mantra, 3);
        assert_eq!(engine.card_random_rng.counter, card_random_before + 1);
        assert_eq!(engine.rng.counter, card_before);
        let existing: Vec<_> = engine
            .state
            .draw_pile
            .iter()
            .filter_map(|card| {
                let name = engine.card_registry.card_name(card.def_id);
                (name != "Insight").then_some(name)
            })
            .collect();
        assert_eq!(existing, vec!["Strike", "Defend", "Worship"]);

        let mut empty = one_enemy_engine("JawWorm", 50, 0);
        ensure_in_hand(&mut empty, "Pray+");
        let card_random_before = empty.card_random_rng.counter;
        assert!(play_self(&mut empty, "Pray+"));
        assert_eq!(empty.card_random_rng.counter, card_random_before);
        assert_eq!(draw_prefix_count(&empty, "Insight"), 1);
    }

    // Source-derived (verify card/PathToVictory): Mark triggers LoseHPAction on
    // every marked enemy. HP_LOSS bypasses block, is capped by Intangible, and
    // is reduced to zero by Buffer while consuming one Buffer stack.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/PressurePoints.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/MarkPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/LoseHPAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/AbstractMonster.java
    #[test]
    fn pressure_points_source_routes_mark_through_hp_loss_defenses() {
        let mut engine = two_enemy_engine(("JawWorm", 20, 0), ("Cultist", 15, 0));
        engine.state.enemies[0].entity.block = 99;
        engine.state.enemies[0].entity.set_status(sid::INTANGIBLE, 1);
        engine.state.enemies[1].entity.block = 99;
        engine.state.enemies[1].entity.set_status(sid::MARK, 5);
        engine.state.enemies[1].entity.set_status(sid::BUFFER, 1);
        ensure_in_hand(&mut engine, "PathToVictory");
        assert!(play_on_enemy(&mut engine, "PathToVictory", 0));

        assert_eq!(engine.state.enemies[0].entity.status(sid::MARK), 8);
        assert_eq!(engine.state.enemies[0].entity.hp, 19);
        assert_eq!(engine.state.enemies[0].entity.block, 99);
        assert_eq!(engine.state.enemies[1].entity.hp, 15);
        assert_eq!(engine.state.enemies[1].entity.block, 99);
        assert_eq!(engine.state.enemies[1].entity.status(sid::BUFFER), 0);
    }

    // Source-derived (verify card/Prostrate): base and upgrade both cost zero
    // and own block 4; only Mantra increases from 2 to 3. Card block still uses
    // the ordinary Dexterity/Frail calculation.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Prostrate.java
    #[test]
    fn prostrate_source_upgrade_changes_only_mantra_and_block_uses_modifiers() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.player.set_status(sid::DEXTERITY, 3);
        engine.state.player.set_status(sid::FRAIL, 1);
        ensure_in_hand(&mut engine, "Prostrate");
        ensure_in_hand(&mut engine, "Prostrate+");
        let energy_before = engine.state.energy;
        assert!(play_self(&mut engine, "Prostrate"));
        assert!(play_self(&mut engine, "Prostrate+"));
        assert_eq!(engine.state.energy, energy_before);
        assert_eq!(engine.state.mantra, 5);
        assert_eq!(engine.state.player.block, 10);
    }
    watcher_test!(
        protect_java_parity_coverage,
        base = ("Protect", "Protect", 2, -1, 12, -1, CardType::Skill, CardTarget::SelfTarget, false, None, ["retain"]),
        plus = ("Protect+", "Protect+", 2, -1, 16, -1, CardType::Skill, CardTarget::SelfTarget, false, None, ["retain"]),
        {}
    );
    // Source-derived (verify card/Protect): selfRetain keeps an unplayed copy;
    // the upgrade changes block 12 -> 16 and ordinary Dexterity/Frail modifiers
    // apply when it is eventually played.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Protect.java
    #[test]
    fn protect_source_self_retains_and_uses_modified_upgraded_block() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.draw_pile = make_deck_n("Strike+", 10);
        engine.state.hand = make_deck(&["Protect+"]);
        end_turn(&mut engine);
        assert_eq!(hand_count(&engine, "Protect+"), 1);
        engine.state.player.set_status(sid::DEXTERITY, 4);
        engine.state.player.set_status(sid::FRAIL, 1);
        assert!(play_self(&mut engine, "Protect+"));
        assert_eq!(engine.state.player.block, 15); // floor((16 + 4) * 0.75)
    }
    watcher_test!(
        ragnarok_java_parity,
        base = ("Ragnarok", "Ragnarok", 3, 5, -1, 5, CardType::Attack, CardTarget::AllEnemy, false, None, []),
        plus = ("Ragnarok+", "Ragnarok+", 3, 6, -1, 6, CardType::Attack, CardTarget::AllEnemy, false, None, []),
        {}
    );
    // Source-derived (verify card/Ragnarok): five separate random-enemy
    // actions each consume cardRandomRng, even when random(0, 0) has only one
    // legal target. The upgrade queues six hits at six damage.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Ragnarok.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/AttackDamageRandomEnemyAction.java
    #[test]
    fn ragnarok_source_consumes_card_random_once_per_hit() {
        let mut engine = one_enemy_engine("JawWorm", 100, 0);
        engine.state.hand = make_deck(&["Ragnarok"]);
        let card_random_before = engine.card_random_rng.counter;
        let card_before = engine.rng.counter;
        assert!(play_on_enemy(&mut engine, "Ragnarok", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 75);
        assert_eq!(engine.card_random_rng.counter, card_random_before + 5);
        assert_eq!(engine.rng.counter, card_before);
    }
    watcher_test!(
        reach_heaven_java_parity,
        base = ("ReachHeaven", "Reach Heaven", 2, 10, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["add_through_violence_to_draw"]),
        plus = ("ReachHeaven+", "Reach Heaven+", 2, 15, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["add_through_violence_to_draw"]),
        {}
    );
    // Source-derived (verify card/ReachHeaven): damage resolves before one base
    // Through Violence is inserted at a cardRandomRng-selected draw index; the
    // existing pile is not shuffled.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ReachHeaven.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInDrawPileAction.java
    #[test]
    fn reach_heaven_source_uses_random_insertion_without_shuffling() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.draw_pile = make_deck(&["Strike", "Defend", "Worship"]);
        engine.state.hand = make_deck(&["ReachHeaven+"]);
        let card_random_before = engine.card_random_rng.counter;
        let card_before = engine.rng.counter;
        assert!(play_on_enemy(&mut engine, "ReachHeaven+", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 35);
        assert_eq!(engine.card_random_rng.counter, card_random_before + 1);
        assert_eq!(engine.rng.counter, card_before);
        let names: Vec<_> = engine
            .state
            .draw_pile
            .iter()
            .map(|card| engine.card_registry.card_name(card.def_id))
            .collect();
        let existing: Vec<_> = names
            .iter()
            .copied()
            .filter(|name| *name != "ThroughViolence")
            .collect();
        assert_eq!(existing, vec!["Strike", "Defend", "Worship"]);
        assert_eq!(names.iter().filter(|name| **name == "ThroughViolence").count(), 1);
    }
    watcher_test!(
        rushdown_java_parity,
        base = ("Adaptation", "Rushdown", 1, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false, None, []),
        plus = ("Adaptation+", "Rushdown+", 0, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false, None, []),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.draw_pile = make_deck(&["Strike", "Defend", "Worship"]);
            ensure_in_hand(&mut engine, "Adaptation");
            ensure_in_hand(&mut engine, "Eruption");
            play_self(&mut engine, "Adaptation");
            let hand_before = engine.state.hand.len();
            play_on_enemy(&mut engine, "Eruption", 0);
            assert_eq!(engine.state.hand.len(), hand_before + 1); // Eruption played (-1) + Rushdown draws 2 on Wrath entry
        }
    );

    // Source-derived (verify card/Adaptation): Rushdown.java (ID Adaptation)
    // applies RushdownPower with magicNumber 2. Its upgrade changes only the
    // cost from 1 to 0; the amount remains 2.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Rushdown.java
    #[test]
    fn adaptation_source_applies_two_rushdown_and_upgrade_only_reduces_cost() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        ensure_in_hand(&mut engine, "Adaptation");
        let energy_before = engine.state.energy;
        play_self(&mut engine, "Adaptation");
        assert_eq!(engine.state.player.status(sid::RUSHDOWN), 2);
        assert_eq!(engine.state.energy, energy_before - 1);

        let base = reg().get("Adaptation").expect("Adaptation registered");
        let plus = reg().get("Adaptation+").expect("Adaptation+ registered");
        assert_eq!((base.cost, base.base_magic), (1, 2));
        assert_eq!((plus.cost, plus.base_magic), (0, 2));
    }
    watcher_test!(
        scrawl_java_parity,
        base = ("Scrawl", "Scrawl", 1, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, ["draw_to_ten"]),
        plus = ("Scrawl+", "Scrawl+", 0, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, ["draw_to_ten"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.hand = make_deck(&["Scrawl", "Strike", "Defend"]);
            engine.state.draw_pile = make_deck(&["Strike", "Strike", "Strike", "Strike", "Strike", "Strike", "Strike", "Strike", "Strike"]);
            ensure_in_hand(&mut engine, "Scrawl");
            play_self(&mut engine, "Scrawl");
            assert_eq!(engine.state.hand.len(), 10);
        }
    );
    // Source-derived (verify card/Scrawl): ExpertiseAction draws exactly the
    // number needed to reach ten cards after Scrawl leaves hand; Scrawl then
    // exhausts, and the upgrade only makes it free.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Scrawl.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ExpertiseAction.java
    #[test]
    fn scrawl_source_draws_only_to_ten_and_exhausts() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.hand = make_deck(&[
            "Scrawl+", "Defend", "Defend", "Defend", "Defend", "Defend", "Defend",
        ]);
        engine.state.draw_pile = make_deck_n("Strike+", 20);
        let draw_before = engine.state.draw_pile.len();
        assert!(play_self(&mut engine, "Scrawl+"));
        assert_eq!(engine.state.hand.len(), 10);
        assert_eq!(engine.state.draw_pile.len(), draw_before - 4);
        assert_eq!(exhaust_prefix_count(&engine, "Scrawl+"), 1);
    }
    watcher_test!(
        spirit_shield_java_parity,
        base = ("SpiritShield", "Spirit Shield", 2, -1, -1, 3, CardType::Skill, CardTarget::SelfTarget, false, None, ["block_per_card_in_hand"]),
        plus = ("SpiritShield+", "Spirit Shield+", 2, -1, -1, 4, CardType::Skill, CardTarget::SelfTarget, false, None, ["block_per_card_in_hand"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.hand = make_deck(&["SpiritShield", "Strike", "Defend", "Worship", "Protect", "Prostrate"]);
            play_self(&mut engine, "SpiritShield");
            assert_eq!(engine.state.player.block, 15);
        }
    );
    // Source-derived (verify card/SpiritShield): applyPowers excludes the card
    // itself, multiplies the remaining hand count by magicNumber, and only then
    // runs the ordinary block-power pipeline once. With four remaining cards,
    // Spirit Shield+ has base block 16; Dexterity 4 and Frail yield floor(20*.75)=15.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/SpiritShield.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/AbstractCard.java
    #[test]
    fn spirit_shield_source_applies_block_modifiers_once_to_the_total() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.hand = make_deck(&[
            "SpiritShield+", "Strike", "Defend", "Worship", "Protect",
        ]);
        engine.state.player.set_status(sid::DEXTERITY, 4);
        engine.state.player.set_status(sid::FRAIL, 1);
        assert!(play_self(&mut engine, "SpiritShield+"));
        assert_eq!(engine.state.player.block, 15);
    }
    watcher_test!(
        study_java_parity,
        base = ("Study", "Study", 2, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, []),
        plus = ("Study+", "Study+", 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, []),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.draw_pile = make_deck_n("Strike", 5);
            ensure_in_hand(&mut engine, "Study");
            play_self(&mut engine, "Study");
            end_turn(&mut engine);
            let total_insights = hand_prefix_count(&engine, "Insight")
                + draw_prefix_count(&engine, "Insight")
                + discard_prefix_count(&engine, "Insight");
            assert_eq!(total_insights, 1); // Study adds exactly 1 Insight at end of turn
        }
    );
    // Source-derived (verify card/Study): StudyPower.atEndOfTurn passes
    // randomSpot=true to MakeTempCardInDrawPileAction once per power stack.
    // CardGroup.addToRandomSpot consumes cardRandomRng and preserves existing
    // draw-pile order instead of shuffling it.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Study.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/StudyPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
    #[test]
    fn study_source_randomly_inserts_each_insight_without_shuffling() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.draw_pile = make_deck(&[
            "Strike", "Defend", "Worship", "Protect", "Prostrate", "Vigilance",
        ]);
        engine.state.hand = make_deck(&["Study", "Study+"]);
        engine.state.energy = 3;
        assert!(play_self(&mut engine, "Study"));
        assert!(play_self(&mut engine, "Study+"));
        assert_eq!(engine.state.player.status(sid::STUDY), 2);
        let card_random_before = engine.card_random_rng.counter;
        let shuffle_before = engine.rng.counter;
        end_turn(&mut engine);
        assert_eq!(engine.card_random_rng.counter, card_random_before + 2);
        assert_eq!(engine.rng.counter, shuffle_before);
        let existing: Vec<_> = engine
            .state
            .draw_pile
            .iter()
            .chain(engine.state.hand.iter())
            .filter_map(|card| {
                let name = engine.card_registry.card_name(card.def_id);
                (name != "Insight").then_some(name)
            })
            .collect();
        assert_eq!(existing.len(), 6);
        let total_insights = hand_prefix_count(&engine, "Insight")
            + draw_prefix_count(&engine, "Insight")
            + discard_prefix_count(&engine, "Insight");
        assert_eq!(total_insights, 2);
    }
    watcher_test!(
        swivel_java_parity,
        base = ("Swivel", "Swivel", 2, -1, 8, -1, CardType::Skill, CardTarget::SelfTarget, false, None, ["next_attack_free"]),
        plus = ("Swivel+", "Swivel+", 2, -1, 11, -1, CardType::Skill, CardTarget::SelfTarget, false, None, ["next_attack_free"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "Swivel");
            ensure_in_hand(&mut engine, "Strike");
            play_self(&mut engine, "Swivel");
            let energy_before = engine.state.energy;
            play_on_enemy(&mut engine, "Strike", 0);
            assert_eq!(engine.state.energy, energy_before);
        }
    );
    // Source-derived (verify card/Swivel): ApplyPowerAction stacks
    // FreeAttackPower, whose onUseCard decrements exactly one amount for each
    // non-purge Attack. Two Swivels therefore make the next two Attacks free.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Swivel.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/FreeAttackPower.java
    #[test]
    fn swivel_source_stacks_and_consumes_one_free_attack_charge_at_a_time() {
        let mut engine = one_enemy_engine("JawWorm", 100, 0);
        engine.state.hand = make_deck(&[
            "Swivel", "Swivel", "Strike", "Strike", "Strike",
        ]);
        engine.state.energy = 10;
        assert!(play_self(&mut engine, "Swivel"));
        assert!(play_self(&mut engine, "Swivel"));
        assert_eq!(engine.state.player.block, 16);
        assert_eq!(engine.state.player.status(sid::NEXT_ATTACK_FREE), 2);
        assert_eq!(engine.state.energy, 6);

        assert!(play_on_enemy(&mut engine, "Strike", 0));
        assert_eq!(engine.state.player.status(sid::NEXT_ATTACK_FREE), 1);
        assert_eq!(engine.state.energy, 6);
        assert!(play_on_enemy(&mut engine, "Strike", 0));
        assert_eq!(engine.state.player.status(sid::NEXT_ATTACK_FREE), 0);
        assert_eq!(engine.state.energy, 6);
        assert!(play_on_enemy(&mut engine, "Strike", 0));
        assert_eq!(engine.state.energy, 5);
    }
    watcher_test!(
        talk_to_the_hand_java_parity,
        base = ("TalkToTheHand", "Talk to the Hand", 1, 5, -1, 2, CardType::Attack, CardTarget::Enemy, true, None, ["apply_block_return"]),
        plus = ("TalkToTheHand+", "Talk to the Hand+", 1, 7, -1, 3, CardType::Attack, CardTarget::Enemy, true, None, ["apply_block_return"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            ensure_in_hand(&mut engine, "TalkToTheHand");
            ensure_in_hand(&mut engine, "Strike");
            play_on_enemy(&mut engine, "TalkToTheHand", 0);
            play_on_enemy(&mut engine, "Strike", 0);
            assert_eq!(engine.state.player.block, 2); // BlockReturn=2 triggered by Strike hit
        }
    );
    // Source-derived (verify card/TalkToTheHand): BlockReturnPower is applied
    // after the card's damage, stacks by amount, and onAttacked queues block for
    // every ordinary attack even if decrementBlock reduced damage to zero.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/TalkToTheHand.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/BlockReturnPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/AbstractMonster.java
    #[test]
    fn talk_to_the_hand_source_triggers_on_a_fully_blocked_attack() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.enemies[0].entity.block = 50;
        engine.state.hand = make_deck(&["TalkToTheHand+", "Strike"]);
        engine.state.energy = 2;
        assert!(play_on_enemy(&mut engine, "TalkToTheHand+", 0));
        assert_eq!(engine.state.enemies[0].entity.status(sid::BLOCK_RETURN), 3);
        assert_eq!(engine.state.player.block, 0);
        assert!(play_on_enemy(&mut engine, "Strike", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 50);
        assert_eq!(engine.state.player.block, 3);
    }
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
    // Source-derived (verify card/Vault): SkipEnemiesTurnAction sets the room
    // flag before PressEndTurnButtonAction. GameActionManager therefore skips
    // both monster actions and monsters.applyEndOfTurnPowers for that round.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Vault.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    #[test]
    fn vault_source_skips_enemy_action_and_enemy_power_decrement() {
        let mut engine = one_enemy_engine("JawWorm", 50, 20);
        engine.state.enemies[0].entity.set_status(sid::VULNERABLE, 2);
        engine.state.hand = make_deck(&["Vault+"]);
        engine.state.draw_pile = make_deck_n("Strike", 5);
        engine.state.energy = 2;
        let hp_before = engine.state.player.hp;
        let turn_before = engine.state.turn;
        assert!(play_self(&mut engine, "Vault+"));
        assert_eq!(engine.state.turn, turn_before + 1);
        assert_eq!(engine.state.player.hp, hp_before);
        assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 2);
        assert_eq!(exhaust_prefix_count(&engine, "Vault+"), 1);
    }
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
    // Source-derived (verify card/Wallop): WallopAction gains exactly
    // target.lastDamageTaken through GainBlockAction. That damage-derived block
    // is not card.block, so Dexterity and Frail do not modify it.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wallop.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/WallopAction.java
    #[test]
    fn wallop_source_returns_last_damage_as_raw_block() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.hand = make_deck(&["Wallop"]);
        engine.state.player.set_status(sid::DEXTERITY, 5);
        engine.state.player.set_status(sid::FRAIL, 1);
        assert!(play_on_enemy(&mut engine, "Wallop", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 41);
        assert_eq!(engine.state.player.block, 9);
    }

    // Source-derived (verify card/WaveOfTheHand): each positive block gain
    // applies the full stacked amount of Weak to every monster. ApplyPowerAction
    // still respects Artifact independently on each target.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/WaveOfTheHand.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/WaveOfTheHandPower.java
    #[test]
    fn wave_of_the_hand_source_triggers_per_block_gain_and_respects_artifact() {
        let mut engine = two_enemy_engine(("JawWorm", 50, 0), ("Cultist", 50, 0));
        engine.state.enemies[0].entity.set_status(sid::ARTIFACT, 1);
        engine.state.hand = make_deck(&["WaveOfTheHand+", "Defend", "Defend"]);
        assert!(play_self(&mut engine, "WaveOfTheHand+"));
        assert!(play_self(&mut engine, "Defend"));
        assert_eq!(engine.state.enemies[0].entity.status(sid::ARTIFACT), 0);
        assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 0);
        assert_eq!(engine.state.enemies[1].entity.status(sid::WEAKENED), 2);
        assert!(play_self(&mut engine, "Defend"));
        assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 2);
        assert_eq!(engine.state.enemies[1].entity.status(sid::WEAKENED), 4);
    }
    watcher_test!(
        weave_java_parity,
        base = ("Weave", "Weave", 0, 4, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["return_on_scry"]),
        plus = ("Weave+", "Weave+", 0, 6, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["return_on_scry"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            // Need cards in draw pile so Scry has something to reveal
            engine.state.draw_pile = make_deck(&["Strike", "Defend", "Worship"]);
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
    // Source-derived (verify card/Weave): DiscardToHandAction checks the hand
    // cap before removing its exact Weave instance from discard. A full hand
    // therefore leaves Weave in discard instead of deleting it.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Weave.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/DiscardToHandAction.java
    #[test]
    fn weave_source_stays_in_discard_when_scry_resolves_with_full_hand() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.hand = make_deck_n("Strike", 10);
        engine.state.draw_pile = make_deck(&["Defend"]);
        engine.state.discard_pile = make_deck(&["Weave+"]);
        engine.do_scry(1);
        assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
        engine.execute_action(&Action::ConfirmSelection);
        assert_eq!(engine.state.hand.len(), 10);
        assert_eq!(discard_prefix_count(&engine, "Weave+"), 1);
    }

    // Source-derived (verify card/WheelKick): DamageAction is queued before
    // DrawCardAction(2). If that damage kills the final monster,
    // clearPostCombatActions removes the queued draw.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/WheelKick.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAction.java
    #[test]
    fn wheel_kick_source_does_not_draw_after_lethal_final_damage() {
        let mut engine = one_enemy_engine("JawWorm", 10, 0);
        engine.state.hand = make_deck(&["WheelKick"]);
        engine.state.draw_pile = make_deck(&["Strike", "Defend"]);
        assert!(play_on_enemy(&mut engine, "WheelKick", 0));
        assert!(engine.state.combat_over);
        assert_eq!(engine.state.draw_pile.len(), 2);
        assert_eq!(engine.state.hand.len(), 0);
    }

    // Source-derived (verify card/WindmillStrike): onRetained calls
    // upgradeDamage(magicNumber) on each exact AbstractCard. Base and upgraded
    // copies therefore grow independently by 4 and 5 per retained turn.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/WindmillStrike.java
    #[test]
    fn windmill_strike_source_growth_is_per_card_instance() {
        let mut engine = one_enemy_engine("JawWorm", 100, 0);
        engine.state.hand = make_deck(&["WindmillStrike", "WindmillStrike+"]);
        end_turn(&mut engine);
        let base = engine.state.hand.iter()
            .find(|card| engine.card_registry.card_name(card.def_id) == "WindmillStrike")
            .expect("base Windmill retained");
        let plus = engine.state.hand.iter()
            .find(|card| engine.card_registry.card_name(card.def_id) == "WindmillStrike+")
            .expect("upgraded Windmill retained");
        assert_eq!(base.misc, 11);
        assert_eq!(plus.misc, 15);

        assert!(play_on_enemy(&mut engine, "WindmillStrike", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 89);
        end_turn(&mut engine);
        let plus = engine.state.hand.iter()
            .find(|card| engine.card_registry.card_name(card.def_id) == "WindmillStrike+")
            .expect("upgraded Windmill retained twice");
        assert_eq!(plus.misc, 20);
    }
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
    // Source-derived (verify card/WreathOfFlame): ApplyPowerAction stacks
    // Vigor, atDamageGive adds the full amount to normal attack damage, and
    // onUseCard removes it after the next Attack. Tantrum's three DamageActions
    // therefore each use the same stacked 13-point Vigor bonus.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/WreathOfFlame.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/VigorPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Tantrum.java
    #[test]
    fn wreath_of_flame_source_stacks_for_every_hit_of_the_next_attack() {
        let mut engine = one_enemy_engine("JawWorm", 100, 0);
        engine.state.hand = make_deck(&[
            "WreathOfFlame", "WreathOfFlame+", "Tantrum",
        ]);
        assert!(play_self(&mut engine, "WreathOfFlame"));
        assert!(play_self(&mut engine, "WreathOfFlame+"));
        assert_eq!(engine.state.player.status(sid::VIGOR), 13);
        assert!(play_on_enemy(&mut engine, "Tantrum", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 52);
        assert_eq!(engine.state.player.status(sid::VIGOR), 0);
    }

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

    // Source-derived (verify card/Brilliance): Brilliance.java adds the total
    // mantra gained this combat to base damage; upgrade() adds 4 base damage.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Brilliance.java
    #[test]
    fn brilliance_source_scales_with_total_mantra_gained() {
        let mut engine = one_enemy_engine("JawWorm", 60, 0);
        engine.gain_mantra(7);
        ensure_in_hand(&mut engine, "Brilliance");
        play_on_enemy(&mut engine, "Brilliance", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 41); // 60 - (12 + 7)

        let base = reg().get("Brilliance").expect("Brilliance registered");
        let plus = reg().get("Brilliance+").expect("Brilliance+ registered");
        assert_eq!((base.cost, base.base_damage), (1, 12));
        assert_eq!((plus.cost, plus.base_damage), (1, 16));
    }
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

    // Source-derived (verify card/Conclude): Conclude.java deals 12 to all
    // enemies before pressing the end-turn button; upgrade() adds 4 damage.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Conclude.java
    #[test]
    fn conclude_source_hits_all_enemies_then_ends_turn() {
        let mut engine = two_enemy_engine(("JawWorm", 40, 0), ("Cultist", 40, 0));
        ensure_in_hand(&mut engine, "Conclude");
        let turn_before = engine.state.turn;
        play_on_enemy(&mut engine, "Conclude", 0);
        assert_eq!(engine.state.enemies[0].entity.hp, 28);
        assert_eq!(engine.state.enemies[1].entity.hp, 28);
        assert_eq!(engine.state.turn, turn_before + 1);

        let base = reg().get("Conclude").expect("Conclude registered");
        let plus = reg().get("Conclude+").expect("Conclude+ registered");
        assert_eq!((base.cost, base.base_damage), (1, 12));
        assert_eq!((plus.cost, plus.base_damage), (1, 16));
    }
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
    // Source-derived (verify card/Devotion): DevotionPower fires post-draw.
    // Ten stacks with no existing Mantra enter Divinity, which must remain
    // active for the new turn rather than being cleared by pre-draw auto-exit.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Devotion.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/DevotionPower.java
    #[test]
    fn devotion_source_enters_divinity_after_draw_and_keeps_it_for_the_turn() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.player.set_status(sid::DEVOTION, 10);
        engine.rebuild_effect_runtime();
        engine.state.draw_pile = make_deck(&["Strike"; 5]);
        end_turn(&mut engine);

        assert_eq!(engine.state.hand.len(), 5);
        assert_eq!(engine.state.stance, Stance::Divinity);
        assert_eq!(engine.state.mantra, 0);
    }
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
    // Source-derived (verify card/DevaForm): DevaPower.stackPower increments
    // both amount and energyGainAmount; recharge grants energyGainAmount and
    // then increments it by amount. Two copies therefore grant 2, then 4.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/DevaForm.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/DevaPower.java
    #[test]
    fn deva_form_source_keeps_stacks_separate_from_escalating_energy() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.energy = 6;
        engine.state.hand = make_deck(&["DevaForm", "DevaForm"]);
        play_self(&mut engine, "DevaForm");
        play_self(&mut engine, "DevaForm");
        assert_eq!(engine.state.player.status(sid::DEVA_FORM), 2);
        assert_eq!(engine.state.player.status(sid::DEVA_FORM_ENERGY), 2);

        end_turn(&mut engine);
        assert_eq!(engine.state.energy, 5);
        assert_eq!(engine.state.player.status(sid::DEVA_FORM), 2);
        assert_eq!(engine.state.player.status(sid::DEVA_FORM_ENERGY), 4);

        end_turn(&mut engine);
        assert_eq!(engine.state.energy, 7);
        assert_eq!(engine.state.player.status(sid::DEVA_FORM_ENERGY), 6);
    }
    // Source-derived (verify card/Discipline): the deprecated power records
    // positive unspent energy at end of turn, then draws that many cards at
    // the next turn start and resets its stored amount.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Discipline.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/deprecated/DEPRECATEDDisciplinePower.java
    watcher_test!(
        discipline_java_parity,
        base = ("Discipline", "Discipline", 2, -1, -1, -1, CardType::Power, CardTarget::SelfTarget, false, None, []),
        plus = ("Discipline+", "Discipline+", 1, -1, -1, -1, CardType::Power, CardTarget::SelfTarget, false, None, []),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.draw_pile = make_deck(&["Strike"; 10]);
            ensure_in_hand(&mut engine, "Discipline+");
            play_self(&mut engine, "Discipline+");
            assert_eq!(engine.state.energy, 2);
            assert_eq!(engine.state.player.status(sid::DISCIPLINE), 1);
            end_turn(&mut engine);
            assert_eq!(engine.state.hand.len(), 7);
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
    // Source-derived (verify card/Establishment): EstablishmentPowerAction
    // reduces only cards with retain/selfRetain. Runic Pyramid merely prevents
    // discard, so an ordinary Strike must not receive the discount. Upgrade is innate.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Establishment.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EstablishmentPowerAction.java
    #[test]
    fn establishment_source_ignores_ordinary_runic_pyramid_cards() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.relics.push("Runic Pyramid".to_string());
        engine.state.hand = make_deck(&["Establishment", "Protect", "Strike"]);
        play_self(&mut engine, "Establishment");
        end_turn(&mut engine);

        let protect = engine.state.hand.iter()
            .find(|card| engine.card_registry.card_name(card.def_id) == "Protect")
            .expect("Protect retained");
        let strike = engine.state.hand.iter()
            .find(|card| engine.card_registry.card_name(card.def_id) == "Strike")
            .expect("Strike held by Runic Pyramid");
        assert_eq!(protect.cost, 1);
        assert_eq!(strike.cost, 1);

        let plus = reg().get("Establishment+").expect("Establishment+ registered");
        assert_eq!((plus.cost, plus.base_magic), (1, 1));
        assert!(plus.runtime_traits().innate);
    }
    // Source-derived (verify card/Evaluate): Evaluate.java gains 6 block (10
    // upgraded), then MakeTempCardInDrawPileAction inserts one Insight at a
    // random spot. CardGroup.addToRandomSpot consumes one RNG tick and leaves
    // the existing top card on top.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Evaluate.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
    #[test]
    fn evaluate_source_inserts_insight_with_one_rng_tick_without_moving_top() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.draw_pile = make_deck(&["Strike", "Defend", "Worship"]);
        ensure_in_hand(&mut engine, "Evaluate+");
        let rng_before = engine.card_random_rng.counter;
        let catch_all_before = engine.rng.counter;
        play_self(&mut engine, "Evaluate+");
        assert_eq!(engine.state.player.block, 10);
        assert_eq!(draw_prefix_count(&engine, "Insight"), 1);
        assert_eq!(engine.card_registry.card_name(engine.state.draw_pile.last().unwrap().def_id), "Worship");
        assert_eq!(engine.card_random_rng.counter, rng_before + 1);
        assert_eq!(engine.rng.counter, catch_all_before);
    }
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
            assert_eq!(engine.state.player.status(sid::ENERGY_DOWN), 1);
            assert_eq!(engine.state.max_energy, 3);
            end_turn(&mut engine);
            assert_eq!(engine.state.energy, 2);
        }
    );
    // Source-derived (verify card/Fasting2): Fasting.java applies Strength and
    // Dexterity before EnergyDownPower(1). Energy Down is a debuff, so Artifact
    // consumes itself and blocks only that power; max energy remains unchanged.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Fasting.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/EnergyDownPower.java
    #[test]
    fn fasting_source_energy_down_is_artifact_blockable_and_not_max_energy() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.player.set_status(sid::ARTIFACT, 1);
        ensure_in_hand(&mut engine, "Fasting2");
        play_self(&mut engine, "Fasting2");
        assert_eq!(engine.state.player.strength(), 3);
        assert_eq!(engine.state.player.dexterity(), 3);
        assert_eq!(engine.state.player.status(sid::ARTIFACT), 0);
        assert_eq!(engine.state.player.status(sid::ENERGY_DOWN), 0);
        assert_eq!(engine.state.max_energy, 3);
        end_turn(&mut engine);
        assert_eq!(engine.state.energy, 3);
    }
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
    // Source-derived (verify card/Judgement): JudgementAction queues
    // InstantKillAction, which sets HP to zero before a zero-damage HP_LOSS
    // callback. Block and Flight remain untouched and no dealt-damage total is
    // added.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/JudgementAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/InstantKillAction.java
    #[test]
    fn judgement_source_instant_kill_bypasses_block_flight_and_damage_totals() {
        let mut engine = one_enemy_engine("JawWorm", 30, 0);
        engine.state.enemies[0].entity.block = 999;
        engine.state.enemies[0].entity.set_status(sid::FLIGHT, 3);
        let damage_before = engine.state.total_damage_dealt;
        ensure_in_hand(&mut engine, "Judgement");
        play_on_enemy(&mut engine, "Judgement", 0);

        assert!(engine.state.enemies[0].entity.is_dead());
        assert_eq!(engine.state.enemies[0].entity.block, 999);
        assert_eq!(engine.state.enemies[0].entity.status(sid::FLIGHT), 3);
        assert_eq!(engine.state.total_damage_dealt, damage_before);
    }
    watcher_test!(
        lesson_learned_java_parity,
        base = ("LessonLearned", "Lesson Learned", 2, 10, -1, -1, CardType::Attack, CardTarget::Enemy, true, None, ["lesson_learned"]),
        plus = ("LessonLearned+", "Lesson Learned+", 2, 13, -1, -1, CardType::Attack, CardTarget::Enemy, true, None, ["lesson_learned"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 10, 0);
            engine.state.draw_pile = make_deck(&["Evaluate"]);
            engine.state.master_deck = make_deck(&["Evaluate"]);
            ensure_in_hand(&mut engine, "LessonLearned");
            play_on_enemy(&mut engine, "LessonLearned", 0);
            assert!(engine.state.master_deck.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Evaluate+"));
            assert!(engine.state.draw_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Evaluate"));
        }
    );
    // Source-derived (verify card/LessonLearned): a qualifying kill chooses one
    // upgradeable card from masterDeck with exactly one miscRng call. Combat
    // copies stay unchanged, and MinionPower/half-dead targets do not qualify.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/LessonLearned.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/LessonLearnedAction.java
    #[test]
    fn lesson_learned_source_upgrades_master_deck_with_misc_rng_and_skips_minions() {
        let mut normal = one_enemy_engine("JawWorm", 10, 0);
        normal.state.draw_pile = make_deck(&["Wallop"]);
        normal.state.master_deck = make_deck(&["Wallop", "Strike+", "AscendersBane"]);
        ensure_in_hand(&mut normal, "LessonLearned");
        let misc_before = normal.misc_rng.counter;
        let card_before = normal.rng.counter;
        play_on_enemy(&mut normal, "LessonLearned", 0);
        assert_eq!(normal.misc_rng.counter, misc_before + 1);
        assert_eq!(normal.rng.counter, card_before);
        assert_eq!(normal.card_registry.card_name(normal.state.master_deck[0].def_id), "Wallop+");
        assert_eq!(normal.card_registry.card_name(normal.state.draw_pile[0].def_id), "Wallop");

        let mut minion = one_enemy_engine("SnakeDagger", 10, 0);
        minion.state.enemies[0].is_minion = true;
        minion.state.master_deck = make_deck(&["Wallop"]);
        ensure_in_hand(&mut minion, "LessonLearned");
        let misc_before = minion.misc_rng.counter;
        play_on_enemy(&mut minion, "LessonLearned", 0);
        assert_eq!(minion.misc_rng.counter, misc_before);
        assert_eq!(minion.card_registry.card_name(minion.state.master_deck[0].def_id), "Wallop");
    }
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
    // Source-derived (verify card/MasterReality): the power has no amount and
    // therefore does not stack. MakeTempCard paths upgrade ordinary created
    // cards, but explicitly exclude Status and Curse cards.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/MasterReality.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/MasterRealityPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInHandAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/vfx/cardManip/ShowCardAndAddToDiscardEffect.java
    #[test]
    fn master_reality_source_is_nonstacking_and_does_not_upgrade_status_cards() {
        let mut engine = one_enemy_engine("JawWorm", 100, 0);
        ensure_in_hand(&mut engine, "MasterReality");
        ensure_in_hand(&mut engine, "MasterReality+");
        ensure_in_hand(&mut engine, "Immolate");

        assert!(play_self(&mut engine, "MasterReality"));
        assert!(play_self(&mut engine, "MasterReality+"));
        assert_eq!(engine.state.player.status(sid::MASTER_REALITY), 1);
        assert!(play_self(&mut engine, "Immolate"));
        let created: Vec<_> = engine
            .state
            .discard_pile
            .iter()
            .map(|card| engine.card_registry.card_name(card.def_id))
            .collect();
        assert_eq!(created.iter().filter(|name| **name == "Burn").count(), 1);
        assert_eq!(created.iter().filter(|name| **name == "Burn+").count(), 0);
    }
    watcher_test!(
        mediate_java_parity,
        base = ("Meditate", "Meditate", 1, -1, -1, 1, CardType::Skill, CardTarget::None, false, Some("Calm"), ["meditate", "end_turn"]),
        plus = ("Meditate+", "Meditate+", 1, -1, -1, 2, CardType::Skill, CardTarget::None, false, Some("Calm"), ["meditate", "end_turn"]),
        {
            let mut engine = one_enemy_engine("JawWorm", 50, 0);
            engine.state.discard_pile = make_deck(&["Strike", "Defend"]);
            ensure_in_hand(&mut engine, "Meditate");
            play_self(&mut engine, "Meditate");
            // Meditate now presents a choice to pick from discard
            assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
            engine.execute_action(&Action::Choose(0));
            engine.execute_action(&Action::ConfirmSelection);
            assert!(engine.state.hand.iter().any(|c| { let n = engine.card_registry.card_name(c.def_id); n == "Strike" || n == "Defend" }));
        }
    );
    // Source-derived (verify card/Meditate): MeditateAction is playable with
    // an empty discard; auto-moves the whole pile when it fits; otherwise
    // requires exactly magicNumber selections. ChangeStanceAction and
    // PressEndTurnButtonAction are queued after that action, and the returned
    // cards' retain flag is cleared after the one retention.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Meditate.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/MeditateAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/RestoreRetainedCardsAction.java
    #[test]
    fn meditate_source_allows_empty_discard_and_auto_returns_small_piles_once() {
        let mut empty = one_enemy_engine("JawWorm", 100, 0);
        empty.state.draw_pile = make_deck_n("Strike+", 10);
        set_stance(&mut empty, Stance::Wrath);
        ensure_in_hand(&mut empty, "Meditate");
        assert!(play_self(&mut empty, "Meditate"));
        assert_eq!(empty.state.stance, Stance::Calm);

        let mut automatic = one_enemy_engine("JawWorm", 100, 0);
        automatic.state.draw_pile = make_deck_n("Strike+", 10);
        automatic.state.discard_pile = make_deck(&["Evaluate", "Worship"]);
        ensure_in_hand(&mut automatic, "Meditate+");
        assert!(play_self(&mut automatic, "Meditate+"));
        assert_ne!(automatic.phase, CombatPhase::AwaitingChoice);
        for name in ["Evaluate", "Worship"] {
            let returned = automatic
                .state
                .hand
                .iter()
                .find(|card| automatic.card_registry.card_name(card.def_id) == name)
                .expect("the whole small discard pile should return");
            assert!(!returned.is_retained(), "retain clears after this turn");
        }
    }

    #[test]
    fn meditate_source_defers_calm_and_requires_the_full_selection() {
        let mut engine = one_enemy_engine("JawWorm", 100, 0);
        engine.state.draw_pile = make_deck_n("Strike+", 10);
        engine.state.discard_pile = make_deck(&["Evaluate", "Defend", "Worship"]);
        set_stance(&mut engine, Stance::Wrath);
        ensure_in_hand(&mut engine, "Meditate+");
        assert!(play_self(&mut engine, "Meditate+"));

        assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
        assert_eq!(engine.state.stance, Stance::Wrath);
        let choice = engine.choice.as_ref().expect("Meditate+ choice");
        assert_eq!((choice.min_picks, choice.max_picks), (2, 2));
        assert!(!engine.get_legal_actions().contains(&Action::ConfirmSelection));

        engine.execute_action(&Action::Choose(0));
        assert!(!engine.get_legal_actions().contains(&Action::ConfirmSelection));
        engine.execute_action(&Action::Choose(1));
        assert!(engine.get_legal_actions().contains(&Action::ConfirmSelection));
        engine.execute_action(&Action::ConfirmSelection);

        assert_eq!(engine.state.stance, Stance::Calm);
        assert!(engine.state.hand.iter().any(|card| engine.card_registry.card_name(card.def_id) == "Evaluate"));
        assert!(engine.state.hand.iter().any(|card| engine.card_registry.card_name(card.def_id) == "Defend"));
        assert!(engine.state.discard_pile.iter().any(|card| engine.card_registry.card_name(card.def_id) == "Worship"));
    }
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
    // Source-derived (verify card/MentalFortress): ApplyPowerAction stacks the
    // amounts, and the power queues its own GainBlockAction only for a real
    // stance-ID change. Power-owned block is already resolved, so Dexterity
    // and Frail do not modify it.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/MentalFortress.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/MentalFortressPower.java
    #[test]
    fn mental_fortress_source_stacks_and_grants_raw_block_only_on_real_change() {
        let mut engine = one_enemy_engine("JawWorm", 100, 0);
        engine.state.energy = 5;
        engine.state.player.set_status(sid::DEXTERITY, 5);
        engine.state.player.set_status(sid::FRAIL, 1);
        ensure_in_hand(&mut engine, "MentalFortress");
        ensure_in_hand(&mut engine, "MentalFortress+");
        ensure_in_hand(&mut engine, "Eruption");
        ensure_in_hand(&mut engine, "Eruption+");

        assert!(play_self(&mut engine, "MentalFortress"));
        assert!(play_self(&mut engine, "MentalFortress+"));
        assert_eq!(engine.state.player.status(sid::MENTAL_FORTRESS), 10);
        assert!(play_on_enemy(&mut engine, "Eruption", 0));
        assert_eq!(engine.state.player.block, 10);
        assert!(play_on_enemy(&mut engine, "Eruption+", 0));
        assert_eq!(engine.state.player.block, 10);
    }
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
    // Source-derived (verify card/Sanctity): SanctityAction examines the card
    // immediately before the current Sanctity and draws exactly two only when
    // it was a Skill; block is always gained first.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Sanctity.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/SanctityAction.java
    #[test]
    fn sanctity_source_draws_only_after_a_previous_skill() {
        let mut skill = one_enemy_engine("JawWorm", 50, 0);
        skill.state.draw_pile = make_deck_n("Strike+", 10);
        skill.state.hand = make_deck(&["Defend", "Sanctity+"]);
        assert!(play_self(&mut skill, "Defend"));
        assert!(play_self(&mut skill, "Sanctity+"));
        assert_eq!(skill.state.player.block, 14);
        assert_eq!(skill.state.hand.len(), 2);

        let mut attack = one_enemy_engine("JawWorm", 50, 0);
        attack.state.draw_pile = make_deck_n("Strike+", 10);
        attack.state.hand = make_deck(&["Strike", "Sanctity"]);
        assert!(play_on_enemy(&mut attack, "Strike", 0));
        assert!(play_self(&mut attack, "Sanctity"));
        assert_eq!(attack.state.player.block, 6);
        assert!(attack.state.hand.is_empty());
    }

    // Source-derived (verify card/SandsOfTime): selfRetain calls
    // modifyCostForCombat(-1) on this exact card every turn, so discounts
    // persist and accumulate 4 -> 3 -> 2.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/SandsOfTime.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/AbstractCard.java
    #[test]
    fn sands_of_time_source_accumulates_per_copy_retained_cost() {
        let mut engine = one_enemy_engine("JawWorm", 100, 0);
        engine.state.draw_pile = make_deck_n("Strike+", 10);
        engine.state.hand = make_deck(&["SandsOfTime+"]);
        end_turn(&mut engine);
        let first = engine
            .state
            .hand
            .iter()
            .find(|card| engine.card_registry.card_name(card.def_id) == "SandsOfTime+")
            .expect("first retain");
        assert_eq!(first.cost, 3);
        end_turn(&mut engine);
        let second = engine
            .state
            .hand
            .iter()
            .find(|card| engine.card_registry.card_name(card.def_id) == "SandsOfTime+")
            .expect("second retain");
        assert_eq!((second.cost, second.base_cost), (2, 2));
        assert!(play_on_enemy(&mut engine, "SandsOfTime+", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, 74);
    }

    // Source-derived (verify card/SashWhip): HeadStompAction checks only the
    // immediately previous card, applying upgraded Weak 2 after an Attack and
    // none after a Skill.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/SashWhip.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/HeadStompAction.java
    #[test]
    fn sash_whip_source_checks_only_the_immediately_previous_card() {
        let mut attack = one_enemy_engine("JawWorm", 50, 0);
        attack.state.hand = make_deck(&["Strike", "SashWhip+"]);
        assert!(play_on_enemy(&mut attack, "Strike", 0));
        assert!(play_on_enemy(&mut attack, "SashWhip+", 0));
        assert_eq!(attack.state.enemies[0].entity.status(sid::WEAKENED), 2);

        let mut skill = one_enemy_engine("JawWorm", 50, 0);
        skill.state.hand = make_deck(&["Defend", "SashWhip+"]);
        assert!(play_self(&mut skill, "Defend"));
        assert!(play_on_enemy(&mut skill, "SashWhip+", 0));
        assert_eq!(skill.state.enemies[0].entity.status(sid::WEAKENED), 0);
    }
    watcher_test!(
        signature_move_java_parity,
        base = ("SignatureMove", "Signature Move", 2, 30, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["only_attack_in_hand"]),
        plus = ("SignatureMove+", "Signature Move+", 2, 40, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, ["only_attack_in_hand"]),
        {}
    );
    // Source-derived (verify card/SignatureMove): every other Attack in hand
    // blocks canUse, including a second Signature Move; Skills do not block it.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/SignatureMove.java
    #[test]
    fn signature_move_source_rejects_another_copy_but_allows_skills() {
        let mut blocked = one_enemy_engine("JawWorm", 100, 0);
        blocked.state.hand = make_deck(&["SignatureMove", "SignatureMove+"]);
        assert!(!blocked.get_legal_actions().iter().any(|action| {
            matches!(action, Action::PlayCard { card_idx: 0 | 1, .. })
        }));

        let mut allowed = one_enemy_engine("JawWorm", 100, 0);
        allowed.state.hand = make_deck(&["Defend", "SignatureMove+"]);
        assert!(play_on_enemy(&mut allowed, "SignatureMove+", 0));
        assert_eq!(allowed.state.enemies[0].entity.hp, 60);
    }
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

    // Source-derived (verify card/ClearTheMind): Tranquility.java (ID
    // ClearTheMind) is self-retaining, enters Calm, and exhausts when played;
    // upgrade() changes only cost 1 -> 0.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Tranquility.java
    #[test]
    fn tranquility_source_retains_unplayed_and_plus_enters_calm_for_zero() {
        let mut retained = one_enemy_engine("JawWorm", 50, 0);
        retained.state.hand = make_deck(&["ClearTheMind"]);
        retained.state.draw_pile.clear();
        retained.state.discard_pile.clear();
        end_turn(&mut retained);
        assert_eq!(hand_count(&retained, "ClearTheMind"), 1);

        let mut played = one_enemy_engine("JawWorm", 50, 0);
        set_stance(&mut played, Stance::Wrath);
        ensure_in_hand(&mut played, "ClearTheMind+");
        let energy_before = played.state.energy;
        play_self(&mut played, "ClearTheMind+");
        assert_eq!(played.state.stance, Stance::Calm);
        assert_eq!(played.state.energy, energy_before);
        assert_eq!(exhaust_prefix_count(&played, "ClearTheMind+"), 1);

        let base = reg().get("ClearTheMind").expect("ClearTheMind registered");
        let plus = reg().get("ClearTheMind+").expect("ClearTheMind+ registered");
        assert_eq!((base.cost, plus.cost), (1, 0));
        assert!(base.runtime_traits().retain);
        assert!(plus.runtime_traits().retain);
    }
    watcher_test!(
        wish_java_parity,
        base = ("Wish", "Wish", 3, 3, 6, 25, CardType::Skill, CardTarget::None, true, None, ["wish"]),
        plus = ("Wish+", "Wish+", 3, 4, 8, 30, CardType::Skill, CardTarget::None, true, None, ["wish"]),
        {}
    );
    // Source-derived (verify card/Wish): Become Almighty, Fame and Fortune,
    // and Live Forever consume Wish's damage/magic/block fields. Wish's empty
    // applyPowers means existing Strength does not inflate the Strength option.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wish.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/optionCards/BecomeAlmighty.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/optionCards/FameAndFortune.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/optionCards/LiveForever.java
    #[test]
    fn wish_source_upgraded_options_use_fixed_four_thirty_and_eight() {
        let mut strength = one_enemy_engine("JawWorm", 50, 0);
        strength.state.hand = make_deck(&["Wish+"]);
        strength.state.player.set_status(sid::STRENGTH, 10);
        assert!(play_self(&mut strength, "Wish+"));
        strength.execute_action(&Action::Choose(0));
        assert_eq!(strength.state.player.status(sid::STRENGTH), 14);

        let mut gold = one_enemy_engine("JawWorm", 50, 0);
        gold.state.hand = make_deck(&["Wish+"]);
        assert!(play_self(&mut gold, "Wish+"));
        gold.execute_action(&Action::Choose(1));
        assert_eq!(gold.state.pending_run_gold, 30);

        let mut armor = one_enemy_engine("JawWorm", 50, 0);
        armor.state.hand = make_deck(&["Wish+"]);
        assert!(play_self(&mut armor, "Wish+"));
        armor.execute_action(&Action::Choose(2));
        assert_eq!(armor.state.player.status(sid::PLATED_ARMOR), 8);
    }
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
    // Source-derived (verify card/Worship): the upgrade sets selfRetain and
    // changes no numeric field. An unplayed Worship+ survives end turn, then
    // grants exactly five Mantra when used.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Worship.java
    #[test]
    fn worship_source_upgrade_only_adds_self_retain() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.hand = make_deck(&["Worship+"]);
        end_turn(&mut engine);
        assert_eq!(hand_count(&engine, "Worship+"), 1);
        assert_eq!(engine.state.mantra, 0);
        assert!(play_self(&mut engine, "Worship+"));
        assert_eq!(engine.state.mantra, 5);
    }

    // Missing Java cards or unsupported parity gaps.
    watcher_test!(
        collect_java_parity,
        base = ("Collect", "Collect", -1, -1, -1, -1, CardType::Skill, CardTarget::SelfTarget, true, None, []),
        plus = ("Collect+", "Collect+", -1, -1, -1, -1, CardType::Skill, CardTarget::SelfTarget, true, None, []),
        {}
    );

    // Source-derived (verify card/Collect): CollectAction.java applies a
    // stackable X-turn CollectPower (+1 upgraded). CollectPower.java creates
    // exactly one upgraded Miracle per energy recharge and decrements by one.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/CollectAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/CollectPower.java
    #[test]
    fn collect_source_stacks_and_generates_one_upgraded_miracle_per_turn() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.energy = 2;
        ensure_in_hand(&mut engine, "Collect");
        play_self(&mut engine, "Collect");
        assert_eq!(engine.state.player.status(sid::COLLECT_MIRACLES), 2);
        assert_eq!(engine.state.energy, 0);

        engine.state.energy = 1;
        engine
            .state
            .hand
            .push(engine.card_registry.make_card("Collect+").set_free(true));
        play_self(&mut engine, "Collect+");
        assert_eq!(engine.state.player.status(sid::COLLECT_MIRACLES), 4);
        assert_eq!(engine.state.energy, 1);

        end_turn(&mut engine);
        assert_eq!(hand_count(&engine, "Miracle+"), 1);
        assert_eq!(hand_count(&engine, "Miracle"), 0);
        assert_eq!(engine.state.player.status(sid::COLLECT_MIRACLES), 3);

        end_turn(&mut engine);
        assert_eq!(engine.state.player.status(sid::COLLECT_MIRACLES), 2);

        let base = reg().get("Collect").expect("Collect registered");
        let plus = reg().get("Collect+").expect("Collect+ registered");
        assert_eq!((base.cost, base.exhaust), (-1, true));
        assert_eq!((plus.cost, plus.exhaust), (-1, true));
    }

    // Source-derived (verify card/Collect): CollectPower.stackPower caps the
    // turn counter at 999.
    #[test]
    fn collect_source_caps_stacked_turns_at_999() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.player.set_status(sid::COLLECT_MIRACLES, 998);
        engine.state.energy = 3;
        ensure_in_hand(&mut engine, "Collect+");
        play_self(&mut engine, "Collect+");
        assert_eq!(engine.state.player.status(sid::COLLECT_MIRACLES), 999);
    }

    // Source-derived (verify card/ConjureBlade): ConjureBladeAction.java stamps
    // Expunger with X hits, adds Chemical X's +2, and the upgrade adds one more.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/ConjureBladeAction.java
    #[test]
    fn conjure_blade_source_stamps_x_chemical_x_and_upgrade_hits() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.energy = 2;
        engine.state.relics.push("Chemical X".to_string());
        ensure_in_hand(&mut engine, "ConjureBlade+");
        play_self(&mut engine, "ConjureBlade+");

        let expunger = engine
            .state
            .draw_pile
            .iter()
            .find(|card| engine.card_registry.card_name(card.def_id) == "Expunger")
            .expect("Conjure Blade+ should create Expunger");
        assert_eq!(expunger.misc, 5); // X=2 + Chemical X=2 + upgrade=1
        assert_eq!(engine.state.energy, 0);
        assert_eq!(exhaust_prefix_count(&engine, "ConjureBlade+"), 1);

        let base = reg().get("ConjureBlade").expect("ConjureBlade registered");
        let plus = reg().get("ConjureBlade+").expect("ConjureBlade+ registered");
        assert_eq!((base.cost, base.exhaust), (-1, true));
        assert_eq!((plus.cost, plus.exhaust), (-1, true));
    }
    watcher_test!(
        deus_ex_machina_java_parity,
        base = ("DeusExMachina", "Deus Ex Machina", -2, -1, -1, 2, CardType::Skill, CardTarget::SelfTarget, true, None, ["unplayable", "deus_ex_machina"]),
        plus = ("DeusExMachina+", "Deus Ex Machina+", -2, -1, -1, 3, CardType::Skill, CardTarget::SelfTarget, true, None, ["unplayable", "deus_ex_machina"]),
        {}
    );
    // Source-derived (verify card/DeusExMachina): triggerWhenDrawn queues
    // exhaustion before MakeTempCardInHandAction(magicNumber). The action sends
    // generated cards beyond the ten-card hand limit to the discard pile.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/DeusExMachina.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInHandAction.java
    #[test]
    fn deus_ex_machina_plus_source_exhausts_then_spills_miracle_overflow() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.hand = make_deck(&["Strike"; 9]);
        engine.state.draw_pile = make_deck(&["DeusExMachina+"]);
        engine.draw_cards(1);

        assert_eq!(engine.state.hand.len(), 10);
        assert_eq!(hand_count(&engine, "Miracle"), 1);
        assert_eq!(discard_prefix_count(&engine, "Miracle"), 2);
        assert_eq!(exhaust_prefix_count(&engine, "DeusExMachina+"), 1);
        assert_eq!(hand_count(&engine, "DeusExMachina+"), 0);
    }
    watcher_test!(
        foresight_java_parity,
        base = ("Wireheading", "Foresight", 1, -1, -1, 3, CardType::Power, CardTarget::None, false, None, []),
        plus = ("Wireheading+", "Foresight+", 1, -1, -1, 4, CardType::Power, CardTarget::None, false, None, []),
        {}
    );
    // Source-derived (verify card/Wireheading): ForesightPower.atStartOfTurn
    // queues ScryAction after the normal start-turn DrawCardAction. With nine
    // cards available, five are drawn before Foresight+ exposes the remaining four.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Foresight.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/ForesightPower.java
    #[test]
    fn foresight_source_scries_after_the_normal_turn_draw() {
        let mut engine = one_enemy_engine("JawWorm", 50, 0);
        engine.state.hand = make_deck(&["Wireheading+"]);
        engine.state.draw_pile = make_deck_n("Strike", 9);
        assert!(play_self(&mut engine, "Wireheading+"));
        end_turn(&mut engine);
        assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
        assert_eq!(engine.state.hand.len(), 5);
        assert_eq!(engine.choice.as_ref().expect("Foresight Scry").options.len(), 4);
    }
    watcher_test!(
        simmering_fury_java_parity,
        base = ("Vengeance", "Simmering Fury", 1, -1, -1, 2, CardType::Skill, CardTarget::None, false, None, []),
        plus = ("Vengeance+", "Simmering Fury+", 1, -1, -1, 3, CardType::Skill, CardTarget::None, false, None, []),
        {}
    );
    // Source-derived (verify card/Vengeance): WrathNextTurnPower is a
    // non-stacking one-shot atStartOfTurn power; Draw Card stacks 2 + 3 and
    // draws after the normal five-card draw before removing itself.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/SimmeringFury.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/WrathNextTurnPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/DrawCardNextTurnPower.java
    #[test]
    fn simmering_fury_source_separates_wrath_and_stacking_post_draw() {
        let mut engine = one_enemy_engine("JawWorm", 100, 0);
        engine.state.hand = make_deck(&["Vengeance", "Vengeance+"]);
        engine.state.draw_pile = make_deck_n("Strike+", 20);
        assert!(play_self(&mut engine, "Vengeance"));
        assert!(play_self(&mut engine, "Vengeance+"));
        assert_eq!(engine.state.player.status(sid::WRATH_NEXT_TURN), 1);
        assert_eq!(engine.state.player.status(sid::SIMMERING_FURY), 5);

        end_turn(&mut engine);
        assert_eq!(engine.state.stance, Stance::Wrath);
        assert_eq!(engine.state.hand.len(), 10);
        assert_eq!(engine.state.player.status(sid::WRATH_NEXT_TURN), 0);
        assert_eq!(engine.state.player.status(sid::SIMMERING_FURY), 0);
    }
}
