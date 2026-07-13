#![cfg(test)]

// Java oracle:
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/red/Rampage.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/green/GlassKnife.java
// - /Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/common/ModifyDamageAction.java

use crate::cards::{global_registry, CardType};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE};
use crate::tests::support::{enemy_no_intent, engine_without_start, force_player_turn, make_deck, play_on_enemy};

fn single_enemy_engine() -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.energy = 3;
    engine
}

#[test]
fn scaling_wave1_registry_exports_typed_played_instance_damage_mutation_surface() {
    let reg = global_registry();

    let rampage = reg.get("Rampage").expect("Rampage");
    assert_eq!(
        rampage.effect_data,
        &[
            E::Simple(SE::DealDamage(crate::effects::declarative::Target::SelectedEnemy, A::Damage)),
            E::Simple(SE::ModifyPlayedCardDamage(A::Magic)),
        ]
    );
    assert!(rampage.complex_hook.is_none());
    assert_eq!(rampage.card_type, CardType::Attack);

    let rampage_plus = reg.get("Rampage+").expect("Rampage+");
    assert_eq!(
        rampage_plus.effect_data,
        &[
            E::Simple(SE::DealDamage(crate::effects::declarative::Target::SelectedEnemy, A::Damage)),
            E::Simple(SE::ModifyPlayedCardDamage(A::Magic)),
        ]
    );
    assert!(rampage_plus.complex_hook.is_none());

    let glass_knife = reg.get("Glass Knife").expect("Glass Knife");
    assert_eq!(
        glass_knife.effect_data,
        &[
            E::Simple(SE::DealDamage(crate::effects::declarative::Target::SelectedEnemy, A::Damage)),
            E::Simple(SE::ModifyPlayedCardDamage(A::Fixed(-2))),
        ]
    );
    assert!(glass_knife.complex_hook.is_none());

    let glass_knife_plus = reg.get("Glass Knife+").expect("Glass Knife+");
    assert_eq!(glass_knife_plus.base_damage, 12);
    assert_eq!(
        glass_knife_plus.effect_data,
        &[
            E::Simple(SE::DealDamage(crate::effects::declarative::Target::SelectedEnemy, A::Damage)),
            E::Simple(SE::ModifyPlayedCardDamage(A::Fixed(-2))),
        ]
    );
    assert!(glass_knife_plus.complex_hook.is_none());
}

#[test]
fn rampage_and_glass_knife_update_the_played_instance_damage_seed_for_future_plays() {
    let mut rampage = single_enemy_engine();
    rampage.state.hand = make_deck(&["Rampage"]);

    assert!(play_on_enemy(&mut rampage, "Rampage", 0));
    assert_eq!(rampage.state.enemies[0].entity.hp, 32);

    let played = rampage
        .state
        .discard_pile
        .pop()
        .expect("played Rampage should land in discard");
    assert_eq!(played.misc, 13);

    rampage.state.hand.clear();
    rampage.state.hand.push(played);
    rampage.state.energy = 1;

    assert!(play_on_enemy(&mut rampage, "Rampage", 0));
    assert_eq!(rampage.state.enemies[0].entity.hp, 19);

    let mut glass_knife = single_enemy_engine();
    glass_knife.state.hand = make_deck(&["Glass Knife"]);

    assert!(play_on_enemy(&mut glass_knife, "Glass Knife", 0));
    assert_eq!(glass_knife.state.enemies[0].entity.hp, 24);

    let played = glass_knife
        .state
        .discard_pile
        .pop()
        .expect("played Glass Knife should land in discard");
    assert_eq!(played.misc, 6);

    glass_knife.state.hand.clear();
    glass_knife.state.hand.push(played);
    glass_knife.state.energy = 1;

    assert!(play_on_enemy(&mut glass_knife, "Glass Knife", 0));
    assert_eq!(glass_knife.state.enemies[0].entity.hp, 12);

    // GlassKnife.upgrade() calls upgradeDamage(4), so the upgraded card deals
    // 12 twice, stores 10, then deals 10 twice on its next play.
    // Java: reference/extracted/methods/card/GlassKnife.java
    let mut upgraded = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 100, 100)],
        3,
    );
    force_player_turn(&mut upgraded);
    upgraded.state.hand = make_deck(&["Glass Knife+"]);

    assert!(play_on_enemy(&mut upgraded, "Glass Knife+", 0));
    assert_eq!(upgraded.state.enemies[0].entity.hp, 76);
    let played = upgraded
        .state
        .discard_pile
        .pop()
        .expect("played Glass Knife+ should land in discard");
    assert_eq!(played.misc, 10);

    upgraded.state.hand.push(played);
    upgraded.state.energy = 1;
    assert!(play_on_enemy(&mut upgraded, "Glass Knife+", 0));
    assert_eq!(upgraded.state.enemies[0].entity.hp, 56);
    assert_eq!(upgraded.state.discard_pile.last().unwrap().misc, 8);
}

#[test]
fn rampage_does_not_grow_when_its_damage_kills_the_final_monster() {
    // Rampage.java queues DamageAction before ModifyDamageAction. DamageAction
    // clears later CARD_MANIPULATION actions only when all monsters are dead,
    // so a nonterminal target kill still grows the played UUID by five.
    let mut nonterminal = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent("JawWorm", 8, 8),
            enemy_no_intent("Cultist", 40, 40),
        ],
        3,
    );
    force_player_turn(&mut nonterminal);
    nonterminal.state.hand = make_deck(&["Rampage"]);
    assert!(play_on_enemy(&mut nonterminal, "Rampage", 0));
    assert_eq!(
        nonterminal.state.discard_pile.last().expect("played Rampage").misc,
        13
    );

    let mut terminal = engine_without_start(
        Vec::new(),
        vec![enemy_no_intent("JawWorm", 8, 8)],
        3,
    );
    force_player_turn(&mut terminal);
    terminal.state.hand = make_deck(&["Rampage"]);
    assert!(play_on_enemy(&mut terminal, "Rampage", 0));
    assert!(terminal.state.combat_over);
    assert_eq!(
        terminal.state.discard_pile.last().expect("played Rampage").misc,
        -1
    );
}
