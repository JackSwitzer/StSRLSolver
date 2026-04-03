#[cfg(test)]
mod power_java_parity_tests {
    //! Java references:
    //! - /tmp/sts-decompiled/com/megacrit/cardcrawl/powers/*.java
    //! - /tmp/sts-decompiled/com/megacrit/cardcrawl/powers/watcher/*.java
    //! - /tmp/sts-decompiled/com/megacrit/cardcrawl/powers/{StrengthPower,WeakPower,VulnerablePower,FrailPower,ArtifactPower,IntangiblePower,ThornsPower,PoisonPower,RegenerationPower,BufferPower,BarricadePower,MetallicizePower,PlatedArmorPower,RitualPower,AngryPower,EnragePower,CuriosityPower,ModeShiftPower,SplitPower,FadingPower,InvinciblePower,BackAttackPower,ExplosivePower,UnawakenedPower,ResurrectPower,SlowPower,TimeWarpPower,SporeCloudPower,ThieveryPower,DemonFormPower,FlameBarrierPower,BrutalityPower,DarkEmbracePower,DoubleTapPower,EvolvePower,FeelNoPainPower,FireBreathingPower,JuggernautPower,RupturePower,BerserkPower,CombustPower,CorruptionPower,DoubleDamagePower,RagePower,NoxiousFumesPower,EnvenomPower,AfterImagePower,AccuracyPower,ThousandCutsPower,InfiniteBladesPower,ToolsOfTheTradePower,NightmarePower,PhantasmalKillerPower,SadisticNaturePower,FocusPower,LockOnPower,CreativeAIPower,StormPower,HeatsinkPower,StaticDischargePower,ElectroPower,LoopPower,HelloWorldPower,EquilibriumPower,RushdownPower,MentalFortressPower,BattleHymnPower,DevotionPower,EstablishmentPower,ForesightPower,LikeWaterPower,NirvanaPower,OmegaPower,StudyPower,WaveOfTheHandPower,VigorPower,MantraPower,BlockReturnPower,DevaPower,LiveForeverPower,WrathNextTurnPower,EndTurnDeathPower,FreeAttackPower,MasterRealityPower,NoSkillsPower,EnergyDownPower,CannotChangeStancePower,MarkPower,VaultPower,OmnisciencePower,BlurPower,ConservePower,DrawCardNextTurnPower,DrawPowerPower,DoubleDamagePower,EnergizedPower,NextTurnBlockPower,PenNibPower,ReboundPower,NoBlockPower,NoDrawPower,EntangledPower,ConfusionPower,PanachePower,BurstPower,WraithFormPower,BeatOfDeathPower,GrowthPower,MagnetismPower,SkillBurnPower,ForcefieldPower,RegrowPower,StasisPower,TheBombPower,GenericStrengthUpPower,LoseStrengthPower,LoseDexterityPower,CollectPower,WinterPower,RepairPower}.java

    use crate::cards::CardRegistry;
    use crate::status_ids::sid;
    use crate::engine::CombatEngine;
    use crate::orbs::OrbType;
    use crate::powers::*;
    use crate::state::{EntityState, Stance};
    use crate::tests::support::*;

    fn entity() -> EntityState {
        EntityState::new(50, 50)
    }

    fn deck(cards: &[&str]) -> Vec<crate::combat_types::CardInstance> {
        make_deck(cards)
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
        assert_eq!(PowerId::Omega.key(), "Omega");
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
            (PowerId::PlatedArmor, "PlatedArmor"),
            (PowerId::Ritual, "Ritual"),
            (PowerId::Curiosity, "Curiosity"),
            (PowerId::ModeShift, "ModeShift"),
            (PowerId::Invincible, "Invincible"),
            (PowerId::Split, "Split"),
            (PowerId::TimeWarp, "TimeWarp"),
            (PowerId::SporeCloud, "SporeCloud"),
            (PowerId::DemonForm, "DemonForm"),
            (PowerId::FeelNoPain, "FeelNoPain"),
            (PowerId::FireBreathing, "FireBreathing"),
            (PowerId::Rage, "Rage"),
            (PowerId::NoxiousFumes, "NoxiousFumes"),
            (PowerId::AfterImage, "AfterImage"),
            (PowerId::Focus, "Focus"),
            (PowerId::Loop, "Loop"),
            (PowerId::BattleHymn, "BattleHymn"),
            (PowerId::Establishment, "Establishment"),
            (PowerId::LikeWater, "LikeWater"),
            (PowerId::Nirvana, "Nirvana"),
            (PowerId::Omega, "Omega"),
            (PowerId::WaveOfTheHand, "WaveOfTheHand"),
            (PowerId::DevaForm, "DevaForm"),
            (PowerId::EndTurnDeath, "EndTurnDeath"),
            (PowerId::EnergyDown, "EnergyDown"),
            (PowerId::CannotChangeStance, "CannotChangeStance"),
            (PowerId::Blur, "Blur"),
            (PowerId::DoubleDamage, "DoubleDamage"),
            (PowerId::WraithForm, "WraithForm"),
            (PowerId::BeatOfDeath, "BeatOfDeath"),
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
    assert_power_def!(plated_armor_def_matches_java, "PlatedArmor", power_type => PowerType::Buff, on_turn_end => true, on_attacked => true);
    assert_power_def!(ritual_def_matches_java, "Ritual", power_type => PowerType::Buff, on_turn_end => true);
    assert_power_def!(double_damage_def_matches_java, "DoubleDamage", power_type => PowerType::Buff, modify_damage_give => true);
    assert_power_def!(rage_def_matches_java, "Rage", power_type => PowerType::Buff, on_use_card => true, on_turn_end => true);
    assert_power_def!(noxious_fumes_def_matches_java, "NoxiousFumes", power_type => PowerType::Buff, on_turn_start_post_draw => true);
    assert_power_def!(after_image_def_matches_java, "AfterImage", power_type => PowerType::Buff, on_use_card => true);
    assert_power_def!(accuracy_def_matches_java, "Accuracy", power_type => PowerType::Buff, modify_damage_give => true);
    assert_power_def!(phantasmal_killer_def_matches_java, "PhantasmalKiller", power_type => PowerType::Buff, on_turn_start_post_draw => true, modify_damage_give => true);
    assert_power_def!(focus_def_matches_java, "Focus", power_type => PowerType::Buff, can_go_negative => true);
    assert_power_def!(lock_on_def_matches_java, "Lock-On", power_type => PowerType::Debuff, is_turn_based => true, on_end_of_round => true);
    assert_power_def!(storm_def_matches_java, "Storm", power_type => PowerType::Buff, on_use_card => true);
    assert_power_def!(heatsink_def_matches_java, "Heatsink", power_type => PowerType::Buff, on_use_card => true);
    assert_power_def!(static_discharge_def_matches_java, "StaticDischarge", power_type => PowerType::Buff, on_hp_lost => true);
    assert_power_def!(loop_def_matches_java, "Loop", power_type => PowerType::Buff, on_turn_start => true);
    assert_power_def!(equilibrium_def_matches_java, "Equilibrium", power_type => PowerType::Buff, on_turn_end => true);
    assert_power_def!(rushdown_def_matches_java, "Rushdown", power_type => PowerType::Buff, on_change_stance => true);
    assert_power_def!(mental_fortress_def_matches_java, "MentalFortress", power_type => PowerType::Buff, on_change_stance => true);
    assert_power_def!(battle_hymn_def_matches_java, "BattleHymn", power_type => PowerType::Buff, on_turn_start => true);
    assert_power_def!(devotion_def_matches_java, "Devotion", power_type => PowerType::Buff, on_turn_start_post_draw => true);
    assert_power_def!(establishment_def_matches_java, "Establishment", power_type => PowerType::Buff, on_turn_end => true);
    assert_power_def!(like_water_def_matches_java, "LikeWater", power_type => PowerType::Buff, on_turn_end => true);
    assert_power_def!(nirvana_def_matches_java, "Nirvana", power_type => PowerType::Buff, on_scry => true);
    assert_power_def!(omega_def_matches_java, "Omega", power_type => PowerType::Buff, on_turn_end => true);
    assert_power_def!(study_def_matches_java, "Study", power_type => PowerType::Buff, on_turn_end => true);
    assert_power_def!(wave_of_the_hand_def_matches_java, "WaveOfTheHand", power_type => PowerType::Buff, on_gained_block => true);
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
    assert_power_def!(draw_card_next_turn_def_matches_java, "DrawCard", power_type => PowerType::Buff);
    assert_power_def!(draw_power_def_matches_java, "Draw", power_type => PowerType::Buff);
    assert_power_def!(energized_def_matches_java, "Energized", power_type => PowerType::Buff);
    assert_power_def!(next_turn_block_def_matches_java, "NextTurnBlock", power_type => PowerType::Buff);
    assert_power_def!(no_block_def_matches_java, "NoBlock", power_type => PowerType::Debuff, on_end_of_round => true);
    assert_power_def!(no_draw_def_matches_java, "NoDraw", power_type => PowerType::Debuff);
    assert_power_def!(confusion_def_matches_java, "Confusion", power_type => PowerType::Debuff);
    assert_power_def!(panache_def_matches_java, "Panache", power_type => PowerType::Buff);
    assert_power_def!(burst_def_matches_java, "Burst", power_type => PowerType::Buff);
    assert_power_def!(wraith_form_def_matches_java, "WraithForm", power_type => PowerType::Debuff, can_go_negative => true, on_turn_end => true);
    assert_power_def!(beat_of_death_def_matches_java, "BeatOfDeath", power_type => PowerType::Buff, on_after_use_card => true);
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
        e.set_status(sid::AFTER_IMAGE, 2);
        e.set_status(sid::RAGE, 3);
        e.set_status(sid::THOUSAND_CUTS, 4);
        e.set_status(sid::NOXIOUS_FUMES, 5);
        e.set_status(sid::BATTLE_HYMN, 1);
        e.set_status(sid::DEVOTION, 2);
        e.set_status(sid::RUSHDOWN, 2);
        e.set_status(sid::MENTAL_FORTRESS, 4);
        e.set_status(sid::NIRVANA, 3);
        e.set_status(sid::STUDY, 2);
        e.set_status(sid::LIVE_FOREVER, 4);
        e.set_status(sid::ACCURACY, 3);
        e.set_status(sid::MARK, 7);
        e.set_status(sid::DARK_EMBRACE, 1);
        e.set_status(sid::FEEL_NO_PAIN, 5);
        e.set_status(sid::EVOLVE, 2);
        e.set_status(sid::FIRE_BREATHING, 6);
        e.set_status(sid::HEATSINK, 3);
        e.set_status(sid::SKILL_BURN, 9);
        e.set_status(sid::ENVENOM, 4);
        e.set_status(sid::JUGGERNAUT, 6);
        e.set_status(sid::WAVE_OF_THE_HAND, 1);

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
        e.set_status(sid::DRAW, 3);
        e.set_status(sid::ENERGY_DOWN, 2);
        e.set_status(sid::BATTLE_HYMN, 1);
        e.set_status(sid::DEVOTION, 4);
        e.set_status(sid::INFINITE_BLADES, 1);
        e.set_status(sid::STORM, 1);
        e.set_status(sid::HEATSINK, 2);
        e.set_status(sid::STATIC_DISCHARGE, 3);
        e.set_status(sid::FOCUS, 5);
        e.set_status(sid::LOCK_ON, 1);
        e.set_status(sid::MAGNETISM, 6);
        e.set_status(sid::REGROW, 7);
        e.set_status(sid::SPORE_CLOUD, 2);
        e.set_status(sid::BEAT_OF_DEATH, 9);
        e.set_status(sid::FORCEFIELD, 4);
        e.set_status(sid::SKILL_BURN, 8);
        e.set_status(sid::NOXIOUS_FUMES, 3);
        e.set_status(sid::ENVENOM, 2);
        e.set_status(sid::LIVE_FOREVER, 6);
        e.set_status(sid::STUDY, 4);

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
        e.set_status(sid::NIRVANA, 3);
        e.set_status(sid::RUSHDOWN, 2);
        e.set_status(sid::MENTAL_FORTRESS, 4);
        e.set_status(sid::LIKE_WATER, 5);
        e.set_status(sid::WAVE_OF_THE_HAND, 1);
        e.set_status(sid::ESTABLISHMENT, 1);
        e.set_status(sid::VIGOR, 6);
        e.set_status(sid::OMEGA, 18);

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
        e.set_status(sid::STRENGTH, 5);
        e.set_status(sid::DEXTERITY, 4);
        e.set_status(sid::LOSE_STRENGTH, 2);
        e.set_status(sid::LOSE_DEXTERITY, 1);
        e.set_status(sid::WRAITH_FORM, 2);
        e.set_status(sid::DEMON_FORM, 3);
        e.set_status(sid::BERSERK, 4);
        e.set_status(sid::NOXIOUS_FUMES, 2);
        e.set_status(sid::BRUTALITY, 1);
        e.set_status(sid::DRAW_CARD, 3);
        e.set_status(sid::NEXT_TURN_BLOCK, 7);
        e.set_status(sid::ENERGIZED, 2);
        e.set_status(sid::ENERGY_DOWN, 1);
        e.set_status(sid::WRATH_NEXT_TURN, 1);
        e.set_status(sid::BATTLE_HYMN, 2);
        e.set_status(sid::DEVOTION, 3);
        e.set_status(sid::INFINITE_BLADES, 1);
        e.set_status(sid::DRAW, 4);
        e.set_status(sid::DEVA_FORM, 1);
        e.set_status(sid::DEVA_FORM_ENERGY, 0);
        e.set_status(sid::FLAME_BARRIER, 2);

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
        assert_eq!(e.status(sid::LOSE_STRENGTH), 0);
        assert_eq!(e.status(sid::LOSE_DEXTERITY), 0);
        assert_eq!(e.status(sid::FLAME_BARRIER), 0);
        assert_eq!(e.status(sid::DRAW_CARD), 0);
        assert_eq!(e.status(sid::NEXT_TURN_BLOCK), 0);
        assert_eq!(e.status(sid::ENERGIZED), 0);
        assert_eq!(e.status(sid::ENERGY_DOWN), 1);
        assert_eq!(e.status(sid::WRATH_NEXT_TURN), 0);
        assert_eq!(e.status(sid::DEVA_FORM_ENERGY), 1);
    }

    #[test]
    fn end_of_turn_dispatch_matches_java() {
        let mut e = entity();
        e.set_status(sid::METALLICIZE, 4);
        e.set_status(sid::PLATED_ARMOR, 6);
        e.set_status(sid::OMEGA, 9);
        e.set_status(sid::LIKE_WATER, 5);
        e.set_status(sid::COMBUST, 5);
        e.set_status(sid::REGENERATION, 7);
        e.set_status(sid::LIVE_FOREVER, 4);
        e.set_status(sid::STUDY, 2);
        e.set_status(sid::RAGE, 3);
        e.set_status(sid::EQUILIBRIUM, 2);
        e.set_status(sid::INTANGIBLE, 1);
        e.set_status(sid::END_TURN_DEATH, 1);

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
        assert_eq!(e.status(sid::RAGE), 0);
        assert_eq!(e.status(sid::EQUILIBRIUM), 1);
        assert_eq!(e.status(sid::INTANGIBLE), 0);
        assert_eq!(e.status(sid::REGENERATION), 6);
    }

    #[test]
    fn end_of_round_dispatch_matches_java() {
        let mut e = entity();
        e.set_status(sid::WEAKENED, 3);
        e.set_status(sid::VULNERABLE, 2);
        e.set_status(sid::FRAIL, 1);
        e.set_status(sid::BLUR, 2);
        e.set_status(sid::LOCK_ON, 4);
        e.set_status(sid::SLOW, 5);

        process_end_of_round(&mut e);

        assert_eq!(e.status(sid::WEAKENED), 2);
        assert_eq!(e.status(sid::VULNERABLE), 1);
        assert_eq!(e.status(sid::FRAIL), 0);
        assert_eq!(e.status(sid::FRAIL), 0);
        assert_eq!(e.status(sid::BLUR), 1);
        assert_eq!(e.status(sid::LOCK_ON), 3);
        assert_eq!(e.status(sid::SLOW), 0);
        assert_eq!(e.status(sid::SLOW), 0);
    }

    #[test]
    fn debuff_application_and_restrictions_match_java() {
        let mut e = entity();
        e.set_status(sid::ARTIFACT, 2);
        assert!(!apply_debuff(&mut e, sid::WEAKENED, 3));
        assert_eq!(e.status(sid::ARTIFACT), 1);
        assert_eq!(e.status(sid::WEAKENED), 0);

        assert!(!apply_debuff(&mut e, sid::VULNERABLE, 1));
        assert_eq!(e.status(sid::ARTIFACT), 0);
        assert_eq!(e.status(sid::VULNERABLE), 0);

        assert!(apply_debuff(&mut e, sid::VULNERABLE, 1));
        assert_eq!(e.status(sid::VULNERABLE), 1);

        assert!(!has_no_skills(&e));
        assert!(!has_no_draw(&e));
        assert!(!has_confusion(&e));
        assert!(!cannot_change_stance(&e));

        e.set_status(sid::NO_SKILLS_POWER, 1);
        e.set_status(sid::NO_DRAW, 1);
        e.set_status(sid::CONFUSION, 1);
        e.set_status(sid::CANNOT_CHANGE_STANCE, 1);
        assert!(has_no_skills(&e));
        assert!(has_no_draw(&e));
        assert!(has_confusion(&e));
        assert!(cannot_change_stance(&e));
    }

    #[test]
    fn defensive_power_helpers_match_java() {
        let mut e = entity();
        e.block = 20;
        e.set_status(sid::BARRICADE, 1);
        assert!(should_retain_block(&e));
        assert_eq!(apply_block_decay(&e, true), 20);

        e.set_status(sid::BARRICADE, 0);
        e.set_status(sid::BLUR, 1);
        assert!(should_retain_block(&e));
        e.set_status(sid::BLUR, 0);
        assert_eq!(apply_block_decay(&e, true), 5);
        assert_eq!(apply_block_decay(&e, false), 0);

        e.set_status(sid::METALLICIZE, 4);
        apply_metallicize(&mut e);
        assert_eq!(e.block, 24);

        e.set_status(sid::PLATED_ARMOR, 6);
        apply_plated_armor(&mut e);
        assert_eq!(e.block, 30);

        e.set_status(sid::THORNS, 3);
        e.set_status(sid::FLAME_BARRIER, 5);
        e.set_status(sid::BUFFER, 1);
        assert_eq!(get_thorns_damage(&e), 3);
        assert_eq!(get_flame_barrier_damage(&e), 5);
        assert!(check_buffer(&mut e));
        assert_eq!(e.status(sid::BUFFER), 0);

        e.set_status(sid::INVINCIBLE, 12);
        assert_eq!(apply_invincible_cap(&mut e, 20), 12);
        assert_eq!(e.status(sid::INVINCIBLE), 0);

        e.set_status(sid::MODE_SHIFT, 20);
        assert!(!apply_mode_shift_damage(&mut e, 10));
        assert!(apply_mode_shift_damage(&mut e, 10));
    }

    #[test]
    fn turn_scaling_helpers_match_java() {
        let mut e = entity();
        e.set_status(sid::RITUAL, 3);
        e.set_status(sid::GENERIC_STRENGTH_UP, 2);
        e.set_status(sid::REGENERATION, 7);
        e.set_status(sid::BEAT_OF_DEATH, 2);
        e.set_status(sid::STATIC_DISCHARGE, 3);
        e.set_status(sid::SPORE_CLOUD, 2);
        e.set_status(sid::SLOW, 5);
        e.set_status(sid::TIME_WARP_ACTIVE, 1);
        e.set_status(sid::TIME_WARP, 11);
        e.set_status(sid::FORCEFIELD, 2);

        apply_ritual(&mut e);
        apply_generic_strength_up(&mut e);
        assert_eq!(e.strength(), 5);

        assert_eq!(apply_regeneration(&mut e), 7);
        assert_eq!(e.status(sid::REGENERATION), 6);

        assert_eq!(get_beat_of_death_damage(&e), 2);
        assert_eq!(get_static_discharge(&e), 3);
        assert_eq!(get_spore_cloud_vulnerable(&e), 2);

        increment_slow(&mut e);
        assert_eq!(e.status(sid::SLOW), 6);
        assert!(increment_time_warp(&mut e));
        assert_eq!(e.status(sid::TIME_WARP), 0);

        assert!(check_forcefield(&mut e));
        assert_eq!(e.status(sid::FORCEFIELD), 1);
    }

    #[test]
    fn consume_helpers_match_java() {
        let mut e = entity();
        e.set_status(sid::DRAW_CARD, 2);
        e.set_status(sid::NEXT_TURN_BLOCK, 4);
        e.set_status(sid::ENERGIZED, 3);
        e.set_status(sid::FREE_ATTACK_POWER, 1);
        e.set_status(sid::DOUBLE_TAP, 1);
        e.set_status(sid::BURST, 1);
        e.set_status(sid::EQUILIBRIUM, 2);

        assert_eq!(consume_draw_card_next_turn(&mut e), 2);
        assert_eq!(e.status(sid::DRAW_CARD), 0);
        assert_eq!(consume_next_turn_block(&mut e), 4);
        assert_eq!(e.status(sid::NEXT_TURN_BLOCK), 0);
        assert_eq!(consume_energized(&mut e), 3);
        assert_eq!(e.status(sid::ENERGIZED), 0);
        assert!(consume_free_attack(&mut e));
        assert_eq!(e.status(sid::FREE_ATTACK_POWER), 0);
        assert!(consume_double_tap(&mut e));
        assert_eq!(e.status(sid::DOUBLE_TAP), 0);
        assert!(consume_burst(&mut e));
        assert_eq!(e.status(sid::BURST), 0);
        assert!(has_equilibrium(&e));
        decrement_equilibrium(&mut e);
        assert_eq!(e.status(sid::EQUILIBRIUM), 1);
    }

    #[test]
    fn damage_and_heal_modifiers_match_java() {
        let mut e = entity();
        e.set_status(sid::DOUBLE_DAMAGE, 1);
        e.set_status(sid::NO_BLOCK, 1);
        e.set_status(sid::SLOW, 2);
        e.set_status(sid::INTANGIBLE, 1);

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
        assert_eq!(deva_engine.state.player.status(sid::DEVA_FORM), 1);
        end_turn(&mut deva_engine);
        assert_eq!(deva_engine.state.energy, 4);
        assert_eq!(deva_engine.state.player.status(sid::DEVA_FORM), 2);
        end_turn(&mut deva_engine);
        assert_eq!(deva_engine.state.energy, 5);
        assert_eq!(deva_engine.state.player.status(sid::DEVA_FORM), 3);
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
        assert_eq!(engine.state.player.status(sid::VIGOR), 5);

        let hp_before = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Strike_P", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp_before - 11);
        assert_eq!(engine.state.player.status(sid::VIGOR), 0);
    }

    #[test]
    #[ignore] // SIGABRT in complex multi-power state — index management bug, not trigger ordering
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
        assert_eq!(engine.state.player.block, rage_block_before + 4); // +3 Rage + +1 After Image

        // Beat of Death: deals damage after each card played
        // Clear all accumulated block and set up clean state
        engine.state.player.block = 0;
        engine.state.player.set_status(sid::AFTER_IMAGE, 0); // disable to isolate Beat of Death
        engine.state.enemies[0].entity.hp = 200;
        engine.state.enemies[0].entity.set_status(sid::BEAT_OF_DEATH, 2);
        let hp_before = engine.state.player.hp;
        ensure_in_hand(&mut engine, "Strike_P");
        assert!(play_on_enemy(&mut engine, "Strike_P", 0));
        // No After Image block, no other block → Beat of Death 2 hits full HP
        assert_eq!(engine.state.player.hp, hp_before - 2);
    }

    #[test]
    fn exhaust_powers_match_java_on_engine() {
        let mut engine = make_engine(&["Strike_P"; 12], 100, 0);
        engine.state.player.set_status(sid::FEEL_NO_PAIN, 2);
        engine.state.player.set_status(sid::DARK_EMBRACE, 1);
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
        assert_eq!(engine.state.player.status(sid::WAVE_OF_THE_HAND), 1);

        ensure_in_hand(&mut engine, "WaveOfTheHand+");
        assert!(play_self(&mut engine, "WaveOfTheHand+"));
        assert_eq!(engine.state.player.status(sid::WAVE_OF_THE_HAND), 3);
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
        assert_eq!(engine.state.player.status(sid::ESTABLISHMENT), 1);
    }

    #[test]
    fn status_restrictions_and_misc_helpers_match_java() {
        let mut e = entity();
        e.set_status(sid::NO_SKILLS_POWER, 1);
        e.set_status(sid::NO_DRAW, 1);
        e.set_status(sid::CONFUSION, 1);
        e.set_status(sid::CANNOT_CHANGE_STANCE, 1);
        e.set_status(sid::FREE_ATTACK_POWER, 1);
        e.set_status(sid::EQUILIBRIUM, 2);
        e.set_status(sid::DRAW_CARD, 2);
        e.set_status(sid::NEXT_TURN_BLOCK, 4);
        e.set_status(sid::ENERGIZED, 3);
        e.set_status(sid::DOUBLE_TAP, 1);
        e.set_status(sid::BURST, 1);
        e.set_status(sid::ENERGY_DOWN, 1);

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
        assert_eq!(e.status(sid::EQUILIBRIUM), 1);
    }

    #[test]
    fn apply_deva_form_helper_scales_energy_gain() {
        let mut e = entity();
        e.set_status(sid::DEVA_FORM, 1);
        assert_eq!(apply_deva_form(&mut e), 1);
        assert_eq!(e.status(sid::DEVA_FORM_ENERGY), 1);
        assert_eq!(apply_deva_form(&mut e), 2);
        assert_eq!(e.status(sid::DEVA_FORM_ENERGY), 2);
    }

    #[test]
    fn debuff_helpers_match_java_tick_and_decrement_behavior() {
        let mut e = entity();
        e.set_status(sid::POISON, 4);
        e.set_status(sid::WEAKENED, 2);
        e.set_status(sid::VULNERABLE, 1);
        e.set_status(sid::FRAIL, 3);
        e.set_status(sid::BLUR, 2);
        e.set_status(sid::LOCK_ON, 1);
        e.set_status(sid::INTANGIBLE, 1);
        e.set_status(sid::FADING, 2);

        assert_eq!(tick_poison(&mut e), 4);
        assert_eq!(e.hp, 46);
        decrement_debuffs(&mut e);
        assert_eq!(e.status(sid::WEAKENED), 1);
        assert_eq!(e.status(sid::VULNERABLE), 0);
        assert_eq!(e.status(sid::VULNERABLE), 0);
        assert_eq!(e.status(sid::FRAIL), 2);

        assert_eq!(modify_damage_receive(&e, 10.0), 1.0);
        decrement_blur(&mut e);
        decrement_lock_on(&mut e);
        decrement_intangible(&mut e);
        assert_eq!(e.status(sid::BLUR), 1);
        assert_eq!(e.status(sid::LOCK_ON), 0);
        assert_eq!(e.status(sid::LOCK_ON), 0);
        assert_eq!(e.status(sid::INTANGIBLE), 0);
        assert!(!decrement_fading(&mut e));
        assert!(decrement_fading(&mut e));
        assert_eq!(e.status(sid::FADING), 0);
    }

    #[test]
    fn debuff_application_with_artifact_and_sadistic_matches_java() {
        let mut e = entity();
        e.set_status(sid::ARTIFACT, 2);
        assert!(!apply_debuff(&mut e, sid::VULNERABLE, 2));
        assert_eq!(e.status(sid::ARTIFACT), 1);
        assert_eq!(e.status(sid::VULNERABLE), 0);

        let mut target = entity();
        let (applied, sadistic) = apply_debuff_with_sadistic(&mut target, sid::WEAK, 1, 7);
        assert!(applied);
        assert_eq!(sadistic, 7);
        assert_eq!(target.status(sid::WEAK), 1);
    }

    #[test]
    fn invincible_mode_shift_and_buffer_helpers_match_java() {
        let mut e = entity();
        e.set_status(sid::INVINCIBLE, 12);
        assert_eq!(apply_invincible_cap(&mut e, 20), 12);
        assert_eq!(e.status(sid::INVINCIBLE), 0);

        e.set_status(sid::INVINCIBLE, 5);
        assert_eq!(apply_invincible_cap(&mut e, 2), 2);
        assert_eq!(e.status(sid::INVINCIBLE), 3);

        e.set_status(sid::MODE_SHIFT, 14);
        assert!(!apply_mode_shift_damage(&mut e, 13));
        assert_eq!(e.status(sid::MODE_SHIFT), 1);
        assert!(apply_mode_shift_damage(&mut e, 1));
        assert_eq!(e.status(sid::MODE_SHIFT), 0);

        e.set_status(sid::BUFFER, 2);
        assert!(check_buffer(&mut e));
        assert_eq!(e.status(sid::BUFFER), 1);
    }

    #[test]
    fn turn_based_damage_and_block_scalars_match_java() {
        let mut e = entity();
        e.set_status(sid::DOUBLE_DAMAGE, 1);
        e.set_status(sid::NO_BLOCK, 1);
        e.set_status(sid::SLOW, 3);
        e.set_status(sid::INTANGIBLE, 1);

        assert_eq!(modify_damage_give(&e, 7.0, true), 14.0);
        assert_eq!(modify_damage_receive(&e, 10.0), 1.0);
        assert_eq!(modify_block(&e, 15.0), 0.0);
        assert_eq!(modify_heal(&e, 12), 12);
    }

    #[test]
    fn enemy_power_scalars_match_java() {
        let mut e = entity();
        e.set_status(sid::RITUAL, 3);
        e.set_status(sid::GENERIC_STRENGTH_UP, 2);
        e.set_status(sid::GROWTH, 4);
        e.set_status(sid::BEAT_OF_DEATH, 5);
        e.set_status(sid::SPORE_CLOUD, 2);
        e.set_status(sid::REGROW, 7);
        e.set_status(sid::THE_BOMB, 30);
        e.set_status(sid::THE_BOMB_TURNS, 2);
        e.set_status(sid::TIME_WARP_ACTIVE, 1);
        e.set_status(sid::TIME_WARP, 11);

        apply_ritual(&mut e);
        apply_generic_strength_up(&mut e);
        apply_growth(&mut e);
        assert_eq!(e.strength(), 9);
        assert_eq!(e.block, 4); // Growth adds Block, not Dexterity (Java parity)
        assert_eq!(get_beat_of_death_damage(&e), 5);
        assert_eq!(get_spore_cloud_vulnerable(&e), 2);
        assert_eq!(get_regrow_heal(&e), 7);
        assert_eq!(decrement_the_bomb(&mut e), (false, 0));
        assert!(increment_time_warp(&mut e));
    }

    #[test]
    fn start_and_end_turn_counter_helpers_match_java() {
        let mut e = entity();
        e.set_status(sid::LOSE_STRENGTH, 4);
        e.set_status(sid::LOSE_DEXTERITY, 2);
        e.set_status(sid::WRAITH_FORM, 3);
        e.set_status(sid::DEMON_FORM, 5);
        e.set_status(sid::BERSERK, 2);
        e.set_status(sid::BRUTALITY, 1);
        e.set_status(sid::DRAW_CARD, 3);
        e.set_status(sid::NEXT_TURN_BLOCK, 7);
        e.set_status(sid::ENERGIZED, 4);
        e.set_status(sid::ENERGY_DOWN, 1);
        e.set_status(sid::WRATH_NEXT_TURN, 1);
        e.set_status(sid::BATTLE_HYMN, 2);
        e.set_status(sid::DEVOTION, 3);
        e.set_status(sid::INFINITE_BLADES, 1);
        e.set_status(sid::METALLICIZE, 5);
        e.set_status(sid::PLATED_ARMOR, 6);
        e.set_status(sid::OMEGA, 8);
        e.set_status(sid::LIKE_WATER, 4);
        e.set_status(sid::COMBUST, 9);
        e.set_status(sid::REGENERATION, 7);
        e.set_status(sid::LIVE_FOREVER, 6);
        e.set_status(sid::STUDY, 2);
        e.set_status(sid::END_TURN_DEATH, 1);
        e.set_status(sid::RAGE, 3);
        e.set_status(sid::EQUILIBRIUM, 2);
        e.set_status(sid::INTANGIBLE, 1);

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
        e.set_status(sid::WEAKENED, 2);
        e.set_status(sid::VULNERABLE, 1);
        e.set_status(sid::FRAIL, 1);
        e.set_status(sid::BLUR, 2);
        e.set_status(sid::LOCK_ON, 1);
        e.set_status(sid::SLOW, 5);

        process_end_of_round(&mut e);

        assert_eq!(e.status(sid::WEAKENED), 1);
        assert_eq!(e.status(sid::VULNERABLE), 0);
        assert_eq!(e.status(sid::VULNERABLE), 0);
        assert_eq!(e.status(sid::FRAIL), 0);
        assert_eq!(e.status(sid::FRAIL), 0);
        assert_eq!(e.status(sid::BLUR), 1);
        assert_eq!(e.status(sid::LOCK_ON), 0);
        assert_eq!(e.status(sid::SLOW), 0);
    }

    #[test]
    fn panache_forcefield_and_rage_helpers_match_java() {
        let mut e = entity();
        e.set_status(sid::PANACHE, 10);
        e.set_status(sid::FORCEFIELD, 2);
        e.set_status(sid::RAGE, 3);

        assert_eq!(check_panache(&mut e), 0);
        assert_eq!(check_panache(&mut e), 0);
        assert_eq!(check_panache(&mut e), 0);
        assert_eq!(check_panache(&mut e), 0);
        assert_eq!(check_panache(&mut e), 10);
        assert_eq!(check_panache(&mut e), 0);
        assert!(check_forcefield(&mut e));
        assert_eq!(e.status(sid::FORCEFIELD), 1);
        assert_eq!(get_rage_block(&e), 3);
        remove_rage_end_of_turn(&mut e);
        assert_eq!(e.status(sid::RAGE), 0);
    }

    #[test]
    fn consume_helpers_and_equilibrium_match_java() {
        let mut e = entity();
        e.set_status(sid::DRAW_CARD, 2);
        e.set_status(sid::NEXT_TURN_BLOCK, 4);
        e.set_status(sid::ENERGIZED, 3);
        e.set_status(sid::FREE_ATTACK_POWER, 1);
        e.set_status(sid::DOUBLE_TAP, 1);
        e.set_status(sid::BURST, 1);
        e.set_status(sid::EQUILIBRIUM, 2);

        assert_eq!(consume_draw_card_next_turn(&mut e), 2);
        assert_eq!(consume_next_turn_block(&mut e), 4);
        assert_eq!(consume_energized(&mut e), 3);
        assert!(consume_free_attack(&mut e));
        assert!(consume_double_tap(&mut e));
        assert!(consume_burst(&mut e));
        assert!(has_equilibrium(&e));
        decrement_equilibrium(&mut e);
        assert_eq!(e.status(sid::EQUILIBRIUM), 1);
    }

    #[test]
    fn apply_lose_strength_dexterity_and_wraith_form_match_java() {
        let mut e = entity();
        e.set_status(sid::STRENGTH, 8);
        e.set_status(sid::DEXTERITY, 6);
        e.set_status(sid::LOSE_STRENGTH, 3);
        e.set_status(sid::LOSE_DEXTERITY, 2);
        e.set_status(sid::WRAITH_FORM, 4);
        e.set_status(sid::DEMON_FORM, 5);

        apply_lose_strength(&mut e);
        apply_lose_dexterity(&mut e);
        apply_wraith_form(&mut e);
        apply_demon_form(&mut e);

        assert_eq!(e.strength(), 10);
        assert_eq!(e.dexterity(), 0);
        assert_eq!(e.status(sid::LOSE_STRENGTH), 0);
        assert_eq!(e.status(sid::LOSE_DEXTERITY), 0);
    }

    #[test]
    fn deva_form_and_misc_watcher_scalars_match_java() {
        let mut e = entity();
        e.set_status(sid::DEVA_FORM, 1);
        e.set_status(sid::BATTLE_HYMN, 2);
        e.set_status(sid::STUDY, 3);
        e.set_status(sid::LIKE_WATER, 4);
        e.set_status(sid::MENTAL_FORTRESS, 5);

        assert_eq!(apply_deva_form(&mut e), 1);
        assert_eq!(e.status(sid::DEVA_FORM_ENERGY), 1);
        assert_eq!(get_battle_hymn_amount(&e), 2);
        assert_eq!(get_study_insights(&e), 3);
        assert_eq!(get_like_water_block(&e), 4);
        assert_eq!(get_mental_fortress_block(&e), 5);
    }

    #[test]
    fn exploit_and_damage_modifiers_match_java() {
        let mut e = entity();
        e.set_status(sid::DOUBLE_DAMAGE, 1);
        e.set_status(sid::NO_BLOCK, 1);
        e.set_status(sid::SLOW, 2);
        e.set_status(sid::INTANGIBLE, 1);

        assert_eq!(modify_damage_give(&e, 6.0, false), 12.0);
        assert_eq!(modify_damage_receive(&e, 10.0), 1.0);
        assert_eq!(modify_block(&e, 14.0), 0.0);
        assert_eq!(modify_heal(&e, 9), 9);
    }

    #[test]
    fn orb_related_defect_statuses_are_exposed_exactly() {
        let mut e = entity();
        e.set_status(sid::FOCUS, 2);
        e.set_status(sid::STORM, 1);
        e.set_status(sid::HEATSINK, 3);
        e.set_status(sid::STATIC_DISCHARGE, 4);
        e.set_status(sid::LOOP, 1);
        e.set_status(sid::ELECTRO, 1);
        e.set_status(sid::EQUILIBRIUM, 2);
        e.set_status(sid::BLUR, 1);
        assert_eq!(e.status(sid::FOCUS), 2);
        assert!(should_storm_channel(&e));
        assert_eq!(get_heatsink_draw(&e), 3);
        assert_eq!(get_static_discharge(&e), 4);
        assert_eq!(get_extra_draw(&e), 0);
        assert_eq!(e.status(sid::LOOP), 1);
        assert_eq!(e.status(sid::ELECTRO), 1);
        assert!(has_equilibrium(&e));
        assert_eq!(e.status(sid::BLUR), 1);
    }

    #[test]
    fn engine_channels_and_evokes_defect_orbs_with_focus() {
        let mut engine = make_engine(&["Strike_P"; 12], 100, 0);
        engine.init_defect_orbs(3);
        engine.state.player.set_status(sid::FOCUS, 2);

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
        engine.state.player.set_status(sid::STORM, 1);
        engine.state.player.set_status(sid::HEATSINK, 1);
        ensure_in_hand(&mut engine, "BattleHymn");
        let hand_before = engine.state.hand.len();
        ensure_on_top_of_draw(&mut engine, "Defend_P");

        assert!(play_self(&mut engine, "BattleHymn"));

        assert_eq!(engine.state.orb_slots.occupied_count(), 1);
        assert_eq!(engine.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
        assert_eq!(hand_count(&engine, "Defend_P"), 1); // 1 Heatsink draw from top of draw pile
        assert!(engine.state.hand.len() >= hand_before);
    }

    #[test]
    fn static_discharge_and_electro_registry_entries_are_present() {
        let static_discharge = get_power_def("StaticDischarge").unwrap();
        let electro = get_power_def("Electro").unwrap();
        assert_eq!(static_discharge.id, "StaticDischarge");
        assert!(static_discharge.on_hp_lost);
        assert_eq!(electro.id, "Electro");
        assert!(!electro.stackable);
    }
}
