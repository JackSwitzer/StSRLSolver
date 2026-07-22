#![cfg(test)]

use crate::actions::Action;
use crate::effects::runtime::EffectOwner;
use crate::engine::{ChoiceReason, CombatPhase};
use crate::orbs::OrbType;
use crate::status_ids::sid;
use crate::tests::support::{
    end_turn, enemy, enemy_no_intent, engine_without_start, force_player_turn, hand_count,
    make_deck,
};

#[test]
fn frozen_core_frost_passive_resolves_before_burn_autoplay() {
    // FrozenCore.onPlayerEndTurn synchronously channels Frost before
    // GameActionManager.callEndOfTurnActions queues TriggerEndOfTurnOrbsAction;
    // Burn is only then queued as an end-turn CardQueueItem. The new Frost's
    // two Block must therefore absorb Burn's two THORNS damage.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/FrozenCore.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/TriggerEndOfTurnOrbsAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/status/Burn.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    engine.init_defect_orbs(1);
    engine.state.relics.push("FrozenCore".to_string());
    engine.start_combat();
    engine.state.hand = make_deck(&["Burn"]);
    engine.state.draw_pile.clear();
    let hp_before = engine.state.player.hp;

    end_turn(&mut engine);

    assert_eq!(engine.state.player.hp, hp_before);
    assert_eq!(engine.state.orb_slots.occupied_count(), 1);
    assert_eq!(engine.state.orb_slots.slots[0].orb_type, OrbType::Frost);
}

#[test]
fn burn_leaves_hand_before_retain_card_power_opens_its_choice() {
    // Burn's end-turn CardQueueItem is played and removed from hand before
    // AbstractRoom.endTurn calls RetainCardPower.atEndOfTurn. Consequently,
    // RetainCardsAction can offer only the two ordinary cards still in hand.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/status/Burn.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/rooms/AbstractRoom.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/RetainCardPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/RetainCardsAction.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.player.set_status(sid::RETAIN_CARDS, 1);
    engine.state.hand = make_deck(&["Burn", "Strike", "Defend"]);
    let hp_before = engine.state.player.hp;

    end_turn(&mut engine);

    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(engine.state.player.hp, hp_before - 2);
    assert_eq!(hand_count(&engine, "Burn"), 0);
    let choice = engine.choice.as_ref().expect("Retain Cards choice");
    assert_eq!(choice.reason, ChoiceReason::RetainFromHand);
    assert_eq!(choice.options.len(), 2);
    assert_eq!(choice.max_picks, 1);
}

#[test]
fn lethal_burn_prevents_later_omega_end_turn_callback() {
    // Status CardQueueItems finish before AbstractRoom.endTurn invokes the
    // player's power atEndOfTurn callbacks. A lethal Burn therefore ends the
    // combat before OmegaPower can queue its source-less THORNS damage.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/status/Burn.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/rooms/AbstractRoom.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/OmegaPower.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 30, 30)], 3);
    force_player_turn(&mut engine);
    engine.state.player.hp = 2;
    engine.state.player.set_status(sid::OMEGA, 50);
    engine.state.hand = make_deck(&["Burn"]);

    end_turn(&mut engine);

    assert_eq!(engine.state.player.hp, 0);
    assert_eq!(engine.state.enemies[0].entity.hp, 30);
    assert!(engine.state.combat_over);
}

#[test]
fn lethal_burn_prevents_later_the_bomb_end_turn_callback() {
    // TheBombPower is likewise a player atEndOfTurn callback. Even an armed
    // amount-one bomb cannot run before the earlier lethal Burn CardQueueItem.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/status/Burn.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/rooms/AbstractRoom.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/TheBombPower.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 30, 30)], 3);
    force_player_turn(&mut engine);
    engine.state.player.hp = 2;
    engine.schedule_the_bomb(1, 40);
    engine.state.hand = make_deck(&["Burn"]);

    end_turn(&mut engine);

    assert_eq!(engine.state.player.hp, 0);
    assert_eq!(engine.state.enemies[0].entity.hp, 30);
    assert!(engine.state.combat_over);
}

#[test]
fn unused_replay_powers_and_entangled_are_removed_at_player_end_of_turn() {
    // Each of these powers implements atEndOfTurn(true) by queuing its own
    // RemoveSpecificPowerAction. They are player-power callbacks, not pre-card
    // callbacks, and all unused charges are removed rather than decremented.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/DoubleTapPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/BurstPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/AmplifyPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/EntanglePower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/core/AbstractCreature.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.player.set_status(sid::DOUBLE_TAP, 2);
    engine.state.player.set_status(sid::BURST, 2);
    engine.state.player.set_status(sid::AMPLIFY, 2);
    engine.state.player.set_status(sid::ENTANGLED, 1);

    end_turn(&mut engine);

    assert_eq!(engine.state.player.status(sid::DOUBLE_TAP), 0);
    assert_eq!(engine.state.player.status(sid::BURST), 0);
    assert_eq!(engine.state.player.status(sid::AMPLIFY), 0);
    assert_eq!(engine.state.player.status(sid::ENTANGLED), 0);
}

#[test]
fn intangible_protects_the_enemy_turn_before_end_of_round_decrement() {
    // MonsterGroup.applyEndOfTurnPowers invokes player atEndOfRound only after
    // the monsters have taken their turns. IntangiblePlayerPower must cap this
    // attack first and lose its final stack afterward.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/MonsterGroup.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/IntangiblePlayerPower.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy("JawWorm", 40, 40, 1, 10, 1)], 3);
    force_player_turn(&mut engine);
    engine.state.player.set_status(sid::INTANGIBLE, 1);
    let hp_before = engine.state.player.hp;

    end_turn(&mut engine);

    assert_eq!(engine.state.player.hp, hp_before - 1);
    assert_eq!(engine.state.player.status(sid::INTANGIBLE), 0);
}

#[test]
fn vault_skips_intangible_end_of_round_decrement_with_the_monster_turn() {
    // Vault sets AbstractRoom.skipMonsterTurn. GameActionManager gates the
    // whole MonsterGroup.applyEndOfTurnPowers call on that flag; that method is
    // also where player powers receive atEndOfRound. Intangible therefore
    // remains installed for the extra turn.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Vault.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/MonsterGroup.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/IntangiblePlayerPower.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy("JawWorm", 40, 40, 1, 10, 1)], 3);
    force_player_turn(&mut engine);
    engine.state.player.set_status(sid::INTANGIBLE, 1);
    engine.state.hand = make_deck(&["Vault"]);
    let hp_before = engine.state.player.hp;

    assert!(crate::tests::support::play_self(&mut engine, "Vault"));

    assert_eq!(engine.state.player.hp, hp_before);
    assert_eq!(engine.state.player.status(sid::INTANGIBLE), 1);
    assert_eq!(engine.state.turn, 2);
}

#[test]
fn cloak_clasp_and_orichalcum_observe_pre_action_block_during_callbacks() {
    // AbstractRoom.applyEndOfTurnRelics invokes both callbacks before queued
    // actions drain. CloakClasp first queues Block equal to hand size, but
    // Orichalcum still observes currentBlock == 0 and queues six more Block.
    // With two cards, all eight Block must exist before this ten-damage attack.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/rooms/AbstractRoom.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/CloakClasp.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Orichalcum.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/GainBlockAction.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy("JawWorm", 40, 40, 1, 10, 1)], 3);
    engine.state.relics = vec!["CloakClasp".to_string(), "Orichalcum".to_string()];
    engine.start_combat();
    engine.state.hand = make_deck(&["Strike", "Defend"]);
    engine.state.draw_pile.clear();
    let hp_before = engine.state.player.hp;

    end_turn(&mut engine);

    assert_eq!(engine.state.player.hp, hp_before - 2);
}

#[test]
fn burn_vacates_a_full_hand_before_runic_cube_draws() {
    // AbstractPlayer.useCard calls Burn.use, removes Burn from hand, and only
    // then does its queued DamageAction resolve. RunicCube.wasHPLost adds its
    // DrawCardAction to the top, so the vacated tenth hand slot accepts the
    // draw. RetainCardPower pauses before discard so this ordering is directly
    // observable here.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/status/Burn.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/RunicCube.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/RetainCardPower.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.relics.push("Runic Cube".to_string());
    engine.state.player.set_status(sid::RETAIN_CARDS, 1);
    engine.state.hand = make_deck(&[
        "Burn", "Defend", "Defend", "Defend", "Defend", "Defend", "Defend", "Defend", "Defend",
        "Defend",
    ]);
    engine.state.draw_pile = make_deck(&["Strike"]);
    let hp_before = engine.state.player.hp;

    end_turn(&mut engine);

    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(engine.state.player.hp, hp_before - 2);
    assert_eq!(hand_count(&engine, "Burn"), 0);
    assert_eq!(hand_count(&engine, "Strike"), 1);
    assert_eq!(engine.state.hand.len(), 10);
    assert!(engine.state.draw_pile.is_empty());
}

#[test]
fn lethal_burn_is_removed_from_hand_before_its_damage_resolves() {
    // Even when Burn's queued DamageAction is lethal, AbstractPlayer.useCard
    // has already removed the autoplay CardQueueItem from hand. Combat cleanup
    // therefore cannot leave a dead player's Burn in the live hand zone.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/status/Burn.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.player.hp = 2;
    engine.state.hand = make_deck(&["Burn", "Defend"]);

    end_turn(&mut engine);

    assert_eq!(engine.state.player.hp, 0);
    assert!(engine.state.combat_over);
    assert_eq!(hand_count(&engine, "Burn"), 0);
}

#[test]
fn doubt_weak_just_applied_survives_the_same_round_end() {
    // Doubt constructs WeakPower(player, 1, true). WeakPower's justApplied
    // latch skips the immediately following atEndOfRound decrement, then the
    // next ordinary round end consumes the one remaining turn.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Doubt.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/WeakPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/monsters/MonsterGroup.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.hand = make_deck(&["Doubt"]);

    end_turn(&mut engine);

    assert_eq!(engine.state.player.status(sid::WEAKENED), 1);
    assert_eq!(engine.state.player.status(sid::WEAKENED_JUST_APPLIED), 0);

    // Prevent the one-card discard from reshuffling, redrawing, and replaying
    // Doubt on the control round; only the installed Weak is under test now.
    engine.state.hand.clear();
    engine.state.draw_pile.clear();
    engine.state.discard_pile.clear();
    end_turn(&mut engine);

    assert_eq!(engine.state.player.status(sid::WEAKENED), 0);
}

#[test]
fn later_regeneration_add_to_top_heals_before_earlier_constricted_damage() {
    // AbstractCreature.applyEndOfTurnTriggers visits powers in list order.
    // Constricted first adds DamageAction to the bottom, but a later
    // RegenPower adds RegenAction to the top. At two HP, healing five before
    // taking three is the difference between surviving at four and dying.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/core/AbstractCreature.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/ConstrictedPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/RegenPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/RegenAction.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.player.hp = 2;
    engine.state.player.set_status_direct(sid::CONSTRICTED, 3);
    engine.state.player.set_status_direct(sid::REGENERATION, 5);

    end_turn(&mut engine);

    assert_eq!(engine.state.player.hp, 4);
    assert_eq!(engine.state.player.status(sid::REGENERATION), 4);
    assert!(!engine.state.combat_over);
}

#[test]
fn delayed_strength_and_dexterity_losses_are_artifact_blockable_then_removed() {
    // LoseStrengthPower and LoseDexterityPower each queue a negative
    // StrengthPower/DexterityPower ApplyPowerAction followed by an unconditional
    // RemoveSpecificPowerAction. Negative stat powers are DEBUFFs, so Artifact
    // consumes one stack and blocks the stat loss without preserving the
    // delayed-loss power.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/LoseStrengthPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/LoseDexterityPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/StrengthPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/DexterityPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ApplyPowerAction.java
    for (stat, delayed_loss) in [
        (sid::STRENGTH, sid::LOSE_STRENGTH),
        (sid::DEXTERITY, sid::LOSE_DEXTERITY),
    ] {
        let mut engine =
            engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
        force_player_turn(&mut engine);
        engine.state.player.set_status(stat, 3);
        engine.state.player.set_status(delayed_loss, 3);
        engine.state.player.set_status(sid::ARTIFACT, 1);

        end_turn(&mut engine);

        assert_eq!(engine.state.player.status(stat), 3);
        assert_eq!(engine.state.player.status(sid::ARTIFACT), 0);
        assert_eq!(engine.state.player.status(delayed_loss), 0);
    }
}

#[test]
fn wraith_form_and_delayed_strength_loss_compete_for_one_artifact_in_power_order() {
    // Player atEndOfTurn callbacks queue actions in the current powers order.
    // With Wraith Form before LoseStrengthPower, its negative Dexterity
    // application spends the sole Artifact first; the later Strength loss then
    // lands, while LoseStrengthPower still removes itself afterward.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/core/AbstractCreature.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/WraithFormPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/LoseStrengthPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ApplyPowerAction.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.player.set_status_direct(sid::ARTIFACT, 1);
    engine.state.player.set_status_direct(sid::WRAITH_FORM, 2);
    engine.state.player.set_status_direct(sid::LOSE_STRENGTH, 3);
    engine.state.player.set_status(sid::STRENGTH, 3);

    end_turn(&mut engine);

    assert_eq!(engine.state.player.status(sid::ARTIFACT), 0);
    assert_eq!(engine.state.player.status(sid::DEXTERITY), 0);
    assert_eq!(engine.state.player.status(sid::STRENGTH), 0);
    assert_eq!(engine.state.player.status(sid::LOSE_STRENGTH), 0);
    assert_eq!(engine.state.player.status(sid::WRAITH_FORM), 2);
}

#[test]
fn lethal_stone_calendar_clears_orb_and_pride_actions_but_not_status_card_queue() {
    // Stone Calendar's lethal DamageAllEnemiesAction calls
    // clearPostCombatActions. TriggerEndOfTurnOrbsAction and Pride's
    // CARD_MANIPULATION MakeTempCardInDrawPileAction are removed, while the
    // separately-held status CardQueueItem still autoplays Burn afterward.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/relics/StoneCalendar.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/TriggerEndOfTurnOrbsAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Pride.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/status/Burn.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    engine.init_defect_orbs(1);
    engine.state.relics.push("StoneCalendar".to_string());
    engine.start_combat();
    engine.channel_orb(OrbType::Frost);
    assert!(engine.set_hidden_effect_value(
        "StoneCalendar",
        EffectOwner::PlayerRelic { slot: 0 },
        0,
        7,
    ));
    engine.state.hand = make_deck(&["Pride", "Burn"]);
    engine.state.draw_pile.clear();
    let hp_before = engine.state.player.hp;

    end_turn(&mut engine);

    assert!(engine.state.is_victory());
    assert_eq!(engine.state.player.block, 0);
    assert_eq!(engine.state.player.hp, hp_before - 2);
    assert_eq!(hand_count(&engine, "Burn"), 0);
    assert_eq!(
        engine
            .state
            .draw_pile
            .iter()
            .filter(|card| engine.card_registry.card_name(card.def_id) == "Pride")
            .count(),
        0,
    );
}

#[test]
fn lethal_omega_clears_later_study_and_retain_actions() {
    // AbstractCreature collects all power callbacks in powers order. Omega's
    // first queued DamageAllEnemiesAction is lethal, so
    // clearPostCombatActions removes Study's CARD_MANIPULATION action and the
    // later RetainCardsAction instead of generating Insight or opening a UI.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/core/AbstractCreature.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/OmegaPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/StudyPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/RetainCardPower.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 20, 20)], 3);
    force_player_turn(&mut engine);
    engine.state.player.set_status_direct(sid::OMEGA, 50);
    engine.state.player.set_status_direct(sid::STUDY, 1);
    engine.state.player.set_status_direct(sid::RETAIN_CARDS, 1);
    engine.state.hand = make_deck(&["Defend"]);
    engine.state.draw_pile.clear();

    end_turn(&mut engine);

    assert!(engine.state.is_victory());
    assert!(engine.choice.is_none());
    assert_ne!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(hand_count(&engine, "Insight"), 0);
    assert_eq!(
        engine
            .state
            .draw_pile
            .iter()
            .filter(|card| engine.card_registry.card_name(card.def_id) == "Insight")
            .count(),
        0,
    );
}

#[test]
fn lethal_omega_preserves_later_combust_damage_class_actions() {
    // Combust's callback runs after Omega's callback and queues LoseHPAction
    // followed by DamageAllEnemiesAction. Both have ActionType.DAMAGE, so
    // Omega's lethal DamageAllEnemiesAction must retain both through
    // clearPostCombatActions. The post-victory HP loss is the directly
    // observable survivor; the later AoE also drains as a DAMAGE survivor even
    // though no living target remains.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/core/AbstractCreature.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/OmegaPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/CombustPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/DamageAllEnemiesAction.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 20, 20)], 3);
    force_player_turn(&mut engine);
    engine.state.player.hp = 10;
    engine.state.player.set_status_direct(sid::OMEGA, 50);
    engine.state.player.set_status_direct(sid::COMBUST, 5);

    end_turn(&mut engine);

    assert!(engine.state.is_victory());
    assert_eq!(engine.state.player.hp, 9);
    assert!(engine.end_turn_actions.is_empty());
}

#[test]
fn retain_cards_reinserts_two_cards_in_reverse_source_order_when_clicked_that_way() {
    // HandCardSelectScreen removes each clicked card and addToTop's it into
    // selectedCards. RetainCardsAction then iterates that click-ordered group
    // and addToTop's each card back to hand. Clicking the second source card
    // and then the first therefore reverses their relative retained order.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/screens/select/HandCardSelectScreen.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/RetainCardsAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 40, 40)], 3);
    force_player_turn(&mut engine);
    engine.state.player.set_status(sid::RETAIN_CARDS, 2);
    engine.state.hand = make_deck(&["Strike", "Defend", "Bash"]);
    engine.state.draw_pile.clear();

    end_turn(&mut engine);

    let choice = engine.choice.as_ref().expect("Retain Cards choice");
    let option_for = |card_name: &str| {
        choice
            .options
            .iter()
            .position(|option| match option {
                crate::engine::ChoiceOption::HandCard(hand_idx) => {
                    engine.state.hand.get(*hand_idx).is_some_and(|card| {
                        engine.card_registry.card_name(card.def_id) == card_name
                    })
                }
                _ => false,
            })
            .unwrap_or_else(|| panic!("missing {card_name} retain option"))
    };
    let defend = option_for("Defend");
    let strike = option_for("Strike");

    engine.execute_action(&Action::Choose(defend));
    engine.execute_action(&Action::Choose(strike));
    engine.execute_action(&Action::ConfirmSelection);

    let names = engine
        .state
        .hand
        .iter()
        .map(|card| engine.card_registry.card_name(card.def_id))
        .collect::<Vec<_>>();
    let defend_pos = names
        .iter()
        .position(|name| *name == "Defend")
        .expect("retained Defend");
    let strike_pos = names
        .iter()
        .position(|name| *name == "Strike")
        .expect("retained Strike");
    assert!(defend_pos < strike_pos, "retained hand order: {names:?}");
}

#[test]
fn lethal_lightning_still_allows_later_frost_end_turn_block() {
    // TriggerEndOfTurnOrbsAction invokes every orb's onEndOfTurn callback in
    // slot order before GameActionManager drains the queued actions. A lethal
    // Lightning action therefore clears only non-survivors; the later Frost
    // GainBlockAction is explicitly retained by clearPostCombatActions.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/TriggerEndOfTurnOrbsAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/orbs/Lightning.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/orbs/Frost.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 3, 3)], 3);
    engine.init_defect_orbs(2);
    engine.channel_orb(OrbType::Lightning);
    engine.channel_orb(OrbType::Frost);
    engine.start_combat();

    end_turn(&mut engine);

    assert!(engine.state.is_victory());
    assert_eq!(engine.state.player.block, 2);
}

#[test]
fn lethal_lightning_does_not_suppress_burn_status_card_queue() {
    // The orb action queue drains before status CardQueueItems, but a lethal
    // Lightning passive does not clear GameActionManager.cardQueue. Burn must
    // still autoplay, damage the player, and leave the hand after orb victory.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/TriggerEndOfTurnOrbsAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/orbs/Lightning.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/status/Burn.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 3, 3)], 3);
    engine.init_defect_orbs(1);
    engine.channel_orb(OrbType::Lightning);
    engine.start_combat();
    engine.state.hand = make_deck(&["Burn"]);
    let hp_before = engine.state.player.hp;

    end_turn(&mut engine);

    assert!(engine.state.is_victory());
    assert_eq!(engine.state.player.hp, hp_before - 2);
    assert_eq!(hand_count(&engine, "Burn"), 0);
}

#[test]
fn lethal_lightning_still_allows_pride_action_already_ahead_of_orb_damage() {
    // callEndOfTurnActions queues TriggerEndOfTurnOrbsAction, then Pride queues
    // MakeTempCardInDrawPileAction. When TriggerEndOfTurnOrbsAction runs, the
    // Lightning child damage is appended behind that already-queued Pride
    // action. Pride therefore copies before Lightning becomes lethal, unlike
    // the earlier Stone Calendar fixture where lethal damage clears both.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/TriggerEndOfTurnOrbsAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/orbs/Lightning.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Pride.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInDrawPileAction.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 3, 3)], 3);
    engine.init_defect_orbs(1);
    engine.channel_orb(OrbType::Lightning);
    engine.start_combat();
    engine.state.hand = make_deck(&["Pride"]);
    engine.state.draw_pile.clear();

    end_turn(&mut engine);

    assert!(engine.state.is_victory());
    assert_eq!(
        engine
            .state
            .draw_pile
            .iter()
            .filter(|card| engine.card_registry.card_name(card.def_id) == "Pride")
            .count(),
        1,
    );
}

#[test]
fn retain_choice_before_lethal_omega_resumes_to_victory_without_discard() {
    // RetainCardPower precedes OmegaPower in powers order, so its queued
    // RetainCardsAction suspends the shared action queue first. After the user
    // confirms, Omega's later DamageAllEnemiesAction becomes lethal and
    // clearPostCombatActions removes ClearCardQueueAction and
    // DiscardAtEndOfTurnAction. Both the selected and unselected cards remain
    // in hand at terminal combat.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/core/AbstractCreature.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/rooms/AbstractRoom.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/RetainCardPower.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/RetainCardsAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/OmegaPower.java
    let mut engine = engine_without_start(Vec::new(), vec![enemy_no_intent("JawWorm", 20, 20)], 3);
    force_player_turn(&mut engine);
    engine.state.player.set_status_direct(sid::RETAIN_CARDS, 2);
    engine.state.player.set_status_direct(sid::OMEGA, 50);
    engine.state.hand = make_deck(&["Strike", "Defend"]);
    engine.state.draw_pile.clear();

    end_turn(&mut engine);

    assert_eq!(engine.phase, CombatPhase::AwaitingChoice);
    let strike = engine
        .choice
        .as_ref()
        .expect("Retain Cards choice")
        .options
        .iter()
        .position(|option| match option {
            crate::engine::ChoiceOption::HandCard(hand_idx) => engine
                .state
                .hand
                .get(*hand_idx)
                .is_some_and(|card| engine.card_registry.card_name(card.def_id) == "Strike"),
            _ => false,
        })
        .expect("Strike retain option");
    engine.execute_action(&Action::Choose(strike));
    engine.execute_action(&Action::ConfirmSelection);

    assert!(engine.state.is_victory());
    assert_ne!(engine.phase, CombatPhase::AwaitingChoice);
    assert_eq!(hand_count(&engine, "Strike"), 1);
    assert_eq!(hand_count(&engine, "Defend"), 1);
    assert!(engine.state.discard_pile.is_empty());
}
