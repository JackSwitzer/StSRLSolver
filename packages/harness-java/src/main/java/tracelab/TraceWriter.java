package tracelab;

import com.google.gson.Gson;
import com.megacrit.cardcrawl.cards.AbstractCard;
import com.megacrit.cardcrawl.characters.AbstractPlayer;
import com.megacrit.cardcrawl.core.CardCrawlGame;
import com.megacrit.cardcrawl.dungeons.AbstractDungeon;
import com.megacrit.cardcrawl.helpers.SeedHelper;
import com.megacrit.cardcrawl.monsters.AbstractMonster;
import com.megacrit.cardcrawl.powers.AbstractPower;
import com.megacrit.cardcrawl.potions.AbstractPotion;
import com.megacrit.cardcrawl.random.Random;
import com.megacrit.cardcrawl.relics.AbstractRelic;
import com.megacrit.cardcrawl.rooms.AbstractRoom;
import com.megacrit.cardcrawl.ui.panels.EnergyPanel;

import java.io.FileWriter;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

/**
 * Per-action JSONL trace records per docs/goal/TOOLING.md T1. State capture
 * evolved from EVTracker's TurnStateCapture (reference/evtracker), trimmed to
 * deterministic fields only (no timestamps, uuids, or derived analysis).
 * RNG stream names follow docs/vault/rng-system-analysis.md.
 */
public class TraceWriter {

    private static final Gson GSON = new Gson();
    private static PrintWriter out;

    public static void init(String path, Script script) throws IOException {
        out = new PrintWriter(new FileWriter(path, false));
        Map<String, Object> header = new LinkedHashMap<String, Object>();
        header.put("v", 1);
        header.put("kind", "header");
        header.put("seed", script.seed.trim().toUpperCase());
        header.put("seed_long", TraceLabMod.parseSeed(script.seed));
        header.put("character", script.character.toUpperCase());
        header.put("ascension", script.ascension);
        header.put("game_version", CardCrawlGame.TRUE_VERSION_NUM);
        List<String> mods = new ArrayList<String>();
        mods.add("basemod");
        mods.add("tracelab");
        header.put("mods", mods);
        writeLine(header);
    }

    public static void writeRecord(int idx, Script.Action action) {
        if (out == null) {
            return;
        }
        Map<String, Object> rec = new LinkedHashMap<String, Object>();
        rec.put("v", 1);
        rec.put("idx", idx);
        rec.put("floor", AbstractDungeon.floorNum);
        rec.put("turn", AbstractDungeon.actionManager != null ? AbstractDungeon.actionManager.turn : 0);
        rec.put("phase", currentPhase());
        rec.put("action", actionMap(action));
        rec.put("post", postState());
        writeLine(rec);
    }

    public static void writeFinal(String status, int recorded, int executed, int scripted) {
        if (out == null) {
            return;
        }
        Map<String, Object> rec = new LinkedHashMap<String, Object>();
        rec.put("v", 1);
        rec.put("kind", "end");
        rec.put("status", status);
        rec.put("recorded", recorded);
        rec.put("executed", executed);
        rec.put("scripted", scripted);
        writeLine(rec);
    }

    public static void close() {
        if (out != null) {
            out.flush();
            out.close();
            out = null;
        }
    }

    private static void writeLine(Map<String, Object> obj) {
        out.println(GSON.toJson(obj));
        out.flush();
    }

    private static String currentPhase() {
        AbstractRoom room = ScriptRunner.currRoom();
        if (room == null) {
            return "NONE";
        }
        if (room.phase == AbstractRoom.RoomPhase.COMBAT) {
            return "COMBAT";
        }
        if (room.event != null) {
            return "EVENT";
        }
        if (AbstractDungeon.screen == AbstractDungeon.CurrentScreen.MAP) {
            return "MAP";
        }
        return room.phase != null ? room.phase.name() : "NONE";
    }

    private static Map<String, Object> actionMap(Script.Action a) {
        Map<String, Object> m = new LinkedHashMap<String, Object>();
        m.put("type", a.type);
        if (a.hand_idx != null) m.put("hand_idx", a.hand_idx);
        if (a.target != null) m.put("target", a.target);
        if (a.choice != null) m.put("choice", a.choice);
        if (a.item != null) m.put("item", a.item);
        if (a.idx != null) m.put("idx", a.idx);
        if (a.card_id != null) m.put("card_id", a.card_id);
        if (a.choice_name != null) m.put("choice_name", a.choice_name);
        return m;
    }

    private static Map<String, Object> postState() {
        Map<String, Object> post = new LinkedHashMap<String, Object>();
        post.put("player", playerState());
        post.put("enemies", enemyStates());
        post.put("piles", piles());
        post.put("relics", relics());
        post.put("potions", potions());
        post.put("rng", rngCounters());
        return post;
    }

    private static Map<String, Object> playerState() {
        AbstractPlayer p = AbstractDungeon.player;
        Map<String, Object> s = new LinkedHashMap<String, Object>();
        s.put("hp", p.currentHealth);
        s.put("max_hp", p.maxHealth);
        s.put("block", p.currentBlock);
        s.put("energy", EnergyPanel.totalCount);
        s.put("stance", p.stance != null ? p.stance.ID : "None");
        s.put("gold", p.gold);
        s.put("powers", powers(p.powers));
        List<Map<String, Object>> orbs = new ArrayList<Map<String, Object>>();
        if (p.orbs != null) {
            for (com.megacrit.cardcrawl.orbs.AbstractOrb orb : p.orbs) {
                Map<String, Object> o = new LinkedHashMap<String, Object>();
                o.put("id", orb.ID);
                orbs.add(o);
            }
        }
        s.put("orbs", orbs);
        return s;
    }

    private static List<Map<String, Object>> enemyStates() {
        List<Map<String, Object>> enemies = new ArrayList<Map<String, Object>>();
        AbstractRoom room = ScriptRunner.currRoom();
        if (room == null || room.phase != AbstractRoom.RoomPhase.COMBAT
                || AbstractDungeon.getMonsters() == null) {
            return enemies;
        }
        int i = 0;
        for (AbstractMonster m : AbstractDungeon.getMonsters().monsters) {
            Map<String, Object> s = new LinkedHashMap<String, Object>();
            s.put("id", m.id);
            s.put("idx", i++);
            s.put("dead", m.isDead || m.isDying);
            s.put("hp", m.currentHealth);
            s.put("max_hp", m.maxHealth);
            s.put("block", m.currentBlock);
            Map<String, Object> intent = new LinkedHashMap<String, Object>();
            intent.put("name", m.intent != null ? m.intent.name() : "NONE");
            intent.put("dmg", m.getIntentDmg());
            intent.put("hits", intentHits(m));
            List<Integer> history = new ArrayList<Integer>();
            for (Byte move : m.moveHistory) {
                history.add(move.intValue());
            }
            intent.put("move_id", history.isEmpty() ? -1 : history.get(history.size() - 1));
            s.put("intent", intent);
            s.put("move_history", history);
            s.put("powers", powers(m.powers));
            enemies.add(s);
        }
        return enemies;
    }

    private static java.lang.reflect.Field multiAmtField;
    private static java.lang.reflect.Field isMultiField;

    // AbstractMonster.intentMultiAmt / isMultiDmg are private (monsters/AbstractMonster.java:122).
    private static int intentHits(AbstractMonster m) {
        try {
            if (multiAmtField == null) {
                multiAmtField = AbstractMonster.class.getDeclaredField("intentMultiAmt");
                multiAmtField.setAccessible(true);
                isMultiField = AbstractMonster.class.getDeclaredField("isMultiDmg");
                isMultiField.setAccessible(true);
            }
            return isMultiField.getBoolean(m) ? multiAmtField.getInt(m) : 1;
        } catch (ReflectiveOperationException e) {
            return 1;
        }
    }

    private static List<Map<String, Object>> powers(ArrayList<AbstractPower> src) {
        List<Map<String, Object>> list = new ArrayList<Map<String, Object>>();
        for (AbstractPower power : src) {
            Map<String, Object> p = new LinkedHashMap<String, Object>();
            p.put("id", power.ID);
            p.put("amt", power.amount);
            list.add(p);
        }
        return list;
    }

    private static Map<String, Object> piles() {
        AbstractPlayer p = AbstractDungeon.player;
        Map<String, Object> piles = new LinkedHashMap<String, Object>();
        piles.put("hand", cardIds(p.hand.group));
        piles.put("draw_ordered", cardIds(p.drawPile.group));
        piles.put("discard", cardIds(p.discardPile.group));
        piles.put("exhaust", cardIds(p.exhaustPile.group));
        return piles;
    }

    private static List<String> cardIds(ArrayList<AbstractCard> cards) {
        List<String> ids = new ArrayList<String>();
        for (AbstractCard c : cards) {
            StringBuilder sb = new StringBuilder(c.cardID);
            if (c.timesUpgraded > 0) {
                sb.append('+');
                if (c.timesUpgraded > 1) {
                    sb.append(c.timesUpgraded);
                }
            }
            ids.add(sb.toString());
        }
        return ids;
    }

    private static List<Map<String, Object>> relics() {
        List<Map<String, Object>> list = new ArrayList<Map<String, Object>>();
        for (AbstractRelic relic : AbstractDungeon.player.relics) {
            Map<String, Object> r = new LinkedHashMap<String, Object>();
            r.put("id", relic.relicId);
            r.put("counter", relic.counter);
            list.add(r);
        }
        return list;
    }

    private static List<String> potions() {
        List<String> list = new ArrayList<String>();
        for (AbstractPotion potion : AbstractDungeon.player.potions) {
            list.add(potion.ID);
        }
        return list;
    }

    private static Map<String, Object> rngCounters() {
        Map<String, Object> rng = new TreeMap<String, Object>();
        putCounter(rng, "card", AbstractDungeon.cardRng);
        putCounter(rng, "cardRandom", AbstractDungeon.cardRandomRng);
        putCounter(rng, "shuffle", AbstractDungeon.shuffleRng);
        putCounter(rng, "monster", AbstractDungeon.monsterRng);
        putCounter(rng, "monsterHp", AbstractDungeon.monsterHpRng);
        putCounter(rng, "ai", AbstractDungeon.aiRng);
        putCounter(rng, "relic", AbstractDungeon.relicRng);
        putCounter(rng, "treasure", AbstractDungeon.treasureRng);
        putCounter(rng, "event", AbstractDungeon.eventRng);
        putCounter(rng, "merchant", AbstractDungeon.merchantRng);
        putCounter(rng, "potion", AbstractDungeon.potionRng);
        putCounter(rng, "map", AbstractDungeon.mapRng);
        putCounter(rng, "misc", AbstractDungeon.miscRng);
        return rng;
    }

    private static void putCounter(Map<String, Object> rng, String key, Random random) {
        rng.put(key, random != null ? random.counter : -1);
    }
}
