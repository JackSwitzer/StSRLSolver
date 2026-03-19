//! Card data and effects — full Watcher card registry for the combat turn loop.
//!
//! Implements ALL Watcher cards (75+) so the Rust engine can handle complete
//! combat simulations for TurnSolver. Complex effects (scry, discover, etc.)
//! are modeled as effect tags that the engine interprets.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Card types (match Python enums)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardType {
    Attack,
    Skill,
    Power,
    Status,
    Curse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardTarget {
    /// Single enemy (requires target selection)
    Enemy,
    /// All enemies (no target needed)
    AllEnemy,
    /// Self only
    SelfTarget,
    /// No target
    None,
}

// ---------------------------------------------------------------------------
// Card definition — static data, no mutation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct CardDef {
    pub id: &'static str,
    pub name: &'static str,
    pub card_type: CardType,
    pub target: CardTarget,
    pub cost: i32,
    pub base_damage: i32,
    pub base_block: i32,
    pub base_magic: i32,
    /// Does this card exhaust when played?
    pub exhaust: bool,
    /// Does this card retain in hand at end of turn?
    pub retain: bool,
    /// Does this card shuffle back into draw pile instead of discard?
    pub shuffle_back: bool,
    /// Does this card change stance?
    pub enter_stance: Option<&'static str>,
    /// Special effect tags for the engine to check
    pub effects: &'static [&'static str],
}

impl CardDef {
    /// Is this card an unplayable status/curse?
    pub fn is_unplayable(&self) -> bool {
        self.cost == -2
    }
}

// ---------------------------------------------------------------------------
// Card registry — lookup by ID (including "+" suffix for upgrades)
// ---------------------------------------------------------------------------

/// Static card registry. Populated with ALL Watcher cards + universals.
/// Cards not in the registry fall back to defaults (cost 1, attack, enemy target).
pub struct CardRegistry {
    cards: HashMap<&'static str, CardDef>,
}

impl CardRegistry {
    pub fn new() -> Self {
        let mut cards = HashMap::new();

        // ==================================================================
        // WATCHER BASIC CARDS
        // ==================================================================

        Self::ins(&mut cards, "Strike_P", "Strike", CardType::Attack,
            CardTarget::Enemy, 1, 6, -1, -1, &[]);
        Self::ins(&mut cards, "Strike_P+", "Strike+", CardType::Attack,
            CardTarget::Enemy, 1, 9, -1, -1, &[]);
        Self::ins(&mut cards, "Defend_P", "Defend", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 5, -1, &[]);
        Self::ins(&mut cards, "Defend_P+", "Defend+", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 8, -1, &[]);

        // Eruption: deal 9 damage, enter Wrath
        Self::ins_stance(&mut cards, "Eruption", "Eruption", CardType::Attack,
            CardTarget::Enemy, 2, 9, -1, -1, "Wrath", &[]);
        Self::ins_stance(&mut cards, "Eruption+", "Eruption+", CardType::Attack,
            CardTarget::Enemy, 1, 9, -1, -1, "Wrath", &[]);

        // Vigilance: gain block, enter Calm
        Self::ins_stance(&mut cards, "Vigilance", "Vigilance", CardType::Skill,
            CardTarget::SelfTarget, 2, -1, 8, -1, "Calm", &[]);
        Self::ins_stance(&mut cards, "Vigilance+", "Vigilance+", CardType::Skill,
            CardTarget::SelfTarget, 2, -1, 12, -1, "Calm", &[]);

        // Miracle: retain, exhaust, gain 1 energy (upgraded: gain 2)
        Self::ins_flags(&mut cards, "Miracle", "Miracle", CardType::Skill,
            CardTarget::SelfTarget, 0, -1, -1, 1, true, true, false,
            None, &["gain_energy"]);
        Self::ins_flags(&mut cards, "Miracle+", "Miracle+", CardType::Skill,
            CardTarget::SelfTarget, 0, -1, -1, 2, true, true, false,
            None, &["gain_energy"]);

        // ==================================================================
        // COMMON ATTACKS
        // ==================================================================

        // Bowling Bash: deal damage * number of enemies
        Self::ins(&mut cards, "BowlingBash", "Bowling Bash", CardType::Attack,
            CardTarget::Enemy, 1, 7, -1, -1, &["damage_per_enemy"]);
        Self::ins(&mut cards, "BowlingBash+", "Bowling Bash+", CardType::Attack,
            CardTarget::Enemy, 1, 10, -1, -1, &["damage_per_enemy"]);

        // Consecrate: deal damage to ALL enemies
        Self::ins(&mut cards, "Consecrate", "Consecrate", CardType::Attack,
            CardTarget::AllEnemy, 0, 5, -1, -1, &[]);
        Self::ins(&mut cards, "Consecrate+", "Consecrate+", CardType::Attack,
            CardTarget::AllEnemy, 0, 8, -1, -1, &[]);

        // Crush Joints: deal damage, if last card was a Skill apply Vulnerable
        Self::ins(&mut cards, "CrushJoints", "Crush Joints", CardType::Attack,
            CardTarget::Enemy, 1, 8, -1, 1, &["vuln_if_last_skill"]);
        Self::ins(&mut cards, "CrushJoints+", "Crush Joints+", CardType::Attack,
            CardTarget::Enemy, 1, 10, -1, 2, &["vuln_if_last_skill"]);

        // Cut Through Fate: deal damage, scry magic, draw 1
        Self::ins(&mut cards, "CutThroughFate", "Cut Through Fate", CardType::Attack,
            CardTarget::Enemy, 1, 7, -1, 2, &["scry", "draw_1"]);
        Self::ins(&mut cards, "CutThroughFate+", "Cut Through Fate+", CardType::Attack,
            CardTarget::Enemy, 1, 9, -1, 3, &["scry", "draw_1"]);

        // Empty Fist: deal damage, exit stance
        Self::ins(&mut cards, "EmptyFist", "Empty Fist", CardType::Attack,
            CardTarget::Enemy, 1, 9, -1, -1, &["exit_stance"]);
        Self::ins(&mut cards, "EmptyFist+", "Empty Fist+", CardType::Attack,
            CardTarget::Enemy, 1, 14, -1, -1, &["exit_stance"]);

        // Flurry of Blows: 0 cost attack (Java ID: FlurryOfBlows, Rust ID: FlurryOfBlows)
        Self::ins(&mut cards, "FlurryOfBlows", "Flurry of Blows", CardType::Attack,
            CardTarget::Enemy, 0, 4, -1, -1, &[]);
        Self::ins(&mut cards, "FlurryOfBlows+", "Flurry of Blows+", CardType::Attack,
            CardTarget::Enemy, 0, 6, -1, -1, &[]);

        // Flying Sleeves: deal damage twice, retain
        Self::ins_flags(&mut cards, "FlyingSleeves", "Flying Sleeves", CardType::Attack,
            CardTarget::Enemy, 1, 4, -1, 2, false, true, false,
            None, &["multi_hit"]);
        Self::ins_flags(&mut cards, "FlyingSleeves+", "Flying Sleeves+", CardType::Attack,
            CardTarget::Enemy, 1, 6, -1, 2, false, true, false,
            None, &["multi_hit"]);

        // Follow-Up: deal damage, if last card was attack gain 1 energy
        Self::ins(&mut cards, "FollowUp", "Follow-Up", CardType::Attack,
            CardTarget::Enemy, 1, 7, -1, -1, &["energy_if_last_attack"]);
        Self::ins(&mut cards, "FollowUp+", "Follow-Up+", CardType::Attack,
            CardTarget::Enemy, 1, 11, -1, -1, &["energy_if_last_attack"]);

        // Sash Whip: deal damage, if last card was attack apply Weak
        Self::ins(&mut cards, "SashWhip", "Sash Whip", CardType::Attack,
            CardTarget::Enemy, 1, 8, -1, 1, &["weak_if_last_attack"]);
        Self::ins(&mut cards, "SashWhip+", "Sash Whip+", CardType::Attack,
            CardTarget::Enemy, 1, 10, -1, 2, &["weak_if_last_attack"]);

        // Just Lucky: scry, gain block, deal damage
        Self::ins(&mut cards, "JustLucky", "Just Lucky", CardType::Attack,
            CardTarget::Enemy, 0, 3, 2, 1, &["scry"]);
        Self::ins(&mut cards, "JustLucky+", "Just Lucky+", CardType::Attack,
            CardTarget::Enemy, 0, 4, 3, 2, &["scry"]);

        // ==================================================================
        // COMMON SKILLS
        // ==================================================================

        // Empty Body: gain block, exit stance (enter Neutral)
        Self::ins_stance(&mut cards, "EmptyBody", "Empty Body", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 7, -1, "Neutral", &["exit_stance"]);
        Self::ins_stance(&mut cards, "EmptyBody+", "Empty Body+", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 11, -1, "Neutral", &["exit_stance"]);

        // Evaluate: gain block, add Insight to draw
        Self::ins(&mut cards, "Evaluate", "Evaluate", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 6, -1, &["add_insight_to_draw"]);
        Self::ins(&mut cards, "Evaluate+", "Evaluate+", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 10, -1, &["add_insight_to_draw"]);

        // Halt: gain block, extra block in Wrath (magic = extra block amount)
        Self::ins(&mut cards, "Halt", "Halt", CardType::Skill,
            CardTarget::SelfTarget, 0, -1, 3, 9, &["extra_block_in_wrath"]);
        Self::ins(&mut cards, "Halt+", "Halt+", CardType::Skill,
            CardTarget::SelfTarget, 0, -1, 4, 14, &["extra_block_in_wrath"]);

        // Prostrate: gain block, gain Mantra (magic = mantra amount)
        Self::ins(&mut cards, "Prostrate", "Prostrate", CardType::Skill,
            CardTarget::SelfTarget, 0, -1, 4, 2, &["mantra"]);
        Self::ins(&mut cards, "Prostrate+", "Prostrate+", CardType::Skill,
            CardTarget::SelfTarget, 0, -1, 4, 3, &["mantra"]);

        // Protect: gain block, retain
        Self::ins_flags(&mut cards, "Protect", "Protect", CardType::Skill,
            CardTarget::SelfTarget, 2, -1, 12, -1, false, true, false,
            None, &[]);
        Self::ins_flags(&mut cards, "Protect+", "Protect+", CardType::Skill,
            CardTarget::SelfTarget, 2, -1, 16, -1, false, true, false,
            None, &[]);

        // Third Eye: gain block, scry magic
        Self::ins(&mut cards, "ThirdEye", "Third Eye", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 7, 3, &["scry"]);
        Self::ins(&mut cards, "ThirdEye+", "Third Eye+", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 9, 5, &["scry"]);

        // Crescendo: enter Wrath, retain, exhaust (cost 1, upgraded 0)
        Self::ins_flags(&mut cards, "Crescendo", "Crescendo", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, -1, -1, true, true, false,
            Some("Wrath"), &[]);
        Self::ins_flags(&mut cards, "Crescendo+", "Crescendo+", CardType::Skill,
            CardTarget::SelfTarget, 0, -1, -1, -1, true, true, false,
            Some("Wrath"), &[]);

        // Tranquility (Java ID: ClearTheMind): enter Calm, retain, exhaust
        Self::ins_flags(&mut cards, "ClearTheMind", "Tranquility", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, -1, -1, true, true, false,
            Some("Calm"), &[]);
        Self::ins_flags(&mut cards, "ClearTheMind+", "Tranquility+", CardType::Skill,
            CardTarget::SelfTarget, 0, -1, -1, -1, true, true, false,
            Some("Calm"), &[]);

        // Pressure Points (Java ID: PathToVictory): apply Mark, trigger all Marks
        Self::ins(&mut cards, "PathToVictory", "Pressure Points", CardType::Skill,
            CardTarget::Enemy, 1, -1, -1, 8, &["apply_mark", "trigger_marks"]);
        Self::ins(&mut cards, "PathToVictory+", "Pressure Points+", CardType::Skill,
            CardTarget::Enemy, 1, -1, -1, 11, &["apply_mark", "trigger_marks"]);

        // ==================================================================
        // UNCOMMON ATTACKS
        // ==================================================================

        // Tantrum: deal damage magic times, enter Wrath, shuffle back
        Self::ins_flags(&mut cards, "Tantrum", "Tantrum", CardType::Attack,
            CardTarget::Enemy, 1, 3, -1, 3, false, false, true,
            Some("Wrath"), &["multi_hit"]);
        Self::ins_flags(&mut cards, "Tantrum+", "Tantrum+", CardType::Attack,
            CardTarget::Enemy, 1, 3, -1, 4, false, false, true,
            Some("Wrath"), &["multi_hit"]);

        // Fear No Evil: deal damage, if enemy attacking enter Calm
        Self::ins(&mut cards, "FearNoEvil", "Fear No Evil", CardType::Attack,
            CardTarget::Enemy, 1, 8, -1, -1, &["calm_if_enemy_attacking"]);
        Self::ins(&mut cards, "FearNoEvil+", "Fear No Evil+", CardType::Attack,
            CardTarget::Enemy, 1, 11, -1, -1, &["calm_if_enemy_attacking"]);

        // Reach Heaven: deal damage, add Through Violence to draw
        Self::ins(&mut cards, "ReachHeaven", "Reach Heaven", CardType::Attack,
            CardTarget::Enemy, 2, 10, -1, -1, &["add_through_violence_to_draw"]);
        Self::ins(&mut cards, "ReachHeaven+", "Reach Heaven+", CardType::Attack,
            CardTarget::Enemy, 2, 15, -1, -1, &["add_through_violence_to_draw"]);

        // Sands of Time: deal damage, retain, cost reduces each turn
        Self::ins_flags(&mut cards, "SandsOfTime", "Sands of Time", CardType::Attack,
            CardTarget::Enemy, 4, 20, -1, -1, false, true, false,
            None, &["cost_reduces_each_turn"]);
        Self::ins_flags(&mut cards, "SandsOfTime+", "Sands of Time+", CardType::Attack,
            CardTarget::Enemy, 4, 26, -1, -1, false, true, false,
            None, &["cost_reduces_each_turn"]);

        // Signature Move: deal 30 damage, only playable if only attack in hand
        Self::ins(&mut cards, "SignatureMove", "Signature Move", CardType::Attack,
            CardTarget::Enemy, 2, 30, -1, -1, &["only_attack_in_hand"]);
        Self::ins(&mut cards, "SignatureMove+", "Signature Move+", CardType::Attack,
            CardTarget::Enemy, 2, 40, -1, -1, &["only_attack_in_hand"]);

        // Talk to the Hand: deal damage, apply Block Return, exhaust
        Self::ins_flags(&mut cards, "TalkToTheHand", "Talk to the Hand", CardType::Attack,
            CardTarget::Enemy, 1, 5, -1, 2, true, false, false,
            None, &["apply_block_return"]);
        Self::ins_flags(&mut cards, "TalkToTheHand+", "Talk to the Hand+", CardType::Attack,
            CardTarget::Enemy, 1, 7, -1, 3, true, false, false,
            None, &["apply_block_return"]);

        // Wallop: deal damage, gain block equal to unblocked damage
        Self::ins(&mut cards, "Wallop", "Wallop", CardType::Attack,
            CardTarget::Enemy, 2, 9, -1, -1, &["gain_block_equal_damage"]);
        Self::ins(&mut cards, "Wallop+", "Wallop+", CardType::Attack,
            CardTarget::Enemy, 2, 12, -1, -1, &["gain_block_equal_damage"]);

        // Weave: 0-cost attack (plays from discard on scry -- reactive, handled by engine)
        Self::ins(&mut cards, "Weave", "Weave", CardType::Attack,
            CardTarget::Enemy, 0, 4, -1, -1, &[]);
        Self::ins(&mut cards, "Weave+", "Weave+", CardType::Attack,
            CardTarget::Enemy, 0, 6, -1, -1, &[]);

        // Wheel Kick: deal damage, draw 2
        Self::ins(&mut cards, "WheelKick", "Wheel Kick", CardType::Attack,
            CardTarget::Enemy, 2, 15, -1, -1, &["draw_2"]);
        Self::ins(&mut cards, "WheelKick+", "Wheel Kick+", CardType::Attack,
            CardTarget::Enemy, 2, 20, -1, -1, &["draw_2"]);

        // Windmill Strike: deal damage, retain, gain +magic damage when retained
        Self::ins_flags(&mut cards, "WindmillStrike", "Windmill Strike", CardType::Attack,
            CardTarget::Enemy, 2, 7, -1, 4, false, true, false,
            None, &["gain_damage_when_retained"]);
        Self::ins_flags(&mut cards, "WindmillStrike+", "Windmill Strike+", CardType::Attack,
            CardTarget::Enemy, 2, 10, -1, 5, false, true, false,
            None, &["gain_damage_when_retained"]);

        // Conclude: deal damage to ALL enemies, end turn
        Self::ins(&mut cards, "Conclude", "Conclude", CardType::Attack,
            CardTarget::AllEnemy, 1, 12, -1, -1, &["end_turn"]);
        Self::ins(&mut cards, "Conclude+", "Conclude+", CardType::Attack,
            CardTarget::AllEnemy, 1, 16, -1, -1, &["end_turn"]);

        // Carve Reality: deal damage, add Smite to hand
        Self::ins(&mut cards, "CarveReality", "Carve Reality", CardType::Attack,
            CardTarget::Enemy, 1, 6, -1, -1, &["add_smite_to_hand"]);
        Self::ins(&mut cards, "CarveReality+", "Carve Reality+", CardType::Attack,
            CardTarget::Enemy, 1, 10, -1, -1, &["add_smite_to_hand"]);

        // ==================================================================
        // UNCOMMON SKILLS
        // ==================================================================

        // Empty Mind: exit stance, draw cards (magic = draw amount)
        Self::ins(&mut cards, "EmptyMind", "Empty Mind", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, -1, 2, &["exit_stance", "draw_cards"]);
        Self::ins(&mut cards, "EmptyMind+", "Empty Mind+", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, -1, 3, &["exit_stance", "draw_cards"]);

        // Inner Peace: if in Calm draw magic, else enter Calm
        Self::ins(&mut cards, "InnerPeace", "Inner Peace", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, -1, 3, &["if_calm_draw_else_calm"]);
        Self::ins(&mut cards, "InnerPeace+", "Inner Peace+", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, -1, 4, &["if_calm_draw_else_calm"]);

        // Collect: X-cost, exhaust, put X Miracles on draw
        Self::ins_flags(&mut cards, "Collect", "Collect", CardType::Skill,
            CardTarget::SelfTarget, -1, -1, -1, -1, true, false, false,
            None, &["put_x_miracles_on_draw"]);
        Self::ins_flags(&mut cards, "Collect+", "Collect+", CardType::Skill,
            CardTarget::SelfTarget, -1, -1, -1, -1, true, false, false,
            None, &["put_x_miracles_on_draw"]);

        // Deceive Reality: gain block, add Safety to hand
        Self::ins(&mut cards, "DeceiveReality", "Deceive Reality", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 4, -1, &["add_safety_to_hand"]);
        Self::ins(&mut cards, "DeceiveReality+", "Deceive Reality+", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 7, -1, &["add_safety_to_hand"]);

        // Indignation: if in Wrath apply Vuln to all, else enter Wrath
        Self::ins(&mut cards, "Indignation", "Indignation", CardType::Skill,
            CardTarget::None, 1, -1, -1, 3, &["if_wrath_vuln_all_else_wrath"]);
        Self::ins(&mut cards, "Indignation+", "Indignation+", CardType::Skill,
            CardTarget::None, 1, -1, -1, 5, &["if_wrath_vuln_all_else_wrath"]);

        // Meditate: return card(s) from discard to hand, enter Calm, end turn
        Self::ins_stance(&mut cards, "Meditate", "Meditate", CardType::Skill,
            CardTarget::None, 1, -1, -1, 1, "Calm", &["meditate", "end_turn"]);
        Self::ins_stance(&mut cards, "Meditate+", "Meditate+", CardType::Skill,
            CardTarget::None, 1, -1, -1, 2, "Calm", &["meditate", "end_turn"]);

        // Perseverance: gain block, retain, gains extra block when retained
        Self::ins_flags(&mut cards, "Perseverance", "Perseverance", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 5, 2, false, true, false,
            None, &["gains_block_when_retained"]);
        Self::ins_flags(&mut cards, "Perseverance+", "Perseverance+", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 7, 3, false, true, false,
            None, &["gains_block_when_retained"]);

        // Pray: gain mantra, add Insight to draw
        Self::ins(&mut cards, "Pray", "Pray", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, -1, 3, &["mantra", "add_insight_to_draw"]);
        Self::ins(&mut cards, "Pray+", "Pray+", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, -1, 4, &["mantra", "add_insight_to_draw"]);

        // Sanctity: gain block, if last card was a Skill draw 2
        Self::ins(&mut cards, "Sanctity", "Sanctity", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 6, 2, &["draw_2_if_last_skill"]);
        Self::ins(&mut cards, "Sanctity+", "Sanctity+", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 9, 2, &["draw_2_if_last_skill"]);

        // Swivel: gain block, next attack this turn costs 0
        Self::ins(&mut cards, "Swivel", "Swivel", CardType::Skill,
            CardTarget::SelfTarget, 2, -1, 8, -1, &["free_attack_next"]);
        Self::ins(&mut cards, "Swivel+", "Swivel+", CardType::Skill,
            CardTarget::SelfTarget, 2, -1, 11, -1, &["free_attack_next"]);

        // Simmering Fury (Java ID: Vengeance): next turn enter Wrath and draw
        Self::ins(&mut cards, "Vengeance", "Simmering Fury", CardType::Skill,
            CardTarget::None, 1, -1, -1, 2, &["wrath_next_turn_draw"]);
        Self::ins(&mut cards, "Vengeance+", "Simmering Fury+", CardType::Skill,
            CardTarget::None, 1, -1, -1, 3, &["wrath_next_turn_draw"]);

        // Wave of the Hand: gain block_return_weak power (magic = weak stacks)
        Self::ins(&mut cards, "WaveOfTheHand", "Wave of the Hand", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, -1, 1, &["block_gain_applies_weak"]);
        Self::ins(&mut cards, "WaveOfTheHand+", "Wave of the Hand+", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, -1, 2, &["block_gain_applies_weak"]);

        // Worship: gain 5 mantra
        Self::ins(&mut cards, "Worship", "Worship", CardType::Skill,
            CardTarget::SelfTarget, 2, -1, -1, 5, &["mantra"]);
        Self::ins_flags(&mut cards, "Worship+", "Worship+", CardType::Skill,
            CardTarget::SelfTarget, 2, -1, -1, 5, false, true, false,
            None, &["mantra"]);

        // Wreath of Flame: next attack deals +magic extra damage
        Self::ins(&mut cards, "WreathOfFlame", "Wreath of Flame", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, -1, 5, &["next_attack_plus_damage"]);
        Self::ins(&mut cards, "WreathOfFlame+", "Wreath of Flame+", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, -1, 8, &["next_attack_plus_damage"]);

        // ==================================================================
        // UNCOMMON POWERS
        // ==================================================================

        // Battle Hymn: add Smite to hand each turn
        Self::ins(&mut cards, "BattleHymn", "Battle Hymn", CardType::Power,
            CardTarget::SelfTarget, 1, -1, -1, 1, &["add_smite_each_turn"]);
        Self::ins(&mut cards, "BattleHymn+", "Battle Hymn+", CardType::Power,
            CardTarget::SelfTarget, 1, -1, -1, 1, &["add_smite_each_turn"]);

        // Foresight (Java ID: Wireheading): scry each turn
        Self::ins(&mut cards, "Wireheading", "Foresight", CardType::Power,
            CardTarget::None, 1, -1, -1, 3, &["scry_each_turn"]);
        Self::ins(&mut cards, "Wireheading+", "Foresight+", CardType::Power,
            CardTarget::None, 1, -1, -1, 4, &["scry_each_turn"]);

        // Like Water: if in Calm at end of turn, gain block
        Self::ins(&mut cards, "LikeWater", "Like Water", CardType::Power,
            CardTarget::None, 1, -1, -1, 5, &["calm_end_turn_block"]);
        Self::ins(&mut cards, "LikeWater+", "Like Water+", CardType::Power,
            CardTarget::None, 1, -1, -1, 7, &["calm_end_turn_block"]);

        // Mental Fortress: gain block on stance change
        Self::ins(&mut cards, "MentalFortress", "Mental Fortress", CardType::Power,
            CardTarget::SelfTarget, 1, -1, -1, 4, &["on_stance_change_block"]);
        Self::ins(&mut cards, "MentalFortress+", "Mental Fortress+", CardType::Power,
            CardTarget::SelfTarget, 1, -1, -1, 6, &["on_stance_change_block"]);

        // Nirvana: gain block when scrying
        Self::ins(&mut cards, "Nirvana", "Nirvana", CardType::Power,
            CardTarget::SelfTarget, 1, -1, -1, 3, &["on_scry_block"]);
        Self::ins(&mut cards, "Nirvana+", "Nirvana+", CardType::Power,
            CardTarget::SelfTarget, 1, -1, -1, 4, &["on_scry_block"]);

        // Rushdown (Java ID: Adaptation): draw when entering Wrath
        Self::ins(&mut cards, "Adaptation", "Rushdown", CardType::Power,
            CardTarget::SelfTarget, 1, -1, -1, 2, &["on_wrath_draw"]);
        Self::ins(&mut cards, "Adaptation+", "Rushdown+", CardType::Power,
            CardTarget::SelfTarget, 0, -1, -1, 2, &["on_wrath_draw"]);

        // Study: add Insight at end of turn
        Self::ins(&mut cards, "Study", "Study", CardType::Power,
            CardTarget::SelfTarget, 2, -1, -1, 1, &["add_insight_end_turn"]);
        Self::ins(&mut cards, "Study+", "Study+", CardType::Power,
            CardTarget::SelfTarget, 1, -1, -1, 1, &["add_insight_end_turn"]);

        // Establishment: retained cards cost 1 less
        Self::ins(&mut cards, "Establishment", "Establishment", CardType::Power,
            CardTarget::SelfTarget, 1, -1, -1, 1, &["retained_cards_cost_less"]);
        Self::ins(&mut cards, "Establishment+", "Establishment+", CardType::Power,
            CardTarget::SelfTarget, 1, -1, -1, 1, &["retained_cards_cost_less"]);

        // ==================================================================
        // RARE ATTACKS
        // ==================================================================

        // Brilliance: deal 12 damage + mantra gained this combat
        Self::ins(&mut cards, "Brilliance", "Brilliance", CardType::Attack,
            CardTarget::Enemy, 1, 12, -1, -1, &["damage_plus_mantra"]);
        Self::ins(&mut cards, "Brilliance+", "Brilliance+", CardType::Attack,
            CardTarget::Enemy, 1, 16, -1, -1, &["damage_plus_mantra"]);

        // Lesson Learned: deal damage, exhaust, if fatal upgrade random card
        Self::ins_flags(&mut cards, "LessonLearned", "Lesson Learned", CardType::Attack,
            CardTarget::Enemy, 2, 10, -1, -1, true, false, false,
            None, &["if_fatal_upgrade_random"]);
        Self::ins_flags(&mut cards, "LessonLearned+", "Lesson Learned+", CardType::Attack,
            CardTarget::Enemy, 2, 13, -1, -1, true, false, false,
            None, &["if_fatal_upgrade_random"]);

        // Ragnarok: deal damage to random enemy magic times
        Self::ins(&mut cards, "Ragnarok", "Ragnarok", CardType::Attack,
            CardTarget::AllEnemy, 3, 5, -1, 5, &["damage_random_x_times"]);
        Self::ins(&mut cards, "Ragnarok+", "Ragnarok+", CardType::Attack,
            CardTarget::AllEnemy, 3, 6, -1, 6, &["damage_random_x_times"]);

        // ==================================================================
        // RARE SKILLS
        // ==================================================================

        // Judgement: if enemy HP <= magic, kill it
        Self::ins(&mut cards, "Judgement", "Judgement", CardType::Skill,
            CardTarget::Enemy, 1, -1, -1, 30, &["if_hp_below_kill"]);
        Self::ins(&mut cards, "Judgement+", "Judgement+", CardType::Skill,
            CardTarget::Enemy, 1, -1, -1, 40, &["if_hp_below_kill"]);

        // Deus Ex Machina: unplayable, on draw add Miracles and exhaust
        Self::ins_flags(&mut cards, "DeusExMachina", "Deus Ex Machina", CardType::Skill,
            CardTarget::SelfTarget, -2, -1, -1, 2, true, false, false,
            None, &["unplayable", "on_draw_add_miracles"]);
        Self::ins_flags(&mut cards, "DeusExMachina+", "Deus Ex Machina+", CardType::Skill,
            CardTarget::SelfTarget, -2, -1, -1, 3, true, false, false,
            None, &["unplayable", "on_draw_add_miracles"]);

        // Alpha: exhaust, shuffle Beta into draw
        Self::ins_flags(&mut cards, "Alpha", "Alpha", CardType::Skill,
            CardTarget::None, 1, -1, -1, -1, true, false, false,
            None, &["shuffle_beta_into_draw"]);
        Self::ins_flags(&mut cards, "Alpha+", "Alpha+", CardType::Skill,
            CardTarget::None, 1, -1, -1, -1, true, false, false,
            None, &["shuffle_beta_into_draw"]);

        // Blasphemy: enter Divinity, die next turn, exhaust (upgraded: retain)
        Self::ins_flags(&mut cards, "Blasphemy", "Blasphemy", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, -1, -1, true, false, false,
            Some("Divinity"), &["die_next_turn"]);
        Self::ins_flags(&mut cards, "Blasphemy+", "Blasphemy+", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, -1, -1, true, true, false,
            Some("Divinity"), &["die_next_turn"]);

        // Conjure Blade: X-cost, exhaust, add Expunger to hand (Xx9 damage)
        Self::ins_flags(&mut cards, "ConjureBlade", "Conjure Blade", CardType::Skill,
            CardTarget::SelfTarget, -1, -1, -1, -1, true, false, false,
            None, &["add_expunger_to_hand"]);
        Self::ins_flags(&mut cards, "ConjureBlade+", "Conjure Blade+", CardType::Skill,
            CardTarget::SelfTarget, -1, -1, -1, -1, true, false, false,
            None, &["add_expunger_to_hand"]);

        // Foreign Influence: choose attack from any class, exhaust
        Self::ins_flags(&mut cards, "ForeignInfluence", "Foreign Influence", CardType::Skill,
            CardTarget::None, 0, -1, -1, -1, true, false, false,
            None, &["choose_attack_any_class"]);
        Self::ins_flags(&mut cards, "ForeignInfluence+", "Foreign Influence+", CardType::Skill,
            CardTarget::None, 0, -1, -1, -1, true, false, false,
            None, &["choose_attack_any_class"]);

        // Omniscience: play card from draw twice, exhaust
        Self::ins_flags(&mut cards, "Omniscience", "Omniscience", CardType::Skill,
            CardTarget::None, 4, -1, -1, -1, true, false, false,
            None, &["play_card_from_draw_twice"]);
        Self::ins_flags(&mut cards, "Omniscience+", "Omniscience+", CardType::Skill,
            CardTarget::None, 3, -1, -1, -1, true, false, false,
            None, &["play_card_from_draw_twice"]);

        // Scrawl: draw until hand is full, exhaust
        Self::ins_flags(&mut cards, "Scrawl", "Scrawl", CardType::Skill,
            CardTarget::None, 1, -1, -1, -1, true, false, false,
            None, &["draw_until_full"]);
        Self::ins_flags(&mut cards, "Scrawl+", "Scrawl+", CardType::Skill,
            CardTarget::None, 0, -1, -1, -1, true, false, false,
            None, &["draw_until_full"]);

        // Spirit Shield: gain magic block per card in hand
        Self::ins(&mut cards, "SpiritShield", "Spirit Shield", CardType::Skill,
            CardTarget::SelfTarget, 2, -1, -1, 3, &["block_per_card_in_hand"]);
        Self::ins(&mut cards, "SpiritShield+", "Spirit Shield+", CardType::Skill,
            CardTarget::SelfTarget, 2, -1, -1, 4, &["block_per_card_in_hand"]);

        // Vault: take extra turn, exhaust
        Self::ins_flags(&mut cards, "Vault", "Vault", CardType::Skill,
            CardTarget::None, 3, -1, -1, -1, true, false, false,
            None, &["take_extra_turn"]);
        Self::ins_flags(&mut cards, "Vault+", "Vault+", CardType::Skill,
            CardTarget::None, 2, -1, -1, -1, true, false, false,
            None, &["take_extra_turn"]);

        // Wish: choose Strength/Plated Armor/Gold, exhaust
        Self::ins_flags(&mut cards, "Wish", "Wish", CardType::Skill,
            CardTarget::None, 3, 3, 6, 25, true, false, false,
            None, &["choose_plated_or_strength_or_gold"]);
        Self::ins_flags(&mut cards, "Wish+", "Wish+", CardType::Skill,
            CardTarget::None, 3, 4, 8, 30, true, false, false,
            None, &["choose_plated_or_strength_or_gold"]);

        // ==================================================================
        // RARE POWERS
        // ==================================================================

        // Deva Form: ethereal, gain +1 energy each turn (stacking)
        Self::ins(&mut cards, "DevaForm", "Deva Form", CardType::Power,
            CardTarget::SelfTarget, 3, -1, -1, 1, &["gain_energy_stacking"]);
        Self::ins(&mut cards, "DevaForm+", "Deva Form+", CardType::Power,
            CardTarget::SelfTarget, 3, -1, -1, 1, &["gain_energy_stacking"]);

        // Devotion: gain mantra each turn
        Self::ins(&mut cards, "Devotion", "Devotion", CardType::Power,
            CardTarget::None, 1, -1, -1, 2, &["gain_mantra_each_turn"]);
        Self::ins(&mut cards, "Devotion+", "Devotion+", CardType::Power,
            CardTarget::None, 1, -1, -1, 3, &["gain_mantra_each_turn"]);

        // Fasting (Java ID: Fasting2): gain Strength and Dexterity = magic
        Self::ins(&mut cards, "Fasting2", "Fasting", CardType::Power,
            CardTarget::SelfTarget, 2, -1, -1, 3, &["gain_str_dex"]);
        Self::ins(&mut cards, "Fasting2+", "Fasting+", CardType::Power,
            CardTarget::SelfTarget, 2, -1, -1, 4, &["gain_str_dex"]);

        // Master Reality: created cards are upgraded
        Self::ins(&mut cards, "MasterReality", "Master Reality", CardType::Power,
            CardTarget::SelfTarget, 1, -1, -1, -1, &["created_cards_upgraded"]);
        Self::ins(&mut cards, "MasterReality+", "Master Reality+", CardType::Power,
            CardTarget::SelfTarget, 0, -1, -1, -1, &["created_cards_upgraded"]);

        // ==================================================================
        // SPECIAL CARDS (generated during combat)
        // ==================================================================

        // Insight: retain, exhaust, draw 2 (upgraded: draw 3)
        Self::ins_flags(&mut cards, "Insight", "Insight", CardType::Skill,
            CardTarget::SelfTarget, 0, -1, -1, 2, true, true, false,
            None, &["draw_cards"]);
        Self::ins_flags(&mut cards, "Insight+", "Insight+", CardType::Skill,
            CardTarget::SelfTarget, 0, -1, -1, 3, true, true, false,
            None, &["draw_cards"]);

        // Smite: retain, exhaust, deal damage
        Self::ins_flags(&mut cards, "Smite", "Smite", CardType::Attack,
            CardTarget::Enemy, 1, 12, -1, -1, true, true, false,
            None, &[]);
        Self::ins_flags(&mut cards, "Smite+", "Smite+", CardType::Attack,
            CardTarget::Enemy, 1, 16, -1, -1, true, true, false,
            None, &[]);

        // Safety: retain, exhaust, gain block
        Self::ins_flags(&mut cards, "Safety", "Safety", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 12, -1, true, true, false,
            None, &[]);
        Self::ins_flags(&mut cards, "Safety+", "Safety+", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 16, -1, true, true, false,
            None, &[]);

        // Through Violence: retain, exhaust, deal damage
        Self::ins_flags(&mut cards, "ThroughViolence", "Through Violence", CardType::Attack,
            CardTarget::Enemy, 0, 20, -1, -1, true, true, false,
            None, &[]);
        Self::ins_flags(&mut cards, "ThroughViolence+", "Through Violence+", CardType::Attack,
            CardTarget::Enemy, 0, 30, -1, -1, true, true, false,
            None, &[]);

        // Expunger: hits X times (magic tracks X from Conjure Blade)
        Self::ins(&mut cards, "Expunger", "Expunger", CardType::Attack,
            CardTarget::Enemy, 1, 9, -1, 1, &["multi_hit"]);
        Self::ins(&mut cards, "Expunger+", "Expunger+", CardType::Attack,
            CardTarget::Enemy, 1, 9, -1, 1, &["multi_hit"]);

        // Beta: exhaust, shuffle Omega into draw
        Self::ins_flags(&mut cards, "Beta", "Beta", CardType::Skill,
            CardTarget::SelfTarget, 2, -1, -1, -1, true, false, false,
            None, &["shuffle_omega_into_draw"]);
        Self::ins_flags(&mut cards, "Beta+", "Beta+", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, -1, -1, true, false, false,
            None, &["shuffle_omega_into_draw"]);

        // Omega: power, deal 50 damage to all enemies at end of turn
        Self::ins(&mut cards, "Omega", "Omega", CardType::Power,
            CardTarget::SelfTarget, 3, -1, -1, 50, &["deal_damage_end_turn"]);
        Self::ins(&mut cards, "Omega+", "Omega+", CardType::Power,
            CardTarget::SelfTarget, 3, -1, -1, 50, &["deal_damage_end_turn"]);

        // ==================================================================
        // UNIVERSAL STATUS/CURSE CARDS
        // ==================================================================

        Self::ins_flags(&mut cards, "Slimed", "Slimed", CardType::Status,
            CardTarget::None, 1, -1, -1, -1, true, false, false,
            None, &[]);
        Self::ins(&mut cards, "Wound", "Wound", CardType::Status,
            CardTarget::None, -2, -1, -1, -1, &["unplayable"]);
        Self::ins(&mut cards, "Daze", "Daze", CardType::Status,
            CardTarget::None, -2, -1, -1, -1, &["unplayable", "ethereal"]);
        Self::ins(&mut cards, "Burn", "Burn", CardType::Status,
            CardTarget::None, -2, -1, -1, -1, &["unplayable"]);
        Self::ins(&mut cards, "VoidCard", "Void", CardType::Status,
            CardTarget::None, -2, -1, -1, -1, &["unplayable", "ethereal"]);
        Self::ins(&mut cards, "AscendersBane", "Ascender's Bane", CardType::Curse,
            CardTarget::None, -2, -1, -1, -1, &["unplayable", "ethereal"]);

        // ---- Colorless basics (Strike/Defend aliases for other characters) ----
        Self::ins(&mut cards, "Strike_R", "Strike", CardType::Attack,
            CardTarget::Enemy, 1, 6, -1, -1, &[]);
        Self::ins(&mut cards, "Strike_R+", "Strike+", CardType::Attack,
            CardTarget::Enemy, 1, 9, -1, -1, &[]);
        Self::ins(&mut cards, "Defend_R", "Defend", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 5, -1, &[]);
        Self::ins(&mut cards, "Defend_R+", "Defend+", CardType::Skill,
            CardTarget::SelfTarget, 1, -1, 8, -1, &[]);

        // Also register the old short IDs for backwards compatibility
        // (the engine previously used "Flurry" for FlurryOfBlows)
        Self::ins(&mut cards, "Flurry", "Flurry of Blows", CardType::Attack,
            CardTarget::Enemy, 0, 4, -1, -1, &[]);
        Self::ins(&mut cards, "Flurry+", "Flurry of Blows+", CardType::Attack,
            CardTarget::Enemy, 0, 6, -1, -1, &[]);

        CardRegistry { cards }
    }

    /// Insert a basic card with default flags (no exhaust, no retain, no shuffle).
    fn ins(
        map: &mut HashMap<&'static str, CardDef>,
        id: &'static str, name: &'static str,
        card_type: CardType, target: CardTarget,
        cost: i32, dmg: i32, blk: i32, magic: i32,
        effects: &'static [&'static str],
    ) {
        map.insert(id, CardDef {
            id, name, card_type, target, cost,
            base_damage: dmg, base_block: blk, base_magic: magic,
            exhaust: false, retain: false, shuffle_back: false,
            enter_stance: None, effects,
        });
    }

    /// Insert a card that enters a stance.
    fn ins_stance(
        map: &mut HashMap<&'static str, CardDef>,
        id: &'static str, name: &'static str,
        card_type: CardType, target: CardTarget,
        cost: i32, dmg: i32, blk: i32, magic: i32,
        stance: &'static str, effects: &'static [&'static str],
    ) {
        map.insert(id, CardDef {
            id, name, card_type, target, cost,
            base_damage: dmg, base_block: blk, base_magic: magic,
            exhaust: false, retain: false, shuffle_back: false,
            enter_stance: Some(stance), effects,
        });
    }

    /// Insert a card with all flags specified.
    fn ins_flags(
        map: &mut HashMap<&'static str, CardDef>,
        id: &'static str, name: &'static str,
        card_type: CardType, target: CardTarget,
        cost: i32, dmg: i32, blk: i32, magic: i32,
        exhaust: bool, retain: bool, shuffle_back: bool,
        enter_stance: Option<&'static str>,
        effects: &'static [&'static str],
    ) {
        map.insert(id, CardDef {
            id, name, card_type, target, cost,
            base_damage: dmg, base_block: blk, base_magic: magic,
            exhaust, retain, shuffle_back, enter_stance, effects,
        });
    }

    /// Look up a card by ID. Falls back to a default attack if not found.
    pub fn get(&self, card_id: &str) -> Option<&CardDef> {
        self.cards.get(card_id)
    }

    /// Get card or a sensible default for unknown cards.
    pub fn get_or_default(&self, card_id: &str) -> CardDef {
        if let Some(card) = self.cards.get(card_id) {
            card.clone()
        } else {
            // Unknown card: default to 1-cost attack targeting enemy, 6 damage
            CardDef {
                id: "Unknown",
                name: "Unknown",
                card_type: CardType::Attack,
                target: CardTarget::Enemy,
                cost: 1,
                base_damage: 6,
                base_block: -1,
                base_magic: -1,
                exhaust: false,
                retain: false,
                shuffle_back: false,
                enter_stance: None,
                effects: &[],
            }
        }
    }

    /// Get total number of registered cards.
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Check if a card ID is a known upgrade ("+" suffix).
    pub fn is_upgraded(card_id: &str) -> bool {
        card_id.ends_with('+')
    }

    /// Get the base ID (strip "+" suffix).
    pub fn base_id(card_id: &str) -> &str {
        card_id.trim_end_matches('+')
    }
}

impl Default for CardRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_lookup() {
        let reg = CardRegistry::new();
        let strike = reg.get("Strike_P").unwrap();
        assert_eq!(strike.base_damage, 6);
        assert_eq!(strike.cost, 1);
        assert_eq!(strike.card_type, CardType::Attack);
    }

    #[test]
    fn test_upgraded_lookup() {
        let reg = CardRegistry::new();
        let strike_plus = reg.get("Strike_P+").unwrap();
        assert_eq!(strike_plus.base_damage, 9);
    }

    #[test]
    fn test_eruption_stance() {
        let reg = CardRegistry::new();
        let eruption = reg.get("Eruption").unwrap();
        assert_eq!(eruption.enter_stance, Some("Wrath"));
        assert_eq!(eruption.cost, 2);

        let eruption_plus = reg.get("Eruption+").unwrap();
        assert_eq!(eruption_plus.cost, 1); // Upgrade reduces cost
    }

    #[test]
    fn test_unknown_card_default() {
        let reg = CardRegistry::new();
        let unknown = reg.get_or_default("SomeWeirdCard");
        assert_eq!(unknown.cost, 1);
        assert_eq!(unknown.card_type, CardType::Attack);
    }

    #[test]
    fn test_is_upgraded() {
        assert!(CardRegistry::is_upgraded("Strike_P+"));
        assert!(!CardRegistry::is_upgraded("Strike_P"));
    }

    // ---- New card coverage tests ----

    #[test]
    fn test_all_watcher_cards_registered() {
        let reg = CardRegistry::new();
        let watcher_ids = vec![
            // Basic
            "Strike_P", "Strike_P+", "Defend_P", "Defend_P+",
            "Eruption", "Eruption+", "Vigilance", "Vigilance+",
            "Miracle", "Miracle+",
            // Common Attacks
            "BowlingBash", "BowlingBash+", "Consecrate", "Consecrate+",
            "CrushJoints", "CrushJoints+", "CutThroughFate", "CutThroughFate+",
            "EmptyFist", "EmptyFist+", "FlurryOfBlows", "FlurryOfBlows+",
            "FlyingSleeves", "FlyingSleeves+", "FollowUp", "FollowUp+",
            "SashWhip", "SashWhip+", "JustLucky", "JustLucky+",
            // Common Skills
            "EmptyBody", "EmptyBody+", "Evaluate", "Evaluate+",
            "Halt", "Halt+", "Prostrate", "Prostrate+",
            "Protect", "Protect+", "ThirdEye", "ThirdEye+",
            "Crescendo", "Crescendo+", "ClearTheMind", "ClearTheMind+",
            "PathToVictory", "PathToVictory+",
            // Uncommon Attacks
            "Tantrum", "Tantrum+", "FearNoEvil", "FearNoEvil+",
            "ReachHeaven", "ReachHeaven+", "SandsOfTime", "SandsOfTime+",
            "SignatureMove", "SignatureMove+", "TalkToTheHand", "TalkToTheHand+",
            "Wallop", "Wallop+", "Weave", "Weave+",
            "WheelKick", "WheelKick+", "WindmillStrike", "WindmillStrike+",
            "Conclude", "Conclude+", "CarveReality", "CarveReality+",
            // Uncommon Skills
            "EmptyMind", "EmptyMind+", "InnerPeace", "InnerPeace+",
            "Collect", "Collect+", "DeceiveReality", "DeceiveReality+",
            "Indignation", "Indignation+", "Meditate", "Meditate+",
            "Perseverance", "Perseverance+", "Pray", "Pray+",
            "Sanctity", "Sanctity+", "Swivel", "Swivel+",
            "Vengeance", "Vengeance+", "WaveOfTheHand", "WaveOfTheHand+",
            "Worship", "Worship+", "WreathOfFlame", "WreathOfFlame+",
            // Uncommon Powers
            "BattleHymn", "BattleHymn+", "Wireheading", "Wireheading+",
            "LikeWater", "LikeWater+", "MentalFortress", "MentalFortress+",
            "Nirvana", "Nirvana+", "Adaptation", "Adaptation+",
            "Study", "Study+", "Establishment", "Establishment+",
            // Rare Attacks
            "Brilliance", "Brilliance+", "LessonLearned", "LessonLearned+",
            "Ragnarok", "Ragnarok+",
            // Rare Skills
            "Judgement", "Judgement+", "DeusExMachina", "DeusExMachina+",
            "Alpha", "Alpha+", "Blasphemy", "Blasphemy+",
            "ConjureBlade", "ConjureBlade+", "ForeignInfluence", "ForeignInfluence+",
            "Omniscience", "Omniscience+", "Scrawl", "Scrawl+",
            "SpiritShield", "SpiritShield+", "Vault", "Vault+",
            "Wish", "Wish+",
            // Rare Powers
            "DevaForm", "DevaForm+", "Devotion", "Devotion+",
            "Fasting2", "Fasting2+", "MasterReality", "MasterReality+",
            // Special
            "Insight", "Insight+", "Smite", "Smite+",
            "Safety", "Safety+", "ThroughViolence", "ThroughViolence+",
            "Expunger", "Expunger+", "Beta", "Beta+",
            "Omega", "Omega+",
        ];

        for id in &watcher_ids {
            assert!(reg.get(id).is_some(), "Card '{}' not found in registry", id);
        }
    }

    #[test]
    fn test_card_flags() {
        let reg = CardRegistry::new();

        // Crescendo: retain + exhaust
        let cresc = reg.get("Crescendo").unwrap();
        assert!(cresc.retain);
        assert!(cresc.exhaust);
        assert_eq!(cresc.enter_stance, Some("Wrath"));

        // Tantrum: shuffle_back
        let tant = reg.get("Tantrum").unwrap();
        assert!(tant.shuffle_back);
        assert!(!tant.exhaust);
        assert_eq!(tant.enter_stance, Some("Wrath"));

        // Protect: retain
        let prot = reg.get("Protect").unwrap();
        assert!(prot.retain);
        assert!(!prot.exhaust);
    }

    #[test]
    fn test_card_stat_values() {
        let reg = CardRegistry::new();

        // Wallop: 2 cost, 9 damage
        let wallop = reg.get("Wallop").unwrap();
        assert_eq!(wallop.cost, 2);
        assert_eq!(wallop.base_damage, 9);

        // Ragnarok: 3 cost, 5 damage, 5 hits
        let rag = reg.get("Ragnarok").unwrap();
        assert_eq!(rag.cost, 3);
        assert_eq!(rag.base_damage, 5);
        assert_eq!(rag.base_magic, 5);

        // Judgement: magic = 30 threshold
        let j = reg.get("Judgement").unwrap();
        assert_eq!(j.base_magic, 30);
        let jp = reg.get("Judgement+").unwrap();
        assert_eq!(jp.base_magic, 40);
    }

    #[test]
    fn test_registry_size() {
        let reg = CardRegistry::new();
        // Should have 130+ cards (all watcher base+upgraded + status/curse + colorless)
        assert!(reg.len() >= 130, "Registry has {} cards, expected 130+", reg.len());
    }
}
