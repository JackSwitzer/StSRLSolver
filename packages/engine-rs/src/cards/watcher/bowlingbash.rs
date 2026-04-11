use crate::cards::prelude::*;

fn bowling_bash_hook(engine: &mut crate::engine::CombatEngine, ctx: &crate::effects::types::CardPlayContext) {
    // Deal additional hits equal to (living_enemy_count - 1) since preamble already did 1 hit
    let living = engine.state.living_enemy_indices().len() as i32;
    let extra_hits = (living - 1).max(0);
    if extra_hits > 0 && ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        let tidx = ctx.target_idx as usize;
        let player_strength = engine.state.player.strength();
        let player_weak = engine.state.player.is_weak();
        let weak_paper_crane = engine.state.has_relic("Paper Crane");
        let stance_mult = engine.state.stance.outgoing_mult();
        let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
        let enemy_intangible = engine.state.enemies[tidx].entity.status(sid::INTANGIBLE) > 0;
        let vuln_paper_frog = engine.state.has_relic("Paper Frog");
        let dmg = crate::damage::calculate_damage_full(
            ctx.card.base_damage.max(0), player_strength, 0, player_weak,
            weak_paper_crane, false, false, stance_mult,
            enemy_vuln, vuln_paper_frog, false, enemy_intangible,
        );
        for _ in 0..extra_hits {
            engine.deal_damage_to_enemy(tidx, dmg);
            if engine.state.enemies[tidx].entity.is_dead() { break; }
        }
    }
}

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Common Watcher Cards ----
    insert(cards, CardDef {
                id: "BowlingBash", name: "Bowling Bash", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["damage_per_enemy"], effect_data: &[], complex_hook: Some(bowling_bash_hook),
            });
    insert(cards, CardDef {
                id: "BowlingBash+", name: "Bowling Bash+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["damage_per_enemy"], effect_data: &[], complex_hook: Some(bowling_bash_hook),
            });
}
