#[cfg(test)]
mod event_java_parity_tests {
    use crate::events::{events_for_act, shrine_events, EventDef, EventEffect};

    fn event(act: i32, name: &str) -> EventDef {
        events_for_act(act)
            .into_iter()
            .find(|event| event.name == name)
            .unwrap_or_else(|| panic!("missing event {name} in act {act}"))
    }

    #[test]
    fn big_fish_has_three_java_options() {
        let e = event(1, "Big Fish");
        assert_eq!(e.options.len(), 3);
    }
    #[test]
    fn big_fish_eat_option_heals_five_hp() {
        let e = event(1, "Big Fish");
        assert!(matches!(e.options[0].effect, EventEffect::Hp(5)));
    }
    #[test]
    fn big_fish_banana_option_gains_two_max_hp() {
        let e = event(1, "Big Fish");
        assert!(matches!(e.options[1].effect, EventEffect::MaxHp(2)));
    }
    #[test]
    fn golden_idol_take_option_matches_rust_port_values() {
        let e = event(1, "Golden Idol");
        assert!(matches!(e.options[0].effect, EventEffect::GoldenIdolTake));
    }
    #[test]
    fn golden_idol_leave_option_does_nothing() {
        let e = event(1, "Golden Idol");
        assert!(matches!(e.options[1].effect, EventEffect::Nothing));
    }
    #[test]
    fn scrap_ooze_reach_inside_is_three_damage_relic_roll() {
        let e = event(1, "Scrap Ooze");
        assert!(matches!(e.options[0].effect, EventEffect::DamageAndGold(-3, 0)));
    }
    #[test]
    fn shining_light_enter_is_ten_damage_placeholder() {
        let e = event(1, "Shining Light");
        assert!(matches!(e.options[0].effect, EventEffect::DamageAndGold(-10, 0)));
    }
    #[test]
    fn living_wall_upgrade_option_exists() {
        let e = event(1, "Living Wall");
        assert!(matches!(e.options[0].effect, EventEffect::UpgradeCard));
    }
    #[test]
    fn living_wall_remove_option_exists() {
        let e = event(1, "Living Wall");
        assert!(matches!(e.options[1].effect, EventEffect::RemoveCard));
    }
    #[test]
    fn forgotten_altar_offer_costs_five_hp_in_rust_port() {
        let e = event(2, "Forgotten Altar");
        assert!(matches!(e.options[0].effect, EventEffect::Hp(-5)));
    }
    #[test]
    fn council_of_ghosts_accept_reduces_max_hp_by_five() {
        let e = event(2, "Council of Ghosts");
        assert!(matches!(e.options[0].effect, EventEffect::MaxHp(-5)));
    }
    #[test]
    fn masked_bandits_pay_option_uses_gold_placeholder() {
        let e = event(2, "Masked Bandits");
        assert!(matches!(e.options[0].effect, EventEffect::Gold(-999)));
    }
    #[test]
    fn knowing_skull_gold_option_matches_current_rust_values() {
        let e = event(2, "Knowing Skull");
        assert!(matches!(e.options[0].effect, EventEffect::DamageAndGold(-6, 90)));
    }
    #[test]
    fn vampires_accept_option_is_remove_card_placeholder() {
        let e = event(2, "Vampires");
        assert!(matches!(e.options[0].effect, EventEffect::RemoveCard));
    }
    #[test]
    fn mysterious_sphere_open_option_grants_relic_placeholder() {
        let e = event(3, "Mysterious Sphere");
        assert!(matches!(e.options[0].effect, EventEffect::GainRelic));
    }
    #[test]
    fn mind_bloom_rich_option_grants_999_gold() {
        let e = event(3, "Mind Bloom");
        assert!(matches!(e.options[2].effect, EventEffect::Gold(999)));
    }
    #[test]
    fn tomb_of_lord_red_mask_mask_option_grants_relic() {
        let e = event(3, "Tomb of Lord Red Mask");
        assert!(matches!(e.options[0].effect, EventEffect::GainRelic));
    }
    #[test]
    fn sensory_stone_focus_option_grants_card() {
        let e = event(3, "Sensory Stone");
        assert!(matches!(e.options[0].effect, EventEffect::GainCard));
    }
    #[test]
    fn secret_portal_enter_option_is_placeholder_nothing() {
        let e = event(3, "Secret Portal");
        assert!(matches!(e.options[0].effect, EventEffect::Nothing));
    }
    #[test]
    fn act1_java_catalog_has_eleven_events_not_five() {
        assert_eq!(events_for_act(1).len(), 11);
    }
    #[test]
    fn act2_java_catalog_has_fifteen_events_not_five() {
        assert_eq!(events_for_act(2).len(), 15);
    }
    #[test]
    fn act3_java_catalog_has_nine_events_not_five() {
        assert_eq!(events_for_act(3).len(), 9);
    }
    #[test]
    fn shrine_catalog_has_seventeen_events() {
        assert_eq!(shrine_events().len(), 17);
    }
    #[test]
    fn shrine_catalog_contains_key_java_events() {
        let names: Vec<String> = shrine_events().into_iter().map(|e| e.name).collect();
        for expected in [
            "Duplicator", "The Woman in Blue", "FaceTrader", "Designer",
            "N'loth", "Accursed Blacksmith", "Bonfire Elementals",
            "Fountain of Cleansing", "Golden Shrine", "Match and Keep!",
            "Wheel of Change", "Lab", "NoteForYourself", "Purifier",
            "Transmorgrifier", "Upgrade Shrine", "WeMeetAgain",
        ] {
            assert!(names.contains(&expected.to_string()), "missing shrine event: {expected}");
        }
    }
}
