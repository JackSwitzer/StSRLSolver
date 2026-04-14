#[cfg(test)]
mod silent_card_java_parity_tests {
    // Java sources referenced:
    // /tmp/sts-decompiled/com/megacrit/cardcrawl/cards/green/*.java

    use crate::actions::Action;
    use crate::status_ids::sid;
    use crate::cards::{CardRegistry, CardTarget, CardType};
    use crate::tests::support::*;

    fn reg() -> &'static CardRegistry {
        crate::cards::global_registry()
    }

    fn assert_card(
        reg: &CardRegistry,
        id: &str,
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
        let card = reg.get(id).unwrap_or_else(|| panic!("missing card {id}"));
        assert_eq!(card.cost, cost, "{id} cost");
        assert_eq!(card.base_damage, damage, "{id} damage");
        assert_eq!(card.base_block, block, "{id} block");
        assert_eq!(card.base_magic, magic, "{id} magic");
        assert_eq!(card.card_type, card_type, "{id} type");
        assert_eq!(card.target, target, "{id} target");
        assert_eq!(card.exhaust, exhaust, "{id} exhaust");
        assert_eq!(card.enter_stance, enter_stance, "{id} stance");
        assert_eq!(card.effects, effects, "{id} effects");
    }

    macro_rules! card_pair_test {
        (
            $name:ident,
            $base_id:expr, $base_cost:expr, $base_damage:expr, $base_block:expr, $base_magic:expr,
            $base_type:expr, $base_target:expr, $base_exhaust:expr, $base_stance:expr, $base_effects:expr,
            $up_id:expr, $up_cost:expr, $up_damage:expr, $up_block:expr, $up_magic:expr,
            $up_type:expr, $up_target:expr, $up_exhaust:expr, $up_stance:expr, $up_effects:expr $(,)?
        ) => {
            #[test]
            fn $name() {
                let reg = reg();
                assert_card(
                    &reg,
                    $base_id,
                    $base_cost,
                    $base_damage,
                    $base_block,
                    $base_magic,
                    $base_type,
                    $base_target,
                    $base_exhaust,
                    $base_stance,
                    $base_effects,
                );
                assert_card(
                    &reg,
                    $up_id,
                    $up_cost,
                    $up_damage,
                    $up_block,
                    $up_magic,
                    $up_type,
                    $up_target,
                    $up_exhaust,
                    $up_stance,
                    $up_effects,
                );
            }
        };
    }

    // ---------------------------------------------------------------------
    // Exact Java parity for every Silent card in /cards/green
    // ---------------------------------------------------------------------

    card_pair_test!(strike_g,
        "Strike_G", 1, 6, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &[],
        "Strike_G+", 1, 9, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &[],
    );
    card_pair_test!(defend_g,
        "Defend_G", 1, -1, 5, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &[],
        "Defend_G+", 1, -1, 8, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &[],
    );
    card_pair_test!(neutralize,
        "Neutralize", 0, 3, -1, 1, CardType::Attack, CardTarget::Enemy, false, None, &["weak"],
        "Neutralize+", 0, 4, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, &["weak"],
    );
    card_pair_test!(survivor,
        "Survivor", 1, -1, 8, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &["discard"],
        "Survivor+", 1, -1, 11, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &["discard"],
    );
    card_pair_test!(acrobatics,
        "Acrobatics", 1, -1, -1, 3, CardType::Skill, CardTarget::None, false, None, &["draw", "discard"],
        "Acrobatics+", 1, -1, -1, 4, CardType::Skill, CardTarget::None, false, None, &["draw", "discard"],
    );
    card_pair_test!(backflip,
        "Backflip", 1, -1, 5, 2, CardType::Skill, CardTarget::SelfTarget, false, None, &["draw"],
        "Backflip+", 1, -1, 8, 2, CardType::Skill, CardTarget::SelfTarget, false, None, &["draw"],
    );
    card_pair_test!(bane,
        "Bane", 1, 7, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["double_if_poisoned"],
        "Bane+", 1, 10, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["double_if_poisoned"],
    );
    card_pair_test!(blade_dance,
        "Blade Dance", 1, -1, -1, 3, CardType::Skill, CardTarget::None, false, None, &["add_shivs"],
        "Blade Dance+", 1, -1, -1, 4, CardType::Skill, CardTarget::None, false, None, &["add_shivs"],
    );
    card_pair_test!(cloak_and_dagger,
        "Cloak and Dagger", 1, -1, 6, 1, CardType::Skill, CardTarget::SelfTarget, false, None, &["add_shivs"],
        "Cloak and Dagger+", 1, -1, 6, 2, CardType::Skill, CardTarget::SelfTarget, false, None, &["add_shivs"],
    );
    card_pair_test!(dagger_spray,
        "Dagger Spray", 1, 4, -1, 2, CardType::Attack, CardTarget::AllEnemy, false, None, &["multi_hit"],
        "Dagger Spray+", 1, 6, -1, 2, CardType::Attack, CardTarget::AllEnemy, false, None, &["multi_hit"],
    );
    card_pair_test!(dagger_throw,
        "Dagger Throw", 1, 9, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["draw", "discard"],
        "Dagger Throw+", 1, 12, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["draw", "discard"],
    );
    card_pair_test!(deadly_poison,
        "Deadly Poison", 1, -1, -1, 5, CardType::Skill, CardTarget::Enemy, false, None, &["poison"],
        "Deadly Poison+", 1, -1, -1, 7, CardType::Skill, CardTarget::Enemy, false, None, &["poison"],
    );
    card_pair_test!(deflect,
        "Deflect", 0, -1, 4, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &[],
        "Deflect+", 0, -1, 7, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &[],
    );
    card_pair_test!(dodge_and_roll,
        "Dodge and Roll", 1, -1, 4, 4, CardType::Skill, CardTarget::SelfTarget, false, None, &["next_turn_block"],
        "Dodge and Roll+", 1, -1, 6, 6, CardType::Skill, CardTarget::SelfTarget, false, None, &["next_turn_block"],
    );
    card_pair_test!(flying_knee,
        "Flying Knee", 1, 8, -1, 1, CardType::Attack, CardTarget::Enemy, false, None, &["next_turn_energy"],
        "Flying Knee+", 1, 11, -1, 1, CardType::Attack, CardTarget::Enemy, false, None, &["next_turn_energy"],
    );
    card_pair_test!(outmaneuver,
        "Outmaneuver", 1, -1, -1, 2, CardType::Skill, CardTarget::None, false, None, &["next_turn_energy"],
        "Outmaneuver+", 1, -1, -1, 3, CardType::Skill, CardTarget::None, false, None, &["next_turn_energy"],
    );
    card_pair_test!(piercing_wail,
        "Piercing Wail", 1, -1, -1, 6, CardType::Skill, CardTarget::AllEnemy, true, None, &["reduce_strength_all_temp"],
        "Piercing Wail+", 1, -1, -1, 8, CardType::Skill, CardTarget::AllEnemy, true, None, &["reduce_strength_all_temp"],
    );
    card_pair_test!(poisoned_stab,
        "Poisoned Stab", 1, 6, -1, 3, CardType::Attack, CardTarget::Enemy, false, None, &["poison"],
        "Poisoned Stab+", 1, 8, -1, 4, CardType::Attack, CardTarget::Enemy, false, None, &["poison"],
    );
    card_pair_test!(prepared,
        "Prepared", 0, -1, -1, 1, CardType::Skill, CardTarget::None, false, None, &["draw", "discard"],
        "Prepared+", 0, -1, -1, 2, CardType::Skill, CardTarget::None, false, None, &["draw", "discard"],
    );
    card_pair_test!(quick_slash,
        "Quick Slash", 1, 8, -1, 1, CardType::Attack, CardTarget::Enemy, false, None, &["draw"],
        "Quick Slash+", 1, 12, -1, 1, CardType::Attack, CardTarget::Enemy, false, None, &["draw"],
    );
    card_pair_test!(slice,
        "Slice", 0, 6, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &[],
        "Slice+", 0, 9, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &[],
    );
    card_pair_test!(sneaky_strike,
        "Sneaky Strike", 2, 12, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, &["refund_energy_on_discard"],
        "Sneaky Strike+", 2, 16, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, &["refund_energy_on_discard"],
    );
    card_pair_test!(sucker_punch,
        "Sucker Punch", 1, 7, -1, 1, CardType::Attack, CardTarget::Enemy, false, None, &["weak"],
        "Sucker Punch+", 1, 9, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, &["weak"],
    );
    card_pair_test!(accuracy,
        "Accuracy", 1, -1, -1, 4, CardType::Power, CardTarget::SelfTarget, false, None, &["accuracy"],
        "Accuracy+", 1, -1, -1, 6, CardType::Power, CardTarget::SelfTarget, false, None, &["accuracy"],
    );
    card_pair_test!(all_out_attack,
        "All-Out Attack", 1, 10, -1, -1, CardType::Attack, CardTarget::AllEnemy, false, None, &["discard_random"],
        "All-Out Attack+", 1, 14, -1, -1, CardType::Attack, CardTarget::AllEnemy, false, None, &["discard_random"],
    );
    card_pair_test!(backstab,
        "Backstab", 0, 11, -1, -1, CardType::Attack, CardTarget::Enemy, true, None, &["innate"],
        "Backstab+", 0, 15, -1, -1, CardType::Attack, CardTarget::Enemy, true, None, &["innate"],
    );
    card_pair_test!(blur,
        "Blur", 1, -1, 5, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &["retain_block"],
        "Blur+", 1, -1, 8, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &["retain_block"],
    );
    card_pair_test!(bouncing_flask,
        "Bouncing Flask", 2, -1, -1, 3, CardType::Skill, CardTarget::AllEnemy, false, None, &["poison_random_multi"],
        "Bouncing Flask+", 2, -1, -1, 4, CardType::Skill, CardTarget::AllEnemy, false, None, &["poison_random_multi"],
    );
    card_pair_test!(calculated_gamble,
        "Calculated Gamble", 0, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, &["calculated_gamble"],
        "Calculated Gamble+", 0, -1, -1, -1, CardType::Skill, CardTarget::None, false, None, &["calculated_gamble"],
    );
    card_pair_test!(caltrops,
        "Caltrops", 1, -1, -1, 3, CardType::Power, CardTarget::SelfTarget, false, None, &["thorns"],
        "Caltrops+", 1, -1, -1, 5, CardType::Power, CardTarget::SelfTarget, false, None, &["thorns"],
    );
    card_pair_test!(catalyst,
        "Catalyst", 1, -1, -1, 2, CardType::Skill, CardTarget::Enemy, true, None, &["catalyst_double"],
        "Catalyst+", 1, -1, -1, 3, CardType::Skill, CardTarget::Enemy, true, None, &["catalyst_triple"],
    );
    card_pair_test!(choke,
        "Choke", 2, 12, -1, 3, CardType::Attack, CardTarget::Enemy, false, None, &["choke"],
        "Choke+", 2, 12, -1, 5, CardType::Attack, CardTarget::Enemy, false, None, &["choke"],
    );
    card_pair_test!(concentrate,
        "Concentrate", 0, -1, -1, 3, CardType::Skill, CardTarget::None, false, None, &["discard_gain_energy"],
        "Concentrate+", 0, -1, -1, 2, CardType::Skill, CardTarget::None, false, None, &["discard_gain_energy"],
    );
    card_pair_test!(crippling_cloud,
        "Crippling Cloud", 2, -1, -1, 4, CardType::Skill, CardTarget::AllEnemy, true, None, &["poison_all", "weak_all"],
        "Crippling Cloud+", 2, -1, -1, 7, CardType::Skill, CardTarget::AllEnemy, true, None, &["poison_all", "weak_all"],
    );
    card_pair_test!(dash,
        "Dash", 2, 10, 10, -1, CardType::Attack, CardTarget::Enemy, false, None, &[],
        "Dash+", 2, 13, 13, -1, CardType::Attack, CardTarget::Enemy, false, None, &[],
    );
    card_pair_test!(distraction,
        "Distraction", 1, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, &["random_skill_to_hand"],
        "Distraction+", 0, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, &["random_skill_to_hand"],
    );
    card_pair_test!(endless_agony,
        "Endless Agony", 0, 4, -1, -1, CardType::Attack, CardTarget::Enemy, true, None, &["copy_on_draw"],
        "Endless Agony+", 0, 6, -1, -1, CardType::Attack, CardTarget::Enemy, true, None, &["copy_on_draw"],
    );
    card_pair_test!(envenom,
        "Envenom", 2, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["envenom"],
        "Envenom+", 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["envenom"],
    );
    card_pair_test!(escape_plan,
        "Escape Plan", 0, -1, 3, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &["block_if_skill"],
        "Escape Plan+", 0, -1, 5, -1, CardType::Skill, CardTarget::SelfTarget, false, None, &["block_if_skill"],
    );
    card_pair_test!(eviscerate,
        "Eviscerate", 3, 7, -1, 3, CardType::Attack, CardTarget::Enemy, false, None, &["multi_hit", "cost_reduce_on_discard"],
        "Eviscerate+", 3, 8, -1, 3, CardType::Attack, CardTarget::Enemy, false, None, &["multi_hit", "cost_reduce_on_discard"],
    );
    card_pair_test!(expertise,
        "Expertise", 1, -1, -1, 6, CardType::Skill, CardTarget::None, false, None, &["draw_to_n"],
        "Expertise+", 1, -1, -1, 7, CardType::Skill, CardTarget::None, false, None, &["draw_to_n"],
    );
    card_pair_test!(finisher,
        "Finisher", 1, 6, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["finisher"],
        "Finisher+", 1, 8, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["finisher"],
    );
    card_pair_test!(flechettes,
        "Flechettes", 1, 4, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["flechettes"],
        "Flechettes+", 1, 6, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["flechettes"],
    );
    card_pair_test!(footwork,
        "Footwork", 1, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false, None, &["gain_dexterity"],
        "Footwork+", 1, -1, -1, 3, CardType::Power, CardTarget::SelfTarget, false, None, &["gain_dexterity"],
    );
    card_pair_test!(heel_hook,
        "Heel Hook", 1, 5, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["if_weak_energy_draw"],
        "Heel Hook+", 1, 8, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["if_weak_energy_draw"],
    );
    card_pair_test!(infinite_blades,
        "Infinite Blades", 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["infinite_blades"],
        "Infinite Blades+", 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["infinite_blades", "innate"],
    );
    card_pair_test!(leg_sweep,
        "Leg Sweep", 2, -1, 11, 2, CardType::Skill, CardTarget::Enemy, false, None, &["weak"],
        "Leg Sweep+", 2, -1, 14, 3, CardType::Skill, CardTarget::Enemy, false, None, &["weak"],
    );
    card_pair_test!(masterful_stab,
        "Masterful Stab", 0, 12, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["cost_increase_on_hp_loss"],
        "Masterful Stab+", 0, 16, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["cost_increase_on_hp_loss"],
    );
    card_pair_test!(noxious_fumes,
        "Noxious Fumes", 1, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false, None, &["noxious_fumes"],
        "Noxious Fumes+", 1, -1, -1, 3, CardType::Power, CardTarget::SelfTarget, false, None, &["noxious_fumes"],
    );
    card_pair_test!(predator,
        "Predator", 2, 15, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, &["draw_next_turn"],
        "Predator+", 2, 20, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, &["draw_next_turn"],
    );
    card_pair_test!(reflex,
        "Reflex", -2, -1, -1, 2, CardType::Skill, CardTarget::None, false, None, &["unplayable", "draw_on_discard"],
        "Reflex+", -2, -1, -1, 3, CardType::Skill, CardTarget::None, false, None, &["unplayable", "draw_on_discard"],
    );
    card_pair_test!(riddle_with_holes,
        "Riddle with Holes", 2, 3, -1, 5, CardType::Attack, CardTarget::Enemy, false, None, &["multi_hit"],
        "Riddle with Holes+", 2, 4, -1, 5, CardType::Attack, CardTarget::Enemy, false, None, &["multi_hit"],
    );
    card_pair_test!(setup,
        "Setup", 1, -1, -1, -1, CardType::Skill, CardTarget::None, false, None, &["setup"],
        "Setup+", 0, -1, -1, -1, CardType::Skill, CardTarget::None, false, None, &["setup"],
    );
    card_pair_test!(skewer,
        "Skewer", -1, 7, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["x_cost"],
        "Skewer+", -1, 10, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["x_cost"],
    );
    card_pair_test!(tactician,
        "Tactician", -2, -1, -1, 1, CardType::Skill, CardTarget::None, false, None, &["unplayable", "energy_on_discard"],
        "Tactician+", -2, -1, -1, 2, CardType::Skill, CardTarget::None, false, None, &["unplayable", "energy_on_discard"],
    );
    card_pair_test!(terror,
        "Terror", 1, -1, -1, 99, CardType::Skill, CardTarget::Enemy, true, None, &["vulnerable"],
        "Terror+", 0, -1, -1, 99, CardType::Skill, CardTarget::Enemy, true, None, &["vulnerable"],
    );
    card_pair_test!(well_laid_plans,
        "Well-Laid Plans", 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["well_laid_plans"],
        "Well-Laid Plans+", 1, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false, None, &["well_laid_plans"],
    );
    card_pair_test!(a_thousand_cuts,
        "A Thousand Cuts", 2, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["thousand_cuts"],
        "A Thousand Cuts+", 2, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false, None, &["thousand_cuts"],
    );
    card_pair_test!(adrenaline,
        "Adrenaline", 0, -1, -1, 2, CardType::Skill, CardTarget::None, true, None, &["gain_energy_1", "draw"],
        "Adrenaline+", 0, -1, -1, 3, CardType::Skill, CardTarget::None, true, None, &["gain_energy_1", "draw"],
    );
    card_pair_test!(after_image,
        "After Image", 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["after_image"],
        "After Image+", 0, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["after_image"],
    );
    card_pair_test!(alchemize,
        "Alchemize", 1, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, &["alchemize"],
        "Alchemize+", 0, -1, -1, -1, CardType::Skill, CardTarget::None, true, None, &["alchemize"],
    );
    card_pair_test!(bullet_time,
        "Bullet Time", 3, -1, -1, -1, CardType::Skill, CardTarget::None, false, None, &["bullet_time"],
        "Bullet Time+", 2, -1, -1, -1, CardType::Skill, CardTarget::None, false, None, &["bullet_time"],
    );
    card_pair_test!(burst,
        "Burst", 1, -1, -1, 1, CardType::Skill, CardTarget::SelfTarget, false, None, &["burst"],
        "Burst+", 1, -1, -1, 2, CardType::Skill, CardTarget::SelfTarget, false, None, &["burst"],
    );
    card_pair_test!(corpse_explosion,
        "Corpse Explosion", 2, -1, -1, 6, CardType::Skill, CardTarget::Enemy, false, None, &["corpse_explosion"],
        "Corpse Explosion+", 2, -1, -1, 9, CardType::Skill, CardTarget::Enemy, false, None, &["corpse_explosion"],
    );
    card_pair_test!(die_die_die,
        "Die Die Die", 1, 13, -1, -1, CardType::Attack, CardTarget::AllEnemy, true, None, &[],
        "Die Die Die+", 1, 17, -1, -1, CardType::Attack, CardTarget::AllEnemy, true, None, &[],
    );
    card_pair_test!(doppelganger,
        "Doppelganger", -1, -1, -1, 0, CardType::Skill, CardTarget::None, true, None, &["x_cost"],
        "Doppelganger+", -1, -1, -1, 1, CardType::Skill, CardTarget::None, true, None, &["x_cost"],
    );
    card_pair_test!(glass_knife,
        "Glass Knife", 1, 8, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, &["multi_hit", "glass_knife"],
        "Glass Knife+", 1, 10, -1, 2, CardType::Attack, CardTarget::Enemy, false, None, &["multi_hit", "glass_knife"],
    );
    card_pair_test!(grand_finale,
        "Grand Finale", 0, 50, -1, -1, CardType::Attack, CardTarget::AllEnemy, false, None, &["only_empty_draw"],
        "Grand Finale+", 0, 60, -1, -1, CardType::Attack, CardTarget::AllEnemy, false, None, &["only_empty_draw"],
    );
    card_pair_test!(malaise,
        "Malaise", -1, -1, -1, 0, CardType::Skill, CardTarget::Enemy, true, None, &["x_cost"],
        "Malaise+", -1, -1, -1, 1, CardType::Skill, CardTarget::Enemy, true, None, &["x_cost"],
    );
    card_pair_test!(nightmare,
        "Nightmare", 3, -1, -1, 3, CardType::Skill, CardTarget::None, true, None, &["nightmare"],
        "Nightmare+", 2, -1, -1, 3, CardType::Skill, CardTarget::None, true, None, &["nightmare"],
    );
    card_pair_test!(phantasmal_killer,
        "Phantasmal Killer", 1, -1, -1, -1, CardType::Skill, CardTarget::None, false, None, &["phantasmal_killer", "ethereal"],
        "Phantasmal Killer+", 1, -1, -1, -1, CardType::Skill, CardTarget::None, false, None, &["phantasmal_killer"],
    );
    card_pair_test!(storm_of_steel,
        "Storm of Steel", 1, -1, -1, -1, CardType::Skill, CardTarget::None, false, None, &["storm_of_steel"],
        "Storm of Steel+", 1, -1, -1, -1, CardType::Skill, CardTarget::None, false, None, &["storm_of_steel"],
    );
    card_pair_test!(tools_of_the_trade,
        "Tools of the Trade", 1, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["tools_of_the_trade"],
        "Tools of the Trade+", 0, -1, -1, 1, CardType::Power, CardTarget::SelfTarget, false, None, &["tools_of_the_trade"],
    );
    card_pair_test!(unload,
        "Unload", 1, 14, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["discard_non_attacks"],
        "Unload+", 1, 18, -1, -1, CardType::Attack, CardTarget::Enemy, false, None, &["discard_non_attacks"],
    );
    card_pair_test!(wraith_form,
        "Wraith Form", 3, -1, -1, 2, CardType::Power, CardTarget::SelfTarget, false, None, &[],
        "Wraith Form+", 3, -1, -1, 3, CardType::Power, CardTarget::SelfTarget, false, None, &[],
    );

    // ---------------------------------------------------------------------
    // Breadth-first runtime checks for cards that the Rust engine already
    // wires up, plus a few exact Java-mechanic coverage checks.
    // ---------------------------------------------------------------------

    #[test]
    fn neutralize_applies_weak_and_damage() {
        let mut engine = engine_with(make_deck_n("Neutralize", 6), 40, 0);
        ensure_in_hand(&mut engine, "Neutralize");
        let hp = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Neutralize", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp - 3);
        assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 1);
    }

    #[test]
    fn backflip_blocks_and_draws() {
        let mut engine = engine_with(make_deck_n("Backflip", 8), 40, 0);
        ensure_in_hand(&mut engine, "Backflip");
        let hand_before = engine.state.hand.len();
        assert!(play_self(&mut engine, "Backflip"));
        assert_eq!(engine.state.player.block, 5);
        assert_eq!(engine.state.hand.len(), hand_before + 1);
    }

    #[test]
    fn quick_slash_draws_one() {
        let mut engine = engine_with(make_deck_n("Quick Slash", 8), 40, 0);
        ensure_in_hand(&mut engine, "Quick Slash");
        let hand_before = engine.state.hand.len();
        let hp = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Quick Slash", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp - 8);
        assert_eq!(engine.state.hand.len(), hand_before);
    }

    #[test]
    fn slice_deals_exact_damage() {
        let mut engine = engine_with(make_deck_n("Slice", 8), 40, 0);
        ensure_in_hand(&mut engine, "Slice");
        let hp = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Slice", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp - 6);
    }

    #[test]
    fn sucker_punch_applies_weak() {
        let mut engine = engine_with(make_deck_n("Sucker Punch", 8), 40, 0);
        ensure_in_hand(&mut engine, "Sucker Punch");
        assert!(play_on_enemy(&mut engine, "Sucker Punch", 0));
        assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 1);
    }

    #[test]
    fn leg_sweep_blocks_and_weakens() {
        let mut engine = engine_with(make_deck_n("Leg Sweep", 8), 40, 0);
        ensure_in_hand(&mut engine, "Leg Sweep");
        assert!(play_on_enemy(&mut engine, "Leg Sweep", 0));
        assert_eq!(engine.state.player.block, 11);
        assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 2);
    }

    #[test]
    fn dash_deals_damage_and_block() {
        let mut engine = engine_with(make_deck_n("Dash", 8), 50, 0);
        ensure_in_hand(&mut engine, "Dash");
        let hp = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Dash", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp - 10);
        assert_eq!(engine.state.player.block, 10);
    }

    #[test]
    fn escape_plan_draws_and_blocks() {
        let mut engine = engine_with(make_deck_n("Escape Plan", 8), 40, 0);
        ensure_in_hand(&mut engine, "Escape Plan");
        let hand_before = engine.state.hand.len();
        assert!(play_self(&mut engine, "Escape Plan"));
        assert_eq!(engine.state.player.block, 3);
        assert_eq!(engine.state.hand.len(), hand_before);
    }

    #[test]
    fn catalyst_doubles_poison() {
        let mut engine = engine_with(make_deck_n("Catalyst", 8), 40, 0);
        engine.state.enemies[0].entity.set_status(sid::POISON, 5);
        ensure_in_hand(&mut engine, "Catalyst");
        assert!(play_on_enemy(&mut engine, "Catalyst", 0));
        assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 10);
        assert!(engine.state.exhaust_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Catalyst"));
    }

    #[test]
    fn catalyst_plus_triples_poison() {
        let mut engine = engine_with(make_deck_n("Catalyst+", 8), 40, 0);
        engine.state.enemies[0].entity.set_status(sid::POISON, 5);
        ensure_in_hand(&mut engine, "Catalyst+");
        assert!(play_on_enemy(&mut engine, "Catalyst+", 0));
        assert_eq!(engine.state.enemies[0].entity.status(sid::POISON), 15);
    }

    #[test]
    fn terror_applies_vulnerable() {
        let mut engine = engine_with(make_deck_n("Terror", 8), 40, 0);
        ensure_in_hand(&mut engine, "Terror");
        assert!(play_on_enemy(&mut engine, "Terror", 0));
        assert_eq!(engine.state.enemies[0].entity.status(sid::VULNERABLE), 99);
        assert!(engine.state.exhaust_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Terror"));
    }

    #[test]
    fn skewer_spends_all_energy() {
        let mut engine = engine_with(make_deck_n("Skewer", 8), 100, 0);
        ensure_in_hand(&mut engine, "Skewer");
        engine.state.energy = 3;
        let hp = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Skewer", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp - 21);
        assert_eq!(engine.state.energy, 0);
    }

    #[test]
    fn riddle_with_holes_hits_five_times() {
        let mut engine = engine_with(make_deck_n("Riddle with Holes", 8), 100, 0);
        ensure_in_hand(&mut engine, "Riddle with Holes");
        let hp = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Riddle with Holes", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp - 15);
    }

    #[test]
    fn all_out_attack_hits_all_enemies() {
        let enemies = vec![
            enemy("A", 40, 40, 1, 0, 1),
            enemy("B", 40, 40, 1, 0, 1),
            enemy("C", 40, 40, 1, 0, 1),
        ];
        let mut engine = engine_with_enemies(make_deck_n("All-Out Attack", 8), enemies, 3);
        ensure_in_hand(&mut engine, "All-Out Attack");
        assert!(play_self(&mut engine, "All-Out Attack"));
        assert_eq!(engine.state.enemies[0].entity.hp, 30);
        assert_eq!(engine.state.enemies[1].entity.hp, 30);
        assert_eq!(engine.state.enemies[2].entity.hp, 30);
    }

    #[test]
    fn die_die_die_hits_all_enemies() {
        let enemies = vec![
            enemy("A", 50, 50, 1, 0, 1),
            enemy("B", 50, 50, 1, 0, 1),
        ];
        let mut engine = engine_with_enemies(make_deck_n("Die Die Die", 8), enemies, 3);
        ensure_in_hand(&mut engine, "Die Die Die");
        assert!(play_self(&mut engine, "Die Die Die"));
        assert_eq!(engine.state.enemies[0].entity.hp, 37);
        assert_eq!(engine.state.enemies[1].entity.hp, 37);
        assert!(engine.state.exhaust_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Die Die Die"));
    }

    #[test]
    fn adrenaline_gains_energy_and_draws() {
        let mut engine = engine_with(make_deck_n("Adrenaline", 8), 40, 0);
        ensure_in_hand(&mut engine, "Adrenaline");
        let energy = engine.state.energy;
        let hand_before = engine.state.hand.len();
        assert!(play_self(&mut engine, "Adrenaline"));
        assert_eq!(engine.state.energy, energy + 1);
        assert_eq!(engine.state.hand.len(), hand_before + 1);
        assert!(engine.state.exhaust_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Adrenaline"));
    }

    #[test]
    fn bullet_time_sets_statuses() {
        let mut engine = engine_with(make_deck_n("Bullet Time", 8), 40, 0);
        ensure_in_hand(&mut engine, "Bullet Time");
        assert!(play_self(&mut engine, "Bullet Time"));
        assert_eq!(engine.state.player.status(sid::BULLET_TIME), 1);
        assert_eq!(engine.state.player.status(sid::NO_DRAW), 1);
    }

    #[test]
    fn doppelganger_sets_next_turn_bonuses() {
        let mut engine = engine_with(make_deck_n("Doppelganger", 8), 40, 0);
        ensure_in_hand(&mut engine, "Doppelganger");
        engine.state.energy = 3;
        assert!(play_self(&mut engine, "Doppelganger"));
        assert_eq!(engine.state.player.status(sid::DOPPELGANGER_DRAW), 3);
        assert_eq!(engine.state.player.status(sid::DOPPELGANGER_ENERGY), 3);
        assert!(engine.state.exhaust_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Doppelganger"));
    }

    #[test]
    fn malaise_applies_weak_and_strength_down() {
        let mut engine = engine_with(make_deck_n("Malaise", 8), 40, 0);
        engine.state.enemies[0].entity.set_status(sid::STRENGTH, 4);
        ensure_in_hand(&mut engine, "Malaise");
        engine.state.energy = 3;
        assert!(play_on_enemy(&mut engine, "Malaise", 0));
        assert_eq!(engine.state.enemies[0].entity.status(sid::WEAKENED), 3);
        assert_eq!(engine.state.enemies[0].entity.strength(), 1);
    }

    #[test]
    fn wraith_form_sets_intangible() {
        let mut engine = engine_with(make_deck_n("Wraith Form", 8), 40, 0);
        ensure_in_hand(&mut engine, "Wraith Form");
        assert!(play_self(&mut engine, "Wraith Form"));
        assert_eq!(engine.state.player.status(sid::INTANGIBLE), 2);
        assert_eq!(engine.state.player.status(sid::WRAITH_FORM), 1);
    }

    #[test]
    fn grand_finale_is_blocked_when_draw_pile_is_not_empty() {
        let state = combat_state_with(
            make_deck(&["Strike_G", "Strike_G", "Strike_G", "Strike_G", "Strike_G", "Defend_G"]),
            vec![enemy("A", 60, 60, 1, 0, 1)],
            3,
        );
        let mut engine = engine_with_state(state);
        ensure_in_hand(&mut engine, "Grand Finale");
        let grand_finale_idx = engine.state.hand.iter().position(|card| engine.card_registry.card_name(card.def_id) == "Grand Finale").expect("Grand Finale should be in hand");
        assert!(
            !engine.get_legal_actions().iter().any(|action| matches!(
                action,
                Action::PlayCard { card_idx, .. } if *card_idx == grand_finale_idx
            ))
        );
        assert_eq!(hand_count(&engine, "Grand Finale"), 1);
    }

    #[test]
    fn grand_finale_hits_for_50_when_draw_pile_empty() {
        let state = combat_state_with(Vec::new(), vec![enemy("A", 90, 90, 1, 0, 1)], 3);
        let mut engine = engine_with_state(state);
        ensure_in_hand(&mut engine, "Grand Finale");
        let hp = engine.state.enemies[0].entity.hp;
        assert!(play_self(&mut engine, "Grand Finale"));
        assert_eq!(engine.state.enemies[0].entity.hp, hp - 50);
    }

    #[test]
    fn backstab_exhausts_on_play() {
        let mut engine = engine_with(make_deck_n("Backstab", 8), 40, 0);
        ensure_in_hand(&mut engine, "Backstab");
        let hp = engine.state.enemies[0].entity.hp;
        assert!(play_on_enemy(&mut engine, "Backstab", 0));
        assert_eq!(engine.state.enemies[0].entity.hp, hp - 11);
        assert!(engine.state.exhaust_pile.iter().any(|c| engine.card_registry.card_name(c.def_id) == "Backstab"));
    }

}
