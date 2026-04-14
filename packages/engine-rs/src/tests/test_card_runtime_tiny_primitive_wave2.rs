#![cfg(test)]

// Java oracle references for this wave:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/FTL.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/FTLAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/AllOutAttack.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/common/DiscardAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Bane.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/BaneAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Feed.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/FeedAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/EscapePlan.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{AmountSource as A, Condition as Cond, Effect as E, Pile as P, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;
use crate::tests::support::{enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_on_enemy, play_self};

fn single_enemy_engine(enemy_hp: i32, energy: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", enemy_hp, enemy_hp.max(1))],
        energy,
    );
    force_player_turn(&mut engine);
    engine.state.turn = 1;
    engine
}

#[test]
fn tiny_primitive_wave2_registry_exports_show_the_typed_primary_surfaces() {
    let reg = global_registry();

    let ftl = reg.get("FTL").expect("FTL");
    assert_eq!(ftl.card_type, CardType::Attack);
    assert_eq!(ftl.target, CardTarget::Enemy);
    assert_eq!(
        ftl.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Conditional(
                Cond::CardsPlayedThisTurnLessThan(3),
                &[E::Simple(SE::DrawCards(A::Magic))],
                &[],
            ),
        ]
    );
    assert!(ftl.complex_hook.is_none());

    let all_out_attack = reg.get("All-Out Attack").expect("All-Out Attack");
    assert_eq!(all_out_attack.card_type, CardType::Attack);
    assert_eq!(all_out_attack.target, CardTarget::AllEnemy);
    assert_eq!(
        all_out_attack.effect_data,
        &[
            E::Simple(SE::DealDamage(T::AllEnemies, A::Damage)),
            E::Simple(SE::DiscardRandomCardsFromPile(P::Hand, 1)),
        ]
    );
    assert!(all_out_attack.complex_hook.is_none());

    let bane = reg.get("Bane").expect("Bane");
    assert_eq!(bane.card_type, CardType::Attack);
    assert_eq!(bane.target, CardTarget::Enemy);
    assert_eq!(
        bane.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Conditional(
                Cond::EnemyAlive,
                &[E::Conditional(
                    Cond::EnemyHasStatus(sid::POISON),
                    &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
                    &[],
                )],
                &[],
            ),
        ]
    );
    assert!(bane.complex_hook.is_none());

    let feed = reg.get("Feed").expect("Feed");
    assert_eq!(feed.card_type, CardType::Attack);
    assert_eq!(feed.target, CardTarget::Enemy);
    assert_eq!(
        feed.effect_data,
        &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            E::Conditional(
                Cond::EnemyKilled,
                &[E::Simple(SE::ModifyMaxHp(A::Magic))],
                &[],
            ),
        ]
    );
    assert!(feed.complex_hook.is_none());

    let escape_plan = reg.get("Escape Plan").expect("Escape Plan");
    assert_eq!(
        escape_plan.effect_data,
        &[E::Simple(SE::DrawCards(A::Fixed(1)))]
    );
    assert!(escape_plan.complex_hook.is_some());

    let enlightenment = reg.get("Enlightenment").expect("Enlightenment");
    assert!(enlightenment.effect_data.is_empty());
    assert!(enlightenment.complex_hook.is_some());
}

#[test]
fn tiny_primitive_wave2_ftl_bane_feed_and_all_out_attack_follow_the_typed_runtime_surface() {
    let mut ftl_draws = single_enemy_engine(40, 3);
    ftl_draws.state.hand = make_deck(&["FTL+"]);
    ftl_draws.state.draw_pile = make_deck(&["Strike_B", "Defend_B", "Zap", "Dualcast"]);
    assert!(play_on_enemy(&mut ftl_draws, "FTL+", 0));
    assert_eq!(ftl_draws.state.enemies[0].entity.hp, 34);
    assert_eq!(ftl_draws.state.hand.len(), 4);

    let mut ftl_gated = single_enemy_engine(40, 3);
    ftl_gated.state.cards_played_this_turn = 3;
    ftl_gated.state.hand = make_deck(&["FTL"]);
    ftl_gated.state.draw_pile = make_deck(&["Strike_B", "Defend_B"]);
    assert!(play_on_enemy(&mut ftl_gated, "FTL", 0));
    assert_eq!(ftl_gated.state.enemies[0].entity.hp, 35);
    assert_eq!(ftl_gated.state.hand.len(), 0);

    let mut bane = single_enemy_engine(40, 3);
    bane.state.hand = make_deck(&["Bane"]);
    bane.state.enemies[0].entity.set_status(sid::POISON, 2);
    let bane_hp_before = bane.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut bane, "Bane", 0));
    assert_eq!(bane.state.enemies[0].entity.hp, bane_hp_before - 14);

    let mut feed = single_enemy_engine(10, 3);
    feed.state.player.hp = 40;
    feed.state.player.max_hp = 60;
    feed.state.hand = make_deck(&["Feed"]);
    let max_hp_before = feed.state.player.max_hp;
    let hp_before = feed.state.player.hp;
    assert!(play_on_enemy(&mut feed, "Feed", 0));
    assert_eq!(feed.state.player.max_hp, max_hp_before + 3);
    assert_eq!(feed.state.player.hp, hp_before);

    let mut all_out_attack = single_enemy_engine(40, 3);
    all_out_attack.state.hand = make_deck(&["All-Out Attack", "Strike_G"]);
    let hand_before = all_out_attack.state.hand.len();
    assert!(play_self(&mut all_out_attack, "All-Out Attack"));
    assert_eq!(all_out_attack.state.enemies[0].entity.hp, 30);
    assert_eq!(all_out_attack.state.hand.len(), hand_before - 2);
    assert_eq!(all_out_attack.state.discard_pile.len(), 2);
    assert_eq!(all_out_attack.state.player.status(sid::DISCARDED_THIS_TURN), 1);
}

#[test]
#[ignore = "Blocked on Java turn-only cost-reduction lifetime semantics for Enlightenment base; the current runtime still needs a typed costForTurn lifetime primitive. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java and /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java"]
fn tiny_primitive_wave2_enlightenment_stays_explicitly_blocked() {}

#[test]
#[ignore = "Blocked on Java last-drawn-card inspection semantics for Escape Plan; the current runtime still needs a typed last-drawn-card predicate primitive. Java oracle: /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/EscapePlan.java"]
fn tiny_primitive_wave2_escape_plan_stays_explicitly_blocked() {}
