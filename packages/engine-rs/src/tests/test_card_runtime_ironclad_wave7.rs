#![cfg(test)]

// Java oracle references for this wave:
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/BattleTrance.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Bloodletting.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Combust.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Entrench.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Inflame.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/Offering.java
// - decompiled/java-src/com/megacrit/cardcrawl/cards/red/ShrugItOff.java

use crate::cards::{CardTarget, CardType, global_registry};
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;
use crate::tests::support::*;

fn one_enemy_engine(enemy_id: &str, hp: i32) -> crate::engine::CombatEngine {
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent(enemy_id, hp, hp)], 3);
    force_player_turn(&mut engine);
    engine
}

#[test]
fn ironclad_wave7_registry_exports_match_typed_runtime_progress() {
    let battle_trance = global_registry()
        .get("Battle Trance+")
        .expect("Battle Trance+ should exist");
    assert_eq!(battle_trance.card_type, CardType::Skill);
    assert_eq!(battle_trance.target, CardTarget::None);
    assert_eq!(
        battle_trance.effect_data,
        &[
            E::Simple(SE::DrawCards(A::Magic)),
            E::Simple(SE::AddStatus(T::Player, sid::NO_DRAW, A::Fixed(1))),
        ]
    );

    let bloodletting = global_registry()
        .get("Bloodletting+")
        .expect("Bloodletting+ should exist");
    assert_eq!(
        bloodletting.effect_data,
        &[
            E::Simple(SE::ModifyHp(A::Fixed(-3))),
            E::Simple(SE::GainEnergy(A::Magic)),
        ]
    );

    let combust = global_registry().get("Combust").expect("Combust should exist");
    assert_eq!(combust.card_type, CardType::Power);
    assert_eq!(
        combust.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::COMBUST, A::Magic))]
    );

    let entrench = global_registry().get("Entrench").expect("Entrench should exist");
    assert_eq!(
        entrench.effect_data,
        &[E::Simple(SE::GainBlock(A::PlayerBlock))]
    );
    assert!(entrench.complex_hook.is_none());

    let inflame = global_registry().get("Inflame+").expect("Inflame+ should exist");
    assert_eq!(
        inflame.effect_data,
        &[E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic))]
    );

    let offering = global_registry().get("Offering").expect("Offering should exist");
    assert!(offering.exhaust);
    assert_eq!(
        offering.effect_data,
        &[
            E::Simple(SE::ModifyHp(A::Fixed(-6))),
            E::Simple(SE::GainEnergy(A::Fixed(2))),
            E::Simple(SE::DrawCards(A::Magic)),
        ]
    );

    let shrug_it_off = global_registry()
        .get("Shrug It Off")
        .expect("Shrug It Off should exist");
    assert_eq!(shrug_it_off.base_block, 8);
    assert_eq!(
        shrug_it_off.effect_data,
        &[E::Simple(SE::DrawCards(A::Fixed(1)))]
    );
}

#[test]
fn ironclad_wave7_battle_trance_bloodletting_and_offering_run_through_typed_effects() {
    let mut battle_trance = one_enemy_engine("JawWorm", 60);
    battle_trance.state.hand = make_deck(&["Battle Trance"]);
    battle_trance.state.draw_pile = make_deck(&["Strike", "Defend", "Bash"]);
    assert!(play_self(&mut battle_trance, "Battle Trance"));
    assert_eq!(battle_trance.state.hand.len(), 3);
    assert_eq!(battle_trance.state.player.status(sid::NO_DRAW), 1);
    let hand_before = battle_trance.state.hand.len();
    battle_trance.draw_cards(1);
    assert_eq!(battle_trance.state.hand.len(), hand_before);

    let mut bloodletting = one_enemy_engine("JawWorm", 60);
    bloodletting.state.hand = make_deck(&["Bloodletting+"]);
    let hp_before = bloodletting.state.player.hp;
    let energy_before = bloodletting.state.energy;
    assert!(play_self(&mut bloodletting, "Bloodletting+"));
    assert_eq!(bloodletting.state.player.hp, hp_before - 3);
    assert_eq!(bloodletting.state.energy, energy_before + 3);

    let mut offering = one_enemy_engine("JawWorm", 60);
    offering.state.hand = make_deck(&["Offering+"]);
    offering.state.draw_pile = make_deck(&["Strike", "Defend", "Bash", "Shrug It Off", "Inflame"]);
    let hp_before = offering.state.player.hp;
    assert!(play_self(&mut offering, "Offering+"));
    assert_eq!(offering.state.player.hp, hp_before - 6);
    assert_eq!(offering.state.energy, 5);
    assert_eq!(offering.state.hand.len(), 5);
    assert_eq!(exhaust_prefix_count(&offering, "Offering"), 1);
}

#[test]
fn offering_loses_fixed_six_through_hp_loss_then_gains_energy_and_draws() {
    // Offering.java queues LoseHPAction(6), GainEnergyAction(2), and then draw
    // magic 3 (5 upgraded). LoseHPAction is HP_LOSS damage, so it bypasses
    // block, triggers Rupture after positive loss, and can be stopped by Buffer.
    let mut base = one_enemy_engine("JawWorm", 60);
    base.state.energy = 0;
    base.state.player.block = 9;
    base.state.player.set_status(sid::RUPTURE, 2);
    base.state.hand = make_deck(&["Offering"]);
    base.state.draw_pile = make_deck(&["Strike", "Defend", "Bash", "Inflame"]);
    let hp_before = base.state.player.hp;

    assert!(play_self(&mut base, "Offering"));

    assert_eq!(base.state.player.hp, hp_before - 6);
    assert_eq!(base.state.player.block, 9);
    assert_eq!(base.state.player.status(sid::STRENGTH), 2);
    assert_eq!(base.state.energy, 2);
    assert_eq!(base.state.hand.len(), 3);
    assert_eq!(exhaust_prefix_count(&base, "Offering"), 1);

    let mut upgraded = one_enemy_engine("JawWorm", 60);
    upgraded.state.energy = 0;
    upgraded.state.player.block = 9;
    upgraded.state.player.set_status(sid::BUFFER, 1);
    upgraded.state.player.set_status(sid::RUPTURE, 2);
    upgraded.state.hand = make_deck(&["Offering+"]);
    upgraded.state.draw_pile = make_deck(&[
        "Strike", "Defend", "Bash", "Shrug It Off", "Inflame", "Strike",
    ]);
    let hp_before = upgraded.state.player.hp;

    assert!(play_self(&mut upgraded, "Offering+"));

    assert_eq!(upgraded.state.player.hp, hp_before);
    assert_eq!(upgraded.state.player.block, 9);
    assert_eq!(upgraded.state.player.status(sid::BUFFER), 0);
    assert_eq!(upgraded.state.player.status(sid::STRENGTH), 0);
    assert_eq!(upgraded.state.energy, 2);
    assert_eq!(upgraded.state.hand.len(), 5);
    assert_eq!(exhaust_prefix_count(&upgraded, "Offering"), 1);
}

#[test]
fn bloodletting_hp_loss_obeys_intangible_buffer_and_tungsten_before_gaining_energy() {
    // Source: Bloodletting.java queues LoseHPAction(3) before GainEnergyAction;
    // HP_LOSS still passes IntangiblePlayerPower, BufferPower, and TungstenRod
    // in AbstractPlayer.damage.
    let mut ordinary = one_enemy_engine("JawWorm", 60);
    ordinary.state.hand = make_deck(&["Bloodletting+"]);
    let hp_before = ordinary.state.player.hp;
    assert!(play_self(&mut ordinary, "Bloodletting+"));
    assert_eq!(ordinary.state.player.hp, hp_before - 3);
    assert_eq!(ordinary.state.energy, 6);
    assert_eq!(ordinary.state.player.status(sid::HP_LOSS_THIS_COMBAT), 1);

    let mut intangible = one_enemy_engine("JawWorm", 60);
    intangible.state.player.set_status(sid::INTANGIBLE, 1);
    intangible.state.hand = make_deck(&["Bloodletting"]);
    let hp_before = intangible.state.player.hp;
    assert!(play_self(&mut intangible, "Bloodletting"));
    assert_eq!(intangible.state.player.hp, hp_before - 1);
    assert_eq!(intangible.state.energy, 5);
    assert_eq!(intangible.state.player.status(sid::HP_LOSS_THIS_COMBAT), 1);

    let mut buffered = one_enemy_engine("JawWorm", 60);
    buffered.state.player.set_status(sid::BUFFER, 1);
    buffered.state.hand = make_deck(&["Bloodletting"]);
    let hp_before = buffered.state.player.hp;
    assert!(play_self(&mut buffered, "Bloodletting"));
    assert_eq!(buffered.state.player.hp, hp_before);
    assert_eq!(buffered.state.player.status(sid::BUFFER), 0);
    assert_eq!(buffered.state.energy, 5);
    assert_eq!(buffered.state.player.status(sid::HP_LOSS_THIS_COMBAT), 0);

    let mut prevented = one_enemy_engine("JawWorm", 60);
    prevented.state.player.set_status(sid::INTANGIBLE, 1);
    prevented.state.relics.push("Tungsten Rod".to_string());
    prevented.state.hand = make_deck(&["Bloodletting"]);
    let hp_before = prevented.state.player.hp;
    assert!(play_self(&mut prevented, "Bloodletting"));
    assert_eq!(prevented.state.player.hp, hp_before);
    assert_eq!(prevented.state.energy, 5);
    assert_eq!(prevented.state.player.status(sid::HP_LOSS_THIS_COMBAT), 0);
}

#[test]
fn battle_trance_plus_draws_four_then_no_draw_expires_at_turn_end() {
    // Sources: BattleTrance.java queues DrawCardAction(4) before applying
    // NoDrawPower; NoDrawPower.java removes itself at the player's turn end.
    let mut engine = one_enemy_engine("JawWorm", 60);
    engine.state.hand = make_deck(&["Battle Trance+"]);
    engine.state.draw_pile = make_deck(&[
        "Strike", "Defend", "Bash", "Shrug It Off", "Inflame", "Strike", "Defend",
        "Bash", "Strike",
    ]);

    assert!(play_self(&mut engine, "Battle Trance+"));
    assert_eq!(engine.state.hand.len(), 4);
    assert_eq!(engine.state.energy, 3);
    assert_eq!(engine.state.player.status(sid::NO_DRAW), 1);

    let hand_before_blocked_draw = engine.state.hand.len();
    engine.draw_cards(1);
    assert_eq!(engine.state.hand.len(), hand_before_blocked_draw);

    end_turn(&mut engine);

    assert_eq!(engine.state.player.status(sid::NO_DRAW), 0);
    assert_eq!(engine.state.hand.len(), 5);

    // NoDrawPower.java is a DEBUFF, so Artifact consumes itself after the
    // Battle Trance draw and prevents the restriction from being installed.
    let mut artifact = one_enemy_engine("JawWorm", 60);
    artifact.state.player.set_status(sid::ARTIFACT, 1);
    artifact.state.hand = make_deck(&["Battle Trance"]);
    artifact.state.draw_pile = make_deck(&["Strike", "Defend", "Bash", "Inflame"]);
    assert!(play_self(&mut artifact, "Battle Trance"));
    assert_eq!(artifact.state.hand.len(), 3);
    assert_eq!(artifact.state.player.status(sid::ARTIFACT), 0);
    assert_eq!(artifact.state.player.status(sid::NO_DRAW), 0);
    artifact.draw_cards(1);
    assert_eq!(artifact.state.hand.len(), 4);
}

#[test]
fn ironclad_wave7_combust_inflame_and_shrug_it_off_follow_engine_path() {
    let mut combust = one_enemy_engine("JawWorm", 50);
    combust.state.hand = make_deck(&["Combust+"]);
    assert!(play_self(&mut combust, "Combust+"));
    assert_eq!(combust.state.player.status(sid::COMBUST), 7);
    end_turn(&mut combust);
    assert_eq!(combust.state.player.hp, 79);
    assert_eq!(combust.state.enemies[0].entity.hp, 43);

    // CombustPower.stackPower adds the incoming damage to amount but
    // increments its private hpLoss field by one. Its end-turn actions lose HP
    // first, then deal source-less THORNS damage to every enemy.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/CombustPower.java
    let mut stacked = engine_without_start(
        Vec::new(),
        vec![
            enemy_no_intent("JawWorm", 50, 50),
            enemy_no_intent("Cultist", 50, 50),
        ],
        3,
    );
    force_player_turn(&mut stacked);
    stacked.state.enemies[0].entity.set_status(sid::MALLEABLE, 1);
    stacked.state.hand = make_deck(&["Combust", "Combust+"]);
    assert!(play_self(&mut stacked, "Combust"));
    assert!(play_self(&mut stacked, "Combust+"));
    assert_eq!(stacked.state.player.status(sid::COMBUST), 12);

    end_turn(&mut stacked);

    assert_eq!(stacked.state.player.hp, 78);
    assert_eq!(stacked.state.enemies[0].entity.hp, 38);
    assert_eq!(stacked.state.enemies[1].entity.hp, 38);
    assert_eq!(stacked.state.enemies[0].entity.block, 0);
    assert_eq!(stacked.state.enemies[0].entity.status(sid::MALLEABLE), 1);

    // Inflame.java applies 2 Strength for one energy, upgradeMagicNumber(1)
    // raises that to 3, and both Power cards leave combat piles after play.
    let mut inflame = one_enemy_engine("JawWorm", 50);
    inflame.state.hand = make_deck(&["Inflame", "Inflame+"]);
    assert!(play_self(&mut inflame, "Inflame"));
    assert!(play_self(&mut inflame, "Inflame+"));
    assert_eq!(inflame.state.player.status(sid::STRENGTH), 5);
    assert_eq!(inflame.state.energy, 1);
    assert!(inflame.state.hand.is_empty());
    assert!(inflame.state.discard_pile.is_empty());

    let mut shrug_it_off = one_enemy_engine("JawWorm", 50);
    shrug_it_off.state.hand = make_deck(&["Shrug It Off+"]);
    shrug_it_off.state.draw_pile = make_deck(&["Strike"]);
    assert!(play_self(&mut shrug_it_off, "Shrug It Off+"));
    assert_eq!(shrug_it_off.state.player.block, 11);
    assert_eq!(hand_count(&shrug_it_off, "Strike"), 1);
}

#[test]
fn ironclad_wave7_entrench_stays_hook_backed_until_double_block_is_typed() {
    let mut entrench = one_enemy_engine("JawWorm", 50);
    entrench.state.hand = make_deck(&["Entrench+"]);
    entrench.state.player.block = 13;

    assert!(play_self(&mut entrench, "Entrench+"));

    assert_eq!(entrench.state.player.block, 26);
}
