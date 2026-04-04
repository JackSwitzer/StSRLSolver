use crate::cards::CardType;
use crate::state::CombatState;
use crate::status_ids::sid;

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
                state.player.add_status(sid::STRENGTH, 1);
            }
            "Oddly Smooth Stone" | "OddlySmoothStone" => {
                // +1 Dexterity at combat start
                state.player.add_status(sid::DEXTERITY, 1);
            }
            "Data Disk" | "DataDisk" => {
                // +1 Focus at combat start
                state.player.add_status(sid::FOCUS, 1);
            }
            "Akabeko" => {
                // 8 Vigor at combat start
                state.player.add_status(sid::VIGOR, 8);
            }
            "Bag of Marbles" => {
                // Apply 1 Vulnerable to ALL enemies
                for enemy in &mut state.enemies {
                    if enemy.is_alive() {
                        enemy.entity.add_status(sid::VULNERABLE, 1);
                    }
                }
            }
            "Red Mask" | "RedMask" => {
                // Apply 1 Weak to ALL enemies
                for enemy in &mut state.enemies {
                    if enemy.is_alive() {
                        enemy.entity.add_status(sid::WEAKENED, 1);
                    }
                }
            }
            "Thread and Needle" => {
                // 4 Plated Armor at combat start
                state.player.add_status(sid::PLATED_ARMOR, 4);
            }
            "Bronze Scales" => {
                // 3 Thorns at combat start
                state.player.add_status(sid::THORNS, 3);
            }
            "Anchor" => {
                // 10 Block at combat start
                state.player.block += 10;
            }
            "Lantern" => {
                // +1 energy on turn 1 (tracked via counter)
                state.player.set_status(sid::LANTERN_READY, 1);
            }
            "Clockwork Souvenir" | "ClockworkSouvenir" => {
                // 1 Artifact at combat start
                state.player.add_status(sid::ARTIFACT, 1);
            }
            "Fossilized Helix" | "FossilizedHelix" => {
                // 1 Buffer at combat start
                state.player.add_status(sid::BUFFER, 1);
            }
            "Mark of Pain" => {
                // 2 Wounds in draw pile
                let registry = crate::cards::CardRegistry::new();
                state.draw_pile.push(registry.make_card("Wound"));
                state.draw_pile.push(registry.make_card("Wound"));
            }
            "Blood Vial" => {
                // Heal 2 HP at combat start
                state.heal_player(2);
            }
            "MutagenicStrength" => {
                // +3 Strength, -3 at end of turn (temporary)
                state.player.add_status(sid::STRENGTH, 3);
                state.player.add_status(sid::LOSE_STRENGTH, 3);
            }

            // --- Card-generation relics (atBattleStartPreDraw) ---
            "PureWater" => {
                // Add a Miracle card to hand at combat start
                let registry = crate::cards::CardRegistry::new();
                state.hand.push(registry.make_card("Miracle"));
            }
            "HolyWater" => {
                // Add 3 Holy Water cards to hand at combat start
                let registry = crate::cards::CardRegistry::new();
                for _ in 0..3 {
                    if state.hand.len() < 10 {
                        state.hand.push(registry.make_card("HolyWater"));
                    }
                }
            }
            "Ninja Scroll" | "NinjaScroll" => {
                // Add 3 Shivs to hand at combat start
                let registry = crate::cards::CardRegistry::new();
                for _ in 0..3 {
                    if state.hand.len() < 10 {
                        state.hand.push(registry.make_card("Shiv"));
                    }
                }
            }

            // --- Draw relics (atBattleStart -> draw) ---
            "Bag of Preparation" => {
                // Draw 2 extra cards at combat start
                state.player.set_status(sid::BAG_OF_PREP_DRAW, 2);
            }
            "Ring of the Snake" => {
                // Draw 2 extra cards at combat start
                state.player.set_status(sid::BAG_OF_PREP_DRAW, 2);
            }

            // --- Philosopher's Stone: +1 energy, all enemies +1 Strength ---
            "Philosopher's Stone" | "PhilosophersStone" => {
                for enemy in &mut state.enemies {
                    if enemy.is_alive() {
                        enemy.entity.add_status(sid::STRENGTH, 1);
                    }
                }
                // Energy bonus handled via max_energy on equip (Python side)
            }

            // --- Pen Nib: track counter ---
            "Pen Nib" => {
                if state.player.status(sid::PEN_NIB_COUNTER) == 0 {
                    state.player.set_status(sid::PEN_NIB_COUNTER, 0);
                }
            }

            // --- Counter-based relics: initialize ---
            "Ornamental Fan" => {
                state.player.set_status(sid::ORNAMENTAL_FAN_COUNTER, 0);
            }
            "Kunai" => {
                state.player.set_status(sid::KUNAI_COUNTER, 0);
            }
            "Shuriken" => {
                state.player.set_status(sid::SHURIKEN_COUNTER, 0);
            }
            "Nunchaku" => {
                // Counter persists across combats, don't reset
            }
            "Letter Opener" => {
                state.player.set_status(sid::LETTER_OPENER_COUNTER, 0);
            }
            "Happy Flower" => {
                // Counter persists across combats (counter field)
                // Initialize if not set
                if state.player.status(sid::HAPPY_FLOWER_COUNTER) == 0 {
                    state.player.set_status(sid::HAPPY_FLOWER_COUNTER, 0);
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
                state.player.set_status(sid::HORN_CLEAT_COUNTER, 0);
            }
            "CaptainsWheel" => {
                state.player.set_status(sid::CAPTAINS_WHEEL_COUNTER, 0);
            }
            "StoneCalendar" => {
                state.player.set_status(sid::STONE_CALENDAR_COUNTER, 0);
            }

            // --- Velvet Choker: card limit ---
            "Velvet Choker" | "VelvetChoker" => {
                state.player.set_status(sid::VELVET_CHOKER_COUNTER, 0);
            }

            // --- Pocketwatch ---
            "Pocketwatch" => {
                state.player.set_status(sid::POCKETWATCH_COUNTER, 0);
                state.player.set_status(sid::POCKETWATCH_FIRST_TURN, 1);
            }

            // --- Violet Lotus: +1 energy on Calm exit (handled in stance change) ---
            "Violet Lotus" | "VioletLotus" => {
                state.player.set_status(sid::VIOLET_LOTUS, 1);
            }

            // --- Emotion Chip: on HP loss, trigger orb passive ---
            "Emotion Chip" | "EmotionChip" => {
                state.player.set_status(sid::EMOTION_CHIP_READY, 1);
            }

            // --- CentennialPuzzle: first HP loss draws 3 ---
            "Centennial Puzzle" | "CentennialPuzzle" => {
                state.player.set_status(sid::CENTENNIAL_PUZZLE_READY, 1);
            }

            // --- ArtOfWar: if no attacks played, +1 energy next turn ---
            "Art of War" => {
                state.player.set_status(sid::ART_OF_WAR_READY, 1);
            }

            // --- Twisted Funnel: apply 4 Poison to all enemies ---
            "TwistedFunnel" => {
                for enemy in &mut state.enemies {
                    if enemy.is_alive() {
                        enemy.entity.add_status(sid::POISON, 4);
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
                state.player.set_status(sid::SNECKO_EYE, 1);
                state.player.set_status(sid::CONFUSION, 1);
                state.player.set_status(sid::BAG_OF_PREP_DRAW, 2);
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
                    state.heal_player(25);
                }
            }

            // --- Sling of Courage: +2 Strength at elite fights ---
            "Sling" => {
                // Elite detection would be Python-side
                // Stub: if sling_elite flag is set
                if state.player.status(sid::SLING_ELITE) > 0 {
                    state.player.add_status(sid::STRENGTH, 2);
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
                if state.player.status(sid::PRESERVED_INSECT_ELITE) > 0 {
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
                let counter = state.player.status(sid::NEOWS_LAMENT_COUNTER);
                if counter > 0 {
                    for enemy in &mut state.enemies {
                        if enemy.is_alive() {
                            enemy.entity.hp = 1;
                        }
                    }
                    state.player.set_status(sid::NEOWS_LAMENT_COUNTER, counter - 1);
                }
            }

            // --- Du-Vu Doll: +1 Strength per curse in deck ---
            "Du-Vu Doll" => {
                let curse_count = state.player.status(sid::DU_VU_DOLL_CURSES);
                if curse_count > 0 {
                    state.player.add_status(sid::STRENGTH, curse_count);
                }
            }

            // --- Girya: Strength from rest site lifting ---
            "Girya" => {
                let lift_count = state.player.status(sid::GIRYA_COUNTER);
                if lift_count > 0 {
                    state.player.add_status(sid::STRENGTH, lift_count);
                }
            }

            // --- Red Skull: +3 Strength when HP <= 50% ---
            "Red Skull" => {
                if state.player.hp <= state.player.max_hp / 2 {
                    state.player.add_status(sid::STRENGTH, 3);
                    state.player.set_status(sid::RED_SKULL_ACTIVE, 1);
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
                state.player.set_status(sid::OP_ATTACK, 0);
                state.player.set_status(sid::OP_SKILL, 0);
                state.player.set_status(sid::OP_POWER, 0);
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

            // ---- Relic modifier flags (checked in pipelines) ----
            "Mark of the Bloom" | "MarkOfTheBloom" => {
                state.player.set_status(sid::HAS_MARK_OF_BLOOM, 1);
            }
            "Magic Flower" | "MagicFlower" => {
                state.player.set_status(sid::HAS_MAGIC_FLOWER, 1);
            }
            "Ginger" => {
                state.player.set_status(sid::HAS_GINGER, 1);
            }
            "Turnip" => {
                state.player.set_status(sid::HAS_TURNIP, 1);
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
            "Omamori" |
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
    if state.turn == 1 && state.player.status(sid::LANTERN_READY) > 0 {
        state.energy += 1;
        state.player.set_status(sid::LANTERN_READY, 0);
    }

    // Bag of Preparation / Ring of the Snake: extra draw on turn 1
    if state.turn == 1 {
        let extra_draw = state.player.status(sid::BAG_OF_PREP_DRAW);
        if extra_draw > 0 {
            state.player.set_status(sid::TURN_START_EXTRA_DRAW, extra_draw);
            state.player.set_status(sid::BAG_OF_PREP_DRAW, 0);
        }
    }

    // Happy Flower: every 3rd turn, +1 energy
    if state.has_relic("Happy Flower") {
        let counter = state.player.status(sid::HAPPY_FLOWER_COUNTER) + 1;
        if counter >= 3 {
            state.energy += 1;
            state.player.set_status(sid::HAPPY_FLOWER_COUNTER, 0);
        } else {
            state.player.set_status(sid::HAPPY_FLOWER_COUNTER, counter);
        }
    }

    // Incense Burner: every 6th turn, gain 1 Intangible
    if state.has_relic("Incense Burner") || state.has_relic("IncenseBurner") {
        let counter = state.player.status(sid::INCENSE_BURNER_COUNTER) + 1;
        if counter >= 6 {
            state.player.add_status(sid::INTANGIBLE, 1);
            state.player.set_status(sid::INCENSE_BURNER_COUNTER, 0);
        } else {
            state.player.set_status(sid::INCENSE_BURNER_COUNTER, counter);
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
        state.player.add_status(sid::STRENGTH, 2);
        for enemy in &mut state.enemies {
            if enemy.is_alive() {
                enemy.entity.add_status(sid::STRENGTH, 1);
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
            state.player.set_status(sid::ENTER_DIVINITY, 1);
        }
    }

    // Inserter: every 2nd turn, gain an orb slot
    if state.has_relic("Inserter") {
        let counter = state.player.status(sid::INSERTER_COUNTER) + 1;
        if counter >= 2 {
            state.player.set_status(sid::INSERTER_COUNTER, 0);
            // Orb slot increase; tracked as status for MCTS
            state.player.add_status(sid::ORB_SLOTS, 1);
        } else {
            state.player.set_status(sid::INSERTER_COUNTER, counter);
        }
    }

    // Horn Cleat: on 2nd turn, gain 14 Block (once)
    if state.has_relic("HornCleat") {
        let counter = state.player.status(sid::HORN_CLEAT_COUNTER);
        if counter >= 0 && counter < 2 {
            let new_counter = counter + 1;
            if new_counter == 2 {
                state.player.block += 14;
                state.player.set_status(sid::HORN_CLEAT_COUNTER, -1); // done
            } else {
                state.player.set_status(sid::HORN_CLEAT_COUNTER, new_counter);
            }
        }
    }

    // Captain's Wheel: on 3rd turn, gain 18 Block (once)
    if state.has_relic("CaptainsWheel") {
        let counter = state.player.status(sid::CAPTAINS_WHEEL_COUNTER);
        if counter >= 0 && counter < 3 {
            let new_counter = counter + 1;
            if new_counter == 3 {
                state.player.block += 18;
                state.player.set_status(sid::CAPTAINS_WHEEL_COUNTER, -1); // done
            } else {
                state.player.set_status(sid::CAPTAINS_WHEEL_COUNTER, new_counter);
            }
        }
    }

    // Stone Calendar: on 7th turn, deal 52 damage to all enemies (at end of turn)
    if state.has_relic("StoneCalendar") {
        let counter = state.player.status(sid::STONE_CALENDAR_COUNTER) + 1;
        state.player.set_status(sid::STONE_CALENDAR_COUNTER, counter);
    }

    // Velvet Choker: reset card play counter
    if state.has_relic("Velvet Choker") || state.has_relic("VelvetChoker") {
        state.player.set_status(sid::VELVET_CHOKER_COUNTER, 0);
    }

    // Pocketwatch: if played <= 3 cards last turn, draw 3 extra
    if state.has_relic("Pocketwatch") {
        let first_turn = state.player.status(sid::POCKETWATCH_FIRST_TURN);
        if first_turn > 0 {
            state.player.set_status(sid::POCKETWATCH_FIRST_TURN, 0);
        } else {
            let counter = state.player.status(sid::POCKETWATCH_COUNTER);
            if counter <= 3 {
                state.player.set_status(sid::TURN_START_EXTRA_DRAW,
                    state.player.status(sid::TURN_START_EXTRA_DRAW) + 3);
            }
        }
        state.player.set_status(sid::POCKETWATCH_COUNTER, 0);
    }

    // Art of War: if no attacks played last turn, +1 energy
    if state.has_relic("Art of War") {
        let ready = state.player.status(sid::ART_OF_WAR_READY);
        if ready > 0 && state.turn > 1 {
            state.energy += 1;
        }
        // Reset: will be set to 0 if attack is played
        state.player.set_status(sid::ART_OF_WAR_READY, 1);
    }

    // Kunai counter reset
    if state.has_relic("Kunai") {
        state.player.set_status(sid::KUNAI_COUNTER, 0);
    }

    // Shuriken counter reset
    if state.has_relic("Shuriken") {
        state.player.set_status(sid::SHURIKEN_COUNTER, 0);
    }

    // Letter Opener counter reset
    if state.has_relic("Letter Opener") {
        state.player.set_status(sid::LETTER_OPENER_COUNTER, 0);
    }

    // Ornamental Fan counter reset
    if state.has_relic("Ornamental Fan") {
        state.player.set_status(sid::ORNAMENTAL_FAN_COUNTER, 0);
    }

    // Orange Pellets: reset type tracking
    if state.has_relic("OrangePellets") {
        state.player.set_status(sid::OP_ATTACK, 0);
        state.player.set_status(sid::OP_SKILL, 0);
        state.player.set_status(sid::OP_POWER, 0);
    }

    // Unceasing Top: draw when hand is empty (handled in engine)

    // Hovering Kite: discard 1 card, gain 1 energy (complex; Python handles)
}

/// Legacy wrapper for Lantern (backward compat).
pub fn apply_lantern_turn_start(state: &mut CombatState) {
    if state.turn == 1 && state.player.status(sid::LANTERN_READY) > 0 {
        state.energy += 1;
        state.player.set_status(sid::LANTERN_READY, 0);
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

    // --- Ornamental Fan: gain 4 block every 3 ATTACKS played ---
    if is_attack && state.has_relic("Ornamental Fan") {
        let counter = state.player.status(sid::ORNAMENTAL_FAN_COUNTER) + 1;
        if counter >= 3 {
            state.player.block += 4;
            state.player.set_status(sid::ORNAMENTAL_FAN_COUNTER, 0);
        } else {
            state.player.set_status(sid::ORNAMENTAL_FAN_COUNTER, counter);
        }
    }

    // --- Kunai: every 3 attacks, +1 Dexterity ---
    if is_attack && state.has_relic("Kunai") {
        let counter = state.player.status(sid::KUNAI_COUNTER) + 1;
        if counter >= 3 {
            state.player.add_status(sid::DEXTERITY, 1);
            state.player.set_status(sid::KUNAI_COUNTER, 0);
        } else {
            state.player.set_status(sid::KUNAI_COUNTER, counter);
        }
    }

    // --- Shuriken: every 3 attacks, +1 Strength ---
    if is_attack && state.has_relic("Shuriken") {
        let counter = state.player.status(sid::SHURIKEN_COUNTER) + 1;
        if counter >= 3 {
            state.player.add_status(sid::STRENGTH, 1);
            state.player.set_status(sid::SHURIKEN_COUNTER, 0);
        } else {
            state.player.set_status(sid::SHURIKEN_COUNTER, counter);
        }
    }

    // --- Letter Opener: every 3 skills, deal 5 damage to ALL enemies ---
    if is_skill && state.has_relic("Letter Opener") {
        let counter = state.player.status(sid::LETTER_OPENER_COUNTER) + 1;
        if counter >= 3 {
            let living = state.living_enemy_indices();
            for idx in living {
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
            state.player.set_status(sid::LETTER_OPENER_COUNTER, 0);
        } else {
            state.player.set_status(sid::LETTER_OPENER_COUNTER, counter);
        }
    }

    // --- Nunchaku: every 10 attacks, +1 energy ---
    if is_attack && state.has_relic("Nunchaku") {
        let counter = state.player.status(sid::NUNCHAKU_COUNTER) + 1;
        if counter >= 10 {
            state.energy += 1;
            state.player.set_status(sid::NUNCHAKU_COUNTER, 0);
        } else {
            state.player.set_status(sid::NUNCHAKU_COUNTER, counter);
        }
    }

    // --- Ink Bottle: every 10 cards, draw 1 ---
    if state.has_relic("InkBottle") {
        let counter = state.player.status(sid::INK_BOTTLE_COUNTER) + 1;
        if counter >= 10 {
            // Draw 1 card (set flag for engine)
            state.player.set_status(sid::INK_BOTTLE_DRAW, 1);
            state.player.set_status(sid::INK_BOTTLE_COUNTER, 0);
        } else {
            state.player.set_status(sid::INK_BOTTLE_COUNTER, counter);
        }
    }

    // --- Pen Nib: handled separately via check_pen_nib ---

    // --- Velvet Choker: track cards played ---
    if state.has_relic("Velvet Choker") || state.has_relic("VelvetChoker") {
        state.player.add_status(sid::VELVET_CHOKER_COUNTER, 1);
    }

    // --- Pocketwatch: track cards played ---
    if state.has_relic("Pocketwatch") {
        state.player.add_status(sid::POCKETWATCH_COUNTER, 1);
    }

    // --- Art of War: if attack played, disable bonus ---
    if is_attack && (state.has_relic("Art of War")) {
        state.player.set_status(sid::ART_OF_WAR_READY, 0);
    }

    // --- Bird-Faced Urn: heal 2 when playing a Power ---
    if is_power && state.has_relic("Bird Faced Urn") {
        state.heal_player(2);
    }

    // --- Mummified Hand: when playing a Power, random card in hand costs 0 ---
    if is_power && state.has_relic("Mummified Hand") {
        // Complex card cost manipulation; set flag for engine
        state.player.set_status(sid::MUMMIFIED_HAND_TRIGGER, 1);
    }

    // --- Duality (Yang): when playing an Attack, gain 1 Dexterity this turn ---
    if is_attack && state.has_relic("Yang") {
        state.player.add_status(sid::DEXTERITY, 1);
        state.player.add_status(sid::LOSE_DEXTERITY, 1);
    }

    // --- Necronomicon: first Attack costing 2+ per turn is played twice ---
    // Handled in engine card play logic

    // --- Orange Pellets: track card types ---
    if state.has_relic("OrangePellets") {
        if is_attack {
            state.player.set_status(sid::OP_ATTACK, 1);
        } else if is_skill {
            state.player.set_status(sid::OP_SKILL, 1);
        } else if is_power {
            state.player.set_status(sid::OP_POWER, 1);
        }
        // If all three types played, remove all debuffs
        if state.player.status(sid::OP_ATTACK) > 0
            && state.player.status(sid::OP_SKILL) > 0
            && state.player.status(sid::OP_POWER) > 0
        {
            // Remove debuffs
            state.player.set_status(sid::WEAKENED, 0);
            state.player.set_status(sid::VULNERABLE, 0);
            state.player.set_status(sid::FRAIL, 0);
            state.player.set_status(sid::ENTANGLED, 0);
            state.player.set_status(sid::NO_DRAW, 0);
            state.player.set_status(sid::OP_ATTACK, 0);
            state.player.set_status(sid::OP_SKILL, 0);
            state.player.set_status(sid::OP_POWER, 0);
        }
    }
}

/// Apply Ornamental Fan: gain 4 block after playing 3 ATTACKS.
/// Legacy wrapper — caller MUST only call for attacks. Use on_card_played for new code.
pub fn check_ornamental_fan(state: &mut CombatState) {
    if !state.has_relic("Ornamental Fan") {
        return;
    }

    let counter = state.player.status(sid::ORNAMENTAL_FAN_COUNTER) + 1;
    if counter >= 3 {
        state.player.block += 4;
        state.player.set_status(sid::ORNAMENTAL_FAN_COUNTER, 0);
    } else {
        state.player.set_status(sid::ORNAMENTAL_FAN_COUNTER, counter);
    }
}

/// Check Pen Nib: every 10th attack deals double damage.
/// Returns true if this attack triggers Pen Nib.
pub fn check_pen_nib(state: &mut CombatState) -> bool {
    if !state.has_relic("Pen Nib") {
        return false;
    }

    let counter = state.player.status(sid::PEN_NIB_COUNTER);
    if counter >= 9 {
        state.player.set_status(sid::PEN_NIB_COUNTER, 0);
        true
    } else {
        state.player.set_status(sid::PEN_NIB_COUNTER, counter + 1);
        false
    }
}

/// Check if Velvet Choker allows playing another card.
pub fn velvet_choker_can_play(state: &CombatState) -> bool {
    if !state.has_relic("Velvet Choker") && !state.has_relic("VelvetChoker") {
        return true;
    }
    state.player.status(sid::VELVET_CHOKER_COUNTER) < 6
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

    // Stone Calendar: on exactly the 7th turn, deal 52 damage to all enemies (once)
    if state.has_relic("StoneCalendar") {
        if state.player.status(sid::STONE_CALENDAR_COUNTER) == 7
            && state.player.status(sid::STONE_CALENDAR_FIRED) == 0
        {
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
            state.player.set_status(sid::STONE_CALENDAR_FIRED, 1);
        }
    }

    // Frozen Core: if empty orb slot, channel Frost (Defect)
    if state.has_relic("FrozenCore") {
        // Orb handling is Python-side; set flag
        state.player.set_status(sid::FROZEN_CORE_TRIGGER, 1);
    }

    // Nilry's Codex: discover a card at end of turn (complex; Python handles)
    // Stub: no combat effect in MCTS
}

