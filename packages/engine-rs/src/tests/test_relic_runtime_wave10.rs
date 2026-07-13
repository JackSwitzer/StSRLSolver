#![cfg(test)]

// Java references:
// - decompiled/java-src/com/megacrit/cardcrawl/relics/HornCleat.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/CaptainsWheel.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/StoneCalendar.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/ArtOfWar.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Orichalcum.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/CloakClasp.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Damaru.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/GoldenEye.java
// - decompiled/java-src/com/megacrit/cardcrawl/actions/utility/ScryAction.java
// - decompiled/java-src/com/megacrit/cardcrawl/relics/Inserter.java

use crate::effects::runtime::EffectOwner;
use crate::engine::CombatPhase;
use crate::state::Stance;
use crate::status_ids::sid;
use crate::tests::support::{
    combat_state_with, end_turn, enemy, enemy_no_intent, engine_with_state, engine_without_start,
    make_deck, make_deck_n,
};

#[test]
fn relic_wave10_delayed_turn_block_relics_follow_runtime_path() {
    let mut horn_state = combat_state_with(
        make_deck_n("Defend", 5),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    horn_state.relics.push("HornCleat".to_string());
    let mut horn = engine_with_state(horn_state);
    assert_eq!(horn.hidden_effect_value("HornCleat", EffectOwner::PlayerRelic { slot: 0 }, 0), 1);
    end_turn(&mut horn);
    assert_eq!(horn.state.turn, 2);
    assert_eq!(horn.state.player.block, 14);
    assert_eq!(horn.hidden_effect_value("HornCleat", EffectOwner::PlayerRelic { slot: 0 }, 0), -1);

    let mut wheel_state = combat_state_with(
        make_deck_n("Defend", 5),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    wheel_state.relics.push("CaptainsWheel".to_string());
    let mut wheel = engine_with_state(wheel_state);
    assert_eq!(wheel.hidden_effect_value("CaptainsWheel", EffectOwner::PlayerRelic { slot: 0 }, 0), 1);
    end_turn(&mut wheel);
    assert_eq!(wheel.hidden_effect_value("CaptainsWheel", EffectOwner::PlayerRelic { slot: 0 }, 0), 2);
    end_turn(&mut wheel);
    assert_eq!(wheel.state.turn, 3);
    assert_eq!(wheel.state.player.block, 18);
    assert_eq!(
        wheel.hidden_effect_value("CaptainsWheel", EffectOwner::PlayerRelic { slot: 0 }, 0),
        -1
    );
}

#[test]
fn captains_wheel_resolves_before_turn_three_draw_and_foresight_choice() {
    // Sources: reference/extracted/methods/relic/CaptainsWheel.java and
    // decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java.
    let mut state = combat_state_with(
        make_deck_n("Defend", 20),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    state.relics.push("CaptainsWheel".to_string());
    let mut engine = engine_with_state(state);
    assert_eq!(engine.hidden_effect_value(
        "CaptainsWheel", EffectOwner::PlayerRelic { slot: 0 }, 0), 1);

    end_turn(&mut engine);
    assert_eq!(engine.state.turn, 2);
    assert_eq!(engine.hidden_effect_value(
        "CaptainsWheel", EffectOwner::PlayerRelic { slot: 0 }, 0), 2);

    engine.state.player.set_status(sid::FORESIGHT, 1);
    end_turn(&mut engine);
    assert_eq!(engine.state.turn, 3);
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(engine.state.player.block, 18,
        "Captain's Wheel resolves before Foresight's post-draw scry choice");
    assert_eq!(engine.hidden_effect_value(
        "CaptainsWheel", EffectOwner::PlayerRelic { slot: 0 }, 0), -1);
}

#[test]
fn damaru_gains_mantra_and_enters_divinity_before_post_draw_choices() {
    // Source: reference/extracted/methods/relic/Damaru.java (`atTurnStart`
    // queues exactly one Mantra before GameActionManager queues the draw).
    let mut state = combat_state_with(
        make_deck_n("Defend", 20),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    state.relics.push("Damaru".to_string());
    let mut engine = engine_with_state(state);
    assert_eq!(engine.state.mantra, 1);

    engine.state.mantra = 9;
    engine.state.player.set_status(sid::FORESIGHT, 1);
    end_turn(&mut engine);
    assert_eq!(engine.state.turn, 2);
    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(engine.state.mantra, 0);
    assert_eq!(engine.state.stance, Stance::Divinity,
        "Damaru's tenth Mantra resolves before Foresight pauses after draw");
}

#[test]
fn golden_eye_adds_exactly_two_cards_to_every_scry_action() {
    // Source: ScryAction.java adds two in its constructor when the player has
    // GoldenEye, then caps the selection group at the current draw-pile size.
    let mut plain = engine_without_start(
        make_deck_n("Defend", 5),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    plain.do_scry(1);
    assert_eq!(plain.choice.as_ref().expect("plain Scry choice").options.len(), 1);

    let mut golden = engine_without_start(
        make_deck_n("Defend", 5),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    golden.state.relics.push("GoldenEye".to_string());
    golden.do_scry(1);
    let choice = golden.choice.as_ref().expect("GoldenEye Scry choice");
    assert_eq!(choice.options.len(), 3);
    assert_eq!(choice.max_picks, 3);
}

#[test]
fn relic_wave10_turn_end_runtime_relics_match_java_timing() {
    let mut calendar_state = combat_state_with(
        make_deck_n("Defend", 10),
        vec![enemy_no_intent("JawWorm", 120, 120), enemy_no_intent("Cultist", 120, 120)],
        3,
    );
    calendar_state.relics.push("StoneCalendar".to_string());
    let mut calendar = engine_with_state(calendar_state);
    for _ in 0..6 {
        end_turn(&mut calendar);
    }
    assert_eq!(calendar.hidden_effect_value("StoneCalendar", EffectOwner::PlayerRelic { slot: 0 }, 0), 7);
    let hp0 = calendar.state.enemies[0].entity.hp;
    let hp1 = calendar.state.enemies[1].entity.hp;
    end_turn(&mut calendar);
    assert_eq!(calendar.state.enemies[0].entity.hp, hp0 - 52);
    assert_eq!(calendar.state.enemies[1].entity.hp, hp1 - 52);

    let mut orichalcum_state = combat_state_with(
        make_deck_n("Defend", 5),
        vec![enemy("JawWorm", 60, 60, 1, 5, 1)],
        3,
    );
    // Source: reference/extracted/methods/relic/Orichalcum.java
    // onPlayerEndTurn queues exactly 6 Block when currentBlock is zero.
    orichalcum_state.relics.push("Orichalcum".to_string());
    let mut orichalcum = engine_with_state(orichalcum_state);
    let hp_before = orichalcum.state.player.hp;
    end_turn(&mut orichalcum);
    assert_eq!(orichalcum.state.player.hp, hp_before);

    let mut blocked_state = combat_state_with(
        make_deck_n("Defend", 5),
        vec![enemy("JawWorm", 60, 60, 1, 5, 1)],
        3,
    );
    blocked_state.relics.push("Orichalcum".to_string());
    blocked_state.player.block = 1;
    let mut blocked = engine_with_state(blocked_state);
    let hp_before = blocked.state.player.hp;
    end_turn(&mut blocked);
    assert_eq!(blocked.state.player.hp, hp_before - 4);

    let mut clasp_state = combat_state_with(
        make_deck(&["Defend", "Defend", "Defend"]),
        vec![enemy("JawWorm", 60, 60, 1, 3, 1)],
        3,
    );
    clasp_state.relics.push("CloakClasp".to_string());
    let mut clasp = engine_with_state(clasp_state);
    clasp.state.hand = make_deck(&["Defend", "Defend"]);
    clasp.state.draw_pile.clear();
    let hp_before = clasp.state.player.hp;
    end_turn(&mut clasp);
    assert_eq!(clasp.state.player.hp, hp_before - 1);
}

#[test]
fn cloak_clasp_counts_the_current_hand_before_discard() {
    // Source: reference/extracted/methods/relic/CloakClasp.java
    // (`onPlayerEndTurn` gains exactly hand.group.size() Block when nonempty).
    let mut four_state = combat_state_with(
        make_deck_n("Defend", 5),
        vec![enemy("JawWorm", 60, 60, 1, 5, 1)],
        3,
    );
    four_state.relics.push("CloakClasp".to_string());
    let mut four = engine_with_state(four_state);
    four.state.hand = make_deck_n("Defend", 4);
    four.state.draw_pile.clear();
    let hp_before = four.state.player.hp;
    end_turn(&mut four);
    assert_eq!(four.state.player.hp, hp_before - 1,
        "four held cards grant four Block before the five-damage attack");

    let mut empty_state = combat_state_with(
        make_deck_n("Defend", 5),
        vec![enemy("JawWorm", 60, 60, 1, 5, 1)],
        3,
    );
    empty_state.relics.push("CloakClasp".to_string());
    let mut empty = engine_with_state(empty_state);
    empty.state.hand.clear();
    empty.state.draw_pile.clear();
    let hp_before = empty.state.player.hp;
    end_turn(&mut empty);
    assert_eq!(empty.state.player.hp, hp_before - 5,
        "an empty hand queues no block");
}

#[test]
fn relic_wave10_turn_progression_relics_follow_runtime_path() {
    let mut art_state = combat_state_with(
        make_deck_n("Defend", 5),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    art_state.relics.push("Art of War".to_string());
    let mut art = engine_with_state(art_state);
    end_turn(&mut art);
    assert_eq!(art.state.turn, 2);
    assert_eq!(art.state.energy, 4);
    assert_eq!(art.hidden_effect_value("Art of War", EffectOwner::PlayerRelic { slot: 0 }, 0), 1);

    let mut damaru_state = combat_state_with(
        make_deck_n("Defend", 5),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    damaru_state.relics.push("Damaru".to_string());
    let mut damaru = engine_with_state(damaru_state);
    assert_eq!(damaru.state.mantra, 1);
    damaru.state.mantra = 9;
    end_turn(&mut damaru);
    assert_eq!(damaru.state.stance, crate::state::Stance::Divinity);
    assert_eq!(damaru.state.mantra, 0);

    let mut inserter_state = combat_state_with(
        make_deck_n("Defend", 5),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    inserter_state.relics.push("Inserter".to_string());
    let mut inserter = engine_with_state(inserter_state);
    assert_eq!(inserter.state.player.status(crate::status_ids::sid::ORB_SLOTS), 0);
    assert_eq!(inserter.hidden_effect_value("Inserter", EffectOwner::PlayerRelic { slot: 0 }, 0), 1);
    end_turn(&mut inserter);
    assert_eq!(inserter.state.player.status(crate::status_ids::sid::ORB_SLOTS), 1);
    assert_eq!(inserter.hidden_effect_value("Inserter", EffectOwner::PlayerRelic { slot: 0 }, 0), 0);
}

#[test]
fn inserter_grows_live_orb_slots_every_second_turn_and_respects_the_cap() {
    // Source: reference/extracted/methods/relic/Inserter.java and
    // IncreaseMaxOrbAction.java. atTurnStart increments the counter, resets it
    // at two, and grows the live orb collection by one up to ten slots.
    let mut state = combat_state_with(
        make_deck_n("Defend", 20),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    state.relics.push("Inserter".to_string());
    let mut engine = engine_with_state(state);
    engine.init_defect_orbs(3);

    assert_eq!(engine.state.orb_slots.get_slot_count(), 3);
    assert_eq!(engine.hidden_effect_value("Inserter", EffectOwner::PlayerRelic { slot: 0 }, 0), 1);
    end_turn(&mut engine);
    assert_eq!(engine.state.orb_slots.get_slot_count(), 4);
    assert_eq!(engine.state.player.status(sid::ORB_SLOTS), 1);
    assert_eq!(engine.hidden_effect_value("Inserter", EffectOwner::PlayerRelic { slot: 0 }, 0), 0);

    end_turn(&mut engine);
    assert_eq!(engine.state.orb_slots.get_slot_count(), 4);
    end_turn(&mut engine);
    assert_eq!(engine.state.orb_slots.get_slot_count(), 5);
    assert_eq!(engine.state.player.status(sid::ORB_SLOTS), 2);

    let mut capped_state = combat_state_with(
        make_deck_n("Defend", 10),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    capped_state.relics.push("Inserter".to_string());
    let mut capped = engine_with_state(capped_state);
    capped.init_defect_orbs(10);
    end_turn(&mut capped);
    assert_eq!(capped.state.orb_slots.get_slot_count(), 10);
    assert_eq!(capped.state.player.status(sid::ORB_SLOTS), 0);
    assert_eq!(capped.hidden_effect_value("Inserter", EffectOwner::PlayerRelic { slot: 0 }, 0), 0);
}
