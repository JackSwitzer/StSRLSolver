#[cfg(test)]
mod defect_card_java_parity_tests {
    // Java references:
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/cards/blue/*.java

    use crate::cards::{CardRegistry, CardType};
    use crate::status_ids::sid;
    use crate::engine::{CombatEngine, CombatPhase};
    use crate::actions::Action;
    use crate::orbs::OrbType;
    use crate::powers::{process_end_of_round, process_end_of_turn, process_start_of_turn};
    use crate::state::EnemyCombatState;
    use crate::tests::support::*;

    macro_rules! defect_test {
        ($name:ident, $body:block) => {
            #[test]
            fn $name() $body
        };
    }

    #[derive(Clone, Copy)]
    struct StatCase {
        id: &'static str,
        cost: i32,
        damage: i32,
        block: i32,
        magic: i32,
        card_type: CardType,
        exhaust: bool,
    }

    fn reg() -> &'static CardRegistry {
        crate::cards::global_registry()
    }

    fn assert_stats(case: StatCase) {
        let registry = reg();
        let card = registry.get(case.id).unwrap();
        assert_eq!(card.cost, case.cost, "{} cost", case.id);
        assert_eq!(card.base_damage, case.damage, "{} damage", case.id);
        assert_eq!(card.base_block, case.block, "{} block", case.id);
        assert_eq!(card.base_magic, case.magic, "{} magic", case.id);
        assert_eq!(card.card_type, case.card_type, "{} type", case.id);
        assert_eq!(card.exhaust, case.exhaust, "{} exhaust", case.id);
    }

    fn filled_engine(card_ids: &[&str], enemy_hp: i32, enemy_dmg: i32) -> CombatEngine {
        let mut engine = engine_with(make_deck(card_ids), enemy_hp, enemy_dmg);
        force_player_turn(&mut engine);
        engine
    }

    fn bare_engine(card_ids: &[&str], enemies: Vec<EnemyCombatState>) -> CombatEngine {
        let mut engine = engine_without_start(make_deck(card_ids), enemies, 3);
        force_player_turn(&mut engine);
        engine
    }

    fn set_orbs(engine: &mut CombatEngine, orbs: &[OrbType]) {
        engine.init_defect_orbs(orbs.len().max(1));
        for orb in orbs {
            engine.channel_orb(*orb);
        }
    }

    fn enemy(id: &str, hp: i32, dmg: i32) -> EnemyCombatState {
        crate::tests::support::enemy(id, hp, hp, 1, dmg, 1)
    }

    // ------------------------------------------------------------------
    // Registry snapshots
    // ------------------------------------------------------------------

    defect_test!(registry_starter_and_basics, {
        let cases = [
            StatCase { id: "Strike", cost: 1, damage: 6, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Strike+", cost: 1, damage: 9, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Defend", cost: 1, damage: -1, block: 5, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Defend+", cost: 1, damage: -1, block: 8, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Zap", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Zap+", cost: 0, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Dualcast", cost: 1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Dualcast+", cost: 0, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: false },
        ];
        for case in cases { assert_stats(case); }
    });

    defect_test!(registry_orb_common, {
        let cases = [
            StatCase { id: "Ball Lightning", cost: 1, damage: 7, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Ball Lightning+", cost: 1, damage: 10, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Barrage", cost: 1, damage: 4, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Barrage+", cost: 1, damage: 6, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Beam Cell", cost: 0, damage: 3, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Beam Cell+", cost: 0, damage: 4, block: -1, magic: 2, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Cold Snap", cost: 1, damage: 6, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Cold Snap+", cost: 1, damage: 9, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Compile Driver", cost: 1, damage: 7, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Compile Driver+", cost: 1, damage: 10, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Conserve Battery", cost: 1, damage: -1, block: 7, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Conserve Battery+", cost: 1, damage: -1, block: 10, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Coolheaded", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Coolheaded+", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Go for the Eyes", cost: 0, damage: 3, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Go for the Eyes+", cost: 0, damage: 4, block: -1, magic: 2, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Hologram", cost: 1, damage: -1, block: 3, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Hologram+", cost: 1, damage: -1, block: 5, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Leap", cost: 1, damage: -1, block: 9, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Leap+", cost: 1, damage: -1, block: 12, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Rebound", cost: 1, damage: 9, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Rebound+", cost: 1, damage: 12, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
        ];
        for case in cases { assert_stats(case); }
    });

    defect_test!(registry_orb_utility, {
        let cases = [
            StatCase { id: "Stack", cost: 1, damage: -1, block: 0, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Stack+", cost: 1, damage: -1, block: 3, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Steam", cost: 0, damage: -1, block: 6, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Steam+", cost: 0, damage: -1, block: 8, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Streamline", cost: 2, damage: 15, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Streamline+", cost: 2, damage: 20, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Sweeping Beam", cost: 1, damage: 6, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Sweeping Beam+", cost: 1, damage: 9, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Turbo", cost: 0, damage: -1, block: -1, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Turbo+", cost: 0, damage: -1, block: -1, magic: 3, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Gash", cost: 0, damage: 3, block: -1, magic: 2, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Gash+", cost: 0, damage: 5, block: -1, magic: 2, card_type: CardType::Attack, exhaust: false },
        ];
        for case in cases { assert_stats(case); }
    });

    defect_test!(registry_uncommon_and_rare_core, {
        let cases = [
            StatCase { id: "Aggregate", cost: 1, damage: -1, block: -1, magic: 4, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Aggregate+", cost: 1, damage: -1, block: -1, magic: 3, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Auto Shields", cost: 1, damage: -1, block: 11, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Auto Shields+", cost: 1, damage: -1, block: 15, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Blizzard", cost: 1, damage: 0, block: -1, magic: 2, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Blizzard+", cost: 1, damage: 0, block: -1, magic: 3, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "BootSequence", cost: 0, damage: -1, block: 10, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "BootSequence+", cost: 0, damage: -1, block: 13, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Capacitor", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Capacitor+", cost: 1, damage: -1, block: -1, magic: 3, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Chaos", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Chaos+", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Chill", cost: 0, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Chill+", cost: 0, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Consume", cost: 2, damage: -1, block: -1, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Consume+", cost: 2, damage: -1, block: -1, magic: 3, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Darkness", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Darkness+", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Defragment", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Defragment+", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Doom and Gloom", cost: 2, damage: 10, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Doom and Gloom+", cost: 2, damage: 14, block: -1, magic: 1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Double Energy", cost: 1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Double Energy+", cost: 0, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Undo", cost: 2, damage: -1, block: 13, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Undo+", cost: 2, damage: -1, block: 16, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Force Field", cost: 4, damage: -1, block: 12, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Force Field+", cost: 4, damage: -1, block: 16, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "FTL", cost: 0, damage: 5, block: -1, magic: 3, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "FTL+", cost: 0, damage: 6, block: -1, magic: 4, card_type: CardType::Attack, exhaust: false },
        ];
        for case in cases { assert_stats(case); }
    });

    defect_test!(registry_rare_power_and_orb_finishers, {
        let cases = [
            StatCase { id: "Fusion", cost: 2, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Fusion+", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Genetic Algorithm", cost: 1, damage: -1, block: 1, magic: 2, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Genetic Algorithm+", cost: 1, damage: -1, block: 0, magic: 3, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Glacier", cost: 2, damage: -1, block: 7, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Glacier+", cost: 2, damage: -1, block: 10, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Heatsinks", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Heatsinks+", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Hello World", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Hello World+", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Lockon", cost: 1, damage: 8, block: -1, magic: 2, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Lockon+", cost: 1, damage: 11, block: -1, magic: 3, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Loop", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Loop+", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Melter", cost: 1, damage: 10, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Melter+", cost: 1, damage: 14, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Steam Power", cost: 0, damage: -1, block: -1, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Steam Power+", cost: 0, damage: -1, block: -1, magic: 3, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Recycle", cost: 1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Recycle+", cost: 0, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Redo", cost: 1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Redo+", cost: 0, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: false },
        ];
        for case in cases { assert_stats(case); }
    });

    defect_test!(registry_rare_powers_and_finishers, {
        let cases = [
            StatCase { id: "Reinforced Body", cost: -1, damage: -1, block: 7, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Reinforced Body+", cost: -1, damage: -1, block: 9, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Reprogram", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Reprogram+", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Rip and Tear", cost: 1, damage: 7, block: -1, magic: 2, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Rip and Tear+", cost: 1, damage: 9, block: -1, magic: 2, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Scrape", cost: 1, damage: 7, block: -1, magic: 4, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Scrape+", cost: 1, damage: 10, block: -1, magic: 5, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Self Repair", cost: 1, damage: -1, block: -1, magic: 7, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Self Repair+", cost: 1, damage: -1, block: -1, magic: 10, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Skim", cost: 1, damage: -1, block: -1, magic: 3, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Skim+", cost: 1, damage: -1, block: -1, magic: 4, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Static Discharge", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Static Discharge+", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Storm", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Storm+", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Sunder", cost: 3, damage: 24, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Sunder+", cost: 3, damage: 32, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Tempest", cost: -1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Tempest+", cost: -1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "White Noise", cost: 1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "White Noise+", cost: 0, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "All For One", cost: 2, damage: 10, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "All For One+", cost: 2, damage: 14, block: -1, magic: -1, card_type: CardType::Attack, exhaust: false },
        ];
        for case in cases { assert_stats(case); }
    });

    defect_test!(registry_final_rare_cards, {
        let cases = [
            StatCase { id: "Amplify", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Amplify+", cost: 1, damage: -1, block: -1, magic: 2, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Biased Cognition", cost: 1, damage: -1, block: -1, magic: 4, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Biased Cognition+", cost: 1, damage: -1, block: -1, magic: 5, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Buffer", cost: 2, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Buffer+", cost: 2, damage: -1, block: -1, magic: 2, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Core Surge", cost: 1, damage: 11, block: -1, magic: 1, card_type: CardType::Attack, exhaust: true },
            StatCase { id: "Core Surge+", cost: 1, damage: 15, block: -1, magic: 1, card_type: CardType::Attack, exhaust: true },
            StatCase { id: "Creative AI", cost: 3, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Creative AI+", cost: 2, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Echo Form", cost: 3, damage: -1, block: -1, magic: -1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Echo Form+", cost: 3, damage: -1, block: -1, magic: -1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Electrodynamics", cost: 2, damage: -1, block: -1, magic: 2, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Electrodynamics+", cost: 2, damage: -1, block: -1, magic: 3, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Fission", cost: 0, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Fission+", cost: 0, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Hyperbeam", cost: 2, damage: 26, block: -1, magic: 3, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Hyperbeam+", cost: 2, damage: 34, block: -1, magic: 3, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Machine Learning", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Machine Learning+", cost: 1, damage: -1, block: -1, magic: 1, card_type: CardType::Power, exhaust: false },
            StatCase { id: "Meteor Strike", cost: 5, damage: 24, block: -1, magic: 3, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Meteor Strike+", cost: 5, damage: 30, block: -1, magic: 3, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Multi-Cast", cost: -1, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Multi-Cast+", cost: -1, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Rainbow", cost: 2, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Rainbow+", cost: 2, damage: -1, block: -1, magic: -1, card_type: CardType::Skill, exhaust: false },
            StatCase { id: "Reboot", cost: 0, damage: -1, block: -1, magic: 4, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Reboot+", cost: 0, damage: -1, block: -1, magic: 6, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Seek", cost: 0, damage: -1, block: -1, magic: 1, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Seek+", cost: 0, damage: -1, block: -1, magic: 2, card_type: CardType::Skill, exhaust: true },
            StatCase { id: "Thunder Strike", cost: 3, damage: 7, block: -1, magic: 0, card_type: CardType::Attack, exhaust: false },
            StatCase { id: "Thunder Strike+", cost: 3, damage: 9, block: -1, magic: 0, card_type: CardType::Attack, exhaust: false },
        ];
        for case in cases { assert_stats(case); }
    });

    // ------------------------------------------------------------------
    // Runtime parity cases for implemented mechanics
    // ------------------------------------------------------------------

    defect_test!(zap_channels_lightning, {
        let mut e = filled_engine(&["Zap"], 40, 0);
        e.init_defect_orbs(1);
        ensure_in_hand(&mut e, "Zap");
        play_self(&mut e, "Zap");
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
        assert_eq!(e.state.energy, 2);
    });

    defect_test!(zap_plus_is_zero_cost_and_channels, {
        let mut e = filled_engine(&["Zap+"], 40, 0);
        e.init_defect_orbs(1);
        ensure_in_hand(&mut e, "Zap+");
        let before = e.state.energy;
        play_self(&mut e, "Zap+");
        assert_eq!(e.state.energy, before);
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
    });

    defect_test!(dualcast_evokes_twice, {
        let mut e = bare_engine(&["Dualcast"], vec![enemy("JawWorm", 40, 0)]);
        e.init_defect_orbs(1);
        e.channel_orb(OrbType::Lightning);
        ensure_in_hand(&mut e, "Dualcast");
        let hp = e.state.enemies[0].entity.hp;
        play_self(&mut e, "Dualcast");
        assert_eq!(e.state.enemies[0].entity.hp, hp - 8);
        assert_eq!(e.state.orb_slots.occupied_count(), 0);
    });

    defect_test!(ball_lightning_channels_and_hits, {
        let mut e = filled_engine(&["Ball Lightning"], 40, 0);
        e.init_defect_orbs(1);
        ensure_in_hand(&mut e, "Ball Lightning");
        let hp = e.state.enemies[0].entity.hp;
        play_on_enemy(&mut e, "Ball Lightning", 0);
        assert_eq!(e.state.enemies[0].entity.hp, hp - 7);
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
    });

    defect_test!(barrage_scales_with_orbs, {
        let mut e = filled_engine(&["Barrage"], 60, 0);
        set_orbs(&mut e, &[OrbType::Lightning, OrbType::Frost, OrbType::Dark]);
        ensure_in_hand(&mut e, "Barrage");
        let hp = e.state.enemies[0].entity.hp;
        play_on_enemy(&mut e, "Barrage", 0);
        assert_eq!(e.state.enemies[0].entity.hp, hp - 12);
    });

    defect_test!(beam_cell_applies_vulnerable, {
        let mut e = filled_engine(&["Beam Cell"], 40, 0);
        ensure_in_hand(&mut e, "Beam Cell");
        play_on_enemy(&mut e, "Beam Cell", 0);
        assert_eq!(e.state.enemies[0].entity.status(sid::VULNERABLE), 1);
    });

    defect_test!(cold_snap_channels_frost, {
        let mut e = filled_engine(&["Cold Snap"], 40, 0);
        e.init_defect_orbs(1);
        ensure_in_hand(&mut e, "Cold Snap");
        let hp = e.state.enemies[0].entity.hp;
        play_on_enemy(&mut e, "Cold Snap", 0);
        assert_eq!(e.state.enemies[0].entity.hp, hp - 6);
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Frost);
    });

    defect_test!(compile_driver_draws_per_unique_orb, {
        let mut e = bare_engine(&["Compile Driver"], vec![enemy("JawWorm", 50, 0)]);
        e.state.draw_pile = make_deck(&["Strike", "Defend", "Zap", "Cold Snap"]);
        set_orbs(&mut e, &[OrbType::Lightning, OrbType::Frost, OrbType::Dark]);
        ensure_in_hand(&mut e, "Compile Driver");
        let hand_before = e.state.hand.len();
        play_on_enemy(&mut e, "Compile Driver", 0);
        assert_eq!(e.state.hand.len(), hand_before + 2);
    });

    defect_test!(consume_reduces_orb_slots_and_gains_focus, {
        let mut e = bare_engine(&["Consume"], vec![enemy("JawWorm", 50, 0)]);
        e.init_defect_orbs(3);
        ensure_in_hand(&mut e, "Consume");
        play_self(&mut e, "Consume");
        assert_eq!(e.state.orb_slots.get_slot_count(), 2);
        assert_eq!(e.state.player.focus(), 2);
    });

    defect_test!(darkness_plus_accumulates_on_entry, {
        let mut e = bare_engine(&["Darkness+"], vec![enemy("JawWorm", 50, 0)]);
        e.init_defect_orbs(1);
        ensure_in_hand(&mut e, "Darkness+");
        play_self(&mut e, "Darkness+");
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Dark);
        assert_eq!(e.state.orb_slots.slots[0].evoke_amount, 12);
    });

    defect_test!(defragment_gains_focus, {
        let mut e = filled_engine(&["Defragment"], 40, 0);
        ensure_in_hand(&mut e, "Defragment");
        play_self(&mut e, "Defragment");
        assert_eq!(e.state.player.focus(), 1);
    });

    defect_test!(double_energy_doubles_current_energy, {
        let mut e = filled_engine(&["Double Energy"], 40, 0);
        ensure_in_hand(&mut e, "Double Energy");
        e.state.energy = 4;
        play_self(&mut e, "Double Energy");
        assert_eq!(e.state.energy, 6);
        assert!(e.state.exhaust_pile.iter().any(|c| e.card_registry.card_name(c.def_id) == "Double Energy"));
    });

    defect_test!(fusion_channels_plasma, {
        let mut e = filled_engine(&["Fusion"], 40, 0);
        e.init_defect_orbs(1);
        ensure_in_hand(&mut e, "Fusion");
        play_self(&mut e, "Fusion");
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Plasma);
    });

    defect_test!(fission_removes_orbs_and_refunds_energy, {
        let mut e = bare_engine(&["Fission"], vec![enemy("JawWorm", 40, 0)]);
        set_orbs(&mut e, &[OrbType::Lightning, OrbType::Frost]);
        ensure_in_hand(&mut e, "Fission");
        let before_energy = e.state.energy;
        play_self(&mut e, "Fission");
        assert_eq!(e.state.energy, before_energy + 2);
        assert_eq!(e.state.orb_slots.occupied_count(), 0);
    });

    defect_test!(fission_plus_evokes_all_orbs, {
        let mut e = bare_engine(&["Fission+"], vec![enemy("JawWorm", 80, 0)]);
        set_orbs(&mut e, &[OrbType::Lightning, OrbType::Frost, OrbType::Dark]);
        ensure_in_hand(&mut e, "Fission+");
        let before_hp = e.state.enemies[0].entity.hp;
        let before_block = e.state.player.block;
        play_self(&mut e, "Fission+");
        assert_eq!(e.state.enemies[0].entity.hp, before_hp - 14); // Lightning evoke 8 + Dark evoke 6
        assert_eq!(e.state.player.block, before_block + 5); // Frost evoke 5
        assert_eq!(e.state.energy, 6);
    });

    defect_test!(glacier_gains_block_and_channels_frost, {
        let mut e = filled_engine(&["Glacier"], 40, 0);
        e.init_defect_orbs(3);
        ensure_in_hand(&mut e, "Glacier");
        play_self(&mut e, "Glacier");
        assert_eq!(e.state.player.block, 7);
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Frost);
    });

    defect_test!(hyperbeam_loses_focus, {
        let mut e = filled_engine(&["Hyperbeam"], 40, 0);
        ensure_in_hand(&mut e, "Hyperbeam");
        e.state.player.set_status(sid::FOCUS, 4);
        play_on_enemy(&mut e, "Hyperbeam", 0);
        assert_eq!(e.state.player.focus(), 1);
    });

    defect_test!(meteo_strike_channels_plasma, {
        let mut e = filled_engine(&["Meteor Strike"], 60, 0);
        e.init_defect_orbs(3);
        e.state.energy = 5;
        ensure_in_hand(&mut e, "Meteor Strike");
        play_on_enemy(&mut e, "Meteor Strike", 0);
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Plasma);
    });

    defect_test!(multi_cast_evokes_front_orb_x_times, {
        let mut e = bare_engine(&["Multi-Cast"], vec![enemy("JawWorm", 80, 0)]);
        set_orbs(&mut e, &[OrbType::Lightning, OrbType::Frost, OrbType::Dark]);
        ensure_in_hand(&mut e, "Multi-Cast");
        let hp = e.state.enemies[0].entity.hp;
        play_self(&mut e, "Multi-Cast");
        assert_eq!(e.state.enemies[0].entity.hp, hp - 14); // X=3: Lightning evoke 8 + Frost block + Dark evoke 6
    });

    defect_test!(reinforced_body_spends_x_for_block, {
        let mut e = bare_engine(&["Reinforced Body"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Reinforced Body");
        e.state.energy = 3;
        play_self(&mut e, "Reinforced Body");
        assert_eq!(e.state.player.block, 21);
        assert_eq!(e.state.energy, 0);
    });

    defect_test!(reprogram_loses_focus_and_gains_stats, {
        let mut e = bare_engine(&["Reprogram"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Reprogram");
        e.state.player.set_status(sid::FOCUS, 3);
        play_self(&mut e, "Reprogram");
        assert_eq!(e.state.player.focus(), 2);
        assert_eq!(e.state.player.strength(), 1);
        assert_eq!(e.state.player.dexterity(), 1);
    });

    defect_test!(rip_and_tear_hits_twice_against_single_enemy, {
        let mut e = filled_engine(&["Rip and Tear"], 40, 0);
        ensure_in_hand(&mut e, "Rip and Tear");
        let hp = e.state.enemies[0].entity.hp;
        play_on_enemy(&mut e, "Rip and Tear", 0);
        assert_eq!(e.state.enemies[0].entity.hp, hp - 14);
    });

    defect_test!(storm_channels_lightning_on_power_play, {
        let mut e = bare_engine(&["Storm", "Defragment"], vec![enemy("JawWorm", 40, 0)]);
        e.init_defect_orbs(1);
        ensure_in_hand(&mut e, "Storm");
        ensure_in_hand(&mut e, "Defragment");
        play_self(&mut e, "Storm");
        play_self(&mut e, "Defragment");
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
    });

    defect_test!(sunder_kill_refunds_energy, {
        let mut e = filled_engine(&["Sunder"], 12, 0);
        ensure_in_hand(&mut e, "Sunder");
        e.state.energy = 3;
        play_on_enemy(&mut e, "Sunder", 0);
        assert_eq!(e.state.energy, 3); // Sunder costs 3, kills 12HP enemy (24 dmg), refunds 3
    });

    defect_test!(tempest_channels_x_lightning, {
        let mut e = bare_engine(&["Tempest"], vec![enemy("JawWorm", 40, 0)]);
        e.init_defect_orbs(3);
        ensure_in_hand(&mut e, "Tempest");
        e.state.energy = 3;
        play_self(&mut e, "Tempest");
        assert_eq!(e.state.orb_slots.occupied_count(), 3);
        assert!(e.state.orb_slots.slots.iter().all(|orb| orb.orb_type == OrbType::Lightning));
    });

    defect_test!(thunder_strike_scales_with_lightning_channel_count, {
        let mut e = filled_engine(&["Thunder Strike"], 40, 0);
        ensure_in_hand(&mut e, "Thunder Strike");
        e.state.player.set_status(sid::LIGHTNING_CHANNELED, 4);
        play_on_enemy(&mut e, "Thunder Strike", 0);
        assert_eq!(e.state.enemies[0].entity.hp, 40 - 28); // 4 hits of 7 base damage = 28
    });

    defect_test!(white_noise_adds_a_power_card, {
        let mut e = bare_engine(&["White Noise"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "White Noise");
        let hand_before = e.state.hand.len();
        play_self(&mut e, "White Noise");
        assert_eq!(e.state.hand.len(), hand_before);
        assert!(e.state.hand.iter().any(|c| e.card_registry.card_name(c.def_id).starts_with("Defragment")));
    });

    defect_test!(all_for_one_returns_zero_cost_cards_from_discard, {
        let mut e = bare_engine(&["All For One"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "All For One");
        e.state.discard_pile = make_deck(&["Zap+", "Turbo", "Strike"]);
        play_on_enemy(&mut e, "All For One", 0);
        assert!(e.state.hand.iter().any(|c| e.card_registry.card_name(c.def_id) == "Zap+"));
        assert!(e.state.hand.iter().any(|c| e.card_registry.card_name(c.def_id) == "Turbo"));
    });

    defect_test!(biased_cognition_gives_focus, {
        let mut e = bare_engine(&["Biased Cognition"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Biased Cognition");
        play_self(&mut e, "Biased Cognition");
        assert_eq!(e.state.player.focus(), 4);
    });

    defect_test!(buffer_installs_buffer, {
        let mut e = bare_engine(&["Buffer"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Buffer");
        play_self(&mut e, "Buffer");
        assert_eq!(e.state.player.status(sid::BUFFER), 1);
    });

    defect_test!(core_surge_grants_artifact, {
        let mut e = filled_engine(&["Core Surge"], 40, 0);
        ensure_in_hand(&mut e, "Core Surge");
        play_on_enemy(&mut e, "Core Surge", 0);
        assert_eq!(e.state.player.status(sid::ARTIFACT), 1);
    });

    defect_test!(creative_ai_installs_power, {
        let mut e = bare_engine(&["Creative AI"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Creative AI");
        play_self(&mut e, "Creative AI");
        assert_eq!(e.state.player.status(sid::CREATIVE_AI), 1);
    });

    defect_test!(echo_form_installs_echo_form, {
        let mut e = bare_engine(&["Echo Form"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Echo Form");
        play_self(&mut e, "Echo Form");
        assert_eq!(e.state.player.status(sid::ECHO_FORM), 1);
    });

    defect_test!(electrodynamics_channels_lightning, {
        let mut e = filled_engine(&["Electrodynamics"], 40, 0);
        e.init_defect_orbs(1);
        ensure_in_hand(&mut e, "Electrodynamics");
        play_self(&mut e, "Electrodynamics");
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
    });

    defect_test!(machine_learning_will_add_draw_status, {
        let mut e = bare_engine(&["Machine Learning"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Machine Learning");
        play_self(&mut e, "Machine Learning");
        assert_eq!(e.state.player.status(sid::DRAW), 1);
    });

    defect_test!(rainbow_channels_all_orbs, {
        let mut e = bare_engine(&["Rainbow"], vec![enemy("JawWorm", 40, 0)]);
        e.init_defect_orbs(3);
        ensure_in_hand(&mut e, "Rainbow");
        play_self(&mut e, "Rainbow");
        assert_eq!(e.state.orb_slots.slots[0].orb_type, OrbType::Lightning);
        assert_eq!(e.state.orb_slots.slots[1].orb_type, OrbType::Frost);
        assert_eq!(e.state.orb_slots.slots[2].orb_type, OrbType::Dark);
    });

    defect_test!(reboot_draws_a_fresh_hand, {
        let mut e = bare_engine(&["Reboot"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Reboot");
        e.state.draw_pile = make_deck(&["Strike", "Defend", "Zap", "Cold Snap"]);
        play_self(&mut e, "Reboot");
        assert_eq!(e.state.hand.len(), 4); // Reboot draws base_magic=4 cards
    });

    defect_test!(seek_is_a_tutor_effect, {
        let mut e = bare_engine(&["Seek"], vec![enemy("JawWorm", 40, 0)]);
        ensure_in_hand(&mut e, "Seek");
        e.state.draw_pile = make_deck(&["Zap", "Turbo", "Defragment"]);
        play_self(&mut e, "Seek");
        // Seek now presents a PickFromDrawPile choice
        assert_eq!(e.phase, CombatPhase::AwaitingChoice);
        e.execute_action(&Action::Choose(0)); // pick first card
        assert!(!e.state.hand.is_empty());
    });

    defect_test!(process_start_of_turn_handles_power_helpers, {
        let mut entity = crate::state::EntityState::new(50, 50);
        entity.set_status(sid::ENERGIZED, 2);
        entity.set_status(sid::DRAW_CARD, 1);
        entity.set_status(sid::NEXT_TURN_BLOCK, 4);
        entity.set_status(sid::BATTLE_HYMN, 3);
        entity.set_status(sid::DEVOTION, 2);
        entity.set_status(sid::DRAW, 1);
        let result = process_start_of_turn(&mut entity);
        assert_eq!(result.extra_energy, 2);
        assert_eq!(result.draw_card_next_turn, 1);
        assert_eq!(result.block_from_next_turn, 4);
        assert_eq!(result.battle_hymn_smites, 3);
        assert_eq!(result.devotion_mantra, 2);
        assert_eq!(result.extra_draw, 1);
    });

    defect_test!(process_end_of_turn_handles_power_helpers, {
        let mut entity = crate::state::EntityState::new(50, 50);
        entity.set_status(sid::METALLICIZE, 4);
        entity.set_status(sid::PLATED_ARMOR, 3);
        entity.set_status(sid::OMEGA, 5);
        entity.set_status(sid::LIKE_WATER, 2);
        let result = process_end_of_turn(&mut entity, true);
        assert_eq!(result.metallicize_block, 4);
        assert_eq!(result.plated_armor_block, 3);
        assert_eq!(result.omega_damage, 5);
        assert_eq!(result.like_water_block, 2);
    });

    defect_test!(process_end_of_round_clears_debuffs, {
        let mut entity = crate::state::EntityState::new(50, 50);
        entity.set_status(sid::WEAKENED, 2);
        entity.set_status(sid::VULNERABLE, 1);
        entity.set_status(sid::FRAIL, 1);
        entity.set_status(sid::BLUR, 1);
        entity.set_status(sid::LOCK_ON, 2);
        process_end_of_round(&mut entity);
        assert_eq!(entity.status(sid::WEAKENED), 1);
        assert_eq!(entity.status(sid::VULNERABLE), 0);
        assert_eq!(entity.status(sid::FRAIL), 0);
        assert_eq!(entity.status(sid::BLUR), 0);
        assert_eq!(entity.status(sid::LOCK_ON), 1);
    });

    defect_test!(orb_passives_and_evokes_match_java_basics, {
        let mut slots = crate::orbs::OrbSlots::new(3);
        slots.channel(OrbType::Lightning, 0);
        slots.channel(OrbType::Frost, 0);
        slots.channel(OrbType::Dark, 0);
        let passives = slots.trigger_end_of_turn_passives(0);
        assert_eq!(passives.len(), 3);
        let evoke = slots.evoke_front(0);
        match evoke {
            crate::orbs::EvokeEffect::LightningDamage(8) => {}
            other => panic!("unexpected evoke effect: {:?}", other),
        }
    });

}
