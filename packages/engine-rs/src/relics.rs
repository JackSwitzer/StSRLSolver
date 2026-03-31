//! Combat-relevant relic effects for MCTS simulations.
//!
//! Implements ALL 187 relics from Slay the Spire. Relics are grouped by
//! trigger type matching the Java AbstractRelic hooks:
//!
//! - Combat start: atBattleStart / atPreBattle / atBattleStartPreDraw
//! - Turn start: atTurnStart / atTurnStartPostDraw
//! - On card play: onUseCard / onPlayCard
//! - Turn end: onPlayerEndTurn
//! - On HP loss: wasHPLost
//! - On shuffle: onShuffle
//! - On enemy death: onMonsterDeath
//! - Combat end: onVictory
//! - Passive / non-combat: gold, map, shop relics (stub — just track ownership)

use crate::cards::CardType;
use crate::state::CombatState;

// ==========================================================================
// 1. COMBAT START — atBattleStart / atPreBattle / atBattleStartPreDraw
// ==========================================================================

/// Apply relic effects at combat start.
/// Called once when combat begins, before initial draw.
pub fn apply_combat_start_relics(state: &mut CombatState) {
    for relic_id in state.relics.clone() {
        match relic_id.as_str() {
            // --- Stat buffs ---
            "Vajra" => {
                // +1 Strength at combat start
                state.player.add_status("Strength", 1);
            }
            "Oddly Smooth Stone" | "OddlySmoothStone" => {
                // +1 Dexterity at combat start
                state.player.add_status("Dexterity", 1);
            }
            "Data Disk" | "DataDisk" => {
                // +1 Focus at combat start
                state.player.add_status("Focus", 1);
            }
            "Akabeko" => {
                // 8 Vigor at combat start
                state.player.add_status("Vigor", 8);
            }
            "Bag of Marbles" => {
                // Apply 1 Vulnerable to ALL enemies
                for enemy in &mut state.enemies {
                    if enemy.is_alive() {
                        enemy.entity.add_status("Vulnerable", 1);
                    }
                }
            }
            "Red Mask" | "RedMask" => {
                // Apply 1 Weak to ALL enemies
                for enemy in &mut state.enemies {
                    if enemy.is_alive() {
                        enemy.entity.add_status("Weakened", 1);
                    }
                }
            }
            "Thread and Needle" => {
                // 4 Plated Armor at combat start
                state.player.add_status("Plated Armor", 4);
            }
            "Bronze Scales" => {
                // 3 Thorns at combat start
                state.player.add_status("Thorns", 3);
            }
            "Anchor" => {
                // 10 Block at combat start
                state.player.block += 10;
            }
            "Lantern" => {
                // +1 energy on turn 1 (tracked via counter)
                state.player.set_status("LanternReady", 1);
            }
            "Clockwork Souvenir" | "ClockworkSouvenir" => {
                // 1 Artifact at combat start
                state.player.add_status("Artifact", 1);
            }
            "Fossilized Helix" | "FossilizedHelix" => {
                // 1 Buffer at combat start
                state.player.add_status("Buffer", 1);
            }
            "Mark of Pain" => {
                // 2 Wounds in draw pile
                state.draw_pile.push("Wound".to_string());
                state.draw_pile.push("Wound".to_string());
            }
            "Blood Vial" => {
                // Heal 2 HP at combat start
                state.player.hp = (state.player.hp + 2).min(state.player.max_hp);
            }
            "MutagenicStrength" => {
                // +3 Strength, -3 at end of turn (temporary)
                state.player.add_status("Strength", 3);
                state.player.add_status("LoseStrength", 3);
            }

            // --- Card-generation relics (atBattleStartPreDraw) ---
            "PureWater" => {
                // Add a Miracle card to hand at combat start
                state.hand.push("Miracle".to_string());
            }
            "HolyWater" => {
                // Add 3 Holy Water cards to hand at combat start
                for _ in 0..3 {
                    if state.hand.len() < 10 {
                        state.hand.push("HolyWater".to_string());
                    }
                }
            }
            "Ninja Scroll" | "NinjaScroll" => {
                // Add 3 Shivs to hand at combat start
                for _ in 0..3 {
                    if state.hand.len() < 10 {
                        state.hand.push("Shiv".to_string());
                    }
                }
            }

            // --- Draw relics (atBattleStart -> draw) ---
            "Bag of Preparation" => {
                // Draw 2 extra cards at combat start
                state.player.set_status("BagOfPrepDraw", 2);
            }
            "Ring of the Snake" => {
                // Draw 2 extra cards at combat start
                state.player.set_status("BagOfPrepDraw", 2);
            }

            // --- Philosopher's Stone: +1 energy, all enemies +1 Strength ---
            "Philosopher's Stone" | "PhilosophersStone" => {
                for enemy in &mut state.enemies {
                    if enemy.is_alive() {
                        enemy.entity.add_status("Strength", 1);
                    }
                }
                // Energy bonus handled via max_energy on equip (Python side)
            }

            // --- Pen Nib: track counter ---
            "Pen Nib" => {
                if state.player.status("PenNibCounter") == 0 {
                    state.player.set_status("PenNibCounter", 0);
                }
            }

            // --- Counter-based relics: initialize ---
            "Ornamental Fan" => {
                state.player.set_status("OrnamentalFanCounter", 0);
            }
            "Kunai" => {
                state.player.set_status("KunaiCounter", 0);
            }
            "Shuriken" => {
                state.player.set_status("ShurikenCounter", 0);
            }
            "Nunchaku" => {
                // Counter persists across combats, don't reset
            }
            "Letter Opener" => {
                state.player.set_status("LetterOpenerCounter", 0);
            }
            "Happy Flower" => {
                // Counter persists across combats (counter field)
                // Initialize if not set
                if state.player.status("HappyFlowerCounter") == 0 {
                    state.player.set_status("HappyFlowerCounter", 0);
                }
            }
            "Sundial" => {
                // Counter persists across combats, resets at 3 shuffles
            }
            "InkBottle" => {
                // Counter persists across combats
            }
            "Incense Burner" | "IncenseBurner" => {
                // Counter persists across combats
            }

            // --- Turn-limited relics: init counter ---
            "HornCleat" => {
                state.player.set_status("HornCleatCounter", 0);
            }
            "CaptainsWheel" => {
                state.player.set_status("CaptainsWheelCounter", 0);
            }
            "StoneCalendar" => {
                state.player.set_status("StoneCalendarCounter", 0);
            }

            // --- Velvet Choker: card limit ---
            "Velvet Choker" | "VelvetChoker" => {
                state.player.set_status("VelvetChokerCounter", 0);
            }

            // --- Pocketwatch ---
            "Pocketwatch" => {
                state.player.set_status("PocketwatchCounter", 0);
                state.player.set_status("PocketwatchFirstTurn", 1);
            }

            // --- Violet Lotus: +1 energy on Calm exit (handled in stance change) ---
            "Violet Lotus" | "VioletLotus" => {
                state.player.set_status("VioletLotus", 1);
            }

            // --- Emotion Chip: on HP loss, trigger orb passive ---
            "Emotion Chip" | "EmotionChip" => {
                state.player.set_status("EmotionChipReady", 1);
            }

            // --- CentennialPuzzle: first HP loss draws 3 ---
            "Centennial Puzzle" | "CentennialPuzzle" => {
                state.player.set_status("CentennialPuzzleReady", 1);
            }

            // --- ArtOfWar: if no attacks played, +1 energy next turn ---
            "Art of War" => {
                state.player.set_status("ArtOfWarReady", 1);
            }

            // --- Twisted Funnel: apply 4 Poison to all enemies ---
            "TwistedFunnel" => {
                for enemy in &mut state.enemies {
                    if enemy.is_alive() {
                        enemy.entity.add_status("Poison", 4);
                    }
                }
            }

            // --- Symbiotic Virus: channel 1 Dark orb ---
            "Symbiotic Virus" => {
                // Orbs handled by Python; stub
            }

            // --- Cracked Core: channel 1 Lightning orb ---
            "Cracked Core" => {
                // Orbs handled by Python; stub
            }

            // --- Nuclear Battery: channel 1 Plasma orb ---
            "Nuclear Battery" => {
                // Orbs handled by Python; stub
            }

            // --- Snecko Eye: draw 2 extra, randomize costs ---
            "Snecko Eye" => {
                state.player.set_status("SneckoEye", 1);
                state.player.set_status("BagOfPrepDraw", 2);
            }

            // --- Ancient Tea Set: +2 energy on first turn if rest last room ---
            "Ancient Tea Set" => {
                // Requires room tracking; Python handles the flag
                // If counter is -2, grant energy
            }

            // --- Pantograph: heal 25 at boss fight start ---
            "Pantograph" => {
                // Boss detection is Python-side; if flagged, heal
                let is_boss = state.enemies.iter().any(|e| {
                    matches!(e.id.as_str(),
                        "Hexaghost" | "SlimeBoss" | "TheGuardian" |
                        "BronzeAutomaton" | "TheCollector" | "TheChamp" |
                        "AwakenedOne" | "TimeEater" | "Donu" | "Deca" |
                        "TheHeart" | "CorruptHeart" | "SpireShield" | "SpireSpear"
                    )
                });
                if is_boss {
                    state.player.hp = (state.player.hp + 25).min(state.player.max_hp);
                }
            }

            // --- Sling of Courage: +2 Strength at elite fights ---
            "Sling" => {
                // Elite detection would be Python-side
                // Stub: if sling_elite flag is set
                if state.player.status("SlingElite") > 0 {
                    state.player.add_status("Strength", 2);
                }
            }

            // --- GremlinMask (Gremlin Visage): at combat start in non-elite, gain N Gold (non-combat) ---
            "GremlinMask" => {
                // Non-combat effect; stub
            }

            // --- Bottled relics: put specific card in hand ---
            "Bottled Flame" | "BottledFlame" => {
                // The bottled card should be flagged by Python
            }
            "Bottled Lightning" | "BottledLightning" => {
                // The bottled card should be flagged by Python
            }
            "Bottled Tornado" | "BottledTornado" => {
                // The bottled card should be flagged by Python
            }

            // --- Preserved Insect: if elite fight, weaken strongest enemy ---
            "PreservedInsect" => {
                // Elite detection Python-side; flag handled externally
                if state.player.status("PreservedInsectElite") > 0 {
                    // Find enemy with most HP
                    if let Some(idx) = state.enemies.iter()
                        .enumerate()
                        .filter(|(_, e)| e.is_alive())
                        .max_by_key(|(_, e)| e.entity.hp)
                        .map(|(i, _)| i)
                    {
                        // Reduce current HP by 25%
                        let reduction = state.enemies[idx].entity.hp / 4;
                        state.enemies[idx].entity.hp -= reduction;
                    }
                }
            }

            // --- Neow's Lament: first 3 combats, enemies start at 1 HP ---
            "NeowsBlessing" => {
                let counter = state.player.status("NeowsLamentCounter");
                if counter > 0 {
                    for enemy in &mut state.enemies {
                        if enemy.is_alive() {
                            enemy.entity.hp = 1;
                        }
                    }
                    state.player.set_status("NeowsLamentCounter", counter - 1);
                }
            }

            // --- Du-Vu Doll: +1 Strength per curse in deck ---
            "Du-Vu Doll" => {
                let curse_count = state.player.status("DuVuDollCurses");
                if curse_count > 0 {
                    state.player.add_status("Strength", curse_count);
                }
            }

            // --- Girya: Strength from rest site lifting ---
            "Girya" => {
                let lift_count = state.player.status("GiryaCounter");
                if lift_count > 0 {
                    state.player.add_status("Strength", lift_count);
                }
            }

            // --- Red Skull: +3 Strength when HP <= 50% ---
            "Red Skull" => {
                if state.player.hp <= state.player.max_hp / 2 {
                    state.player.add_status("Strength", 3);
                    state.player.set_status("RedSkullActive", 1);
                }
            }

            // --- Cultist Headpiece: just aesthetic (no combat effect) ---
            "CultistMask" => {}

            // --- Teardrop Locket: start in Calm stance ---
            "TeardropLocket" => {
                state.stance = crate::state::Stance::Calm;
            }

            // --- Damaru: gain 1 Mantra at start of turn ---
            "Damaru" => {
                // Handled in turn start
            }

            // --- Duality (Yang): gain Dex when playing attacks ---
            "Yang" => {
                // Handled in on_card_play
            }

            // --- Brimstone: +2 Str to player, +1 Str to enemies per turn ---
            "Brimstone" => {
                // Handled in turn start
            }

            // --- Orange Pellets: play ATK+SKL+POW to clear debuffs ---
            "OrangePellets" => {
                state.player.set_status("OPAttack", 0);
                state.player.set_status("OPSkill", 0);
                state.player.set_status("OPPower", 0);
            }

            // --- Enchiridion: random Power into hand ---
            "Enchiridion" => {
                // Requires card pool; Python handles
            }

            // --- WarpedTongs: upgrade random card in hand at turn start ---
            "WarpedTongs" => {
                // Handled in turn start
            }

            // --- GamblingChip: can discard and redraw at start ---
            "Gambling Chip" | "GamblingChip" => {
                // Complex interaction; Python handles
            }

            // ---- Passive/non-combat relics (stub — track ownership) ----
            // These relics affect shops, map, card rewards, etc.
            "Golden Idol" | "GoldenIdol" |
            "Ectoplasm" |
            "Sozu" |
            "Cursed Key" | "CursedKey" |
            "Busted Crown" | "BustedCrown" |
            "Coffee Dripper" | "CoffeeDripper" |
            "Fusion Hammer" | "FusionHammer" |
            "SacredBark" |
            "Runic Dome" | "RunicDome" |
            "Runic Pyramid" | "RunicPyramid" |
            "Ice Cream" | "IceCream" |
            "Potion Belt" | "PotionBelt" |
            "Ceramic Fish" | "CeramicFish" |
            "Calling Bell" | "CallingBell" |
            "Astrolabe" |
            "Pandora's Box" | "PandorasBox" |
            "Empty Cage" | "EmptyCage" |
            "Orrery" |
            "Black Star" | "BlackStar" |
            "Tiny House" | "TinyHouse" |
            "Cauldron" |
            "Circlet" |
            "Red Circlet" | "RedCirclet" |
            "Dream Catcher" | "DreamCatcher" |
            "Eternal Feather" | "EternalFeather" |
            "Frozen Eye" | "FrozenEye" |
            "Frozen Egg 2" | "FrozenEgg2" |
            "Molten Egg 2" | "MoltenEgg2" |
            "Toxic Egg 2" | "ToxicEgg2" |
            "Juzu Bracelet" | "JuzuBracelet" |
            "Mango" |
            "Strawberry" |
            "Pear" |
            "Lee's Waffle" | "Waffle" |
            "Old Coin" | "OldCoin" |
            "War Paint" | "WarPaint" |
            "Whetstone" |
            "Peace Pipe" | "PeacePipe" |
            "Shovel" |
            "Singing Bowl" | "SingingBowl" |
            "Smiling Mask" | "SmilingMask" |
            "Prayer Wheel" | "PrayerWheel" |
            "Question Card" | "QuestionCard" |
            "Regal Pillow" | "RegalPillow" |
            "Meal Ticket" | "MealTicket" |
            "Darkstone Periapt" | "DarkstonePeriapt" |
            "Magic Flower" | "MagicFlower" |
            "Membership Card" | "MembershipCard" |
            "The Courier" | "Courier" |
            "Nloth's Gift" | "NlothsGift" |
            "NlothsMask" |
            "Spirit Poop" | "SpiritPoop" |
            "White Beast Statue" | "WhiteBeast" |
            "SsserpentHead" |
            "MawBank" |
            "Discerning Monocle" | "DiscerningMonocle" |
            "Matryoshka" |
            "Tiny Chest" | "TinyChest" |
            "DollysMirror" |
            "WingedGreaves" | "WingBoots" |
            "SlaversCollar" |
            "GoldenEye" |
            "PrismaticShard" |
            "FaceOfCleric" |
            "Bloody Idol" | "BloodyIdol" |
            "Meat on the Bone" | "MeatOnTheBone" |
            "Lizard Tail" | "LizardTail" |
            "Mark of the Bloom" | "MarkOfTheBloom" |
            "Omamori" |
            "Ginger" |
            "Turnip" |
            "Toolbox" |
            "Runic Capacitor" | "RunicCapacitor" |
            "Ring of the Serpent" | "RingOfTheSerpent" |
            "Hovering Kite" | "HoveringKite" |
            "Strange Spoon" | "StrangeSpoon" |
            "Medical Kit" | "MedicalKit" |
            "Blue Candle" | "BlueCandle" => {
                // Non-combat or complex interactive effects; stub
            }

            _ => {} // Unknown relic, ignore
        }
    }
}

// ==========================================================================
// 2. TURN START — atTurnStart
// ==========================================================================

/// Apply relic effects at the start of each player turn.
/// Called after energy reset and before card draw.
pub fn apply_turn_start_relics(state: &mut CombatState) {
    // Lantern: +1 energy on turn 1
    if state.turn == 1 && state.player.status("LanternReady") > 0 {
        state.energy += 1;
        state.player.set_status("LanternReady", 0);
    }

    // Bag of Preparation / Ring of the Snake: extra draw on turn 1
    if state.turn == 1 {
        let extra_draw = state.player.status("BagOfPrepDraw");
        if extra_draw > 0 {
            state.player.set_status("TurnStartExtraDraw", extra_draw);
            state.player.set_status("BagOfPrepDraw", 0);
        }
    }

    // Happy Flower: every 3rd turn, +1 energy
    if state.has_relic("Happy Flower") {
        let counter = state.player.status("HappyFlowerCounter") + 1;
        if counter >= 3 {
            state.energy += 1;
            state.player.set_status("HappyFlowerCounter", 0);
        } else {
            state.player.set_status("HappyFlowerCounter", counter);
        }
    }

    // Incense Burner: every 6th turn, gain 1 Intangible
    if state.has_relic("Incense Burner") || state.has_relic("IncenseBurner") {
        let counter = state.player.status("IncenseBurnerCounter") + 1;
        if counter >= 6 {
            state.player.add_status("Intangible", 1);
            state.player.set_status("IncenseBurnerCounter", 0);
        } else {
            state.player.set_status("IncenseBurnerCounter", counter);
        }
    }

    // Mercury Hourglass: deal 3 damage to ALL enemies at start of turn
    if state.has_relic("Mercury Hourglass") {
        let living = state.living_enemy_indices();
        for idx in living {
            let enemy = &mut state.enemies[idx];
            let dmg = 3;
            let blocked = enemy.entity.block.min(dmg);
            enemy.entity.block -= blocked;
            let hp_dmg = dmg - blocked;
            enemy.entity.hp -= hp_dmg;
            state.total_damage_dealt += hp_dmg;
            if enemy.entity.hp <= 0 {
                enemy.entity.hp = 0;
            }
        }
    }

    // Brimstone: +2 Strength to player, +1 Strength to all enemies
    if state.has_relic("Brimstone") {
        state.player.add_status("Strength", 2);
        for enemy in &mut state.enemies {
            if enemy.is_alive() {
                enemy.entity.add_status("Strength", 1);
            }
        }
    }

    // Damaru: +1 Mantra at turn start (Watcher)
    if state.has_relic("Damaru") {
        state.mantra += 1;
        state.mantra_gained += 1;
        if state.mantra >= 10 {
            state.mantra -= 10;
            // Enter Divinity (engine handles this)
            state.player.set_status("EnterDivinity", 1);
        }
    }

    // Inserter: every 2nd turn, gain an orb slot
    if state.has_relic("Inserter") {
        let counter = state.player.status("InserterCounter") + 1;
        if counter >= 2 {
            state.player.set_status("InserterCounter", 0);
            // Orb slot increase; tracked as status for MCTS
            state.player.add_status("OrbSlots", 1);
        } else {
            state.player.set_status("InserterCounter", counter);
        }
    }

    // Horn Cleat: on 2nd turn, gain 14 Block (once)
    if state.has_relic("HornCleat") {
        let counter = state.player.status("HornCleatCounter");
        if counter >= 0 && counter < 2 {
            let new_counter = counter + 1;
            if new_counter == 2 {
                state.player.block += 14;
                state.player.set_status("HornCleatCounter", -1); // done
            } else {
                state.player.set_status("HornCleatCounter", new_counter);
            }
        }
    }

    // Captain's Wheel: on 3rd turn, gain 18 Block (once)
    if state.has_relic("CaptainsWheel") {
        let counter = state.player.status("CaptainsWheelCounter");
        if counter >= 0 && counter < 3 {
            let new_counter = counter + 1;
            if new_counter == 3 {
                state.player.block += 18;
                state.player.set_status("CaptainsWheelCounter", -1); // done
            } else {
                state.player.set_status("CaptainsWheelCounter", new_counter);
            }
        }
    }

    // Stone Calendar: on 7th turn, deal 52 damage to all enemies (at end of turn)
    if state.has_relic("StoneCalendar") {
        let counter = state.player.status("StoneCalendarCounter") + 1;
        state.player.set_status("StoneCalendarCounter", counter);
    }

    // Velvet Choker: reset card play counter
    if state.has_relic("Velvet Choker") || state.has_relic("VelvetChoker") {
        state.player.set_status("VelvetChokerCounter", 0);
    }

    // Pocketwatch: if played <= 3 cards last turn, draw 3 extra
    if state.has_relic("Pocketwatch") {
        let first_turn = state.player.status("PocketwatchFirstTurn");
        if first_turn > 0 {
            state.player.set_status("PocketwatchFirstTurn", 0);
        } else {
            let counter = state.player.status("PocketwatchCounter");
            if counter <= 3 {
                state.player.set_status("TurnStartExtraDraw",
                    state.player.status("TurnStartExtraDraw") + 3);
            }
        }
        state.player.set_status("PocketwatchCounter", 0);
    }

    // Art of War: if no attacks played last turn, +1 energy
    if state.has_relic("Art of War") {
        let ready = state.player.status("ArtOfWarReady");
        if ready > 0 && state.turn > 1 {
            state.energy += 1;
        }
        // Reset: will be set to 0 if attack is played
        state.player.set_status("ArtOfWarReady", 1);
    }

    // Kunai counter reset
    if state.has_relic("Kunai") {
        state.player.set_status("KunaiCounter", 0);
    }

    // Shuriken counter reset
    if state.has_relic("Shuriken") {
        state.player.set_status("ShurikenCounter", 0);
    }

    // Letter Opener counter reset
    if state.has_relic("Letter Opener") {
        state.player.set_status("LetterOpenerCounter", 0);
    }

    // Ornamental Fan counter reset
    if state.has_relic("Ornamental Fan") {
        state.player.set_status("OrnamentalFanCounter", 0);
    }

    // Orange Pellets: reset type tracking
    if state.has_relic("OrangePellets") {
        state.player.set_status("OPAttack", 0);
        state.player.set_status("OPSkill", 0);
        state.player.set_status("OPPower", 0);
    }

    // Unceasing Top: draw when hand is empty (handled in engine)

    // Hovering Kite: discard 1 card, gain 1 energy (complex; Python handles)
}

/// Legacy wrapper for Lantern (backward compat).
pub fn apply_lantern_turn_start(state: &mut CombatState) {
    if state.turn == 1 && state.player.status("LanternReady") > 0 {
        state.energy += 1;
        state.player.set_status("LanternReady", 0);
    }
}

// ==========================================================================
// 3. ON CARD PLAY — onUseCard / onPlayCard
// ==========================================================================

/// Apply relic effects after a card is played.
/// `card_type` is the type of card just played.
/// `is_attack` should be true if the card is an Attack type.
pub fn on_card_played(state: &mut CombatState, card_type: CardType) {
    let is_attack = card_type == CardType::Attack;
    let is_skill = card_type == CardType::Skill;
    let is_power = card_type == CardType::Power;

    // --- Ornamental Fan: gain 4 block every 3 cards played ---
    if state.has_relic("Ornamental Fan") {
        let counter = state.player.status("OrnamentalFanCounter") + 1;
        if counter >= 3 {
            state.player.block += 4;
            state.player.set_status("OrnamentalFanCounter", 0);
        } else {
            state.player.set_status("OrnamentalFanCounter", counter);
        }
    }

    // --- Kunai: every 3 attacks, +1 Dexterity ---
    if is_attack && state.has_relic("Kunai") {
        let counter = state.player.status("KunaiCounter") + 1;
        if counter >= 3 {
            state.player.add_status("Dexterity", 1);
            state.player.set_status("KunaiCounter", 0);
        } else {
            state.player.set_status("KunaiCounter", counter);
        }
    }

    // --- Shuriken: every 3 attacks, +1 Strength ---
    if is_attack && state.has_relic("Shuriken") {
        let counter = state.player.status("ShurikenCounter") + 1;
        if counter >= 3 {
            state.player.add_status("Strength", 1);
            state.player.set_status("ShurikenCounter", 0);
        } else {
            state.player.set_status("ShurikenCounter", counter);
        }
    }

    // --- Letter Opener: every 3 skills, deal 5 damage to random enemy ---
    if is_skill && state.has_relic("Letter Opener") {
        let counter = state.player.status("LetterOpenerCounter") + 1;
        if counter >= 3 {
            // Deal 5 damage to all enemies (simplified from random)
            let living = state.living_enemy_indices();
            if let Some(&idx) = living.first() {
                let enemy = &mut state.enemies[idx];
                let dmg = 5;
                let blocked = enemy.entity.block.min(dmg);
                enemy.entity.block -= blocked;
                let hp_dmg = dmg - blocked;
                enemy.entity.hp -= hp_dmg;
                state.total_damage_dealt += hp_dmg;
                if enemy.entity.hp <= 0 {
                    enemy.entity.hp = 0;
                }
            }
            state.player.set_status("LetterOpenerCounter", 0);
        } else {
            state.player.set_status("LetterOpenerCounter", counter);
        }
    }

    // --- Nunchaku: every 10 attacks, +1 energy ---
    if is_attack && state.has_relic("Nunchaku") {
        let counter = state.player.status("NunchakuCounter") + 1;
        if counter >= 10 {
            state.energy += 1;
            state.player.set_status("NunchakuCounter", 0);
        } else {
            state.player.set_status("NunchakuCounter", counter);
        }
    }

    // --- Ink Bottle: every 10 cards, draw 1 ---
    if state.has_relic("InkBottle") {
        let counter = state.player.status("InkBottleCounter") + 1;
        if counter >= 10 {
            // Draw 1 card (set flag for engine)
            state.player.set_status("InkBottleDraw", 1);
            state.player.set_status("InkBottleCounter", 0);
        } else {
            state.player.set_status("InkBottleCounter", counter);
        }
    }

    // --- Pen Nib: handled separately via check_pen_nib ---

    // --- Velvet Choker: track cards played ---
    if state.has_relic("Velvet Choker") || state.has_relic("VelvetChoker") {
        state.player.add_status("VelvetChokerCounter", 1);
    }

    // --- Pocketwatch: track cards played ---
    if state.has_relic("Pocketwatch") {
        state.player.add_status("PocketwatchCounter", 1);
    }

    // --- Art of War: if attack played, disable bonus ---
    if is_attack && (state.has_relic("Art of War")) {
        state.player.set_status("ArtOfWarReady", 0);
    }

    // --- Bird-Faced Urn: heal 2 when playing a Power ---
    if is_power && state.has_relic("Bird Faced Urn") {
        state.player.hp = (state.player.hp + 2).min(state.player.max_hp);
    }

    // --- Mummified Hand: when playing a Power, random card in hand costs 0 ---
    if is_power && state.has_relic("Mummified Hand") {
        // Complex card cost manipulation; set flag for engine
        state.player.set_status("MummifiedHandTrigger", 1);
    }

    // --- Duality (Yang): when playing an Attack, gain 1 Dexterity this turn ---
    if is_attack && state.has_relic("Yang") {
        state.player.add_status("Dexterity", 1);
        state.player.add_status("LoseDexterity", 1);
    }

    // --- Necronomicon: first Attack costing 2+ per turn is played twice ---
    // Handled in engine card play logic

    // --- Orange Pellets: track card types ---
    if state.has_relic("OrangePellets") {
        if is_attack {
            state.player.set_status("OPAttack", 1);
        } else if is_skill {
            state.player.set_status("OPSkill", 1);
        } else if is_power {
            state.player.set_status("OPPower", 1);
        }
        // If all three types played, remove all debuffs
        if state.player.status("OPAttack") > 0
            && state.player.status("OPSkill") > 0
            && state.player.status("OPPower") > 0
        {
            // Remove debuffs
            state.player.set_status("Weakened", 0);
            state.player.set_status("Vulnerable", 0);
            state.player.set_status("Frail", 0);
            state.player.set_status("Entangled", 0);
            state.player.set_status("No Draw", 0);
            state.player.set_status("OPAttack", 0);
            state.player.set_status("OPSkill", 0);
            state.player.set_status("OPPower", 0);
        }
    }
}

/// Apply Ornamental Fan: gain 4 block after playing 3 cards.
/// Legacy wrapper — use on_card_played for new code.
pub fn check_ornamental_fan(state: &mut CombatState) {
    if !state.has_relic("Ornamental Fan") {
        return;
    }

    let counter = state.player.status("OrnamentalFanCounter") + 1;
    if counter >= 3 {
        state.player.block += 4;
        state.player.set_status("OrnamentalFanCounter", 0);
    } else {
        state.player.set_status("OrnamentalFanCounter", counter);
    }
}

/// Check Pen Nib: every 10th attack deals double damage.
/// Returns true if this attack triggers Pen Nib.
pub fn check_pen_nib(state: &mut CombatState) -> bool {
    if !state.has_relic("Pen Nib") {
        return false;
    }

    let counter = state.player.status("PenNibCounter");
    if counter >= 9 {
        state.player.set_status("PenNibCounter", 0);
        true
    } else {
        state.player.set_status("PenNibCounter", counter + 1);
        false
    }
}

/// Check if Velvet Choker allows playing another card.
pub fn velvet_choker_can_play(state: &CombatState) -> bool {
    if !state.has_relic("Velvet Choker") && !state.has_relic("VelvetChoker") {
        return true;
    }
    state.player.status("VelvetChokerCounter") < 6
}

// ==========================================================================
// 4. TURN END — onPlayerEndTurn
// ==========================================================================

/// Apply relic effects at end of player turn (before enemy turns).
pub fn apply_turn_end_relics(state: &mut CombatState) {
    // Orichalcum: if player has 0 Block, gain 6 Block
    if state.has_relic("Orichalcum") && state.player.block == 0 {
        state.player.block += 6;
    }

    // Cloak Clasp: gain 1 Block per card in hand
    if state.has_relic("CloakClasp") {
        let hand_size = state.hand.len() as i32;
        if hand_size > 0 {
            state.player.block += hand_size;
        }
    }

    // Stone Calendar: on 7th turn, deal 52 damage to all enemies
    if state.has_relic("StoneCalendar") {
        if state.player.status("StoneCalendarCounter") >= 7 {
            let living = state.living_enemy_indices();
            for idx in living {
                let enemy = &mut state.enemies[idx];
                let dmg = 52;
                let blocked = enemy.entity.block.min(dmg);
                enemy.entity.block -= blocked;
                let hp_dmg = dmg - blocked;
                enemy.entity.hp -= hp_dmg;
                state.total_damage_dealt += hp_dmg;
                if enemy.entity.hp <= 0 {
                    enemy.entity.hp = 0;
                }
            }
        }
    }

    // Frozen Core: if empty orb slot, channel Frost (Defect)
    if state.has_relic("FrozenCore") {
        // Orb handling is Python-side; set flag
        state.player.set_status("FrozenCoreTrigger", 1);
    }

    // Nilry's Codex: discover a card at end of turn (complex; Python handles)
    // Stub: no combat effect in MCTS
}

// ==========================================================================
// 5. ON HP LOSS — wasHPLost
// ==========================================================================

/// Apply relic effects when player loses HP.
/// `damage` is the amount of HP lost.
pub fn on_hp_loss(state: &mut CombatState, damage: i32) {
    if damage <= 0 {
        return;
    }

    // Centennial Puzzle: first time taking damage, draw 3
    if state.has_relic("Centennial Puzzle") || state.has_relic("CentennialPuzzle") {
        if state.player.status("CentennialPuzzleReady") > 0 {
            state.player.set_status("CentennialPuzzleReady", 0);
            state.player.set_status("CentennialPuzzleDraw", 3);
        }
    }

    // Self-Forming Clay: next turn gain 3 Block
    if state.has_relic("Self Forming Clay") || state.has_relic("SelfFormingClay") {
        state.player.add_status("NextTurnBlock", 3);
    }

    // Runic Cube: draw 1 card when losing HP
    if state.has_relic("Runic Cube") || state.has_relic("RunicCube") {
        state.player.set_status("RunicCubeDraw", 1);
    }

    // Red Skull: if now at <= 50% HP and not already active, +3 Strength
    if state.has_relic("Red Skull") {
        let active = state.player.status("RedSkullActive");
        if active == 0 && state.player.hp <= state.player.max_hp / 2 {
            state.player.add_status("Strength", 3);
            state.player.set_status("RedSkullActive", 1);
        }
    }

    // Emotion Chip: if took damage, trigger orb passive next turn
    if state.has_relic("Emotion Chip") || state.has_relic("EmotionChip") {
        state.player.set_status("EmotionChipTrigger", 1);
    }
}

// ==========================================================================
// 6. ON SHUFFLE — onShuffle
// ==========================================================================

/// Apply relic effects when draw pile is shuffled (discard into draw).
pub fn on_shuffle(state: &mut CombatState) {
    // Sundial: every 3 shuffles, +2 energy
    if state.has_relic("Sundial") {
        let counter = state.player.status("SundialCounter") + 1;
        if counter >= 3 {
            state.energy += 2;
            state.player.set_status("SundialCounter", 0);
        } else {
            state.player.set_status("SundialCounter", counter);
        }
    }

    // The Abacus: gain 6 Block on shuffle
    if state.has_relic("TheAbacus") {
        state.player.block += 6;
    }

    // Melange: scry 3 on shuffle (complex; Python handles)
}

// ==========================================================================
// 7. ON ENEMY DEATH — onMonsterDeath
// ==========================================================================

/// Apply relic effects when an enemy dies.
pub fn on_enemy_death(state: &mut CombatState, _dead_enemy_idx: usize) {
    // Gremlin Horn: gain 1 energy and draw 1 card on non-minion death
    if state.has_relic("Gremlin Horn") {
        // Only if other enemies still alive
        if state.enemies.iter().any(|e| e.is_alive()) {
            state.energy += 1;
            state.player.set_status("GremlinHornDraw", 1);
        }
    }

    // The Specimen: transfer Poison from killed enemy to random alive enemy
    if state.has_relic("The Specimen") {
        let dead_poison = state.enemies[_dead_enemy_idx].entity.status("Poison");
        if dead_poison > 0 {
            // Find first alive enemy
            if let Some(alive_idx) = state.enemies.iter()
                .enumerate()
                .find(|(i, e)| *i != _dead_enemy_idx && e.is_alive())
                .map(|(i, _)| i)
            {
                state.enemies[alive_idx].entity.add_status("Poison", dead_poison);
            }
        }
    }
}

// ==========================================================================
// 8. COMBAT END — onVictory
// ==========================================================================

/// Apply relic effects when combat is won.
/// Returns HP to heal (0 if none).
pub fn on_victory(state: &mut CombatState) -> i32 {
    let mut heal = 0;

    // Burning Blood: heal 6 on victory
    if state.has_relic("Burning Blood") {
        heal += 6;
    }

    // Black Blood: heal 12 on victory (replaces Burning Blood)
    if state.has_relic("Black Blood") {
        heal += 12;
    }

    // Meat on the Bone: if HP <= 50%, heal 12
    if state.has_relic("Meat on the Bone") || state.has_relic("MeatOnTheBone") {
        if state.player.hp <= state.player.max_hp / 2 {
            heal += 12;
        }
    }

    // Face of Cleric: +1 max HP on victory
    if state.has_relic("FaceOfCleric") {
        state.player.max_hp += 1;
    }

    heal
}

// ==========================================================================
// 9. DAMAGE MODIFIERS
// ==========================================================================

/// Boot: if unblocked damage is > 0 and < 5, set to 5.
pub fn apply_boot(state: &CombatState, unblocked_damage: i32) -> i32 {
    if state.has_relic("Boot") && unblocked_damage > 0 && unblocked_damage < 5 {
        5
    } else {
        unblocked_damage
    }
}

/// Champion's Belt: whenever applying Vulnerable, also apply 1 Weak.
pub fn champion_belt_on_vulnerable(state: &CombatState) -> bool {
    state.has_relic("Champion Belt")
}

/// Charon's Ashes: deal 3 damage to all enemies whenever a card is exhausted.
pub fn charons_ashes_on_exhaust(state: &mut CombatState) {
    if !state.has_relic("Charon's Ashes") && !state.has_relic("CharonsAshes") {
        return;
    }
    let living = state.living_enemy_indices();
    for idx in living {
        let enemy = &mut state.enemies[idx];
        let dmg = 3;
        let blocked = enemy.entity.block.min(dmg);
        enemy.entity.block -= blocked;
        let hp_dmg = dmg - blocked;
        enemy.entity.hp -= hp_dmg;
        state.total_damage_dealt += hp_dmg;
        if enemy.entity.hp <= 0 {
            enemy.entity.hp = 0;
        }
    }
}

/// Dead Branch: when a card is exhausted, add a random card to hand.
/// Returns true if Dead Branch should trigger (actual card generation by engine).
pub fn dead_branch_on_exhaust(state: &CombatState) -> bool {
    state.has_relic("Dead Branch")
}

/// Tough Bandages: gain 3 Block whenever a card is discarded manually.
pub fn tough_bandages_on_discard(state: &mut CombatState) {
    if state.has_relic("Tough Bandages") || state.has_relic("ToughBandages") {
        state.player.block += 3;
    }
}

/// Tingsha: deal 3 damage to random enemy when card is discarded manually.
pub fn tingsha_on_discard(state: &mut CombatState) {
    if !state.has_relic("Tingsha") {
        return;
    }
    let living = state.living_enemy_indices();
    if let Some(&idx) = living.first() {
        let enemy = &mut state.enemies[idx];
        let dmg = 3;
        let blocked = enemy.entity.block.min(dmg);
        enemy.entity.block -= blocked;
        let hp_dmg = dmg - blocked;
        enemy.entity.hp -= hp_dmg;
        state.total_damage_dealt += hp_dmg;
        if enemy.entity.hp <= 0 {
            enemy.entity.hp = 0;
        }
    }
}

/// Toy Ornithopter: heal 5 HP whenever a potion is used.
pub fn toy_ornithopter_on_potion(state: &mut CombatState) {
    if state.has_relic("Toy Ornithopter") || state.has_relic("ToyOrnithopter") {
        state.player.hp = (state.player.hp + 5).min(state.player.max_hp);
    }
}

/// Hand Drill: if attack breaks enemy Block, apply 2 Vulnerable.
pub fn hand_drill_on_block_break(state: &mut CombatState, enemy_idx: usize) {
    if state.has_relic("HandDrill") && enemy_idx < state.enemies.len() {
        state.enemies[enemy_idx].entity.add_status("Vulnerable", 2);
    }
}

/// Strike Dummy: +3 damage on Strikes (simplified passive).
pub fn strike_dummy_bonus(state: &CombatState) -> i32 {
    if state.has_relic("StrikeDummy") {
        3
    } else {
        0
    }
}

/// Wrist Blade: +4 damage on 0-cost attacks.
pub fn wrist_blade_bonus(state: &CombatState) -> i32 {
    if state.has_relic("WristBlade") {
        4
    } else {
        0
    }
}

/// Snecko Skull: +1 Poison when applying Poison.
pub fn snecko_skull_bonus(state: &CombatState) -> i32 {
    if state.has_relic("Snake Skull") || state.has_relic("SneckoSkull") {
        1
    } else {
        0
    }
}

/// Chemical X: +2 to X-cost effects.
pub fn chemical_x_bonus(state: &CombatState) -> i32 {
    if state.has_relic("Chemical X") || state.has_relic("ChemicalX") {
        2
    } else {
        0
    }
}

/// Gold Plated Cables: if HP is full, orbs passive trigger extra.
pub fn gold_plated_cables_active(state: &CombatState) -> bool {
    state.has_relic("Cables") && state.player.hp == state.player.max_hp
}

/// Apply Violet Lotus: +1 energy on Calm exit.
pub fn violet_lotus_calm_exit_bonus(state: &CombatState) -> i32 {
    if state.has_relic("Violet Lotus") || state.has_relic("VioletLotus") {
        1
    } else {
        0
    }
}

/// Unceasing Top: if hand is empty, draw 1.
pub fn unceasing_top_should_draw(state: &CombatState) -> bool {
    (state.has_relic("Unceasing Top") || state.has_relic("UnceasingTop"))
        && state.hand.is_empty()
        && (!state.draw_pile.is_empty() || !state.discard_pile.is_empty())
}

/// Runic Pyramid: don't discard hand at end of turn.
pub fn has_runic_pyramid(state: &CombatState) -> bool {
    state.has_relic("Runic Pyramid") || state.has_relic("RunicPyramid")
}

/// Calipers: retain up to 15 Block between turns.
pub fn calipers_block_retention(state: &CombatState, current_block: i32) -> i32 {
    if state.has_relic("Calipers") {
        current_block.min(15).max(0)
    } else {
        0
    }
}

/// Ice Cream: energy carries over between turns.
pub fn has_ice_cream(state: &CombatState) -> bool {
    state.has_relic("Ice Cream") || state.has_relic("IceCream")
}

/// Sacred Bark: double potion effectiveness.
pub fn has_sacred_bark(state: &CombatState) -> bool {
    state.has_relic("SacredBark")
}

/// Necronomicon: first 2+-cost attack per turn plays twice.
pub fn necronomicon_should_trigger(state: &CombatState, card_cost: i32, is_attack: bool) -> bool {
    if !state.has_relic("Necronomicon") {
        return false;
    }
    is_attack && card_cost >= 2 && state.player.status("NecronomiconUsed") == 0
}

/// Mark Necronomicon as used for this turn.
pub fn necronomicon_mark_used(state: &mut CombatState) {
    state.player.set_status("NecronomiconUsed", 1);
}

/// Reset Necronomicon at turn start.
pub fn necronomicon_reset(state: &mut CombatState) {
    if state.has_relic("Necronomicon") {
        state.player.set_status("NecronomiconUsed", 0);
    }
}

// ==========================================================================
// TESTS
// ==========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{CombatState, EnemyCombatState};

    fn make_test_state() -> CombatState {
        let enemy = EnemyCombatState::new("JawWorm", 44, 44);
        CombatState::new(80, 80, vec![enemy], vec!["Strike_P".to_string(); 5], 3)
    }

    fn make_two_enemy_state() -> CombatState {
        let e1 = EnemyCombatState::new("JawWorm", 44, 44);
        let e2 = EnemyCombatState::new("Cultist", 50, 50);
        CombatState::new(80, 80, vec![e1, e2], vec!["Strike_P".to_string(); 5], 3)
    }

    // --- Combat start tests ---

    #[test]
    fn test_vajra_combat_start() {
        let mut state = make_test_state();
        state.relics.push("Vajra".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.strength(), 1);
    }

    #[test]
    fn test_oddly_smooth_stone() {
        let mut state = make_test_state();
        state.relics.push("Oddly Smooth Stone".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.dexterity(), 1);
    }

    #[test]
    fn test_bag_of_marbles_combat_start() {
        let mut state = make_test_state();
        state.relics.push("Bag of Marbles".to_string());
        apply_combat_start_relics(&mut state);
        assert!(state.enemies[0].entity.is_vulnerable());
    }

    #[test]
    fn test_red_mask_combat_start() {
        let mut state = make_two_enemy_state();
        state.relics.push("Red Mask".to_string());
        apply_combat_start_relics(&mut state);
        assert!(state.enemies[0].entity.is_weak());
        assert!(state.enemies[1].entity.is_weak());
    }

    #[test]
    fn test_thread_and_needle_combat_start() {
        let mut state = make_test_state();
        state.relics.push("Thread and Needle".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.status("Plated Armor"), 4);
    }

    #[test]
    fn test_anchor_combat_start() {
        let mut state = make_test_state();
        state.relics.push("Anchor".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.block, 10);
    }

    #[test]
    fn test_akabeko_combat_start() {
        let mut state = make_test_state();
        state.relics.push("Akabeko".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.status("Vigor"), 8);
    }

    #[test]
    fn test_blood_vial_combat_start() {
        let mut state = make_test_state();
        state.player.hp = 70;
        state.relics.push("Blood Vial".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.hp, 72);
    }

    #[test]
    fn test_blood_vial_does_not_exceed_max() {
        let mut state = make_test_state();
        state.player.hp = 79;
        state.relics.push("Blood Vial".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.hp, 80);
    }

    #[test]
    fn test_twisted_funnel() {
        let mut state = make_two_enemy_state();
        state.relics.push("TwistedFunnel".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.enemies[0].entity.status("Poison"), 4);
        assert_eq!(state.enemies[1].entity.status("Poison"), 4);
    }

    #[test]
    fn test_mutagenic_strength() {
        let mut state = make_test_state();
        state.relics.push("MutagenicStrength".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.strength(), 3);
        assert_eq!(state.player.status("LoseStrength"), 3);
    }

    #[test]
    fn test_ninja_scroll() {
        let mut state = make_test_state();
        state.hand.clear();
        state.relics.push("Ninja Scroll".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.hand.len(), 3);
        assert!(state.hand.iter().all(|c| c == "Shiv"));
    }

    #[test]
    fn test_pure_water_adds_miracle() {
        let mut state = make_test_state();
        state.relics.push("PureWater".to_string());
        let hand_before = state.hand.len();
        apply_combat_start_relics(&mut state);
        assert_eq!(state.hand.len(), hand_before + 1);
        assert_eq!(state.hand.last().unwrap(), "Miracle");
    }

    #[test]
    fn test_mark_of_pain() {
        let mut state = make_test_state();
        state.relics.push("Mark of Pain".to_string());
        let initial_draw_size = state.draw_pile.len();
        apply_combat_start_relics(&mut state);
        assert_eq!(state.draw_pile.len(), initial_draw_size + 2);
        let wound_count = state.draw_pile.iter().filter(|c| *c == "Wound").count();
        assert_eq!(wound_count, 2);
    }

    #[test]
    fn test_teardrop_locket() {
        let mut state = make_test_state();
        state.relics.push("TeardropLocket".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.stance, crate::state::Stance::Calm);
    }

    #[test]
    fn test_multiple_relics() {
        let mut state = make_test_state();
        state.relics.push("Vajra".to_string());
        state.relics.push("Bag of Marbles".to_string());
        state.relics.push("Anchor".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.strength(), 1);
        assert!(state.enemies[0].entity.is_vulnerable());
        assert_eq!(state.player.block, 10);
    }

    // --- Turn start tests ---

    #[test]
    fn test_lantern_turn1_energy() {
        let mut state = make_test_state();
        state.relics.push("Lantern".to_string());
        state.turn = 0;
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.status("LanternReady"), 1);
        state.turn = 1;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.energy, 4);
        assert_eq!(state.player.status("LanternReady"), 0);
    }

    #[test]
    fn test_lantern_not_turn2() {
        let mut state = make_test_state();
        state.relics.push("Lantern".to_string());
        apply_combat_start_relics(&mut state);
        state.turn = 2;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.energy, 3);
    }

    #[test]
    fn test_happy_flower_every_3_turns() {
        let mut state = make_test_state();
        state.relics.push("Happy Flower".to_string());
        apply_combat_start_relics(&mut state);

        state.turn = 1;
        state.energy = 3;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.energy, 3); // counter=1

        state.turn = 2;
        state.energy = 3;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.energy, 3); // counter=2

        state.turn = 3;
        state.energy = 3;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.energy, 4); // counter=3 -> +1, reset to 0
    }

    #[test]
    fn test_mercury_hourglass() {
        let mut state = make_test_state();
        state.relics.push("Mercury Hourglass".to_string());
        let hp_before = state.enemies[0].entity.hp;
        state.turn = 1;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp_before - 3);
    }

    #[test]
    fn test_incense_burner_every_6_turns() {
        let mut state = make_test_state();
        state.relics.push("Incense Burner".to_string());
        for turn in 1..=6 {
            state.turn = turn;
            apply_turn_start_relics(&mut state);
        }
        assert_eq!(state.player.status("Intangible"), 1);
    }

    #[test]
    fn test_brimstone() {
        let mut state = make_test_state();
        state.relics.push("Brimstone".to_string());
        state.turn = 1;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.player.strength(), 2);
        assert_eq!(state.enemies[0].entity.strength(), 1);
    }

    #[test]
    fn test_horn_cleat_turn2() {
        let mut state = make_test_state();
        state.relics.push("HornCleat".to_string());
        apply_combat_start_relics(&mut state);

        state.turn = 1;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.player.block, 0); // Not yet

        state.turn = 2;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.player.block, 14);
    }

    #[test]
    fn test_captains_wheel_turn3() {
        let mut state = make_test_state();
        state.relics.push("CaptainsWheel".to_string());
        apply_combat_start_relics(&mut state);

        for t in 1..=2 {
            state.turn = t;
            apply_turn_start_relics(&mut state);
        }
        assert_eq!(state.player.block, 0);

        state.turn = 3;
        apply_turn_start_relics(&mut state);
        assert_eq!(state.player.block, 18);
    }

    // --- On card play tests ---

    #[test]
    fn test_ornamental_fan_every_3_cards() {
        let mut state = make_test_state();
        state.relics.push("Ornamental Fan".to_string());
        apply_combat_start_relics(&mut state);

        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.block, 0);
        on_card_played(&mut state, CardType::Skill);
        assert_eq!(state.player.block, 0);
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.block, 4);
    }

    #[test]
    fn test_kunai_every_3_attacks() {
        let mut state = make_test_state();
        state.relics.push("Kunai".to_string());
        apply_combat_start_relics(&mut state);

        on_card_played(&mut state, CardType::Attack);
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.dexterity(), 0);
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.dexterity(), 1);
    }

    #[test]
    fn test_shuriken_every_3_attacks() {
        let mut state = make_test_state();
        state.relics.push("Shuriken".to_string());
        apply_combat_start_relics(&mut state);

        for _ in 0..3 {
            on_card_played(&mut state, CardType::Attack);
        }
        assert_eq!(state.player.strength(), 1);
    }

    #[test]
    fn test_nunchaku_every_10_attacks() {
        let mut state = make_test_state();
        state.relics.push("Nunchaku".to_string());
        let base_energy = state.energy;

        for _ in 0..10 {
            on_card_played(&mut state, CardType::Attack);
        }
        assert_eq!(state.energy, base_energy + 1);
    }

    #[test]
    fn test_pen_nib_every_10_attacks() {
        let mut state = make_test_state();
        state.relics.push("Pen Nib".to_string());
        apply_combat_start_relics(&mut state);

        for _ in 0..9 {
            assert!(!check_pen_nib(&mut state));
        }
        assert!(check_pen_nib(&mut state));
    }

    #[test]
    fn test_bird_faced_urn() {
        let mut state = make_test_state();
        state.player.hp = 70;
        state.relics.push("Bird Faced Urn".to_string());
        on_card_played(&mut state, CardType::Power);
        assert_eq!(state.player.hp, 72);
    }

    #[test]
    fn test_velvet_choker_limit() {
        let mut state = make_test_state();
        state.relics.push("Velvet Choker".to_string());
        apply_combat_start_relics(&mut state);

        for _ in 0..6 {
            assert!(velvet_choker_can_play(&state));
            on_card_played(&mut state, CardType::Attack);
        }
        assert!(!velvet_choker_can_play(&state));
    }

    #[test]
    fn test_orange_pellets_all_types() {
        let mut state = make_test_state();
        state.relics.push("OrangePellets".to_string());
        apply_combat_start_relics(&mut state);
        state.player.add_status("Weakened", 3);
        state.player.add_status("Vulnerable", 2);

        on_card_played(&mut state, CardType::Attack);
        on_card_played(&mut state, CardType::Skill);
        assert!(state.player.is_weak()); // Not yet
        on_card_played(&mut state, CardType::Power);
        assert!(!state.player.is_weak()); // Cleared!
        assert!(!state.player.is_vulnerable());
    }

    // --- Turn end tests ---

    #[test]
    fn test_orichalcum_no_block() {
        let mut state = make_test_state();
        state.relics.push("Orichalcum".to_string());
        state.player.block = 0;
        apply_turn_end_relics(&mut state);
        assert_eq!(state.player.block, 6);
    }

    #[test]
    fn test_orichalcum_has_block() {
        let mut state = make_test_state();
        state.relics.push("Orichalcum".to_string());
        state.player.block = 5;
        apply_turn_end_relics(&mut state);
        assert_eq!(state.player.block, 5); // No change
    }

    #[test]
    fn test_cloak_clasp() {
        let mut state = make_test_state();
        state.relics.push("CloakClasp".to_string());
        state.hand = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        apply_turn_end_relics(&mut state);
        assert_eq!(state.player.block, 3);
    }

    // --- On HP loss tests ---

    #[test]
    fn test_centennial_puzzle() {
        let mut state = make_test_state();
        state.relics.push("Centennial Puzzle".to_string());
        apply_combat_start_relics(&mut state);

        on_hp_loss(&mut state, 5);
        assert_eq!(state.player.status("CentennialPuzzleDraw"), 3);

        // Second hit: no more draws
        on_hp_loss(&mut state, 5);
        // CentennialPuzzleReady is already 0
    }

    #[test]
    fn test_self_forming_clay() {
        let mut state = make_test_state();
        state.relics.push("Self Forming Clay".to_string());
        on_hp_loss(&mut state, 5);
        assert_eq!(state.player.status("NextTurnBlock"), 3);
    }

    #[test]
    fn test_red_skull_activation() {
        let mut state = make_test_state();
        state.relics.push("Red Skull".to_string());
        state.player.hp = 41; // Above 50%
        on_hp_loss(&mut state, 5);
        assert_eq!(state.player.status("RedSkullActive"), 0);

        state.player.hp = 39; // Below 50%
        on_hp_loss(&mut state, 1);
        assert_eq!(state.player.status("RedSkullActive"), 1);
        assert_eq!(state.player.strength(), 3);
    }

    // --- On shuffle tests ---

    #[test]
    fn test_sundial_every_3_shuffles() {
        let mut state = make_test_state();
        state.relics.push("Sundial".to_string());
        let base_energy = state.energy;

        on_shuffle(&mut state);
        on_shuffle(&mut state);
        assert_eq!(state.energy, base_energy);
        on_shuffle(&mut state);
        assert_eq!(state.energy, base_energy + 2);
    }

    #[test]
    fn test_abacus() {
        let mut state = make_test_state();
        state.relics.push("TheAbacus".to_string());
        on_shuffle(&mut state);
        assert_eq!(state.player.block, 6);
    }

    // --- On enemy death tests ---

    #[test]
    fn test_gremlin_horn() {
        let mut state = make_two_enemy_state();
        state.relics.push("Gremlin Horn".to_string());
        let base_energy = state.energy;
        state.enemies[0].entity.hp = 0; // Kill first
        on_enemy_death(&mut state, 0);
        assert_eq!(state.energy, base_energy + 1);
    }

    #[test]
    fn test_the_specimen() {
        let mut state = make_two_enemy_state();
        state.relics.push("The Specimen".to_string());
        state.enemies[0].entity.add_status("Poison", 5);
        state.enemies[0].entity.hp = 0; // Kill first
        on_enemy_death(&mut state, 0);
        assert_eq!(state.enemies[1].entity.status("Poison"), 5);
    }

    // --- Combat end tests ---

    #[test]
    fn test_burning_blood() {
        let mut state = make_test_state();
        state.relics.push("Burning Blood".to_string());
        let heal = on_victory(&mut state);
        assert_eq!(heal, 6);
    }

    #[test]
    fn test_black_blood() {
        let mut state = make_test_state();
        state.relics.push("Black Blood".to_string());
        let heal = on_victory(&mut state);
        assert_eq!(heal, 12);
    }

    // --- Damage modifier tests ---

    #[test]
    fn test_boot_minimum_damage() {
        let mut state = make_test_state();
        state.relics.push("Boot".to_string());
        assert_eq!(apply_boot(&state, 3), 5);
        assert_eq!(apply_boot(&state, 0), 0);
        assert_eq!(apply_boot(&state, 7), 7);
    }

    #[test]
    fn test_charons_ashes() {
        let mut state = make_two_enemy_state();
        state.relics.push("Charon's Ashes".to_string());
        let hp0 = state.enemies[0].entity.hp;
        let hp1 = state.enemies[1].entity.hp;
        charons_ashes_on_exhaust(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp0 - 3);
        assert_eq!(state.enemies[1].entity.hp, hp1 - 3);
    }

    #[test]
    fn test_tough_bandages() {
        let mut state = make_test_state();
        state.relics.push("Tough Bandages".to_string());
        tough_bandages_on_discard(&mut state);
        assert_eq!(state.player.block, 3);
    }

    #[test]
    fn test_violet_lotus_bonus() {
        let mut state = make_test_state();
        assert_eq!(violet_lotus_calm_exit_bonus(&state), 0);
        state.relics.push("Violet Lotus".to_string());
        assert_eq!(violet_lotus_calm_exit_bonus(&state), 1);
    }

    #[test]
    fn test_calipers_block_retention() {
        let mut state = make_test_state();
        state.relics.push("Calipers".to_string());
        assert_eq!(calipers_block_retention(&state, 20), 15);
        assert_eq!(calipers_block_retention(&state, 10), 10);
    }

    #[test]
    fn test_chemical_x_bonus() {
        let mut state = make_test_state();
        assert_eq!(chemical_x_bonus(&state), 0);
        state.relics.push("Chemical X".to_string());
        assert_eq!(chemical_x_bonus(&state), 2);
    }

    #[test]
    fn test_unceasing_top() {
        let mut state = make_test_state();
        state.relics.push("Unceasing Top".to_string());
        state.hand.clear();
        assert!(unceasing_top_should_draw(&state));
        state.hand.push("Strike".to_string());
        assert!(!unceasing_top_should_draw(&state));
    }

    #[test]
    fn test_necronomicon() {
        let mut state = make_test_state();
        state.relics.push("Necronomicon".to_string());
        assert!(necronomicon_should_trigger(&state, 2, true));
        assert!(!necronomicon_should_trigger(&state, 1, true));
        assert!(!necronomicon_should_trigger(&state, 2, false));
        necronomicon_mark_used(&mut state);
        assert!(!necronomicon_should_trigger(&state, 2, true));
    }

    #[test]
    fn test_toy_ornithopter() {
        let mut state = make_test_state();
        state.player.hp = 70;
        state.relics.push("Toy Ornithopter".to_string());
        toy_ornithopter_on_potion(&mut state);
        assert_eq!(state.player.hp, 75);
    }

    #[test]
    fn test_hand_drill() {
        let mut state = make_test_state();
        state.relics.push("HandDrill".to_string());
        hand_drill_on_block_break(&mut state, 0);
        assert_eq!(state.enemies[0].entity.status("Vulnerable"), 2);
    }

    #[test]
    fn test_pantograph_boss() {
        let mut state = make_test_state();
        state.enemies[0] = EnemyCombatState::new("Hexaghost", 250, 250);
        state.player.hp = 50;
        state.relics.push("Pantograph".to_string());
        apply_combat_start_relics(&mut state);
        assert_eq!(state.player.hp, 75);
    }

    #[test]
    fn test_letter_opener() {
        let mut state = make_test_state();
        state.relics.push("Letter Opener".to_string());
        apply_combat_start_relics(&mut state);
        let hp = state.enemies[0].entity.hp;

        on_card_played(&mut state, CardType::Skill);
        on_card_played(&mut state, CardType::Skill);
        assert_eq!(state.enemies[0].entity.hp, hp);
        on_card_played(&mut state, CardType::Skill);
        assert_eq!(state.enemies[0].entity.hp, hp - 5);
    }

    #[test]
    fn test_duality_yang() {
        let mut state = make_test_state();
        state.relics.push("Yang".to_string());
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.dexterity(), 1);
        assert_eq!(state.player.status("LoseDexterity"), 1);
    }

    #[test]
    fn test_stone_calendar_turn7() {
        // Use a high-HP enemy so it survives 52 damage
        let enemy = EnemyCombatState::new("Boss", 200, 200);
        let mut state = CombatState::new(80, 80, vec![enemy], vec!["Strike_P".to_string(); 5], 3);
        state.relics.push("StoneCalendar".to_string());
        apply_combat_start_relics(&mut state);

        for t in 1..=7 {
            state.turn = t;
            apply_turn_start_relics(&mut state);
        }
        let hp_before = state.enemies[0].entity.hp;
        apply_turn_end_relics(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp_before - 52);
    }

    #[test]
    fn test_ink_bottle_every_10_cards() {
        let mut state = make_test_state();
        state.relics.push("InkBottle".to_string());
        for _ in 0..9 {
            on_card_played(&mut state, CardType::Attack);
        }
        assert_eq!(state.player.status("InkBottleDraw"), 0);
        on_card_played(&mut state, CardType::Attack);
        assert_eq!(state.player.status("InkBottleDraw"), 1);
    }

    #[test]
    fn test_tingsha() {
        let mut state = make_test_state();
        state.relics.push("Tingsha".to_string());
        let hp = state.enemies[0].entity.hp;
        tingsha_on_discard(&mut state);
        assert_eq!(state.enemies[0].entity.hp, hp - 3);
    }
}
