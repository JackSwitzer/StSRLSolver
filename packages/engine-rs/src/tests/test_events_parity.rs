// Java references:
// /tmp/sts-decompiled/com/megacrit/cardcrawl/events/exordium/{BigFish.java,GoldenIdolEvent.java,ScrapOoze.java,ShiningLight.java,LivingWall.java}
// /tmp/sts-decompiled/com/megacrit/cardcrawl/events/city/{ForgottenAltar.java,Ghosts.java,MaskedBandits.java,KnowingSkull.java,Vampires.java}
// /tmp/sts-decompiled/com/megacrit/cardcrawl/events/beyond/{MindBloom.java,MysteriousSphere.java,SensoryStone.java,SecretPortal.java,TombRedMask.java}
// /tmp/sts-decompiled/com/megacrit/cardcrawl/events/shrines/*.java

#[cfg(test)]
mod event_java_parity_tests {
    use crate::events::{events_for_act, EventDef, EventEffect};

    fn event(act: i32, name: &str) -> EventDef {
        events_for_act(act)
            .into_iter()
            .find(|event| event.name == name)
            .unwrap_or_else(|| panic!("missing event {name} in act {act}"))
    }

    #[test]
    fn big_fish_has_three_java_options() {
        let event = event(1, "Big Fish");
        assert_eq!(event.options.len(), 3);
    }

    #[test]
    fn big_fish_eat_option_heals_five_hp() {
        let event = event(1, "Big Fish");
        assert!(matches!(event.options[0].effect, EventEffect::Hp(5)));
    }

    #[test]
    fn big_fish_banana_option_gains_two_max_hp() {
        let event = event(1, "Big Fish");
        assert!(matches!(event.options[1].effect, EventEffect::MaxHp(2)));
    }

    #[test]
    fn golden_idol_take_option_matches_rust_port_values() {
        let event = event(1, "Golden Idol");
        assert!(matches!(event.options[0].effect, EventEffect::GoldenIdolTake));
    }

    #[test]
    fn golden_idol_leave_option_does_nothing() {
        let event = event(1, "Golden Idol");
        assert!(matches!(event.options[1].effect, EventEffect::Nothing));
    }

    #[test]
    fn scrap_ooze_reach_inside_is_three_damage_relic_roll() {
        let event = event(1, "Scrap Ooze");
        assert!(matches!(event.options[0].effect, EventEffect::DamageAndGold(-3, 0)));
    }

    #[test]
    fn shining_light_enter_is_ten_damage_placeholder() {
        let event = event(1, "Shining Light");
        assert!(matches!(event.options[0].effect, EventEffect::DamageAndGold(-10, 0)));
    }

    #[test]
    fn living_wall_upgrade_option_exists() {
        let event = event(1, "Living Wall");
        assert!(matches!(event.options[0].effect, EventEffect::UpgradeCard));
    }

    #[test]
    fn living_wall_remove_option_exists() {
        let event = event(1, "Living Wall");
        assert!(matches!(event.options[1].effect, EventEffect::RemoveCard));
    }

    #[test]
    fn forgotten_altar_offer_costs_five_hp_in_rust_port() {
        let event = event(2, "Forgotten Altar");
        assert!(matches!(event.options[0].effect, EventEffect::Hp(-5)));
    }

    #[test]
    fn council_of_ghosts_accept_reduces_max_hp_by_five() {
        let event = event(2, "Council of Ghosts");
        assert!(matches!(event.options[0].effect, EventEffect::MaxHp(-5)));
    }

    #[test]
    fn masked_bandits_pay_option_uses_gold_placeholder() {
        let event = event(2, "Masked Bandits");
        assert!(matches!(event.options[0].effect, EventEffect::Gold(-999)));
    }

    #[test]
    fn knowing_skull_gold_option_matches_current_rust_values() {
        let event = event(2, "Knowing Skull");
        assert!(matches!(event.options[0].effect, EventEffect::DamageAndGold(-6, 90)));
    }

    #[test]
    fn vampires_accept_option_is_remove_card_placeholder() {
        let event = event(2, "Vampires");
        assert!(matches!(event.options[0].effect, EventEffect::RemoveCard));
    }

    #[test]
    fn mysterious_sphere_open_option_grants_relic_placeholder() {
        let event = event(3, "Mysterious Sphere");
        assert!(matches!(event.options[0].effect, EventEffect::GainRelic));
    }

    #[test]
    fn mind_bloom_rich_option_grants_999_gold() {
        let event = event(3, "Mind Bloom");
        assert!(matches!(event.options[2].effect, EventEffect::Gold(999)));
    }

    #[test]
    fn tomb_of_lord_red_mask_mask_option_grants_relic() {
        let event = event(3, "Tomb of Lord Red Mask");
        assert!(matches!(event.options[0].effect, EventEffect::GainRelic));
    }

    #[test]
    fn sensory_stone_focus_option_grants_card() {
        let event = event(3, "Sensory Stone");
        assert!(matches!(event.options[0].effect, EventEffect::GainCard));
    }

    #[test]
    fn secret_portal_enter_option_is_placeholder_nothing() {
        let event = event(3, "Secret Portal");
        assert!(matches!(event.options[0].effect, EventEffect::Nothing));
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
    fn shrine_catalog_exists_in_java_but_not_in_rust_dispatch() {
        let modeled_names: Vec<String> = [1, 2, 3]
            .into_iter()
            .flat_map(events_for_act)
            .map(|event| {
                event
                    .name
                    .chars()
                    .filter(|ch| ch.is_ascii_alphanumeric())
                    .collect::<String>()
                    .to_ascii_lowercase()
            })
            .collect();
        for shrine_name in ["duplicator", "womaninblue", "facetrader", "designer", "nloth"] {
            assert!(
                !modeled_names.iter().any(|name| name == shrine_name),
                "expected shrine event {shrine_name} to still be missing from the Rust dispatch"
            );
        }
    }
}
