#![cfg(test)]

use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, engine_without_start, make_deck_n};

fn start_with_relics(relics: &[&str], deck_size: usize) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(
        make_deck_n("Strike", deck_size),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    engine.state.relics = relics.iter().map(|id| (*id).to_string()).collect();
    engine.start_combat();
    engine
}

#[test]
fn blood_vial_top_action_resolves_before_red_skull_bottom_action() {
    // BloodVial queues HealAction with addToTop, while RedSkull queues its
    // bloodied check with addToBot. The heal therefore wins in either relic
    // inventory order.
    // Java: BloodVial.java::atBattleStart; RedSkull.java::atBattleStart.
    for relics in [["Red Skull", "Blood Vial"], ["Blood Vial", "Red Skull"]] {
        let mut engine = engine_without_start(
            make_deck_n("Strike", 5),
            vec![enemy_no_intent("JawWorm", 50, 50)],
            3,
        );
        engine.state.player.hp = 39;
        engine.state.relics = relics.iter().map(|id| (*id).to_string()).collect();
        engine.start_combat();

        assert_eq!(engine.state.player.hp, 41);
        assert_eq!(engine.state.player.status(sid::STRENGTH), 0);
    }
}

#[test]
fn clockwork_top_action_blocks_gremlin_mask_bottom_action_in_both_orders() {
    // ClockworkSouvenir uses addToTop and GremlinMask uses addToBot, so the
    // Artifact is present before Weak is attempted regardless of inventory.
    // Java: ClockworkSouvenir.java and GremlinMask.java.
    for relics in [
        ["GremlinMask", "ClockworkSouvenir"],
        ["ClockworkSouvenir", "GremlinMask"],
    ] {
        let engine = start_with_relics(&relics, 5);
        assert_eq!(engine.state.player.status(sid::WEAKENED), 0);
        assert_eq!(engine.state.player.status(sid::ARTIFACT), 0);
    }
}

#[test]
fn top_actions_reverse_relic_inventory_order_like_java() {
    // Later atBattleStart callbacks put their actions above earlier callbacks.
    // This makes Clockwork protect Mutagenic Strength only when Clockwork is
    // later in the inventory.
    // Java: AbstractPlayer.java::applyStartOfCombatLogic,
    // ClockworkSouvenir.java, MutagenicStrength.java.
    let clockwork_first = start_with_relics(&["ClockworkSouvenir", "MutagenicStrength"], 5);
    assert_eq!(clockwork_first.state.player.status(sid::STRENGTH), 3);
    assert_eq!(clockwork_first.state.player.status(sid::LOSE_STRENGTH), 3);
    assert_eq!(clockwork_first.state.player.status(sid::ARTIFACT), 1);

    let clockwork_last = start_with_relics(&["MutagenicStrength", "ClockworkSouvenir"], 5);
    assert_eq!(clockwork_last.state.player.status(sid::STRENGTH), 3);
    assert_eq!(clockwork_last.state.player.status(sid::LOSE_STRENGTH), 0);
    assert_eq!(clockwork_last.state.player.status(sid::ARTIFACT), 0);
}

#[test]
fn mark_of_pain_inserts_wounds_after_the_opening_draw() {
    // AbstractRoom queues the opening DrawCardAction before MarkOfPain's
    // addToBot MakeTempCardInDrawPileAction. A five-card deck is therefore
    // fully drawn before either Wound is inserted.
    // Java: AbstractRoom.java:240-258; MarkOfPain.java::atBattleStart.
    let engine = start_with_relics(&["Mark of Pain"], 5);

    assert_eq!(engine.state.hand.len(), 5);
    assert!(engine
        .state
        .hand
        .iter()
        .all(|card| engine.card_registry.card_name(card.def_id) == "Strike"));
    assert_eq!(engine.state.draw_pile.len(), 2);
    assert!(engine
        .state
        .draw_pile
        .iter()
        .all(|card| engine.card_registry.card_name(card.def_id) == "Wound"));
}

#[test]
fn direct_combat_start_power_precedes_queued_bottom_apply_power() {
    // AbstractPlayer invokes all atBattleStart callbacks before the action
    // manager drains. Philosopher's Stone directly appends Strength during its
    // callback; Bag of Marbles only queues ApplyPowerAction. That later sorted
    // apply therefore keeps Java's [Strength, Vulnerable] power-list order,
    // even when Bag of Marbles is earlier in relic inventory order.
    // Java: AbstractPlayer.java::applyStartOfCombatLogic;
    // PhilosopherStone.java::atBattleStart; BagOfMarbles.java::atBattleStart;
    // ApplyPowerAction.java::update.
    let engine = start_with_relics(&["Bag of Marbles", "Philosopher's Stone"], 5);

    assert_eq!(
        engine.state.enemies[0].entity.ordered_status_ids(),
        vec![sid::STRENGTH, sid::VULNERABLE]
    );
}

#[test]
fn direct_combat_start_hp_mutations_precede_turn_start_damage() {
    // Neow's Lament and Preserved Insect mutate monster HP synchronously in
    // their atBattleStart callbacks. Mercury Hourglass only queues its
    // atTurnStart DAMAGE action later. Java therefore applies the HP mutation
    // first in both cases.
    // Java: NeowsLament.java::atBattleStart;
    // PreservedInsect.java::atBattleStart; MercuryHourglass.java::atTurnStart.
    let mut lament = engine_without_start(
        make_deck_n("Strike", 5),
        vec![enemy_no_intent("JawWorm", 20, 20)],
        3,
    );
    lament.state.relics = vec!["NeowsBlessing".to_string(), "Mercury Hourglass".to_string()];
    lament.state.relic_counters[crate::relic_flags::counter::NEOWS_LAMENT] = 3;
    lament.start_combat();
    assert_eq!(lament.state.enemies[0].entity.hp, 0);

    let mut insect = engine_without_start(
        make_deck_n("Strike", 5),
        vec![enemy_no_intent("Sentry", 20, 20)],
        3,
    );
    insect.state.relics = vec![
        "PreservedInsect".to_string(),
        "Mercury Hourglass".to_string(),
    ];
    insect.start_combat();
    assert_eq!(insect.state.enemies[0].entity.hp, 12);
}

#[test]
fn combat_start_fifo_settles_before_first_turn_relic_actions() {
    // AbstractRoom queues atBattleStart bottom actions before it visits
    // atTurnStart relics. Both powers have Java priority 5, so Fossilized
    // Helix's Buffer must enter the ordered list before Damaru's Mantra.
    // Java: AbstractRoom.java::update; FossilizedHelix.java::atBattleStart;
    // Damaru.java::atTurnStart; ApplyPowerAction.java::update.
    let engine = start_with_relics(&["FossilizedHelix", "Damaru"], 5);
    assert_eq!(
        engine.state.player.ordered_status_ids(),
        vec![sid::BUFFER, sid::MANTRA]
    );
}

#[test]
fn all_stateful_add_to_top_combat_start_relics_use_the_top_phase() {
    // Each Java atBattleStart implementation below calls
    // addToTop(ApplyPowerAction). Pinning the phase prevents inventory-order
    // callbacks from being flattened into the FIFO action pass again.
    // Java: the matching relic class for every ID in this table.
    for relic_id in [
        "Bronze Scales",
        "DataDisk",
        "Du-Vu Doll",
        "Girya",
        "Oddly Smooth Stone",
        "Sling",
        "Thread and Needle",
    ] {
        let def = crate::relics::defs::relic_def_by_id(relic_id).expect(relic_id);
        assert_eq!(
            def.triggers[0].trigger,
            crate::effects::trigger::Trigger::CombatStartTop,
            "{relic_id}"
        );
    }
}

#[test]
fn ring_of_the_snake_and_mark_of_pain_preserve_inventory_fifo_order() {
    // Both relics use addToBot in atBattleStart, so inventory order is causal.
    // With only the five opening cards, Ring first finds no cards and Mark then
    // inserts two Wounds; Mark first inserts the Wounds (consuming its random
    // insertion path) and Ring then draws both. Modeling Ring as atTurnStart
    // incorrectly forced Mark to run first in both inventories.
    // Java: SnakeRing.java::atBattleStart; MarkOfPain.java::atBattleStart;
    // MakeTempCardInDrawPileAction.java; DrawCardAction.java.
    let ring_first = start_with_relics(&["Ring of the Snake", "Mark of Pain"], 5);
    assert_eq!(ring_first.state.hand.len(), 5);
    assert_eq!(ring_first.state.draw_pile.len(), 2);
    assert!(ring_first
        .state
        .draw_pile
        .iter()
        .all(|card| ring_first.card_registry.card_name(card.def_id) == "Wound"));

    let mark_first = start_with_relics(&["Mark of Pain", "Ring of the Snake"], 5);
    assert_eq!(mark_first.state.hand.len(), 7);
    assert!(mark_first.state.draw_pile.is_empty());
    assert_eq!(
        mark_first
            .state
            .hand
            .iter()
            .filter(|card| mark_first.card_registry.card_name(card.def_id) == "Wound")
            .count(),
        2
    );
    assert_eq!(
        ring_first.rng_counters()["cardRandom"],
        mark_first.rng_counters()["cardRandom"],
        "only action order changes; Mark's insertion draw count stays exact"
    );
}

#[test]
fn toolbox_choice_suspends_later_pre_draw_relic_actions_and_resumes_queue() {
    // atBattleStartPreDraw callbacks enqueue FIFO actions in relic inventory
    // order. Toolbox's ChooseOneColorless pauses GameActionManager, so a later
    // Ninja Scroll has not created its Shivs at the choice checkpoint; the
    // reverse inventory creates them before Toolbox opens. Resolving Toolbox
    // resumes the remaining queued action rather than replaying either relic.
    // Java: AbstractPlayer.java::applyStartOfCombatPreDrawLogic;
    // Toolbox.java; ChooseOneColorless.java; NinjaScroll.java.
    let mut toolbox_first = start_with_relics(&["Toolbox", "Ninja Scroll"], 5);
    assert_eq!(
        toolbox_first.phase,
        crate::engine::CombatPhase::AwaitingChoice
    );
    assert_eq!(
        toolbox_first
            .state
            .hand
            .iter()
            .filter(|card| toolbox_first.card_registry.card_name(card.def_id) == "Shiv")
            .count(),
        0
    );
    toolbox_first.execute_action(&crate::actions::Action::Choose(0));
    assert_eq!(
        toolbox_first
            .state
            .hand
            .iter()
            .filter(|card| toolbox_first.card_registry.card_name(card.def_id) == "Shiv")
            .count(),
        3
    );

    let ninja_first = start_with_relics(&["Ninja Scroll", "Toolbox"], 5);
    assert_eq!(
        ninja_first.phase,
        crate::engine::CombatPhase::AwaitingChoice
    );
    assert_eq!(
        ninja_first
            .state
            .hand
            .iter()
            .filter(|card| ninja_first.card_registry.card_name(card.def_id) == "Shiv")
            .count(),
        3
    );
}
