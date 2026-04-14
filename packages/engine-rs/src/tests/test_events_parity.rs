#[cfg(test)]
mod event_java_parity_tests {
    use crate::events::{
        events_for_act, shrine_events, typed_events_for_act, typed_shrine_events,
        EventProgramOp, EventRuntimeStatus, TypedEventDef,
    };

    fn typed_event(act: i32, name: &str) -> TypedEventDef {
        typed_events_for_act(act)
            .into_iter()
            .find(|event| event.name == name)
            .unwrap_or_else(|| panic!("missing typed event {name} in act {act}"))
    }

    fn typed_shrine_event(name: &str) -> TypedEventDef {
        typed_shrine_events()
            .into_iter()
            .find(|event| event.name == name)
            .unwrap_or_else(|| panic!("missing typed shrine event {name}"))
    }

    fn assert_blocked(event: &TypedEventDef, option_index: usize, reason: &str) {
        match &event.options[option_index].status {
            EventRuntimeStatus::Blocked { reason: actual } => {
                assert!(
                    actual.contains(reason),
                    "blocked reason `{actual}` did not contain `{reason}`"
                );
            }
            other => panic!("expected blocked option, found {other:?}"),
        }
    }

    #[test]
    fn typed_and_legacy_catalog_sizes_match_current_port_target() {
        assert_eq!(typed_events_for_act(1).len(), events_for_act(1).len());
        assert_eq!(typed_events_for_act(2).len(), events_for_act(2).len());
        assert_eq!(typed_events_for_act(3).len(), events_for_act(3).len());
        assert_eq!(typed_shrine_events().len(), shrine_events().len());
    }

    #[test]
    fn typed_catalog_contains_expected_java_event_names() {
        let names: Vec<String> = typed_shrine_events().into_iter().map(|e| e.name).collect();
        for expected in [
            "Duplicator",
            "The Woman in Blue",
            "FaceTrader",
            "Designer",
            "N'loth",
            "Accursed Blacksmith",
            "Bonfire Elementals",
            "Fountain of Cleansing",
            "Golden Shrine",
            "Match and Keep!",
            "Wheel of Change",
            "Lab",
            "NoteForYourself",
            "Purifier",
            "Transmorgrifier",
            "Upgrade Shrine",
            "WeMeetAgain",
        ] {
            assert!(names.contains(&expected.to_string()), "missing shrine event: {expected}");
        }
    }

    #[test]
    fn typed_event_programs_express_reward_like_outcomes() {
        let big_fish = typed_event(1, "Big Fish");
        assert!(matches!(
            big_fish.options[0].program.ops.as_slice(),
            [EventProgramOp::AdjustHp { amount: 5 }]
        ));
        assert!(matches!(
            big_fish.options[1].program.ops.as_slice(),
            [EventProgramOp::AdjustMaxHp { amount: 2 }]
        ));

        let golden_idol = typed_event(1, "Golden Idol");
        assert!(matches!(
            golden_idol.options[0].program.ops.as_slice(),
            [
                EventProgramOp::LosePercentHp { percent: 25 },
                EventProgramOp::Reward(_)
            ]
        ));

        let cleric = typed_event(1, "The Cleric");
        assert!(matches!(
            cleric.options[0].program.ops.as_slice(),
            [
                EventProgramOp::AdjustGold { amount: -35 },
                EventProgramOp::HealPercentHp { percent: 25 }
            ]
        ));
        assert!(matches!(
            cleric.options[1].program.ops.as_slice(),
            [
                EventProgramOp::AdjustGold { amount: -50 },
                EventProgramOp::DeckMutation(_)
            ]
        ));

        let golden_shrine = typed_shrine_event("Golden Shrine");
        assert!(matches!(
            golden_shrine.options[1].program.ops.as_slice(),
            [
                EventProgramOp::AdjustGold { amount: 275 },
                EventProgramOp::Reward(_)
            ]
        ));

        let woman_in_blue = typed_shrine_event("The Woman in Blue");
        assert!(matches!(
            woman_in_blue.options[2].program.ops.as_slice(),
            [
                EventProgramOp::AdjustGold { amount: -40 },
                EventProgramOp::Reward(_)
            ]
        ));
    }

    #[test]
    fn blocked_placeholder_events_are_explicit_in_the_typed_catalog() {
        let dead_adventurer = typed_event(1, "Dead Adventurer");
        assert_blocked(
            &dead_adventurer,
            0,
            "requires persistent search-state tracking for encounter chance ramp",
        );

        let golden_wing = typed_event(1, "Golden Wing");
        assert_blocked(
            &golden_wing,
            1,
            "requires Java-style deck damage-threshold evaluation",
        );

    }

    #[test]
    fn typed_runtime_supported_branches_are_no_longer_marked_blocked() {
        let golden_wing = typed_event(1, "Golden Wing");
        assert!(matches!(
            golden_wing.options[0].status,
            EventRuntimeStatus::Supported
        ));

        let addict = typed_event(2, "Addict");
        assert!(matches!(addict.options[0].status, EventRuntimeStatus::Supported));
        assert!(matches!(addict.options[1].status, EventRuntimeStatus::Supported));

        let the_joust = typed_event(2, "The Joust");
        assert!(matches!(
            the_joust.options[0].status,
            EventRuntimeStatus::Supported
        ));
        assert!(matches!(
            the_joust.options[1].status,
            EventRuntimeStatus::Supported
        ));

        let the_library = typed_event(2, "The Library");
        assert!(matches!(
            the_library.options[0].status,
            EventRuntimeStatus::Supported
        ));
        assert!(matches!(
            the_library.options[1].status,
            EventRuntimeStatus::Supported
        ));

        let colosseum = typed_event(2, "Colosseum");
        assert!(matches!(
            colosseum.options[0].status,
            EventRuntimeStatus::Supported
        ));

        let cursed_tome = typed_event(2, "Cursed Tome");
        assert!(matches!(
            cursed_tome.options[0].status,
            EventRuntimeStatus::Supported
        ));
        assert!(matches!(
            cursed_tome.options[1].status,
            EventRuntimeStatus::Supported
        ));

        let winding_halls = typed_event(3, "Winding Halls");
        assert!(matches!(
            winding_halls.options[0].status,
            EventRuntimeStatus::Supported
        ));
        assert!(matches!(
            winding_halls.options[1].status,
            EventRuntimeStatus::Supported
        ));
        assert!(matches!(
            winding_halls.options[2].status,
            EventRuntimeStatus::Supported
        ));

        let mind_bloom = typed_event(3, "Mind Bloom");
        assert!(matches!(
            mind_bloom.options[0].status,
            EventRuntimeStatus::Supported
        ));
        assert!(matches!(
            mind_bloom.options[1].status,
            EventRuntimeStatus::Supported
        ));
        assert!(matches!(
            mind_bloom.options[2].status,
            EventRuntimeStatus::Supported
        ));

        let mushrooms = typed_event(1, "Mushrooms");
        assert!(matches!(
            mushrooms.options[0].status,
            EventRuntimeStatus::Supported
        ));

        let mysterious_sphere = typed_event(3, "Mysterious Sphere");
        assert!(matches!(
            mysterious_sphere.options[0].status,
            EventRuntimeStatus::Supported
        ));

        let secret_portal = typed_event(3, "Secret Portal");
        assert!(matches!(
            secret_portal.options[0].status,
            EventRuntimeStatus::Supported
        ));

        let spire_heart = typed_event(3, "Spire Heart");
        assert!(matches!(
            spire_heart.options[0].status,
            EventRuntimeStatus::Supported
        ));

        let wheel = typed_shrine_event("Wheel of Change");
        assert!(matches!(wheel.options[0].status, EventRuntimeStatus::Supported));

        let bonfire = typed_shrine_event("Bonfire Elementals");
        assert!(matches!(
            bonfire.options[0].status,
            EventRuntimeStatus::Supported
        ));
    }

    #[test]
    fn blocked_placeholder_op_count_matches_remaining_shared_runtime_gaps() {
        let blocked_placeholder_count: usize = typed_events_for_act(1)
            .into_iter()
            .chain(typed_events_for_act(2))
            .chain(typed_events_for_act(3))
            .chain(typed_shrine_events())
            .flat_map(|event| event.options.into_iter())
            .flat_map(|option| option.program.ops.into_iter())
            .filter(|op| matches!(op, EventProgramOp::BlockedPlaceholder { .. }))
            .count();
        assert_eq!(blocked_placeholder_count, 0);
    }

    #[test]
    fn legacy_effects_remain_available_for_run_rs_compatibility() {
        let big_fish = events_for_act(1)
            .into_iter()
            .find(|event| event.name == "Big Fish")
            .expect("missing Big Fish");
        assert_eq!(big_fish.options.len(), 3);

        let golden_idol = events_for_act(1)
            .into_iter()
            .find(|event| event.name == "Golden Idol")
            .expect("missing Golden Idol");
        assert!(matches!(golden_idol.options[0].effect, crate::events::EventEffect::GoldenIdolTake));

        let we_meet_again = shrine_events()
            .into_iter()
            .find(|event| event.name == "WeMeetAgain")
            .expect("missing WeMeetAgain shrine");
        assert_eq!(we_meet_again.options.len(), 4);
    }
}
