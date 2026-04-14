#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Brilliance.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/FlurryOfBlows.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Halt.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Perseverance.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/SandsOfTime.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/SignatureMove.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Weave.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/WindmillStrike.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Foresight.java

use crate::actions::Action;
use crate::cards::global_registry;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;
use crate::tests::support::*;

fn one_enemy_engine(enemy_id: &str, hp: i32, dmg: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy(enemy_id, hp, hp, 1, dmg, 1)], 3);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn watcher_wave11_registry_exports_safe_typed_surface_moves() {
    let registry = global_registry();

    let flurry = registry.get("FlurryOfBlows").expect("FlurryOfBlows should be registered");
    assert_eq!(flurry.effect_data, &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]);

    let signature = registry.get("SignatureMove").expect("SignatureMove should be registered");
    assert_eq!(signature.effect_data, &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]);
    assert!(signature.effects.contains(&"only_attack_in_hand"));

    let weave = registry.get("Weave").expect("Weave should be registered");
    assert_eq!(weave.effect_data, &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]);
    assert!(weave.effects.contains(&"return_on_scry"));

    let wireheading = registry.get("Wireheading").expect("Wireheading should be registered");
    assert_eq!(
        wireheading.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::FORESIGHT, A::Magic))]
    );
    assert!(wireheading.effects.is_empty());
}

#[test]
fn watcher_wave11_flurry_weave_signature_and_wireheading_follow_engine_path() {
    let mut flurry = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut flurry, "FlurryOfBlows+");
    assert!(play_on_enemy(&mut flurry, "FlurryOfBlows+", 0));
    assert_eq!(flurry.state.enemies[0].entity.hp, 44);

    let mut weave = one_enemy_engine("JawWorm", 50, 0);
    ensure_in_hand(&mut weave, "Weave+");
    assert!(play_on_enemy(&mut weave, "Weave+", 0));
    assert_eq!(weave.state.enemies[0].entity.hp, 44);

    let mut signature = one_enemy_engine("JawWorm", 60, 0);
    signature.state.hand = make_deck(&["SignatureMove+"]);
    assert!(play_on_enemy(&mut signature, "SignatureMove+", 0));
    assert_eq!(signature.state.enemies[0].entity.hp, 20);

    let mut wireheading = one_enemy_engine("JawWorm", 60, 0);
    ensure_in_hand(&mut wireheading, "Wireheading+");
    assert!(play_self(&mut wireheading, "Wireheading+"));
    assert_eq!(wireheading.state.player.status(sid::FORESIGHT), 4);
}

#[test]
fn watcher_wave11_signature_move_legality_stays_hook_backed_and_java_correct() {
    let mut blocked = one_enemy_engine("JawWorm", 60, 0);
    blocked.state.hand = make_deck(&["SignatureMove", "Strike_P"]);
    let signature_idx = blocked
        .state
        .hand
        .iter()
        .position(|card| blocked.card_registry.card_name(card.def_id) == "SignatureMove")
        .expect("SignatureMove should be in hand");
    assert!(!blocked.get_legal_actions().iter().any(|action| matches!(
        action,
        Action::PlayCard { card_idx, .. } if *card_idx == signature_idx
    )));
}

#[test]
#[ignore = "Brilliance still needs a typed mantra-gained-this-combat amount source; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Brilliance.java"]
fn watcher_wave11_brilliance_remains_blocked_on_mantra_amount_source() {}

#[test]
#[ignore = "Halt still needs a typed block pipeline that can derive the Wrath bonus before resolution exactly like Java HaltAction; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Halt.java"]
fn watcher_wave11_halt_remains_blocked_on_wrath_scaled_block_amount() {}

#[test]
#[ignore = "Perseverance still needs retained card-owned block growth rather than a shared metadata approximation; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Perseverance.java"]
fn watcher_wave11_perseverance_remains_blocked_on_retained_card_state() {}

#[test]
#[ignore = "Sands of Time still needs retained card-owned cost reduction semantics; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/SandsOfTime.java"]
fn watcher_wave11_sands_of_time_remains_blocked_on_retained_cost_state() {}

#[test]
#[ignore = "Windmill Strike still needs retained card-owned damage growth semantics; Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/purple/WindmillStrike.java"]
fn watcher_wave11_windmill_strike_remains_blocked_on_retained_damage_state() {}
