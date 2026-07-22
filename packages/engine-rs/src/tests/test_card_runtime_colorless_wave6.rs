#![cfg(test)]

// Java oracle sources:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/MindBlast.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Forethought.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/ForethoughtAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Enlightenment.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EnlightenmentAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Impatience.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Madness.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/RitualDagger.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/unique/RitualDaggerAction.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Violence.java

use crate::cards::{global_registry, CardTarget, CardType};
use crate::effects::declarative::{
    AmountSource as A, Effect as E, SimpleEffect as SE, Target as T,
};
use crate::engine::CombatPhase;
use crate::state::Stance;
use crate::status_ids::sid;
use crate::tests::support::{
    enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_on_enemy,
};

#[test]
fn colorless_wave6_registry_exports_match_typed_surface_for_mind_blast() {
    let registry = global_registry();

    let mind_blast = registry.get("Mind Blast").expect("Mind Blast should exist");
    let mind_blast_plus = registry
        .get("Mind Blast+")
        .expect("Mind Blast+ should exist");

    // MindBlast.java constructor: cost 2, Attack/Enemy, Innate, baseDamage 0.
    assert_eq!(mind_blast.card_type, CardType::Attack);
    assert_eq!(mind_blast.target, CardTarget::Enemy);
    assert_eq!(mind_blast.cost, 2);
    assert_eq!(mind_blast.base_damage, 0);
    assert!(mind_blast.runtime_traits().innate);
    assert!(!mind_blast.exhaust);
    // upgrade() calls only upgradeName() and upgradeBaseCost(1).
    assert_eq!(mind_blast_plus.cost, 1);
    assert_eq!(mind_blast_plus.base_damage, 0);
    assert!(mind_blast_plus.runtime_traits().innate);
    assert_eq!(
        mind_blast.effect_data,
        &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::DrawPileSize))]
    );
    assert!(mind_blast.complex_hook.is_none());
}

#[test]
fn mind_blast_uses_live_draw_pile_size_as_normal_attack_base_damage() {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 80, 80)], 3);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Mind Blast"]);
    engine.state.draw_pile = make_deck(&[
        "Strike", "Defend", "Strike", "Defend", "Strike", "Defend", "Strike",
    ]);
    engine.state.player.set_status(sid::STRENGTH, 2);
    engine.state.stance = Stance::Wrath;
    engine.state.enemies[0]
        .entity
        .set_status(sid::VULNERABLE, 1);
    engine.state.enemies[0].entity.block = 5;

    assert!(play_on_enemy(&mut engine, "Mind Blast", 0));
    assert_eq!(engine.phase, CombatPhase::PlayerTurn);
    // applyPowers sets baseDamage to 7. NORMAL damage then applies Strength
    // (7 + 2), Wrath (x2), Vulnerable (x1.5), and block: 27 - 5 = 22 HP.
    assert_eq!(engine.state.enemies[0].entity.hp, 58);
    assert_eq!(engine.state.enemies[0].entity.block, 0);
}
