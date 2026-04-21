use super::*;
use crate::actions::Action;
use crate::status_ids::sid;
use crate::tests::support::{engine_with, make_deck};

fn legal_play_names(engine: &CombatEngine) -> Vec<&'static str> {
    engine
        .get_legal_actions()
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard { card_idx, .. } => Some(engine.card_registry.card_name(engine.state.hand[card_idx].def_id)),
            _ => None,
        })
        .collect()
}

#[test]
fn canonical_legality_helper_blocks_and_allows_card_specific_plays() {
    let mut clash_blocked = engine_with(make_deck(&["Clash", "Defend"]), 50, 0);
    clash_blocked.state.hand = make_deck(&["Clash", "Defend"]);
    clash_blocked.state.draw_pile.clear();
    clash_blocked.state.discard_pile.clear();
    assert!(!legal_play_names(&clash_blocked).contains(&"Clash"));

    let mut signature_blocked = engine_with(make_deck(&["SignatureMove", "Strike"]), 50, 0);
    signature_blocked.state.hand = make_deck(&["SignatureMove", "Strike"]);
    signature_blocked.state.draw_pile.clear();
    signature_blocked.state.discard_pile.clear();
    assert!(!legal_play_names(&signature_blocked).contains(&"Signature Move"));

    let mut finale_blocked = engine_with(make_deck(&["Grand Finale", "Strike"]), 50, 0);
    finale_blocked.state.hand = make_deck(&["Grand Finale"]);
    finale_blocked.state.draw_pile = make_deck(&["Strike"]);
    finale_blocked.state.discard_pile.clear();
    assert!(!legal_play_names(&finale_blocked).contains(&"Grand Finale"));

    let mut finale_allowed = engine_with(make_deck(&["Grand Finale"]), 50, 0);
    finale_allowed.state.hand = make_deck(&["Grand Finale"]);
    finale_allowed.state.draw_pile.clear();
    finale_allowed.state.discard_pile.clear();
    assert!(legal_play_names(&finale_allowed).contains(&"Grand Finale"));
}

#[test]
fn canonical_cost_helper_matches_runtime_scaling_rules() {
    let registry = crate::cards::global_registry();
    let mut engine = engine_with(make_deck(&["Strike"]), 50, 0);

    engine.state.player.set_status(sid::HP_LOSS_THIS_COMBAT, 2);
    let blood_for_blood = registry.get("Blood for Blood").expect("Blood for Blood");
    let blood_for_blood_inst = registry.make_card("Blood for Blood");
    assert_eq!(engine.effective_cost_inst(blood_for_blood, blood_for_blood_inst), 2);

    engine.state.player.set_status(sid::DEMON_FORM, 1);
    engine.state.player.set_status(sid::NOXIOUS_FUMES, 1);
    let force_field = registry.get("Force Field").expect("Force Field");
    let force_field_inst = registry.make_card("Force Field");
    assert_eq!(engine.effective_cost_inst(force_field, force_field_inst), 2);

    engine.state.player.set_status(sid::DISCARDED_THIS_TURN, 2);
    let eviscerate = registry.get("Eviscerate").expect("Eviscerate");
    let eviscerate_inst = registry.make_card("Eviscerate");
    assert_eq!(engine.effective_cost_inst(eviscerate, eviscerate_inst), 1);

    engine.state.total_damage_taken = 3;
    let masterful_stab = registry.get("Masterful Stab").expect("Masterful Stab");
    let masterful_stab_inst = registry.make_card("Masterful Stab");
    assert_eq!(engine.effective_cost_inst(masterful_stab, masterful_stab_inst), 3);
}

#[test]
fn canonical_cost_helper_controls_legal_actions_for_unaffordable_cards() {
    let mut blocked = engine_with(make_deck(&["Masterful Stab"]), 50, 0);
    blocked.state.hand = make_deck(&["Masterful Stab"]);
    blocked.state.draw_pile.clear();
    blocked.state.discard_pile.clear();
    blocked.state.energy = 2;
    blocked.state.total_damage_taken = 3;
    assert!(!legal_play_names(&blocked).contains(&"Masterful Stab"));

    let mut allowed = engine_with(make_deck(&["Masterful Stab"]), 50, 0);
    allowed.state.hand = make_deck(&["Masterful Stab"]);
    allowed.state.draw_pile.clear();
    allowed.state.discard_pile.clear();
    allowed.state.energy = 3;
    allowed.state.total_damage_taken = 3;
    assert!(legal_play_names(&allowed).contains(&"Masterful Stab"));
}
