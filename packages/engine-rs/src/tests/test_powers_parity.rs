#[cfg(test)]
mod power_java_parity_tests {
    //! Java references:
    //! - /tmp/sts-decompiled/com/megacrit/cardcrawl/powers/*.java
    //! - /tmp/sts-decompiled/com/megacrit/cardcrawl/powers/watcher/*.java
    //! - /tmp/sts-decompiled/com/megacrit/cardcrawl/powers/{StrengthPower,WeakPower,VulnerablePower,FrailPower,ArtifactPower,IntangiblePower,ThornsPower,PoisonPower,RegenerationPower,BufferPower,BarricadePower,MetallicizePower,PlatedArmorPower,RitualPower,AngryPower,EnragePower,CuriosityPower,ModeShiftPower,SplitPower,FadingPower,InvinciblePower,BackAttackPower,ExplosivePower,UnawakenedPower,ResurrectPower,SlowPower,TimeWarpPower,SporeCloudPower,ThieveryPower,DemonFormPower,FlameBarrierPower,BrutalityPower,DarkEmbracePower,DoubleTapPower,EvolvePower,FeelNoPainPower,FireBreathingPower,JuggernautPower,RupturePower,BerserkPower,CombustPower,CorruptionPower,DoubleDamagePower,RagePower,NoxiousFumesPower,EnvenomPower,AfterImagePower,AccuracyPower,ThousandCutsPower,InfiniteBladesPower,ToolsOfTheTradePower,NightmarePower,PhantasmalKillerPower,SadisticNaturePower,FocusPower,LockOnPower,CreativeAIPower,StormPower,HeatsinkPower,StaticDischargePower,ElectroPower,LoopPower,HelloWorldPower,EquilibriumPower,RushdownPower,MentalFortressPower,BattleHymnPower,DevotionPower,EstablishmentPower,ForesightPower,LikeWaterPower,NirvanaPower,OmegaPower,StudyPower,WaveOfTheHandPower,VigorPower,MantraPower,BlockReturnPower,DevaPower,LiveForeverPower,WrathNextTurnPower,EndTurnDeathPower,FreeAttackPower,MasterRealityPower,NoSkillsPower,EnergyDownPower,CannotChangeStancePower,MarkPower,VaultPower,OmnisciencePower,BlurPower,ConservePower,DrawCardNextTurnPower,DrawPowerPower,DoubleDamagePower,EnergizedPower,NextTurnBlockPower,PenNibPower,ReboundPower,NoBlockPower,NoDrawPower,EntangledPower,ConfusionPower,PanachePower,BurstPower,WraithFormPower,BeatOfDeathPower,GrowthPower,MagnetismPower,SkillBurnPower,ForcefieldPower,RegrowPower,StasisPower,TheBombPower,GenericStrengthUpPower,LoseStrengthPower,LoseDexterityPower,CollectPower,WinterPower,RepairPower}.java

    use crate::cards::CardRegistry;
    use crate::engine::CombatEngine;
    use crate::orbs::OrbType;
    use crate::powers::*;
    use crate::state::{EntityState, Stance};
    use crate::tests::support::*;

    fn entity() -> EntityState {
        EntityState::new(50, 50)
    }

    fn deck(cards: &[&str]) -> Vec<String> {
        cards.iter().map(|c| c.to_string()).collect()
    }

    fn make_engine(cards: &[&str], enemy_hp: i32, enemy_dmg: i32) -> CombatEngine {
        engine_with(deck(cards), enemy_hp, enemy_dmg)
    }

    fn make_two_enemy_engine(cards: &[&str], hp_a: i32, hp_b: i32, dmg: i32) -> CombatEngine {
        let enemies = vec![
            enemy("A", hp_a, hp_a, 1, dmg, 1),
            enemy("B", hp_b, hp_b, 1, dmg, 1),
        ];
        engine_with_enemies(deck(cards), enemies, 3)
    }

    macro_rules! assert_power_def {
        ($name:ident, $key:expr, $($field:ident => $value:expr),+ $(,)?) => {
            #[test]
            fn $name() {
                let def = get_power_def($key).unwrap();
                assert_eq!(def.id, $key);
                $(assert_eq!(def.$field, $value);)+
            }
        };
    }

    #[test]
    fn power_id_keys_match_java() {
        assert_eq!(PowerId::Strength.key(), "Strength");
        assert_eq!(PowerId::Weakened.key(), "Weakened");
        assert_eq!(PowerId::Vulnerable.key(), "Vulnerable");
        assert_eq!(PowerId::Focus.key(), "Focus");
        assert_eq!(PowerId::DoubleDamage.key(), "DoubleDamage");
        assert_eq!(PowerId::Rushdown.key(), "Rushdown");
        assert_eq!(PowerId::MentalFortress.key(), "MentalFortress");
        assert_eq!(PowerId::DevaForm.key(), "DevaForm");
        assert_eq!(PowerId::Omega.key(), "OmegaPower");
        assert_eq!(PowerId::Vigor.key(), "Vigor");
    }

    #[test]
    fn power_id_keys_cover_status_and_boss_powers() {
        let cases = [
            (PowerId::Dexterity, "Dexterity"),
            (PowerId::Frail, "Frail"),
            (PowerId::Artifact, "Artifact"),
            (PowerId::Barricade, "Barricade"),
            (PowerId::Metallicize, "Metallicize"),
            (PowerId::PlatedArmor, "Plated Armor"),
            (PowerId::Ritual, "Ritual"),
            (PowerId::Curiosity, "Curiosity"),
            (PowerId::ModeShift, "Mode Shift"),
            (PowerId::Invincible, "Invincible"),
            (PowerId::Split, "Split"),
            (PowerId::TimeWarp, "Time Warp"),
            (PowerId::SporeCloud, "Spore Cloud"),
            (PowerId::DemonForm, "Demon Form"),
            (PowerId::FeelNoPain, "Feel No Pain"),
            (PowerId::FireBreathing, "Fire Breathing"),
            (PowerId::Rage, "Rage"),
            (PowerId::NoxiousFumes, "Noxious Fumes"),
            (PowerId::AfterImage, "After Image"),
            (PowerId::Focus, "Focus"),
            (PowerId::Loop, "Loop"),
            (PowerId::BattleHymn, "BattleHymn"),
            (PowerId::Establishment, "Establishment"),
            (PowerId::LikeWater, "Like Water"),
            (PowerId::Nirvana, "Nirvana"),
            (PowerId::Omega, "OmegaPower"),
            (PowerId::WaveOfTheHand, "Wave of the Hand"),
            (PowerId::DevaForm, "DevaForm"),
            (PowerId::EndTurnDeath, "EndTurnDeath"),
            (PowerId::EnergyDown, "EnergyDown"),
            (PowerId::CannotChangeStance, "CannotChangeStance"),
            (PowerId::Blur, "Blur"),
            (PowerId::DoubleDamage, "DoubleDamage"),
            (PowerId::WraithForm, "Wraith Form"),
            (PowerId::BeatOfDeath, "Beat of Death"),
            (PowerId::Growth, "Growth"),
            (PowerId::Forcefield, "Forcefield"),
            (PowerId::TheBomb, "TheBomb"),
        ];

        for (id, key) in cases {
            assert_eq!(id.key(), key);
        }
    }

    assert_power_def!(strength_def_matches_java, "Strength", power_type => PowerType::Buff, can_go_negative => true, modify_damage_give => true);
    assert_power_def!(dexterity_def_matches_java, "Dexterity", power_type => PowerType::Buff, can_go_negative => true, modify_block => true);
    assert_power_def!(weakened_def_matches_java, "Weakened", power_type => PowerType::Debuff, is_turn_based => true, on_end_of_round => true, modify_damage_give => true);
    assert_power_def!(vulnerable_def_matches_java, "Vulnerable", power_type => PowerType::Debuff, is_turn_based => true, on_end_of_round => true, modify_damage_receive => true);
    assert_power_def!(frail_def_matches_java, "Frail", power_type => PowerType::Debuff, is_turn_based => true, on_end_of_round => true, modify_block => true);
    assert_power_def!(artifact_def_matches_java, "Artifact", power_type => PowerType::Buff, on_apply_power => true);
    assert_power_def!(intangible_def_matches_java, "Intangible", power_type => PowerType::Buff, is_turn_based => true, on_turn_end => true, modify_damage_receive => true);
    assert_power_def!(barricade_def_matches_java, "Barricade", power_type => PowerType::Buff, stackable => false);
    assert_power_def!(metallicize_def_matches_java, "Metallicize", power_type => PowerType::Buff, on_turn_end => true);
    assert_power_def!(plated_armor_def_matches_java, "Plated Armor", power_type => PowerType::Buff, on_turn_end => true, on_attacked => true);
    assert_power_def!(ritual_def_matches_java, "Ritual", power_type => PowerType::Buff, on_turn_end => true);
    assert_power_def!(double_damage_def_matches_java, "DoubleDamage", power_type => PowerType::Buff, modify_damage_give => true);
    assert_power_def!(rage_def_matches_java, "Rage", power_type => PowerType::Buff, on_use_card => true, on_turn_end => true);
    assert_power_def!(noxious_fumes_def_matches_java, "Noxious Fumes", power_type => PowerType::Buff, on_turn_start_post_draw => true);
    assert_power_def!(after_image_def_matches_java, "After Image", power_type => PowerType::Buff, on_use_card => true);
    assert_power_def!(accuracy_def_matches_java, "Accuracy", power_type => PowerType::Buff, modify_damage_give => true);
    assert_power_def!(phantasmal_killer_def_matches_java, "Phantasmal Killer", power_type => PowerType::Buff, on_turn_start_post_draw => true, modify_damage_give => true);
    assert_power_def!(focus_def_matches_java, "Focus", power_type => PowerType::Buff, can_go_negative => true);
    assert_power_def!(lock_on_def_matches_java, "Lock-On", power_type => PowerType::Debuff, is_turn_based => true, on_end_of_round => true);
    assert_power_def!(storm_def_matches_java, "Storm", power_type => PowerType::Buff, on_use_card => true);
    assert_power_def!(heatsink_def_matches_java, "Heatsink", power_type => PowerType::Buff, on_use_card => true);
    assert_power_def!(static_discharge_def_matches_java, "Static Discharge", power_type => PowerType::Buff, on_hp_lost => true);
    assert_power_def!(loop_def_matches_java, "Loop", power_type => PowerType::Buff, on_turn_start => true);
    assert_power_def!(equilibrium_def_matches_java, "Equilibrium", power_type => PowerType::Buff, on_turn_end => true);
    assert_power_def!(rushdown_def_matches_java, "Rushdown", power_type => PowerType::Buff, on_change_stance => true);
    assert_power_def!(mental_fortress_def_matches_java, "MentalFortress", power_type => PowerType::Buff, on_change_stance => true);
    assert_power_def!(battle_hymn_def_matches_java, "BattleHymn", power_type => PowerType::Buff, on_turn_start => true);
    assert_power_def!(devotion_def_matches_java, "Devotion", power_type => PowerType::Buff, on_turn_start_post_draw => true);
    assert_power_def!(establishment_def_matches_java, "Establishment", power_type => PowerType::Buff, on_turn_end => true);
    assert_power_def!(like_water_def_matches_java, "Like Water", power_type => PowerType::Buff, on_turn_end => true);
    assert_power_def!(nirvana_def_matches_java, "Nirvana", power_type => PowerType::Buff, on_scry => true);
    assert_power_def!(omega_def_matches_java, "OmegaPower", power_type => PowerType::Buff, on_turn_end => true);
    assert_power_def!(study_def_matches_java, "Study", power_type => PowerType::Buff, on_turn_end => true);
    assert_power_def!(wave_of_the_hand_def_matches_java, "Wave of the Hand", power_type => PowerType::Buff, on_gained_block => true);
    assert_power_def!(vigor_def_matches_java, "Vigor", power_type => PowerType::Buff);
    assert_power_def!(deva_form_def_matches_java, "DevaForm", power_type => PowerType::Buff, on_energy_recharge => true);
    assert_power_def!(live_forever_def_matches_java, "LiveForever", power_type => PowerType::Buff, on_turn_end => true);
    assert_power_def!(wrath_next_turn_def_matches_java, "WrathNextTurn", power_type => PowerType::Buff, on_turn_start => true);
    assert_power_def!(end_turn_death_def_matches_java, "EndTurnDeath", power_type => PowerType::Debuff);
    assert_power_def!(free_attack_def_matches_java, "FreeAttackPower", power_type => PowerType::Buff);
    assert_power_def!(energy_down_def_matches_java, "EnergyDown", power_type => PowerType::Debuff);
    assert_power_def!(cannot_change_stance_def_matches_java, "CannotChangeStance", power_type => PowerType::Debuff);
    assert_power_def!(mark_def_matches_java, "Mark", power_type => PowerType::Debuff);
    assert_power_def!(blur_def_matches_java, "Blur", power_type => PowerType::Buff, on_end_of_round => true);
    assert_power_def!(draw_card_next_turn_def_matches_java, "Draw Card", power_type => PowerType::Buff);
    assert_power_def!(draw_power_def_matches_java, "Draw", power_type => PowerType::Buff);
    assert_power_def!(energized_def_matches_java, "Energized", power_type => PowerType::Buff);
    assert_power_def!(next_turn_block_def_matches_java, "Next Turn Block", power_type => PowerType::Buff);
    assert_power_def!(no_block_def_matches_java, "No Block", power_type => PowerType::Debuff, on_end_of_round => true);
    assert_power_def!(no_draw_def_matches_java, "No Draw", power_type => PowerType::Debuff);
    assert_power_def!(confusion_def_matches_java, "Confusion", power_type => PowerType::Debuff);
    assert_power_def!(panache_def_matches_java, "Panache", power_type => PowerType::Buff);
    assert_power_def!(burst_def_matches_java, "Burst", power_type => PowerType::Buff);
    assert_power_def!(wraith_form_def_matches_java, "Wraith Form", power_type => PowerType::Debuff, can_go_negative => true, on_turn_end => true);
    assert_power_def!(beat_of_death_def_matches_java, "Beat of Death", power_type => PowerType::Buff, on_after_use_card => true);
    assert_power_def!(growth_def_matches_java, "Growth", power_type => PowerType::Buff, on_end_of_round => true);
    assert_power_def!(magnetism_def_matches_java, "Magnetism", power_type => PowerType::Buff, on_turn_start_post_draw => true);
    assert_power_def!(skill_burn_def_matches_java, "SkillBurn", power_type => PowerType::Buff, on_after_use_card => true);
    assert_power_def!(forcefield_def_matches_java, "Forcefield", power_type => PowerType::Buff, on_after_use_card => true);
    assert_power_def!(regrow_def_matches_java, "Regrow", power_type => PowerType::Buff, on_turn_end => true);
    assert_power_def!(stasis_def_matches_java, "Stasis", power_type => PowerType::Buff, stackable => false);
    assert_power_def!(the_bomb_def_matches_java, "TheBomb", power_type => PowerType::Buff, is_turn_based => true, on_end_of_round => true);
    assert_power_def!(generic_strength_up_def_matches_java, "GenericStrengthUp", power_type => PowerType::Buff, on_turn_end => true);
    assert_power_def!(lose_strength_def_matches_java, "LoseStrength", power_type => PowerType::Debuff, on_turn_start => true);
    assert_power_def!(lose_dexterity_def_matches_java, "LoseDexterity", power_type => PowerType::Debuff, on_turn_start => true);
    assert_power_def!(collect_def_matches_java, "Collect", power_type => PowerType::Buff, is_turn_based => true, on_turn_start_post_draw => true);
    assert_power_def!(winter_def_matches_java, "Winter", power_type => PowerType::Buff, on_turn_end => true);
    assert_power_def!(repair_def_matches_java, "Repair", power_type => PowerType::Buff);

    #[test]
    fn watcher_power_card_values_match_java() {
        let reg = CardRegistry::new();

        let battle_hymn = reg.get("BattleHymn").unwrap();
        assert_eq!(battle_hymn.cost, 1);
        assert_eq!(battle_hymn.base_magic, 1);
        assert!(battle_hymn.effects.contains(&"battle_hymn"));
        let battle_hymn_plus = reg.get("BattleHymn+").unwrap();
        assert!(battle_hymn_plus.effects.contains(&"innate"));

        let like_water = reg.get("LikeWater").unwrap();
        assert_eq!(like_water.base_magic, 5);
        assert_eq!(reg.get("LikeWater+").unwrap().base_magic, 7);

        let nirvana = reg.get("Nirvana").unwrap();
        assert_eq!(nirvana.base_magic, 3);
        assert_eq!(reg.get("Nirvana+").unwrap().base_magic, 4);

        let study = reg.get("Study").unwrap();
        assert_eq!(study.cost, 2);
        assert_eq!(reg.get("Study+").unwrap().cost, 1);

        let rushdown = reg.get("Adaptation").unwrap();
        assert_eq!(rushdown.base_magic, 2);
        assert_eq!(reg.get("Adaptation+").unwrap().cost, 0);

        let mental_fortress = reg.get("MentalFortress").unwrap();
        assert_eq!(mental_fortress.base_magic, 4);
        assert_eq!(reg.get("MentalFortress+").unwrap().base_magic, 6);

        let deva_form = reg.get("DevaForm").unwrap();
        assert_eq!(deva_form.cost, 3);
        assert!(deva_form.effects.contains(&"ethereal"));
        assert!(!reg.get("DevaForm+").unwrap().effects.contains(&"ethereal"));

        let devotion = reg.get("Devotion").unwrap();
        assert_eq!(devotion.base_magic, 2);
        assert_eq!(reg.get("Devotion+").unwrap().base_magic, 3);

        let establishment = reg.get("Establishment").unwrap();
        assert_eq!(establishment.base_magic, 1);
        assert!(reg.get("Establishment+").unwrap().effects.contains(&"innate"));

        let wave = reg.get("WaveOfTheHand").unwrap();
        assert_eq!(wave.base_magic, 1);
        assert_eq!(reg.get("WaveOfTheHand+").unwrap().base_magic, 2);

        let wreath = reg.get("WreathOfFlame").unwrap();
        assert_eq!(wreath.base_magic, 5);
        assert_eq!(reg.get("WreathOfFlame+").unwrap().base_magic, 8);

        let after_image = reg.get("After Image").unwrap();
        assert_eq!(after_image.base_magic, 1);
        assert_eq!(reg.get("After Image+").unwrap().cost, 0);

        let rage = reg.get("Rage").unwrap();
        assert_eq!(rage.base_magic, 3);
        assert_eq!(reg.get("Rage+").unwrap().base_magic, 5);
    }

    #[test]
    fn power_amount_getters_match_java() {
        let mut e = entity();
        e.set_status("After Image", 2);
        e.set_status("Rage", 3);
        e.set_status("A Thousand Cuts", 4);
        e.set_status("Noxious Fumes", 5);
        e.set_status("BattleHymn", 1);
        e.set_status("Devotion", 2);
        e.set_status("Rushdown", 2);
        e.set_status("MentalFortress", 4);
        e.set_status("Nirvana", 3);
        e.set_status("Study", 2);
        e.set_status("LiveForever", 4);
        e.set_status("Accuracy", 3);
        e.set_status("Mark", 7);
        e.set_status("Dark Embrace", 1);
        e.set_status("Feel No Pain", 5);
        e.set_status("Evolve", 2);
        e.set_status("Fire Breathing", 6);
        e.set_status("Heatsink", 3);
        e.set_status("SkillBurn", 9);
        e.set_status("Envenom", 4);
        e.set_status("Juggernaut", 6);
        e.set_status("Wave of the Hand", 1);

        assert_eq!(get_after_image_block(&e), 2);
        assert_eq!(get_rage_block(&e), 3);
        assert_eq!(get_thousand_cuts_damage(&e), 4);
        assert_eq!(get_noxious_fumes_amount(&e), 5);
        assert_eq!(get_battle_hymn_amount(&e), 1);
        assert_eq!(get_devotion_amount(&e), 2);
        assert_eq!(get_rushdown_draw(&e), 2);
        assert_eq!(get_mental_fortress_block(&e), 4);
        assert_eq!(get_nirvana_block(&e), 3);
        assert_eq!(get_study_insights(&e), 2);
        assert_eq!(get_live_forever_block(&e), 4);
        assert_eq!(get_accuracy_bonus(&e), 3);
        assert_eq!(get_mark(&e), 7);
        assert_eq!(get_dark_embrace_draw(&e), 1);
        assert_eq!(get_feel_no_pain_block(&e), 5);
        assert_eq!(get_evolve_draw(&e), 2);
        assert_eq!(get_fire_breathing_damage(&e), 6);
        assert_eq!(get_heatsink_draw(&e), 3);
        assert_eq!(get_skill_burn_damage(&e), 9);
        assert_eq!(get_envenom_amount(&e), 4);
        assert_eq!(get_juggernaut_damage(&e), 6);
        assert_eq!(get_wave_of_the_hand_weak(&e), 1);
        assert_eq!(get_omega_damage(&e), 0);
    }

    #[test]
    fn direct_power_amount_helpers_return_exact_stacks() {
        let mut e = entity();
        e.set_status("Draw", 3);
        e.set_status("EnergyDown", 2);
        e.set_status("BattleHymn", 1);
        e.set_status("Devotion", 4);
        e.set_status("Infinite Blades", 1);
        e.set_status("Storm", 1);
        e.set_status("Heatsink", 2);
        e.set_status("Static Discharge", 3);
        e.set_status("Focus", 5);
        e.set_status("Lock-On", 1);
        e.set_status("Magnetism", 6);
        e.set_status("Regrow", 7);
        e.set_status("Spore Cloud", 2);
        e.set_status("Beat of Death", 9);
        e.set_status("Forcefield", 4);
        e.set_status("SkillBurn", 8);
        e.set_status("Noxious Fumes", 3);
        e.set_status("Envenom", 2);
        e.set_status("LiveForever", 6);
        e.set_status("Study", 4);

        assert_eq!(get_extra_draw(&e), 3);
        assert_eq!(get_energy_down(&e), 2);
        assert_eq!(get_battle_hymn_amount(&e), 1);
        assert_eq!(get_devotion_amount(&e), 4);
        assert_eq!(get_infinite_blades(&e), 1);
        assert!(should_storm_channel(&e));
        assert_eq!(get_heatsink_draw(&e), 2);
        assert_eq!(get_static_discharge(&e), 3);
        assert_eq!(get_beat_of_death_damage(&e), 9);
        assert_eq!(get_regrow_heal(&e), 7);
        assert_eq!(get_spore_cloud_vulnerable(&e), 2);
        assert_eq!(get_skill_burn_damage(&e), 8);
        assert_eq!(get_noxious_fumes_amount(&e), 3);
        assert_eq!(get_envenom_amount(&e), 2);
        assert_eq!(get_live_forever_block(&e), 6);
        assert_eq!(get_study_insights(&e), 4);
        assert_eq!(get_mark(&e), 0);
        assert_eq!(get_mental_fortress_block(&e), 0);
        assert_eq!(get_nirvana_block(&e), 0);
        assert_eq!(get_like_water_block(&e), 0);
        assert_eq!(get_wave_of_the_hand_weak(&e), 0);
    }

    #[test]
    fn on_scry_and_stance_helpers_match_java_amounts() {
        let mut e = entity();
        e.set_status("Nirvana", 3);
        e.set_status("Rushdown", 2);
        e.set_status("MentalFortress", 4);
        e.set_status("Like Water", 5);
        e.set_status("Wave of the Hand", 1);
        e.set_status("Establishment", 1);
        e.set_status("Vigor", 6);
        e.set_status("OmegaPower", 18);

        assert_eq!(get_nirvana_block(&e), 3);
        assert_eq!(get_rushdown_draw(&e), 2);
        assert_eq!(get_mental_fortress_block(&e), 4);
        assert_eq!(get_like_water_block(&e), 5);
        assert_eq!(get_wave_of_the_hand_weak(&e), 1);
        assert_eq!(get_omega_damage(&e), 18);
    }

    #[test]
    fn start_of_turn_dispatch_matches_java() {
        let mut e = entity();
        e.set_status("Strength", 5);
        e.set_status("Dexterity", 4);
        e.set_status("LoseStrength", 2);
        e.set_status("LoseDexterity", 1);
        e.set_status("Wraith Form", 2);
        e.set_status("Demon Form", 3);
        e.set_status("Berserk", 4);
        e.set_status("Noxious Fumes", 2);
        e.set_status("Brutality", 1);
        e.set_status("Draw Card", 3);
        e.set_status("Next Turn Block", 7);
        e.set_status("Energized", 2);
        e.set_status("EnergyDown", 1);
        e.set_status("WrathNextTurn", 1);
        e.set_status("BattleHymn", 2);
        e.set_status("Devotion", 3);
        e.set_status("Infinite Blades", 1);
        e.set_status("Draw", 4);
        e.set_status("DevaForm", 1);
        e.set_status("DevaFormEnergy", 0);
        e.set_status("Flame Barrier", 2);

        let result = process_start_of_turn(&mut e);

        assert_eq!(result.extra_energy, 2);
        assert_eq!(result.extra_draw, 4);
        assert_eq!(result.noxious_fumes_poison, 2);
        assert!(result.demon_form_strength);
        assert_eq!(result.brutality_draw, 1);
        assert_eq!(result.block_from_next_turn, 7);
        assert!(result.enter_wrath);
        assert_eq!(result.battle_hymn_smites, 2);
        assert_eq!(result.devotion_mantra, 3);
        assert!(result.infinite_blades);
        assert_eq!(result.draw_card_next_turn, 3);
        assert!(result.wraith_form_dex_loss);
        assert_eq!(result.berserk_energy, 4);
        assert_eq!(e.strength(), 6);
        assert_eq!(e.dexterity(), 1);
        assert_eq!(e.status("LoseStrength"), 0);
        assert_eq!(e.status("LoseDexterity"), 0);
        assert_eq!(e.status("Flame Barrier"), 0);
        assert_eq!(e.status("Draw Card"), 0);
        assert_eq!(e.status("Next Turn Block"), 0);
        assert_eq!(e.status("Energized"), 0);
        assert_eq!(e.status("EnergyDown"), 1);
        assert_eq!(e.status("WrathNextTurn"), 0);
        assert_eq!(e.status("DevaFormEnergy"), 1);
    }

    #[test]
    fn end_of_turn_dispatch_matches_java() {
        let mut e = entity();
        e.set_status("Metallicize", 4);
        e.set_status("Plated Armor", 6);
        e.set_status("OmegaPower", 9);
        e.set_status("Like Water", 5);
        e.set_status("Combust", 5);
        e.set_status("Regeneration", 7);
        e.set_status("LiveForever", 4);
        e.set_status("Study", 2);
        e.set_status("Rage", 3);
        e.set_status("Equilibrium", 2);
        e.set_status("Intangible", 1);
        e.set_status("EndTurnDeath", 1);

        let result = process_end_of_turn(&mut e, true);

        assert_eq!(result.metallicize_block, 4);
        assert_eq!(result.plated_armor_block, 6);
        assert_eq!(result.omega_damage, 9);
        assert_eq!(result.like_water_block, 5);
        assert_eq!(result.combust_hp_loss, 1);
        assert_eq!(result.combust_damage, 5);
        assert_eq!(result.regen_heal, 7);
        assert_eq!(result.live_forever_block, 4);
        assert_eq!(result.study_insights, 2);
        assert!(result.should_die);
        assert_eq!(e.status("Rage"), 0);
        assert_eq!(e.status("Equilibrium"), 1);
        assert_eq!(e.status("Intangible"), 0);
        assert_eq!(e.status("Regeneration"), 6);
    }

    #[test]
    fn end_of_round_dispatch_matches_java() {
        let mut e = entity();
        e.set_status("Weakened", 3);
        e.set_status("Vulnerable", 2);
        e.set_status("Frail", 1);
        e.set_status("Blur", 2);
        e.set_status("Lock-On", 4);
        e.set_status("Slow", 5);

        process_end_of_round(&mut e);

        assert_eq!(e.status("Weakened"), 2);
        assert_eq!(e.status("Vulnerable"), 1);
        assert_eq!(e.status("Frail"), 0);
        assert!(!e.statuses.contains_key("Frail"));
        assert_eq!(e.status("Blur"), 1);
        assert_eq!(e.status("Lock-On"), 3);
        assert_eq!(e.status("Slow"), 0);
        assert!(!e.statuses.contains_key("Slow"));
    }

    #[test]
    fn debuff_application_and_restrictions_match_java() {
        let mut e = entity();
        e.set_status("Artifact", 2);
        assert!(!apply_debuff(&mut e, "Weakened", 3));
        assert_eq!(e.status("Artifact"), 1);
        assert_eq!(e.status("Weakened"), 0);

        assert!(!apply_debuff(&mut e, "Vulnerable", 1));
        assert_eq!(e.status("Artifact"), 0);
        assert_eq!(e.status("Vulnerable"), 0);

        assert!(apply_debuff(&mut e, "Vulnerable", 1));
        assert_eq!(e.status("Vulnerable"), 1);

        assert!(!has_no_skills(&e));
        assert!(!has_no_draw(&e));
        assert!(!has_confusion(&e));
        assert!(!cannot_change_stance(&e));

        e.set_status("NoSkillsPower", 1);
        e.set_status("No Draw", 1);
        e.set_status("Confusion", 1);
        e.set_status("CannotChangeStance", 1);
        assert!(has_no_skills(&e));
        assert!(has_no_draw(&e));
        assert!(has_confusion(&e));
        assert!(cannot_change_stance(&e));
    }

    #[test]
    fn defensive_power_helpers_match_java() {
        let mut e = entity();
        e.block = 20;
        e.set_status("Barricade", 1);
        assert!(should_retain_block(&e));
        assert_eq!(apply_block_decay(&e, true), 20);

        e.set_status("Barricade", 0);
        e.set_status("Blur", 1);
        assert!(should_retain_block(&e));
        e.set_status("Blur", 0);
        assert_eq!(apply_block_decay(&e, true), 5);
        assert_eq!(apply_block_decay(&e, false), 0);

        e.set_status("Metallicize", 4);
        apply_metallicize(&mut e);
        assert_eq!(e.block, 24);

        e.set_status("Plated Armor", 6);
        apply_plated_armor(&mut e);
        assert_eq!(e.block, 30);

        e.set_status("Thorns", 3);
        e.set_status("Flame Barrier", 5);
        e.set_status("Buffer", 1);
        assert_eq!(get_thorns_damage(&e), 3);
        assert_eq!(get_flame_barrier_damage(&e), 5);
        assert!(check_buffer(&mut e));
        assert_eq!(e.status("Buffer"), 0);

        e.set_status("Invincible", 12);
        assert_eq!(apply_invincible_cap(&mut e, 20), 12);
        assert_eq!(e.status("Invincible"), 0);

        e.set_status("Mode Shift", 20);
        assert!(!apply_mode_shift_damage(&mut e, 10));
        assert!(apply_mode_shift_damage(&mut e, 10));
    }

    #[test]
    fn turn_scaling_helpers_match_java() {
        let mut e = entity();
        e.set_status("Ritual", 3);
        e.set_status("GenericStrengthUp", 2);
        e.set_status("Regeneration", 7);
        e.set_status("Beat of Death", 2);
        e.set_status("Static Discharge", 3);
        e.set_status("Spore Cloud", 2);
        e.set_status("Slow", 5);
        e.set_status("TimeWarpActive", 1);
        e.set_status("Time Warp", 11);
        e.set_status("Forcefield", 2);

        apply_ritual(&mut e);
        apply_generic_strength_up(&mut e);
        assert_eq!(e.strength(), 5);

        assert_eq!(apply_regeneration(&mut e), 7);
        assert_eq!(e.status("Regeneration"), 6);

        assert_eq!(get_beat_of_death_damage(&e), 2);
        assert_eq!(get_static_discharge(&e), 3);
        assert_eq!(get_spore_cloud_vulnerable(&e), 2);

        increment_slow(&mut e);
        assert_eq!(e.status("Slow"), 6);
        assert!(increment_time_warp(&mut e));
        assert_eq!(e.status("Time Warp"), 0);

        assert!(check_forcefield(&mut e));
        assert_eq!(e.status("Forcefield"), 1);
    }

    #[test]
    fn consume_helpers_match_java() {
        let mut e = entity();
        e.set_status("Draw Card", 2);
        e.set_status("Next Turn Block", 4);
        e.set_status("Energized", 3);
        e.set_status("FreeAttackPower", 1);
        e.set_status("Double Tap", 1);
        e.set_status("Burst", 1);
        e.set_status("Equilibrium", 2);

        assert_eq!(consume_draw_card_next_turn(&mut e), 2);
        assert_eq!(e.status("Draw Card"), 0);
        assert_eq!(consume_next_turn_block(&mut e), 4);
        assert_eq!(e.status("Next Turn Block"), 0);
        assert_eq!(consume_energized(&mut e), 3);
        assert_eq!(e.status("Energized"), 0);
        assert!(consume_free_attack(&mut e));
        assert_eq!(e.status("FreeAttackPower"), 0);
        assert!(consume_double_tap(&mut e));
        assert_eq!(e.status("Double Tap"), 0);
        assert!(consume_burst(&mut e));
        assert_eq!(e.status("Burst"), 0);
        assert!(has_equilibrium(&e));
        decrement_equilibrium(&mut e);
        assert_eq!(e.status("Equilibrium"), 1);
    }

    #[test]
    fn damage_and_heal_modifiers_match_java() {
        let mut e = entity();
        e.set_status("DoubleDamage", 1);
        e.set_status("No Block", 1);
        e.set_status("Slow", 2);
        e.set_status("Intangible", 1);

        assert_eq!(modify_damage_give(&e, 6.0, true), 12.0);
        assert_eq!(modify_block(&e, 14.0), 0.0);
        assert_eq!(modify_damage_receive(&e, 10.0), 1.0);
        assert_eq!(modify_heal(&e, 9), 9);
    }

    #[test]
    fn start_turn_card_powers_fire_on_engine() {
        let mut engine = make_engine(&["Strike_P"; 12], 100, 0);
        ensure_in_hand(&mut engine, "BattleHymn");
        assert!(play_self(&mut engine, "BattleHymn"));
        end_turn(&mut engine);
        assert_eq!(hand_prefix_count(&engine, "Smite"), 1);

        let mut devotion_engine = make_engine(&["Strike_P"; 12], 100, 0);
        ensure_in_hand(&mut devotion_engine, "Devotion");
        assert!(play_self(&mut devotion_engine, "Devotion"));
        end_turn(&mut devotion_engine);
        assert_eq!(devotion_engine.state.mantra, 2);

        let mut deva_engine = make_engine(&["Strike_P"; 12], 100, 0);
        ensure_in_hand(&mut deva_engine, "DevaForm");
        assert!(play_self(&mut deva_engine, "DevaForm"));
        assert_eq!(deva_engine.state.player.status("DevaForm"), 1);
        end_turn(&mut deva_engine);
        assert_eq!(deva_engine.state.energy, 4);
        assert_eq!(deva_engine.state.player.status("DevaForm"), 2);
        end_turn(&mut deva_engine);
        assert_eq!(deva_engine.state.energy, 5);
        assert_eq!(deva_engine.state.player.status("DevaForm"), 3);
    }

    #[test]
    fn devotion_enters_divinity_at_ten() {
        let mut engine = make_engine(&["Strike_P"; 12], 100, 0);
        engine.state.mantra = 8;
        ensure_in_hand(&mut engine, "Devotion");
        assert!(play_self(&mut engine, "Devotion"));
        end_turn(&mut engine);
        assert_eq!(engine.state.stance, Stance::Divinity);
        assert_eq!(engine.state.energy, 6);
        assert_eq!(engine.state.mantra, 0);
    }

    #[test]
    fn watcher_stance_powers_fire_on_engine() {
        let mut engine = make_engine(&["Strike_P"; 12], 100, 0);
        ensure_in_hand(&mut engine, "Adaptation");
        assert!(play_self(&mut engine, "Adaptation"));
        let hand_before = engine.state.hand.len();
        engine.change_stance(Stance::Wrath);
        assert_eq!(engine.state.hand.len(), hand_before + 2);

        ensure_in_hand(&mut engine, "MentalFortress");
        assert!(play_self(&mut engine, "MentalFortress"));
        let block_before = engine.state.player.block;
        engine.change_stance(Stance::Calm);
        assert_eq!(engine.state.player.block, block_before + 4);
    }

    #[test]
    fn end_turn_card_powers_fire_on_engine() {
        let mut engine = make_engine(&["Strike_P"; 12], 100, 0);
        ensure_in_hand(&mut engine, "Study");
        assert!(play_self(&mut engine, "Study"));
        end_turn(&mut engine);
        let insight_count = hand_prefix_count(&engine, "Insight") + draw_prefix_count(&engine, "Insight");
        assert_eq!(insight_count, 1);

        let mut like_water_engine = make_engine(&["Strike_P"; 12], 100, 5);
        ensure_in_hand(&mut like_water_engine, "LikeWater");
        assert!(play_self(&mut like_water_engine, "LikeWater"));
        like_water_engine.state.stance = Stance::Calm;
        let hp_before = like_water_engine.state.player.hp;
        end_turn(&mut like_water_engine);
        assert_eq!(like_water_engine.state.player.hp, hp_before);

        let mut omega_engine = make_two_enemy_engine(&["Omega"; 12], 100, 90, 0);
        ensure_in_hand(&mut omega_engine, "Omega");
        assert!(play_self(&mut omega_engine, "Omega"));
        end_turn(&mut omega_engine);
        assert_eq!(omega_engine.state.enemies[0].entity.hp, 50);
        assert_eq!(omega_engine.state.enemies[1].entity.hp, 40);
    }

    #[test]
    fn scry_and_vigor_match_java_on_engine() {
        let mut engine = make_engine(&["Strike_P"; 12], 100, 0);
        ensure_in_hand(&mut engine, "Nirvana");
        assert!(play_self(&mut engine, "Nirvana"));
        engine.do_scry(2);
        assert_eq!(engine.state.player.block, 3);

        ensure_in_hand(&mut engine, "WreathOfFlame");
        assert!(play_self(&mut engine, "WreathOfFlame"));
        assert_eq!(engine.state.player.status("Vigor"), 5);

        let hp_before = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Strike_P", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 11);
        assert_eq!(engine.state.player.status("Vigor"), 0);
    }

    #[test]
    fn on_card_play_powers_match_java_on_engine() {
        let mut engine = make_engine(&["Strike_P", "Defend_P", "Strike_P", "Defend_P", "Strike_P", "Defend_P", "Strike_P", "Defend_P"], 100, 0);
        ensure_in_hand(&mut engine, "After Image");
        assert!(play_self(&mut engine, "After Image"));
        let block_before = engine.state.player.block;
        assert!(play_on_enemy(&mut engine, "Strike_P", 0));
        assert_eq!(engine.state.player.block, block_before + 1);

        ensure_in_hand(&mut engine, "Rage");
        assert!(play_self(&mut engine, "Rage"));
        let rage_block_before = engine.state.player.block;
        assert!(play_on_enemy(&mut engine, "Strike_P", 0));
        assert_eq!(engine.state.player.block, rage_block_before + 3);

        let hp_before = engine.state.player.hp;
        engine.state.enemies[0].entity.set_status("Beat of Death", 2);
        assert!(play_on_enemy(&mut engine, "Defend_P", 0));
        assert_eq!(engine.state.player.hp, hp_before - 2);
    }

    #[test]
    fn exhaust_powers_match_java_on_engine() {
        let mut engine = make_engine(&["Strike_P"; 12], 100, 0);
        engine.state.player.set_status("Feel No Pain", 2);
        engine.state.player.set_status("Dark Embrace", 1);
        ensure_in_hand(&mut engine, "Miracle");
        let hand_before = engine.state.hand.len();
        assert!(play_self(&mut engine, "Miracle"));
        assert_eq!(engine.state.player.block, 2);
        assert_eq!(engine.state.hand.len(), hand_before);
        assert_eq!(exhaust_prefix_count(&engine, "Miracle"), 1);
    }

    #[test]
    fn wave_of_the_hand_card_sets_runtime_status_amounts() {
        let mut engine = make_engine(&["Strike_P"; 12], 100, 0);
        ensure_in_hand(&mut engine, "WaveOfTheHand");
        assert!(play_self(&mut engine, "WaveOfTheHand"));
        assert_eq!(engine.state.player.status("WaveOfTheHand"), 1);

        ensure_in_hand(&mut engine, "WaveOfTheHand+");
        assert!(play_self(&mut engine, "WaveOfTheHand+"));
        assert_eq!(engine.state.player.status("WaveOfTheHand"), 3);
    }

    #[test]
    fn wave_of_the_hand_helper_reads_installed_amount_like_java_power() {
        let mut engine = make_engine(&["Strike_P"; 12], 100, 0);
        ensure_in_hand(&mut engine, "WaveOfTheHand");
        assert!(play_self(&mut engine, "WaveOfTheHand"));
        assert_eq!(get_wave_of_the_hand_weak(&engine.state.player), 1);
    }

    #[test]
    fn establishment_card_sets_runtime_status_amount() {
        let mut engine = make_engine(&["Strike_P"; 12], 100, 0);
        ensure_in_hand(&mut engine, "Establishment");
        assert!(play_self(&mut engine, "Establishment"));
        assert_eq!(engine.state.player.status("Establishment"), 1);
    }

    #[test]
    fn status_restrictions_and_misc_helpers_match_java() {
        let mut e = entity();
        e.set_status("NoSkillsPower", 1);
        e.set_status("No Draw", 1);
        e.set_status("Confusion", 1);
        e.set_status("CannotChangeStance", 1);
        e.set_status("FreeAttackPower", 1);
        e.set_status("Equilibrium", 2);
        e.set_status("Draw Card", 2);
        e.set_status("Next Turn Block", 4);
        e.set_status("Energized", 3);
        e.set_status("Double Tap", 1);
        e.set_status("Burst", 1);
        e.set_status("EnergyDown", 1);

        assert!(has_no_skills(&e));
        assert!(has_no_draw(&e));
        assert!(has_confusion(&e));
        assert!(cannot_change_stance(&e));
        assert!(consume_free_attack(&mut e));
        assert!(has_equilibrium(&e));
        assert_eq!(consume_draw_card_next_turn(&mut e), 2);
        assert_eq!(consume_next_turn_block(&mut e), 4);
        assert_eq!(consume_energized(&mut e), 3);
        assert!(consume_double_tap(&mut e));
        assert!(consume_burst(&mut e));
        assert_eq!(get_energy_down(&e), 1);
        decrement_equilibrium(&mut e);
        assert_eq!(e.status("Equilibrium"), 1);
    }

    #[test]
    fn apply_deva_form_helper_scales_energy_gain() {
        let mut e = entity();
        e.set_status("DevaForm", 1);
        assert_eq!(apply_deva_form(&mut e), 1);
        assert_eq!(e.status("DevaFormEnergy"), 1);
        assert_eq!(apply_deva_form(&mut e), 2);
        assert_eq!(e.status("DevaFormEnergy"), 2);
    }

    #[test]
    fn debuff_helpers_match_java_tick_and_decrement_behavior() {
        let mut e = entity();
        e.set_status("Poison", 4);
        e.set_status("Weakened", 2);
        e.set_status("Vulnerable", 1);
        e.set_status("Frail", 3);
        e.set_status("Blur", 2);
        e.set_status("Lock-On", 1);
        e.set_status("Intangible", 1);
        e.set_status("Fading", 2);

        assert_eq!(tick_poison(&mut e), 4);
        assert_eq!(e.hp, 46);
        decrement_debuffs(&mut e);
        assert_eq!(e.status("Weakened"), 1);
        assert_eq!(e.status("Vulnerable"), 0);
        assert!(!e.statuses.contains_key("Vulnerable"));
        assert_eq!(e.status("Frail"), 2);

        assert_eq!(modify_damage_receive(&e, 10.0), 1.0);
        decrement_blur(&mut e);
        decrement_lock_on(&mut e);
        decrement_intangible(&mut e);
        assert_eq!(e.status("Blur"), 1);
        assert_eq!(e.status("Lock-On"), 0);
        assert!(!e.statuses.contains_key("Lock-On"));
        assert_eq!(e.status("Intangible"), 0);
        assert!(!decrement_fading(&mut e));
        assert!(decrement_fading(&mut e));
        assert_eq!(e.status("Fading"), 0);
    }

    #[test]
    fn debuff_application_with_artifact_and_sadistic_matches_java() {
        let mut e = entity();
        e.set_status("Artifact", 2);
        assert!(!apply_debuff(&mut e, "Vulnerable", 2));
        assert_eq!(e.status("Artifact"), 1);
        assert_eq!(e.status("Vulnerable"), 0);

        let mut target = entity();
        let (applied, sadistic) = apply_debuff_with_sadistic(&mut target, "Weak", 1, 7);
        assert!(applied);
        assert_eq!(sadistic, 7);
        assert_eq!(target.status("Weak"), 1);
    }

    #[test]
    fn invincible_mode_shift_and_buffer_helpers_match_java() {
        let mut e = entity();
        e.set_status("Invincible", 12);
        assert_eq!(apply_invincible_cap(&mut e, 20), 12);
        assert_eq!(e.status("Invincible"), 0);

        e.set_status("Invincible", 5);
        assert_eq!(apply_invincible_cap(&mut e, 2), 2);
        assert_eq!(e.status("Invincible"), 3);

        e.set_status("Mode Shift", 14);
        assert!(!apply_mode_shift_damage(&mut e, 13));
        assert_eq!(e.status("Mode Shift"), 1);
        assert!(apply_mode_shift_damage(&mut e, 1));
        assert_eq!(e.status("Mode Shift"), 0);

        e.set_status("Buffer", 2);
        assert!(check_buffer(&mut e));
        assert_eq!(e.status("Buffer"), 1);
    }

    #[test]
    fn turn_based_damage_and_block_scalars_match_java() {
        let mut e = entity();
        e.set_status("DoubleDamage", 1);
        e.set_status("No Block", 1);
        e.set_status("Slow", 3);
        e.set_status("Intangible", 1);

        assert_eq!(modify_damage_give(&e, 7.0, true), 14.0);
        assert_eq!(modify_damage_receive(&e, 10.0), 1.0);
        assert_eq!(modify_block(&e, 15.0), 0.0);
        assert_eq!(modify_heal(&e, 12), 12);
    }

    #[test]
    fn enemy_power_scalars_match_java() {
        let mut e = entity();
        e.set_status("Ritual", 3);
        e.set_status("GenericStrengthUp", 2);
        e.set_status("Growth", 4);
        e.set_status("Beat of Death", 5);
        e.set_status("Spore Cloud", 2);
        e.set_status("Regrow", 7);
        e.set_status("TheBomb", 30);
        e.set_status("TheBombTurns", 2);
        e.set_status("TimeWarpActive", 1);
        e.set_status("Time Warp", 11);

        apply_ritual(&mut e);
        apply_generic_strength_up(&mut e);
        apply_growth(&mut e);
        assert_eq!(e.strength(), 9);
        assert_eq!(e.dexterity(), 4);
        assert_eq!(get_beat_of_death_damage(&e), 5);
        assert_eq!(get_spore_cloud_vulnerable(&e), 2);
        assert_eq!(get_regrow_heal(&e), 7);
        assert_eq!(decrement_the_bomb(&mut e), (false, 0));
        assert!(increment_time_warp(&mut e));
    }

    #[test]
    fn start_and_end_turn_counter_helpers_match_java() {
        let mut e = entity();
        e.set_status("LoseStrength", 4);
        e.set_status("LoseDexterity", 2);
        e.set_status("Wraith Form", 3);
        e.set_status("Demon Form", 5);
        e.set_status("Berserk", 2);
        e.set_status("Brutality", 1);
        e.set_status("Draw Card", 3);
        e.set_status("Next Turn Block", 7);
        e.set_status("Energized", 4);
        e.set_status("EnergyDown", 1);
        e.set_status("WrathNextTurn", 1);
        e.set_status("BattleHymn", 2);
        e.set_status("Devotion", 3);
        e.set_status("Infinite Blades", 1);
        e.set_status("Metallicize", 5);
        e.set_status("Plated Armor", 6);
        e.set_status("OmegaPower", 8);
        e.set_status("Like Water", 4);
        e.set_status("Combust", 9);
        e.set_status("Regeneration", 7);
        e.set_status("LiveForever", 6);
        e.set_status("Study", 2);
        e.set_status("EndTurnDeath", 1);
        e.set_status("Rage", 3);
        e.set_status("Equilibrium", 2);
        e.set_status("Intangible", 1);

        let start = process_start_of_turn(&mut e);
        assert_eq!(start.extra_energy, 3);
        assert_eq!(start.extra_draw, 0);
        assert_eq!(start.noxious_fumes_poison, 0);
        assert_eq!(start.brutality_draw, 1);
        assert_eq!(start.berserk_energy, 2);
        assert_eq!(start.battle_hymn_smites, 2);
        assert_eq!(start.devotion_mantra, 3);
        assert!(start.infinite_blades);
        assert!(start.demon_form_strength);
        assert!(start.wraith_form_dex_loss);

        let end = process_end_of_turn(&mut e, true);
        assert_eq!(end.metallicize_block, 5);
        assert_eq!(end.plated_armor_block, 6);
        assert_eq!(end.omega_damage, 8);
        assert_eq!(end.like_water_block, 4);
        assert_eq!(end.combust_hp_loss, 1);
        assert_eq!(end.combust_damage, 9);
        assert_eq!(end.regen_heal, 7);
        assert_eq!(end.live_forever_block, 6);
        assert_eq!(end.study_insights, 2);
        assert!(end.should_die);
    }

    #[test]
    fn process_end_of_round_matches_java_status_decay() {
        let mut e = entity();
        e.set_status("Weakened", 2);
        e.set_status("Vulnerable", 1);
        e.set_status("Frail", 1);
        e.set_status("Blur", 2);
        e.set_status("Lock-On", 1);
        e.set_status("Slow", 5);

        process_end_of_round(&mut e);

        assert_eq!(e.status("Weakened"), 1);
        assert_eq!(e.status("Vulnerable"), 0);
        assert!(!e.statuses.contains_key("Vulnerable"));
        assert_eq!(e.status("Frail"), 0);
        assert!(!e.statuses.contains_key("Frail"));
        assert_eq!(e.status("Blur"), 1);
        assert_eq!(e.status("Lock-On"), 0);
        assert_eq!(e.status("Slow"), 0);
    }

    #[test]
    fn panache_forcefield_and_rage_helpers_match_java() {
        let mut e = entity();
        e.set_status("Panache", 10);
        e.set_status("Forcefield", 2);
        e.set_status("Rage", 3);

        assert_eq!(check_panache(&mut e), 0);
        assert_eq!(check_panache(&mut e), 0);
        assert_eq!(check_panache(&mut e), 0);
        assert_eq!(check_panache(&mut e), 0);
        assert_eq!(check_panache(&mut e), 10);
        assert_eq!(check_panache(&mut e), 0);
        assert!(check_forcefield(&mut e));
        assert_eq!(e.status("Forcefield"), 1);
        assert_eq!(get_rage_block(&e), 3);
        remove_rage_end_of_turn(&mut e);
        assert_eq!(e.status("Rage"), 0);
    }

    #[test]
    fn consume_helpers_and_equilibrium_match_java() {
        let mut e = entity();
        e.set_status("Draw Card", 2);
        e.set_status("Next Turn Block", 4);
        e.set_status("Energized", 3);
        e.set_status("FreeAttackPower", 1);
        e.set_status("Double Tap", 1);
        e.set_status("Burst", 1);
        e.set_status("Equilibrium", 2);

        assert_eq!(consume_draw_card_next_turn(&mut e), 2);
        assert_eq!(consume_next_turn_block(&mut e), 4);
        assert_eq!(consume_energized(&mut e), 3);
        assert!(consume_free_attack(&mut e));
        assert!(consume_double_tap(&mut e));
        assert!(consume_burst(&mut e));
        assert!(has_equilibrium(&e));
        decrement_equilibrium(&mut e);
        assert_eq!(e.status("Equilibrium"), 1);
    }

    #[test]
    fn apply_lose_strength_dexterity_and_wraith_form_match_java() {
        let mut e = entity();
        e.set_status("Strength", 8);
        e.set_status("Dexterity", 6);
        e.set_status("LoseStrength", 3);
        e.set_status("LoseDexterity", 2);
        e.set_status("Wraith Form", 4);
        e.set_status("Demon Form", 5);

        apply_lose_strength(&mut e);
        apply_lose_dexterity(&mut e);
        apply_wraith_form(&mut e);
        apply_demon_form(&mut e);

        assert_eq!(e.strength(), 10);
        assert_eq!(e.dexterity(), 0);
        assert_eq!(e.status("LoseStrength"), 0);
        assert_eq!(e.status("LoseDexterity"), 0);
    }

    #[test]
    fn deva_form_and_misc_watcher_scalars_match_java() {
        let mut e = entity();
        e.set_status("DevaForm", 1);
        e.set_status("BattleHymn", 2);
        e.set_status("Study", 3);
        e.set_status("Like Water", 4);
        e.set_status("MentalFortress", 5);

        assert_eq!(apply_deva_form(&mut e), 1);
        assert_eq!(e.status("DevaFormEnergy"), 1);
        assert_eq!(get_battle_hymn_amount(&e), 2);
        assert_eq!(get_study_insights(&e), 3);
        assert_eq!(get_like_water_block(&e), 4);
        assert_eq!(get_mental_fortress_block(&e), 5);
    }

    #[test]
    fn exploit_and_damage_modifiers_match_java() {
        let mut e = entity();
        e.set_status("DoubleDamage", 1);
        e.set_status("No Block", 1);
        e.set_status("Slow", 2);
        e.set_status("Intangible", 1);

        assert_eq!(modify_damage_give(&e, 6.0, false), 12.0);
        assert_eq!(modify_damage_receive(&e, 10.0), 1.0);
        assert_eq!(modify_block(&e, 14.0), 0.0);
        assert_eq!(modify_heal(&e, 9), 9);
    }

    #[test]
    fn orb_related_defect_statuses_are_exposed_exactly() {
        let mut e = entity();
        e.set_status("Focus", 2);
        e.set_status("Storm", 1);
        e.set_status("Heatsink", 3);
        e.set_status("Static Discharge", 4);
        e.set_status("Loop", 1);
        e.set_status("Electro", 1);
        e.set_status("Equilibrium", 2);
        e.set_status("Blur", 1);
        assert_eq!(e.status("Focus"), 2);
        assert!(should_storm_channel(&e));
        assert_eq!(get_heatsink_draw(&e), 3);
        assert_eq!(get_static_discharge(&e), 4);
        assert_eq!(get_extra_draw(&e), 0);
        assert_eq!(e.status("Loop"), 1);
        assert_eq!(e.status("Electro"), 1);
        assert!(has_equilibrium(&e));
        assert_eq!(e.status("Blur"), 1);
    }

    #[test]
    fn engine_channels_and_evokes_defect_orbs_with_focus() {
        let mut engine = make_engine(&["Strike_P"; 12], 100, 0);
        engine.init_defect_orbs(3);
        engine.state.player.set_status("Focus", 2);

        engine.channel_orb(OrbType::Lightning);
        assert_eq!(engine.state.orb_slots.occupied_count(), 1);
        assert_eq!(engine.state.orb_slots.slots[0].orb_type, OrbType::Lightning);

        let hp_before = engine.state.enemies[0].entity.hp;
        engine.evoke_front_orb();
        assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 10);
        assert_eq!(engine.state.orb_slots.occupied_count(), 0);
    }

    #[test]
    fn storm_and_heatsink_trigger_on_power_play() {
        let mut engine = make_engine(&["Strike_P", "Strike_P", "Strike_P", "Strike_P"], 100, 0);
        engine.init_defect_orbs(3);
        engine.state.player.set_status("Storm", 1);
        engine.state.player.set_status("Heatsink", 1);
        ensure_in_hand(&mut engine, "BattleHymn");
        let hand_before = engine.state.hand.len();
        ensure_on_top_of_draw(&mut engine, "Defend_P");

        assert!(play_self(&mut engine, "BattleHymn"));

        assert_eq!(engine.state.orb_slots.occupied_count(), 1);
        assert_eq!(engine.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
        assert_eq!(hand_count(&engine, "Defend_P"), 1);
        assert!(engine.state.hand.len() >= hand_before);
    }

    #[test]
    fn static_discharge_and_electro_registry_entries_are_present() {
        let static_discharge = get_power_def("Static Discharge").unwrap();
        let electro = get_power_def("Electro").unwrap();
        assert_eq!(static_discharge.id, "Static Discharge");
        assert!(static_discharge.on_hp_lost);
        assert_eq!(electro.id, "Electro");
        assert!(!electro.stackable);
    }
}
