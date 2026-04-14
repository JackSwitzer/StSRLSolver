#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/AllOutAttack.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Bane.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/EscapePlan.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Expertise.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/Flechettes.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/GlassKnife.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::status_ids::sid;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::tests::support::*;

#[test]
fn silent_wave10_registry_exports_show_typed_primary_surfaces() {
    let registry = global_registry();

    let all_out_attack = registry.get("All-Out Attack").expect("All-Out Attack should exist");
    assert_eq!(all_out_attack.card_type, CardType::Attack);
    assert_eq!(all_out_attack.target, CardTarget::AllEnemy);
    assert_eq!(
        all_out_attack.effect_data,
        &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))]
    );

    let bane = registry.get("Bane").expect("Bane should exist");
    assert_eq!(bane.card_type, CardType::Attack);
    assert_eq!(bane.target, CardTarget::Enemy);
    assert_eq!(
        bane.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );

    let escape_plan = registry.get("Escape Plan").expect("Escape Plan should exist");
    assert_eq!(escape_plan.card_type, CardType::Skill);
    assert_eq!(escape_plan.target, CardTarget::SelfTarget);
    assert_eq!(
        escape_plan.effect_data,
        &[E::Simple(SE::DrawCards(A::Fixed(1)))]
    );

    let flechettes = registry.get("Flechettes").expect("Flechettes should exist");
    assert_eq!(flechettes.card_type, CardType::Attack);
    assert_eq!(flechettes.target, CardTarget::Enemy);
    assert_eq!(
        flechettes.effect_data,
        &[
            E::ExtraHits(A::SkillsInHand),
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ]
    );

    let glass_knife = registry.get("Glass Knife").expect("Glass Knife should exist");
    assert_eq!(glass_knife.card_type, CardType::Attack);
    assert_eq!(glass_knife.target, CardTarget::Enemy);
    assert_eq!(
        glass_knife.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))]
    );

    let expertise = registry.get("Expertise").expect("Expertise should exist");
    assert_eq!(
        expertise.effect_data,
        &[E::Simple(SE::DrawToHandSize(A::Magic))]
    );
    assert!(expertise.complex_hook.is_none());
    assert!(expertise.effects.contains(&"draw_to_n"));
}

#[test]
fn silent_wave10_typed_primary_surfaces_follow_java_oracle_on_engine_path() {
    let mut aoa = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40), enemy_no_intent("Cultist", 40, 40)],
        3,
    );
    force_player_turn(&mut aoa);
    aoa.state.hand = make_deck(&["All-Out Attack", "Strike_G", "Defend_G"]);
    let hp0 = aoa.state.enemies[0].entity.hp;
    let hp1 = aoa.state.enemies[1].entity.hp;
    assert!(play_on_enemy(&mut aoa, "All-Out Attack", 0));
    assert_eq!(aoa.state.enemies[0].entity.hp, hp0 - 10);
    assert_eq!(aoa.state.enemies[1].entity.hp, hp1 - 10);
    assert_eq!(discard_prefix_count(&aoa, "All-Out Attack"), 1);
    assert_eq!(aoa.state.discard_pile.len(), 2);
    assert_eq!(aoa.state.hand.len(), 1);

    let mut bane = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut bane);
    bane.state.hand = make_deck(&["Bane"]);
    bane.state.enemies[0].entity.set_status(sid::POISON, 2);
    let hp_before = bane.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut bane, "Bane", 0));
    assert_eq!(bane.state.enemies[0].entity.hp, hp_before - 14);

    let mut escape_plan = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut escape_plan);
    escape_plan.state.hand = make_deck(&["Escape Plan"]);
    escape_plan.state.draw_pile.clear();
    escape_plan.state.draw_pile.push(escape_plan.card_registry.make_card("Defend_G"));
    assert!(play_self(&mut escape_plan, "Escape Plan"));
    assert_eq!(escape_plan.state.player.block, 3);
    assert_eq!(hand_count(&escape_plan, "Defend_G"), 1);
    assert_eq!(discard_prefix_count(&escape_plan, "Escape Plan"), 1);

    let mut flechettes = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 40, 40)],
        3,
    );
    force_player_turn(&mut flechettes);
    flechettes.state.hand = make_deck(&["Flechettes", "Defend_G", "Escape Plan", "Strike_G"]);
    let flechettes_hp = flechettes.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut flechettes, "Flechettes", 0));
    assert_eq!(flechettes.state.enemies[0].entity.hp, flechettes_hp - 8);

    let mut glass_knife = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 60, 60)],
        3,
    );
    force_player_turn(&mut glass_knife);
    glass_knife.state.hand = make_deck(&["Glass Knife"]);
    let glass_hp = glass_knife.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut glass_knife, "Glass Knife", 0));
    assert_eq!(glass_knife.state.enemies[0].entity.hp, glass_hp - 16);
    assert_eq!(discard_prefix_count(&glass_knife, "Glass Knife"), 1);
    end_turn(&mut glass_knife);
    assert_eq!(hand_count(&glass_knife, "Glass Knife"), 1);
    let glass_hp2 = glass_knife.state.enemies[0].entity.hp;
    assert!(play_on_enemy(&mut glass_knife, "Glass Knife", 0));
    assert_eq!(glass_knife.state.enemies[0].entity.hp, glass_hp2 - 12);
}

#[test]
fn silent_wave10_expertise_draws_to_n_on_engine_path() {
    let mut engine = engine_without_start(
        make_deck(&["Strike_G", "Strike_G", "Strike_G", "Strike_G", "Strike_G", "Strike_G"]),
        vec![enemy_no_intent("JawWorm", 50, 50)],
        3,
    );
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Expertise"]);
    engine.state.draw_pile = make_deck(&["Strike_G", "Strike_G", "Strike_G", "Strike_G", "Strike_G", "Strike_G"]);

    assert!(play_self(&mut engine, "Expertise"));
    assert_eq!(engine.state.hand.len(), 6);
    assert_eq!(discard_prefix_count(&engine, "Expertise"), 1);
}
