#[cfg(test)]
mod relic_java_parity_tests {
    // Java references:
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Vajra.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/OddlySmoothStone.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/DataDisk.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Akabeko.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/BagOfMarbles.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/RedMask.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/ThreadAndNeedle.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/BronzeScales.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Anchor.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/BloodVial.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/ClockworkSouvenir.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/FossilizedHelix.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/MarkOfPain.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/MutagenicStrength.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/PureWater.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/HolyWater.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/NinjaScroll.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/BagOfPreparation.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/RingOfTheSnake.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/PhilosopherStone.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/PenNib.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/OrnamentalFan.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Kunai.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Shuriken.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/LetterOpener.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Nunchaku.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/InkBottle.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/VelvetChoker.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Pocketwatch.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/ArtOfWar.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/BirdFacedUrn.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/MummifiedHand.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/OrangePellets.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/SneckoEye.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/HappyFlower.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/IncenseBurner.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/MercuryHourglass.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Brimstone.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Damaru.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Inserter.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/HornCleat.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/CaptainsWheel.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/StoneCalendar.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Orichalcum.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/CloakClasp.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/FrozenCore.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/CharonsAshes.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/ToughBandages.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Tingsha.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/ToyOrnithopter.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/HandDrill.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/StrikeDummy.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/WristBlade.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/SneckoSkull.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Boot.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Torii.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/TungstenRod.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/ChemicalX.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/GoldenCables.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/RunicPyramid.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/IceCream.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/SacredBark.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Necronomicon.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/CentennialPuzzle.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/SelfFormingClay.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/RunicCube.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/RedSkull.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/EmotionChip.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/GremlinHorn.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/TheSpecimen.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/BurningBlood.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/BlackBlood.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/MeatOnTheBone.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/FaceOfCleric.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/GremlinMask.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Pantograph.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Sling.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/PreservedInsect.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/NeowsLament.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/DuVuDoll.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/Girya.java
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/relics/TeardropLocket.java

    use crate::cards::CardType;
    use crate::relics::*;
    use crate::state::{CombatState, EnemyCombatState, Stance};

    fn base_state() -> CombatState {
        let enemy = EnemyCombatState::new("JawWorm", 50, 50);
        CombatState::new(80, 80, vec![enemy], vec!["Strike_P".to_string(); 5], 3)
    }

    fn two_enemy_state() -> CombatState {
        let e1 = EnemyCombatState::new("JawWorm", 40, 40);
        let e2 = EnemyCombatState::new("Cultist", 50, 50);
        CombatState::new(80, 80, vec![e1, e2], vec!["Strike_P".to_string(); 5], 3)
    }

    fn state_with_relic(relic: &str) -> CombatState {
        let mut state = base_state();
        state.relics.push(relic.to_string());
        state
    }

    fn state_with_enemies(relic: &str, enemies: Vec<EnemyCombatState>) -> CombatState {
        let mut state = CombatState::new(80, 80, enemies, vec!["Strike_P".to_string(); 5], 3);
        state.relics.push(relic.to_string());
        state
    }

    fn start_with(relic: &str) -> CombatState {
        let mut state = state_with_relic(relic);
        apply_combat_start_relics(&mut state);
        state
    }

    fn turn_start(state: &mut CombatState, turn: i32) {
        state.turn = turn;
        apply_turn_start_relics(state);
    }

    fn turn_end(state: &mut CombatState, turn: i32) {
        state.turn = turn;
        apply_turn_end_relics(state);
    }

    fn hand(cards: &[&str]) -> Vec<String> {
        cards.iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn vajra_grants_one_strength() {
        let state = start_with("Vajra");
        assert_eq!(state.player.strength(), 1);
    }

    #[test]
    fn oddly_smooth_stone_grants_one_dexterity() {
        let state = start_with("Oddly Smooth Stone");
        assert_eq!(state.player.dexterity(), 1);
    }

    #[test]
    fn data_disk_grants_one_focus() {
        let state = start_with("Data Disk");
        assert_eq!(state.player.status("Focus"), 1);
    }

    #[test]
    fn akabeko_grants_eight_vigor() {
        let state = start_with("Akabeko");
        assert_eq!(state.player.status("Vigor"), 8);
    }

    #[test]
    fn bag_of_marbles_hits_every_enemy() {
        let state = start_with("Bag of Marbles");
        assert!(state.enemies.iter().all(|e| e.entity.is_vulnerable()));
        assert_eq!(state.enemies[0].entity.status("Vulnerable"), 1);
    }

    #[test]
    fn red_mask_weakens_every_enemy() {
        let state = start_with("Red Mask");
        assert!(state.enemies.iter().all(|e| e.entity.is_weak()));
    }

    #[test]
    fn thread_and_needle_grants_four_plated_armor() {
        let state = start_with("Thread and Needle");
        assert_eq!(state.player.status("Plated Armor"), 4);
    }

    #[test]
    fn bronze_scales_grants_three_thorns() {
        let state = start_with("Bronze Scales");
        assert_eq!(state.player.status("Thorns"), 3);
    }

    #[test]
    fn anchor_grants_ten_block() {
        let state = start_with("Anchor");
        assert_eq!(state.player.block, 10);
    }

    #[test]
    fn blood_vial_heals_two_at_combat_start() {
        let mut state = base_state();
        state.player.hp = 70;
        state.relics.push("Blood Vial".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.hp, 72);
    }

    #[test]
    fn clockwork_souvenir_grants_artifact_one() {
        let state = start_with("Clockwork Souvenir");
        assert_eq!(state.player.status("Artifact"), 1);
    }

    #[test]
    fn fossilized_helix_grants_buffer_one() {
        let state = start_with("Fossilized Helix");
        assert_eq!(state.player.status("Buffer"), 1);
    }

    #[test]
    fn mark_of_pain_adds_two_wounds_to_draw_pile() {
        let state = start_with("Mark of Pain");
        let wound_count = state.draw_pile.iter().filter(|c| *c == "Wound").count();
        assert_eq!(wound_count, 2);
    }

    #[test]
    fn mutagenic_strength_adds_three_strength_and_loses_it_later() {
        let state = start_with("MutagenicStrength");
        assert_eq!(state.player.strength(), 3);
        assert_eq!(state.player.status("LoseStrength"), 3);
    }

    #[test]
    fn pure_water_adds_miracle_to_hand() {
        let state = start_with("PureWater");
        assert_eq!(state.hand, vec!["Miracle".to_string()]);
    }

    #[test]
    fn holy_water_adds_three_cards_and_caps_at_ten() {
        let mut state = base_state();
        state.hand = hand(&["Strike_P"; 9]);
        state.relics.push("HolyWater".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.hand.len(), 10);
        assert_eq!(state.hand.last().unwrap(), "HolyWater");
    }

    #[test]
    fn ninja_scroll_adds_three_shivs() {
        let state = start_with("Ninja Scroll");
        assert_eq!(state.hand, vec!["Shiv".to_string(), "Shiv".to_string(), "Shiv".to_string()]);
    }

    #[test]
    fn bag_of_preparation_sets_turn_one_extra_draw() {
        let mut state = start_with("Bag of Preparation");
        turn_start(&mut state, 1);
        assert_eq!(state.player.status("TurnStartExtraDraw"), 2);
        assert_eq!(state.player.status("BagOfPrepDraw"), 0);
    }

    #[test]
    fn ring_of_the_snake_sets_turn_one_extra_draw() {
        let mut state = start_with("Ring of the Snake");
        turn_start(&mut state, 1);
        assert_eq!(state.player.status("TurnStartExtraDraw"), 2);
        assert_eq!(state.player.status("BagOfPrepDraw"), 0);
    }

    #[test]
    fn philosopher_stone_grants_enemy_strength() {
        let state = start_with("Philosopher's Stone");
        assert_eq!(state.enemies[0].entity.strength(), 1);
    }

    #[test]
    fn pen_nib_initializes_counter() {
        let state = start_with("Pen Nib");
        assert_eq!(state.player.status("PenNibCounter"), 0);
    }

    #[test]
    fn ornamental_fan_initializes_counter() {
        let state = start_with("Ornamental Fan");
        assert_eq!(state.player.status("OrnamentalFanCounter"), 0);
    }

    #[test]
    fn kunai_initializes_counter() {
        let state = start_with("Kunai");
        assert_eq!(state.player.status("KunaiCounter"), 0);
    }

    #[test]
    fn shuriken_initializes_counter() {
        let state = start_with("Shuriken");
        assert_eq!(state.player.status("ShurikenCounter"), 0);
    }

    #[test]
    fn letter_opener_initializes_counter() {
        let state = start_with("Letter Opener");
        assert_eq!(state.player.status("LetterOpenerCounter"), 0);
    }

    #[test]
    fn happy_flower_initializes_counter() {
        let state = start_with("Happy Flower");
        assert_eq!(state.player.status("HappyFlowerCounter"), 0);
    }

    #[test]
    fn incense_burner_initializes_counter() {
        let state = start_with("Incense Burner");
        assert_eq!(state.player.status("IncenseBurnerCounter"), 0);
    }

    #[test]
    fn horn_cleat_initializes_counter() {
        let state = start_with("HornCleat");
        assert_eq!(state.player.status("HornCleatCounter"), 0);
    }

    #[test]
    fn captains_wheel_initializes_counter() {
        let state = start_with("CaptainsWheel");
        assert_eq!(state.player.status("CaptainsWheelCounter"), 0);
    }

    #[test]
    fn stone_calendar_initializes_counter() {
        let state = start_with("StoneCalendar");
        assert_eq!(state.player.status("StoneCalendarCounter"), 0);
    }

    #[test]
    fn velvet_choker_initializes_counter() {
        let state = start_with("Velvet Choker");
        assert_eq!(state.player.status("VelvetChokerCounter"), 0);
    }

    #[test]
    fn pocketwatch_initializes_counter() {
        let state = start_with("Pocketwatch");
        assert_eq!(state.player.status("PocketwatchCounter"), 0);
        assert_eq!(state.player.status("PocketwatchFirstTurn"), 1);
    }

    #[test]
    fn violet_lotus_sets_flag() {
        let state = start_with("Violet Lotus");
        assert_eq!(state.player.status("VioletLotus"), 1);
    }

    #[test]
    fn emotion_chip_sets_flag() {
        let state = start_with("EmotionChip");
        assert_eq!(state.player.status("EmotionChipReady"), 1);
    }

    #[test]
    fn centennial_puzzle_sets_flag() {
        let state = start_with("CentennialPuzzle");
        assert_eq!(state.player.status("CentennialPuzzleReady"), 1);
    }

    #[test]
    fn art_of_war_sets_flag() {
        let state = start_with("Art of War");
        assert_eq!(state.player.status("ArtOfWarReady"), 1);
    }

    #[test]
    fn twisted_funnel_applies_four_poison() {
        let state = start_with("TwistedFunnel");
        assert_eq!(state.enemies[0].entity.status("Poison"), 4);
    }

    #[test]
    fn snecko_eye_sets_draw_and_cost_flag() {
        let state = start_with("Snecko Eye");
        assert_eq!(state.player.status("SneckoEye"), 1);
        assert_eq!(state.player.status("BagOfPrepDraw"), 2);
    }

    #[test]
    fn sling_elite_flag_grants_two_strength() {
        let mut state = base_state();
        state.player.set_status("SlingElite", 1);
        state.relics.push("Sling".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.strength(), 2);
    }

    #[test]
    fn preserved_insect_elite_reduces_highest_hp_enemy() {
        let e1 = EnemyCombatState::new("JawWorm", 20, 20);
        let e2 = EnemyCombatState::new("Cultist", 40, 40);
        let mut state = state_with_enemies("PreservedInsect", vec![e1, e2]);
        state.player.set_status("PreservedInsectElite", 1);
        apply_combat_start_relics(&mut state);
        assert_eq!(state.enemies[0].entity.hp, 20);
        assert_eq!(state.enemies[1].entity.hp, 30);
    }

    #[test]
    fn neows_blessing_sets_enemies_to_one_hp() {
        let mut state = base_state();
        state.relics.push("NeowsBlessing".to_string());
        state.player.set_status("NeowsLamentCounter", 3);
        apply_combat_start_relics(&mut state);
        assert_eq!(state.enemies[0].entity.hp, 1);
        assert_eq!(state.player.status("NeowsLamentCounter"), 2);
    }

    #[test]
    fn du_vu_doll_grants_strength_per_curse() {
        let mut state = base_state();
        state.relics.push("Du-Vu Doll".to_string());
        state.player.set_status("DuVuDollCurses", 4);
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.strength(), 4);
    }

    #[test]
    fn girya_grants_strength_per_lift() {
        let mut state = base_state();
        state.relics.push("Girya".to_string());
        state.player.set_status("GiryaCounter", 2);
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.strength(), 2);
    }

    #[test]
    fn red_skull_triggers_below_half_hp() {
        let mut state = base_state();
        state.player.hp = 40;
        state.relics.push("Red Skull".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.strength(), 3);
        assert_eq!(state.player.status("RedSkullActive"), 1);
    }

    #[test]
    fn teardrop_locket_starts_in_calm() {
        let state = start_with("TeardropLocket");
        assert_eq!(state.stance, Stance::Calm);
    }

    #[test]
    fn orange_pellets_clears_type_tracking_at_combat_start() {
        let mut state = base_state();
        state.player.set_status("OPAttack", 1);
        state.player.set_status("OPSkill", 1);
        state.player.set_status("OPPower", 1);
        state.relics.push("OrangePellets".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.status("OPAttack"), 0);
        assert_eq!(state.player.status("OPSkill"), 0);
        assert_eq!(state.player.status("OPPower"), 0);
    }

    #[test]
    fn pantograph_heals_boss_fight() {
        let mut state = base_state();
        state.player.hp = 50;
        state.enemies[0] = EnemyCombatState::new("Hexaghost", 250, 250);
        state.relics.push("Pantograph".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.hp, 75);
    }

    #[test]
    fn lantern_gives_one_energy_on_first_turn() {
        let mut state = start_with("Lantern");
        turn_start(&mut state, 1);
        assert_eq!(state.energy, 4);
        assert_eq!(state.player.status("LanternReady"), 0);
    }

    #[test]
    fn bag_of_preparation_grants_two_extra_draw_on_first_turn() {
        let mut state = start_with("Bag of Preparation");
        turn_start(&mut state, 1);
        assert_eq!(state.player.status("TurnStartExtraDraw"), 2);
    }

    #[test]
    fn ring_of_the_snake_grants_two_extra_draw_on_first_turn() {
        let mut state = start_with("Ring of the Snake");
        turn_start(&mut state, 1);
        assert_eq!(state.player.status("TurnStartExtraDraw"), 2);
    }

    #[test]
    fn happy_flower_grants_energy_every_third_turn() {
        let mut state = start_with("Happy Flower");
        turn_start(&mut state, 1);
        assert_eq!(state.energy, 3);
        turn_start(&mut state, 2);
        assert_eq!(state.energy, 3);
        turn_start(&mut state, 3);
        assert_eq!(state.energy, 4);
        assert_eq!(state.player.status("HappyFlowerCounter"), 0);
    }

    #[test]
    fn incense_burner_grants_intangible_every_sixth_turn() {
        let mut state = start_with("Incense Burner");
        for turn in 1..=5 {
            turn_start(&mut state, turn);
            assert_eq!(state.player.status("Intangible"), 0);
        }
        turn_start(&mut state, 6);
        assert_eq!(state.player.status("Intangible"), 1);
        assert_eq!(state.player.status("IncenseBurnerCounter"), 0);
    }

    #[test]
    fn mercury_hourglass_deals_three_to_each_enemy() {
        let mut state = two_enemy_state();
        state.relics.push("Mercury Hourglass".to_string());
        let hp0 = state.enemies[0].entity.hp;
        let hp1 = state.enemies[1].entity.hp;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp0 - 3);
        assert_eq!(state.enemies[1].entity.hp, hp1 - 3);
    }

    #[test]
    fn brimstone_grants_strength_to_player_and_enemies() {
        let mut state = two_enemy_state();
        state.relics.push("Brimstone".to_string());
        apply_turn_start_relics(&mut state);
        assert_eq!(state.player.strength(), 2);
        assert_eq!(state.enemies[0].entity.strength(), 1);
        assert_eq!(state.enemies[1].entity.strength(), 1);
    }

    #[test]
    fn damaru_increments_mantra() {
        let mut state = start_with("Damaru");
        turn_start(&mut state, 1);
        assert_eq!(state.mantra, 1);
        assert_eq!(state.mantra_gained, 1);
    }

    #[test]
    fn damaru_enters_divinity_at_ten() {
        let mut state = start_with("Damaru");
        state.mantra = 9;
        turn_start(&mut state, 2);
        assert_eq!(state.mantra, 0);
        assert_eq!(state.mantra_gained, 1);
        assert_eq!(state.player.status("EnterDivinity"), 1);
    }

    #[test]
    fn inserter_adds_orb_slot_on_second_turn() {
        let mut state = start_with("Inserter");
        state.player.set_status("InserterCounter", 1);
        turn_start(&mut state, 2);
        assert_eq!(state.player.status("OrbSlots"), 1);
        assert_eq!(state.player.status("InserterCounter"), 0);
    }

    #[test]
    fn horn_cleat_grants_fourteen_block_on_second_turn() {
        let mut state = start_with("HornCleat");
        state.player.set_status("HornCleatCounter", 1);
        turn_start(&mut state, 2);
        assert_eq!(state.player.block, 14);
        assert_eq!(state.player.status("HornCleatCounter"), -1);
    }

    #[test]
    fn captains_wheel_grants_eighteen_block_on_third_turn() {
        let mut state = start_with("CaptainsWheel");
        state.player.set_status("CaptainsWheelCounter", 2);
        turn_start(&mut state, 3);
        assert_eq!(state.player.block, 18);
        assert_eq!(state.player.status("CaptainsWheelCounter"), -1);
    }

    #[test]
    fn stone_calendar_deals_fifty_two_on_seventh_end_only_once() {
        let mut state = state_with_enemies(
            "StoneCalendar",
            vec![EnemyCombatState::new("JawWorm", 80, 80)],
        );
        state.player.set_status("StoneCalendarCounter", 7);
        let hp = state.enemies[0].entity.hp;
        turn_end(&mut state, 7);
        assert_eq!(state.enemies[0].entity.hp, hp - 52);
        let hp_after = state.enemies[0].entity.hp;
        turn_end(&mut state, 8);
        assert_eq!(state.enemies[0].entity.hp, hp_after);
    }

    #[test]
    fn pocketwatch_adds_three_draw_when_short_turn() {
        let mut state = start_with("Pocketwatch");
        state.player.set_status("PocketwatchFirstTurn", 0);
        state.player.set_status("PocketwatchCounter", 3);
        turn_start(&mut state, 2);
        assert_eq!(state.player.status("TurnStartExtraDraw"), 3);
        assert_eq!(state.player.status("PocketwatchCounter"), 0);
    }

    #[test]
    fn art_of_war_grants_energy_after_attackless_turn() {
        let mut state = start_with("Art of War");
        state.player.set_status("ArtOfWarReady", 1);
        turn_start(&mut state, 2);
        assert_eq!(state.energy, 4);
        assert_eq!(state.player.status("ArtOfWarReady"), 1);
    }

    #[test]
    fn art_of_war_clears_on_attack_play() {
        let mut state = start_with("Art of War");
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.status("ArtOfWarReady"), 0);
    }

    #[test]
    fn kunai_grants_dexterity_every_three_attacks() {
        let mut state = start_with("Kunai");
        for _ in 0..2 {
            on_card_played(&mut state, CardType::Attack);
        }
        assert_eq!(state.player.dexterity(), 0);
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.dexterity(), 1);
        assert_eq!(state.player.status("KunaiCounter"), 0);
    }

    #[test]
    fn shuriken_grants_strength_every_three_attacks() {
        let mut state = start_with("Shuriken");
        for _ in 0..2 {
            on_card_played(&mut state, CardType::Attack);
        }
        assert_eq!(state.player.strength(), 0);
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.strength(), 1);
        assert_eq!(state.player.status("ShurikenCounter"), 0);
    }

    #[test]
    fn letter_opener_deals_five_to_all_enemies_every_three_skills() {
        let mut state = two_enemy_state();
        state.relics.push("Letter Opener".to_string());
        apply_combat_start_relics(&mut state);
        let hp0 = state.enemies[0].entity.hp;
        let hp1 = state.enemies[1].entity.hp;
        on_card_played(&mut state, CardType::Skill);
        on_card_played(&mut state, CardType::Skill);
        on_card_played(&mut state, CardType::Skill);
        assert_eq!(state.enemies[0].entity.hp, hp0 - 5);
        assert_eq!(state.enemies[1].entity.hp, hp1 - 5);
    }

    #[test]
    fn nunchaku_grants_energy_every_ten_attacks() {
        let mut state = start_with("Nunchaku");
        for _ in 0..9 {
            on_card_played(&mut state, CardType::Attack);
        }
        assert_eq!(state.energy, 3);
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.energy, 4);
        assert_eq!(state.player.status("NunchakuCounter"), 0);
    }

    #[test]
    fn ink_bottle_sets_draw_flag_every_ten_cards() {
        let mut state = start_with("InkBottle");
        for _ in 0..9 {
            on_card_played(&mut state, CardType::Attack);
        }
        assert_eq!(state.player.status("InkBottleDraw"), 0);
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.status("InkBottleDraw"), 1);
        assert_eq!(state.player.status("InkBottleCounter"), 0);
    }

    #[test]
    fn orichalcum_grants_six_block_if_you_end_with_zero() {
        let mut state = start_with("Orichalcum");
        state.player.block = 0;
        turn_end(&mut state, 1);
        assert_eq!(state.player.block, 6);
    }

    #[test]
    fn cloak_clasp_grants_block_equal_to_hand_size() {
        let mut state = start_with("CloakClasp");
        state.hand = hand(&["Strike_P", "Defend_P", "Strike_P"]);
        turn_end(&mut state, 1);
        assert_eq!(state.player.block, 3);
    }

    #[test]
    fn frozen_core_sets_trigger_flag_at_turn_end() {
        let mut state = start_with("FrozenCore");
        turn_end(&mut state, 1);
        assert_eq!(state.player.status("FrozenCoreTrigger"), 1);
    }

    #[test]
    fn velvet_choker_allows_six_cards_but_not_seven() {
        let mut state = start_with("Velvet Choker");
        state.player.set_status("VelvetChokerCounter", 5);
        assert!(velvet_choker_can_play(&state));
        state.player.set_status("VelvetChokerCounter", 6);
        assert!(!velvet_choker_can_play(&state));
    }

    #[test]
    fn ornamental_fan_grants_four_block_every_three_attacks() {
        let mut state = start_with("Ornamental Fan");
        for _ in 0..2 {
            on_card_played(&mut state, CardType::Attack);
        }
        assert_eq!(state.player.block, 0);
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.block, 4);
        assert_eq!(state.player.status("OrnamentalFanCounter"), 0);
    }

    #[test]
    fn pen_nib_triggers_on_tenth_attack() {
        let mut state = start_with("Pen Nib");
        for _ in 0..9 {
            assert!(!check_pen_nib(&mut state));
        }
        assert!(check_pen_nib(&mut state));
        assert_eq!(state.player.status("PenNibCounter"), 0);
    }

    #[test]
    fn bird_faced_urn_heals_on_power_play() {
        let mut state = start_with("Bird Faced Urn");
        state.player.hp = 70;
        on_card_played(&mut state, CardType::Power);
        assert_eq!(state.player.hp, 72);
    }

    #[test]
    fn mummified_hand_sets_flag_on_power_play() {
        let mut state = start_with("Mummified Hand");
        on_card_played(&mut state, CardType::Power);
        assert_eq!(state.player.status("MummifiedHandTrigger"), 1);
    }

    #[test]
    fn yang_grants_dexterity_and_temporary_loss_on_attack() {
        let mut state = start_with("Yang");
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.dexterity(), 1);
        assert_eq!(state.player.status("LoseDexterity"), 1);
    }

    #[test]
    fn orange_pellets_clears_debuffs_after_attack_skill_power() {
        let mut state = start_with("OrangePellets");
        state.player.set_status("Weakened", 2);
        state.player.set_status("Vulnerable", 2);
        state.player.set_status("Frail", 2);
        state.player.set_status("Entangled", 1);
        state.player.set_status("No Draw", 1);
        on_card_played(&mut state, CardType::Attack);
        on_card_played(&mut state, CardType::Skill);
        on_card_played(&mut state, CardType::Power);
        assert_eq!(state.player.status("Weakened"), 0);
        assert_eq!(state.player.status("Vulnerable"), 0);
        assert_eq!(state.player.status("Frail"), 0);
        assert_eq!(state.player.status("Entangled"), 0);
        assert_eq!(state.player.status("No Draw"), 0);
    }

    #[test]
    fn dead_branch_returns_true_when_owned() {
        let mut state = base_state();
        state.relics.push("Dead Branch".to_string());
        assert!(dead_branch_on_exhaust(&state));
        state.relics.clear();
        assert!(!dead_branch_on_exhaust(&state));
    }

    #[test]
    fn charons_ashes_deals_three_to_all_enemies_on_exhaust() {
        let mut state = two_enemy_state();
        state.relics.push("Charon's Ashes".to_string());
        let hp0 = state.enemies[0].entity.hp;
        let hp1 = state.enemies[1].entity.hp;
        charons_ashes_on_exhaust(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp0 - 3);
        assert_eq!(state.enemies[1].entity.hp, hp1 - 3);
    }

    #[test]
    fn tough_bandages_give_three_block_on_discard() {
        let mut state = base_state();
        state.relics.push("Tough Bandages".to_string());
        tough_bandages_on_discard(&mut state);
        assert_eq!(state.player.block, 3);
    }

    #[test]
    fn tingsha_deals_three_to_first_alive_enemy_on_discard() {
        let mut state = base_state();
        state.relics.push("Tingsha".to_string());
        let hp = state.enemies[0].entity.hp;
        tingsha_on_discard(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp - 3);
    }

    #[test]
    fn toy_ornithopter_heals_five_on_potion() {
        let mut state = base_state();
        state.relics.push("Toy Ornithopter".to_string());
        state.player.hp = 70;
        toy_ornithopter_on_potion(&mut state);
        assert_eq!(state.player.hp, 75);
    }

    #[test]
    fn hand_drill_applies_two_vulnerable_when_block_break() {
        let mut state = base_state();
        state.relics.push("HandDrill".to_string());
        hand_drill_on_block_break(&mut state, 0);
        assert_eq!(state.enemies[0].entity.status("Vulnerable"), 2);
    }

    #[test]
    fn strike_dummy_bonus_is_three() {
        let mut state = base_state();
        assert_eq!(strike_dummy_bonus(&state), 0);
        state.relics.push("StrikeDummy".to_string());
        assert_eq!(strike_dummy_bonus(&state), 3);
    }

    #[test]
    fn wrist_blade_bonus_is_four() {
        let mut state = base_state();
        assert_eq!(wrist_blade_bonus(&state), 0);
        state.relics.push("WristBlade".to_string());
        assert_eq!(wrist_blade_bonus(&state), 4);
    }

    #[test]
    fn snecko_skull_bonus_is_one() {
        let mut state = base_state();
        assert_eq!(snecko_skull_bonus(&state), 0);
        state.relics.push("SneckoSkull".to_string());
        assert_eq!(snecko_skull_bonus(&state), 1);
    }

    #[test]
    fn apply_boot_raises_small_damage_to_five() {
        let mut state = base_state();
        state.relics.push("Boot".to_string());
        assert_eq!(apply_boot(&state, 3), 5);
        assert_eq!(apply_boot(&state, 0), 0);
        assert_eq!(apply_boot(&state, 7), 7);
    }

    #[test]
    fn apply_torii_reduces_small_damage_to_one() {
        let mut state = base_state();
        state.relics.push("Torii".to_string());
        assert_eq!(apply_torii(&state, 2), 1);
        assert_eq!(apply_torii(&state, 5), 1);
        assert_eq!(apply_torii(&state, 6), 6);
    }

    #[test]
    fn apply_tungsten_rod_reduces_hp_loss_by_one() {
        let mut state = base_state();
        state.relics.push("TungstenRod".to_string());
        assert_eq!(apply_tungsten_rod(&state, 5), 4);
        assert_eq!(apply_tungsten_rod(&state, 1), 0);
        assert_eq!(apply_tungsten_rod(&state, 0), 0);
    }

    #[test]
    fn chemical_x_bonus_is_two() {
        let mut state = base_state();
        assert_eq!(chemical_x_bonus(&state), 0);
        state.relics.push("Chemical X".to_string());
        assert_eq!(chemical_x_bonus(&state), 2);
    }

    #[test]
    fn gold_plated_cables_only_active_at_full_hp() {
        let mut state = base_state();
        state.relics.push("Cables".to_string());
        assert!(gold_plated_cables_active(&state));
        state.player.hp = 70;
        assert!(!gold_plated_cables_active(&state));
    }

    #[test]
    fn runic_pyramid_presence_check() {
        let mut state = base_state();
        state.relics.push("Runic Pyramid".to_string());
        assert!(has_runic_pyramid(&state));
    }

    #[test]
    fn ice_cream_presence_check() {
        let mut state = base_state();
        state.relics.push("Ice Cream".to_string());
        assert!(has_ice_cream(&state));
    }

    #[test]
    fn sacred_bark_presence_check() {
        let mut state = base_state();
        state.relics.push("SacredBark".to_string());
        assert!(has_sacred_bark(&state));
    }

    #[test]
    fn calipers_retains_up_to_fifteen_block() {
        let mut state = base_state();
        state.relics.push("Calipers".to_string());
        assert_eq!(calipers_block_retention(&state, 20), 15);
        assert_eq!(calipers_block_retention(&state, 10), 10);
    }

    #[test]
    fn unceasing_top_draws_when_hand_empty() {
        let mut state = base_state();
        state.relics.push("Unceasing Top".to_string());
        state.hand.clear();
        state.draw_pile.push("Strike_P".to_string());
        assert!(unceasing_top_should_draw(&state));
        state.hand.push("Defend_P".to_string());
        assert!(!unceasing_top_should_draw(&state));
    }

    #[test]
    fn necronomicon_triggers_once_for_first_two_cost_attack() {
        let mut state = base_state();
        state.relics.push("Necronomicon".to_string());
        assert!(necronomicon_should_trigger(&state, 2, true));
        assert!(!necronomicon_should_trigger(&state, 1, true));
        assert!(!necronomicon_should_trigger(&state, 2, false));
        necronomicon_mark_used(&mut state);
        assert!(!necronomicon_should_trigger(&state, 2, true));
    }

    #[test]
    fn necronomicon_reset_clears_flag() {
        let mut state = base_state();
        state.relics.push("Necronomicon".to_string());
        state.player.set_status("NecronomiconUsed", 1);
        necronomicon_reset(&mut state);
        assert_eq!(state.player.status("NecronomiconUsed"), 0);
    }

    #[test]
    fn on_hp_loss_centennial_puzzle_sets_draw_flag() {
        let mut state = base_state();
        state.relics.push("Centennial Puzzle".to_string());
        state.player.set_status("CentennialPuzzleReady", 1);
        on_hp_loss(&mut state, 5);
        assert_eq!(state.player.status("CentennialPuzzleReady"), 0);
        assert_eq!(state.player.status("CentennialPuzzleDraw"), 3);
    }

    #[test]
    fn on_hp_loss_self_forming_clay_sets_next_turn_block() {
        let mut state = base_state();
        state.relics.push("Self Forming Clay".to_string());
        on_hp_loss(&mut state, 5);
        assert_eq!(state.player.status("NextTurnBlock"), 3);
    }

    #[test]
    fn on_hp_loss_runic_cube_sets_draw_flag() {
        let mut state = base_state();
        state.relics.push("Runic Cube".to_string());
        on_hp_loss(&mut state, 5);
        assert_eq!(state.player.status("RunicCubeDraw"), 1);
    }

    #[test]
    fn on_hp_loss_red_skull_grants_strength_when_below_half() {
        let mut state = base_state();
        state.player.hp = 40;
        state.relics.push("Red Skull".to_string());
        on_hp_loss(&mut state, 5);
        assert_eq!(state.player.strength(), 3);
        assert_eq!(state.player.status("RedSkullActive"), 1);
    }

    #[test]
    fn on_hp_loss_emotion_chip_sets_trigger_flag() {
        let mut state = base_state();
        state.relics.push("EmotionChip".to_string());
        on_hp_loss(&mut state, 5);
        assert_eq!(state.player.status("EmotionChipTrigger"), 1);
    }

    #[test]
    fn on_shuffle_sundial_grants_two_energy_on_third_shuffle() {
        let mut state = base_state();
        state.relics.push("Sundial".to_string());
        on_shuffle(&mut state);
        assert_eq!(state.player.status("SundialCounter"), 1);
        on_shuffle(&mut state);
        assert_eq!(state.player.status("SundialCounter"), 2);
        on_shuffle(&mut state);
        assert_eq!(state.energy, 5);
        assert_eq!(state.player.status("SundialCounter"), 0);
    }

    #[test]
    fn on_shuffle_the_abacus_grants_six_block() {
        let mut state = base_state();
        state.relics.push("TheAbacus".to_string());
        on_shuffle(&mut state);
        assert_eq!(state.player.block, 6);
    }

    #[test]
    fn on_enemy_death_gremlin_horn_grants_energy_if_other_enemy_lives() {
        let mut state = two_enemy_state();
        state.relics.push("Gremlin Horn".to_string());
        let energy = state.energy;
        on_enemy_death(&mut state, 0);
        assert_eq!(state.energy, energy + 1);
        assert_eq!(state.player.status("GremlinHornDraw"), 1);
    }

    #[test]
    fn on_enemy_death_the_specimen_transfers_poison() {
        let mut state = two_enemy_state();
        state.relics.push("The Specimen".to_string());
        state.enemies[0].entity.add_status("Poison", 5);
        on_enemy_death(&mut state, 0);
        assert_eq!(state.enemies[1].entity.status("Poison"), 5);
    }

    #[test]
    fn on_victory_burning_blood_heals_six() {
        let mut state = base_state();
        state.relics.push("Burning Blood".to_string());
        assert_eq!(on_victory(&mut state), 6);
    }

    #[test]
    fn on_victory_black_blood_heals_twelve() {
        let mut state = base_state();
        state.relics.push("Black Blood".to_string());
        assert_eq!(on_victory(&mut state), 12);
    }

    #[test]
    fn on_victory_meat_on_the_bone_heals_at_half_or_below() {
        let mut state = base_state();
        state.player.hp = 40;
        state.relics.push("Meat on the Bone".to_string());
        assert_eq!(on_victory(&mut state), 12);
    }

    #[test]
    fn on_victory_face_of_cleric_increases_max_hp() {
        let mut state = base_state();
        state.relics.push("FaceOfCleric".to_string());
        assert_eq!(on_victory(&mut state), 0);
        assert_eq!(state.player.max_hp, 81);
    }

    macro_rules! extra_case {
        ($name:ident, $body:block) => {
            #[test]
            fn $name() $body
        };
    }

    extra_case!(oddly_smooth_stone_compact_name, {
        let state = start_with("OddlySmoothStone");
        assert_eq!(state.player.dexterity(), 1);
    });

    extra_case!(data_disk_compact_name, {
        let state = start_with("DataDisk");
        assert_eq!(state.player.status("Focus"), 1);
    });

    extra_case!(clockwork_souvenir_compact_name, {
        let state = start_with("ClockworkSouvenir");
        assert_eq!(state.player.status("Artifact"), 1);
    });

    extra_case!(fossilized_helix_compact_name, {
        let state = start_with("FossilizedHelix");
        assert_eq!(state.player.status("Buffer"), 1);
    });

    extra_case!(philosopher_stone_compact_name, {
        let state = start_with("PhilosophersStone");
        assert_eq!(state.enemies[0].entity.strength(), 1);
    });

    extra_case!(violet_lotus_compact_name, {
        let mut state = base_state();
        state.relics.push("VioletLotus".to_string());
        assert_eq!(violet_lotus_calm_exit_bonus(&state), 1);
    });

    extra_case!(ice_cream_compact_name, {
        let mut state = base_state();
        state.relics.push("IceCream".to_string());
        assert!(has_ice_cream(&state));
    });

    extra_case!(runic_pyramid_compact_name, {
        let mut state = base_state();
        state.relics.push("RunicPyramid".to_string());
        assert!(has_runic_pyramid(&state));
    });

    extra_case!(sacred_bark_negative, {
        let state = base_state();
        assert!(!has_sacred_bark(&state));
    });

    extra_case!(runic_pyramid_negative, {
        let state = base_state();
        assert!(!has_runic_pyramid(&state));
    });

    extra_case!(ice_cream_negative, {
        let state = base_state();
        assert!(!has_ice_cream(&state));
    });

    extra_case!(calipers_no_relic_returns_zero, {
        let state = base_state();
        assert_eq!(calipers_block_retention(&state, 20), 0);
    });

    extra_case!(calipers_zero_block_returns_zero, {
        let mut state = base_state();
        state.relics.push("Calipers".to_string());
        assert_eq!(calipers_block_retention(&state, 0), 0);
    });

    extra_case!(gold_plated_cables_no_relic, {
        let state = base_state();
        assert!(!gold_plated_cables_active(&state));
    });

    extra_case!(gold_plated_cables_not_full, {
        let mut state = base_state();
        state.relics.push("Cables".to_string());
        state.player.hp = 70;
        assert!(!gold_plated_cables_active(&state));
    });

    extra_case!(unceasing_top_no_relic, {
        let state = base_state();
        assert!(!unceasing_top_should_draw(&state));
    });

    extra_case!(unceasing_top_nonempty_hand, {
        let mut state = base_state();
        state.relics.push("Unceasing Top".to_string());
        state.hand.push("Defend_P".to_string());
        state.draw_pile.push("Strike_P".to_string());
        assert!(!unceasing_top_should_draw(&state));
    });

    extra_case!(chemical_x_no_relic, {
        let state = base_state();
        assert_eq!(chemical_x_bonus(&state), 0);
    });

    extra_case!(strike_dummy_no_relic, {
        let state = base_state();
        assert_eq!(strike_dummy_bonus(&state), 0);
    });

    extra_case!(wrist_blade_no_relic, {
        let state = base_state();
        assert_eq!(wrist_blade_bonus(&state), 0);
    });

    extra_case!(snecko_skull_no_relic, {
        let state = base_state();
        assert_eq!(snecko_skull_bonus(&state), 0);
    });

    extra_case!(boot_no_relic, {
        let state = base_state();
        assert_eq!(apply_boot(&state, 3), 3);
    });

    extra_case!(torii_no_relic, {
        let state = base_state();
        assert_eq!(apply_torii(&state, 3), 3);
    });

    extra_case!(tungsten_no_relic, {
        let state = base_state();
        assert_eq!(apply_tungsten_rod(&state, 5), 5);
    });

    extra_case!(boot_minimum_damage_is_five, {
        let mut state = base_state();
        state.relics.push("Boot".to_string());
        assert_eq!(apply_boot(&state, 4), 5);
    });

    extra_case!(boot_high_damage_unchanged, {
        let mut state = base_state();
        state.relics.push("Boot".to_string());
        assert_eq!(apply_boot(&state, 8), 8);
    });

    extra_case!(torii_one_damage_unchanged, {
        let mut state = base_state();
        state.relics.push("Torii".to_string());
        assert_eq!(apply_torii(&state, 1), 1);
    });

    extra_case!(torii_zero_damage_unchanged, {
        let mut state = base_state();
        state.relics.push("Torii".to_string());
        assert_eq!(apply_torii(&state, 0), 0);
    });

    extra_case!(tungsten_two_damage_reduces_by_one, {
        let mut state = base_state();
        state.relics.push("TungstenRod".to_string());
        assert_eq!(apply_tungsten_rod(&state, 2), 1);
    });

    extra_case!(tungsten_ten_damage_reduces_by_one, {
        let mut state = base_state();
        state.relics.push("TungstenRod".to_string());
        assert_eq!(apply_tungsten_rod(&state, 10), 9);
    });

    extra_case!(necronomicon_no_relic_false, {
        let state = base_state();
        assert!(!necronomicon_should_trigger(&state, 2, true));
    });

    extra_case!(necronomicon_non_attack_false, {
        let mut state = base_state();
        state.relics.push("Necronomicon".to_string());
        assert!(!necronomicon_should_trigger(&state, 2, false));
    });

    extra_case!(necronomicon_one_cost_attack_false, {
        let mut state = base_state();
        state.relics.push("Necronomicon".to_string());
        assert!(!necronomicon_should_trigger(&state, 1, true));
    });

    extra_case!(on_hp_loss_zero_damage_no_triggers, {
        let mut state = base_state();
        state.relics.push("Centennial Puzzle".to_string());
        state.player.set_status("CentennialPuzzleReady", 1);
        on_hp_loss(&mut state, 0);
        assert_eq!(state.player.status("CentennialPuzzleReady"), 1);
        assert_eq!(state.player.status("CentennialPuzzleDraw"), 0);
    });

    extra_case!(on_shuffle_no_relic_no_effect, {
        let mut state = base_state();
        let hp = state.player.hp;
        let block = state.player.block;
        on_shuffle(&mut state);
        assert_eq!(state.player.hp, hp);
        assert_eq!(state.player.block, block);
    });

    extra_case!(on_enemy_death_no_poison_no_transfer, {
        let mut state = two_enemy_state();
        state.relics.push("The Specimen".to_string());
        on_enemy_death(&mut state, 0);
        assert_eq!(state.enemies[1].entity.status("Poison"), 0);
    });

    extra_case!(on_victory_meat_above_half_no_heal, {
        let mut state = base_state();
        state.player.hp = 60;
        state.relics.push("Meat on the Bone".to_string());
        assert_eq!(on_victory(&mut state), 0);
    });

    extra_case!(happy_flower_turn2_no_energy, {
        let mut state = start_with("Happy Flower");
        turn_start(&mut state, 1);
        turn_start(&mut state, 2);
        assert_eq!(state.energy, 3);
    });

    extra_case!(incense_burner_turn5_no_intangible, {
        let mut state = start_with("Incense Burner");
        for turn in 1..=5 {
            turn_start(&mut state, turn);
        }
        assert_eq!(state.player.status("Intangible"), 0);
    });

    extra_case!(horn_cleat_turn1_no_block, {
        let mut state = start_with("HornCleat");
        turn_start(&mut state, 1);
        assert_eq!(state.player.block, 0);
    });

    extra_case!(captains_wheel_turn2_no_block, {
        let mut state = start_with("CaptainsWheel");
        state.player.set_status("CaptainsWheelCounter", 1);
        turn_start(&mut state, 2);
        assert_eq!(state.player.block, 0);
    });

    extra_case!(pocketwatch_first_turn_no_bonus, {
        let mut state = start_with("Pocketwatch");
        state.player.set_status("PocketwatchFirstTurn", 1);
        turn_start(&mut state, 1);
        assert_eq!(state.player.status("TurnStartExtraDraw"), 0);
        assert_eq!(state.player.status("PocketwatchFirstTurn"), 0);
    });

    extra_case!(art_of_war_turn1_no_energy, {
        let mut state = start_with("Art of War");
        state.player.set_status("ArtOfWarReady", 1);
        turn_start(&mut state, 1);
        assert_eq!(state.energy, 3);
    });

    extra_case!(inserter_turn1_no_orb_slot, {
        let mut state = start_with("Inserter");
        state.player.set_status("InserterCounter", 0);
        turn_start(&mut state, 1);
        assert_eq!(state.player.status("OrbSlots"), 0);
    });

    extra_case!(kunai_six_attacks_two_dex, {
        let mut state = start_with("Kunai");
        for _ in 0..6 {
            on_card_played(&mut state, CardType::Attack);
        }
        assert_eq!(state.player.dexterity(), 2);
    });

    extra_case!(shuriken_six_attacks_two_str, {
        let mut state = start_with("Shuriken");
        for _ in 0..6 {
            on_card_played(&mut state, CardType::Attack);
        }
        assert_eq!(state.player.strength(), 2);
    });

    extra_case!(letter_opener_six_skills_second_trigger, {
        let mut state = two_enemy_state();
        state.relics.push("Letter Opener".to_string());
        apply_combat_start_relics(&mut state);
        let hp0 = state.enemies[0].entity.hp;
        for _ in 0..6 {
            on_card_played(&mut state, CardType::Skill);
        }
        assert_eq!(state.enemies[0].entity.hp, hp0 - 10);
    });

    extra_case!(nunchaku_nine_attacks_no_energy, {
        let mut state = start_with("Nunchaku");
        for _ in 0..9 {
            on_card_played(&mut state, CardType::Attack);
        }
        assert_eq!(state.energy, 3);
    });

    extra_case!(ink_bottle_nine_cards_no_draw, {
        let mut state = start_with("InkBottle");
        for _ in 0..9 {
            on_card_played(&mut state, CardType::Attack);
        }
        assert_eq!(state.player.status("InkBottleDraw"), 0);
    });

    extra_case!(orichalcum_with_block_no_bonus, {
        let mut state = start_with("Orichalcum");
        state.player.block = 4;
        turn_end(&mut state, 1);
        assert_eq!(state.player.block, 4);
    });

    extra_case!(cloak_clasp_empty_hand_no_block, {
        let mut state = start_with("CloakClasp");
        state.hand.clear();
        turn_end(&mut state, 1);
        assert_eq!(state.player.block, 0);
    });

    extra_case!(stone_calendar_sixth_end_no_fire, {
        let mut state = base_state();
        state.relics.push("StoneCalendar".to_string());
        state.player.set_status("StoneCalendarCounter", 6);
        let hp = state.enemies[0].entity.hp;
        turn_end(&mut state, 6);
        assert_eq!(state.enemies[0].entity.hp, hp);
    });

    extra_case!(velvet_choker_counter_five_allowed, {
        let mut state = start_with("Velvet Choker");
        state.player.set_status("VelvetChokerCounter", 5);
        assert!(velvet_choker_can_play(&state));
    });

    extra_case!(velvet_choker_counter_six_blocked, {
        let mut state = start_with("Velvet Choker");
        state.player.set_status("VelvetChokerCounter", 6);
        assert!(!velvet_choker_can_play(&state));
    });

    extra_case!(pen_nib_ninth_attack_not_trigger, {
        let mut state = start_with("Pen Nib");
        for _ in 0..9 {
            assert!(!check_pen_nib(&mut state));
        }
    });

    extra_case!(bird_faced_urn_non_power_no_heal, {
        let mut state = start_with("Bird Faced Urn");
        state.player.hp = 70;
        on_card_played(&mut state, CardType::Skill);
        assert_eq!(state.player.hp, 70);
    });

    extra_case!(mummified_hand_non_power_no_flag, {
        let mut state = start_with("Mummified Hand");
        on_card_played(&mut state, CardType::Skill);
        assert_eq!(state.player.status("MummifiedHandTrigger"), 0);
    });

    extra_case!(yang_skill_no_dex, {
        let mut state = start_with("Yang");
        on_card_played(&mut state, CardType::Skill);
        assert_eq!(state.player.dexterity(), 0);
    });

    extra_case!(orange_pellets_one_type_does_not_clear, {
        let mut state = start_with("OrangePellets");
        state.player.set_status("Weakened", 1);
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.status("Weakened"), 1);
    });

    extra_case!(charons_ashes_no_relic_no_damage, {
        let mut state = base_state();
        let hp = state.enemies[0].entity.hp;
        charons_ashes_on_exhaust(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp);
    });

    extra_case!(tough_bandages_no_relic_no_block, {
        let mut state = base_state();
        tough_bandages_on_discard(&mut state);
        assert_eq!(state.player.block, 0);
    });

    extra_case!(tingsha_no_relic_no_damage, {
        let mut state = base_state();
        let hp = state.enemies[0].entity.hp;
        tingsha_on_discard(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp);
    });

    extra_case!(toy_ornithopter_no_relic_no_heal, {
        let mut state = base_state();
        state.player.hp = 70;
        toy_ornithopter_on_potion(&mut state);
        assert_eq!(state.player.hp, 70);
    });

    extra_case!(hand_drill_no_relic_no_vuln, {
        let mut state = base_state();
        hand_drill_on_block_break(&mut state, 0);
        assert_eq!(state.enemies[0].entity.status("Vulnerable"), 0);
    });

    extra_case!(on_shuffle_sundial_first_no_energy, {
        let mut state = base_state();
        state.relics.push("Sundial".to_string());
        on_shuffle(&mut state);
        assert_eq!(state.energy, 3);
    });

    extra_case!(on_shuffle_sundial_second_no_energy, {
        let mut state = base_state();
        state.relics.push("Sundial".to_string());
        on_shuffle(&mut state);
        on_shuffle(&mut state);
        assert_eq!(state.energy, 3);
    });

    extra_case!(on_shuffle_abacus_no_relic_no_block, {
        let mut state = base_state();
        on_shuffle(&mut state);
        assert_eq!(state.player.block, 0);
    });

    extra_case!(on_enemy_death_gremlin_horn_no_other_enemy_no_energy, {
        let mut state = base_state();
        state.relics.push("Gremlin Horn".to_string());
        state.enemies.clear();
        state.enemies.push(EnemyCombatState::new("JawWorm", 0, 50));
        on_enemy_death(&mut state, 0);
        assert_eq!(state.energy, 3);
    });

    extra_case!(red_skull_above_half_no_trigger, {
        let mut state = base_state();
        state.player.hp = 70;
        state.relics.push("Red Skull".to_string());
        on_hp_loss(&mut state, 5);
        assert_eq!(state.player.strength(), 0);
    });

    extra_case!(centennial_puzzle_zero_damage_no_flag, {
        let mut state = base_state();
        state.relics.push("Centennial Puzzle".to_string());
        state.player.set_status("CentennialPuzzleReady", 1);
        on_hp_loss(&mut state, 0);
        assert_eq!(state.player.status("CentennialPuzzleReady"), 1);
    });

    extra_case!(self_forming_clay_zero_damage_no_flag, {
        let mut state = base_state();
        state.relics.push("Self Forming Clay".to_string());
        on_hp_loss(&mut state, 0);
        assert_eq!(state.player.status("NextTurnBlock"), 0);
    });

    extra_case!(runic_cube_zero_damage_no_flag, {
        let mut state = base_state();
        state.relics.push("Runic Cube".to_string());
        on_hp_loss(&mut state, 0);
        assert_eq!(state.player.status("RunicCubeDraw"), 0);
    });

    extra_case!(emotion_chip_zero_damage_no_flag, {
        let mut state = base_state();
        state.relics.push("EmotionChip".to_string());
        on_hp_loss(&mut state, 0);
        assert_eq!(state.player.status("EmotionChipTrigger"), 0);
    });

    extra_case!(burning_blood_on_victory, {
        let mut state = base_state();
        state.relics.push("Burning Blood".to_string());
        assert_eq!(on_victory(&mut state), 6);
    });

    extra_case!(black_blood_on_victory, {
        let mut state = base_state();
        state.relics.push("Black Blood".to_string());
        assert_eq!(on_victory(&mut state), 12);
    });

    extra_case!(face_of_cleric_on_victory_max_hp_plus1, {
        let mut state = base_state();
        state.relics.push("FaceOfCleric".to_string());
        let heal = on_victory(&mut state);
        assert_eq!(heal, 0);
        assert_eq!(state.player.max_hp, 81);
    });

    extra_case!(sling_no_flag_no_strength, {
        let mut state = base_state();
        state.relics.push("Sling".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.strength(), 0);
    });

    extra_case!(preserved_insect_no_flag_no_damage, {
        let e1 = EnemyCombatState::new("JawWorm", 20, 20);
        let e2 = EnemyCombatState::new("Cultist", 40, 40);
        let mut state = state_with_enemies("PreservedInsect", vec![e1, e2]);
        apply_combat_start_relics(&mut state);
        assert_eq!(state.enemies[0].entity.hp, 20);
        assert_eq!(state.enemies[1].entity.hp, 40);
    });

    extra_case!(neows_blessing_no_counter_no_change, {
        let mut state = base_state();
        state.relics.push("NeowsBlessing".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.enemies[0].entity.hp, 50);
    });

    extra_case!(du_vu_doll_no_curses_no_strength, {
        let mut state = base_state();
        state.relics.push("Du-Vu Doll".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.strength(), 0);
    });

    extra_case!(girya_no_lifts_no_strength, {
        let mut state = base_state();
        state.relics.push("Girya".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.strength(), 0);
    });
}
