import SwiftUI

struct BossFightView: View {
    let episodes: [Episode]

    /// Episodes that reached floor 14+ with boss combats, most recent first
    private var bossEpisodes: [BossRun] {
        episodes
            .filter { $0.effectiveFloor >= 14 }
            .compactMap { ep -> BossRun? in
                guard let combats = ep.combats else { return nil }
                let bossFights = combats.filter { $0.roomCategory == .boss }
                guard !bossFights.isEmpty else { return nil }
                return BossRun(episode: ep, bossCombats: bossFights)
            }
            .sorted { $0.episode.effectiveFloor > $1.episode.effectiveFloor }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SectionHeader(title: "Boss Fight Deep-Dive")

            if bossEpisodes.isEmpty {
                Text("No boss fights recorded yet")
                    .font(.stsBody)
                    .foregroundStyle(Color.stsTextMuted)
                    .frame(maxWidth: .infinity, minHeight: 60)
            } else {
                ScrollView {
                    LazyVStack(spacing: 6) {
                        ForEach(bossEpisodes.prefix(20)) { run in
                            BossRunRow(run: run)
                        }
                    }
                }
                .frame(maxHeight: 400)
            }
        }
    }
}

// MARK: - Data model

private struct BossRun: Identifiable {
    let id = UUID()
    let episode: Episode
    let bossCombats: [Combat]

    var bossName: String {
        bossCombats.first?.enemy ?? "Unknown"
    }

    /// HP when arriving at boss: death HP + total HP lost in boss fights
    var hpAtBoss: Int {
        let totalLost = bossCombats.compactMap(\.hpLost).reduce(0, +)
        return episode.effectiveHp + totalLost
    }
}

// MARK: - Row per boss run

private struct BossRunRow: View {
    let run: BossRun
    @State private var isExpanded = false

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            // Summary row
            Button(action: { withAnimation(.easeInOut(duration: 0.15)) { isExpanded.toggle() } }) {
                HStack(spacing: 10) {
                    Image(systemName: isExpanded ? "chevron.down" : "chevron.right")
                        .font(.system(size: 8))
                        .foregroundStyle(Color.stsTextDim)
                        .frame(width: 10)

                    Text("F\(run.episode.effectiveFloor)")
                        .font(.stsValue)
                        .foregroundStyle(floorColor(run.episode.effectiveFloor))

                    Text(run.bossName)
                        .font(.stsBody)
                        .foregroundStyle(Color.stsRed)
                        .lineLimit(1)

                    Spacer()

                    Text("HP \(run.hpAtBoss)")
                        .font(.stsBody)
                        .foregroundStyle(Color.stsText)

                    if run.episode.won {
                        Text("WIN")
                            .font(.stsLabel)
                            .fontWeight(.bold)
                            .foregroundStyle(Color.stsAccent)
                    }

                    Text(String(run.episode.seed.prefix(8)))
                        .font(.stsLabel)
                        .foregroundStyle(Color.stsTextMuted)
                }
                .padding(.vertical, 4)
                .padding(.horizontal, 6)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)

            // Expandable detail
            if isExpanded {
                VStack(alignment: .leading, spacing: 4) {
                    ForEach(run.bossCombats, id: \.floor) { combat in
                        BossCombatDetail(combat: combat)
                    }
                }
                .padding(.leading, 20)
                .padding(.trailing, 6)
                .padding(.bottom, 6)
            }
        }
        .background(Color.stsBg.opacity(0.5))
        .clipShape(RoundedRectangle(cornerRadius: 4))
        .overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.stsBorderDim, lineWidth: 1))
    }

    private func floorColor(_ floor: Int) -> Color {
        if floor >= 51 { return Color.stsAccent }
        if floor >= 34 { return Color.stsBlue }
        if floor >= 17 { return Color.stsYellow }
        return Color.stsText
    }
}

// MARK: - Per-combat turn breakdown

private struct BossCombatDetail: View {
    let combat: Combat

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            // Combat header
            HStack(spacing: 8) {
                Text(combat.enemy)
                    .font(.stsBody)
                    .fontWeight(.medium)
                    .foregroundStyle(Color.stsRed)

                if let potions = combat.potionsUsed, potions > 0 {
                    Text("\(potions) pot")
                        .font(.stsLabel)
                        .foregroundStyle(Color.stsGold)
                        .padding(.horizontal, 4)
                        .padding(.vertical, 1)
                        .background(Color.stsGold.opacity(0.15))
                        .clipShape(RoundedRectangle(cornerRadius: 3))
                }

                Spacer()

                if let hpLost = combat.hpLost {
                    Text("-\(hpLost) HP")
                        .font(.stsLabel)
                        .foregroundStyle(hpLost > 30 ? Color.stsRed : Color.stsOrange)
                }

                if let turns = combat.turns {
                    Text("\(turns)T")
                        .font(.stsLabel)
                        .foregroundStyle(Color.stsTextDim)
                }
            }

            // Turn-by-turn detail
            if let turns = combat.turnsDetail, !turns.isEmpty {
                ForEach(Array(turns.enumerated()), id: \.offset) { idx, turn in
                    TurnRow(turn: turn, index: idx)
                }
            }
        }
        .padding(.vertical, 4)
    }
}

// MARK: - Single turn row

private struct TurnRow: View {
    let turn: TurnDetail
    let index: Int

    var body: some View {
        HStack(alignment: .top, spacing: 6) {
            // Turn number + stance dot
            HStack(spacing: 3) {
                Text("T\(turn.turn ?? (index + 1))")
                    .font(.stsLabel)
                    .foregroundStyle(Color.stsTextDim)
                    .frame(width: 20, alignment: .trailing)

                Circle()
                    .fill(stanceColor)
                    .frame(width: 6, height: 6)
            }

            // Cards played as compact chips
            if let cards = turn.cards, !cards.isEmpty {
                cardChips(cards)
            }

            Spacer()

            // Stats cluster
            HStack(spacing: 6) {
                if let hp = turn.playerHp {
                    Text("\(hp)hp")
                        .font(.stsLabel)
                        .foregroundStyle(hp < 20 ? Color.stsRed : Color.stsText)
                }

                if let block = turn.playerBlock, block > 0 {
                    Text("\(block)bl")
                        .font(.stsLabel)
                        .foregroundStyle(Color.stsBlue)
                }

                if let energy = turn.energyLeft, energy > 0 {
                    Text("\(energy)e")
                        .font(.stsLabel)
                        .foregroundStyle(Color.stsYellow)
                }

                if let unplayed = turn.playableUnplayed, unplayed > 0 {
                    Text("\(unplayed)skip")
                        .font(.stsLabel)
                        .foregroundStyle(Color.stsOrange)
                }
            }
        }
        .padding(.vertical, 1)
    }

    private var stanceColor: Color {
        switch turn.stance?.lowercased() {
        case "wrath": return Color.stanceWrath
        case "calm": return Color.stanceCalm
        case "divinity": return Color.stanceDivinity
        default: return Color.stanceNeutral
        }
    }

    private func cardChips(_ cards: [String]) -> some View {
        // Compact: show up to 6 cards, then "+N"
        let display = Array(cards.prefix(6))
        let overflow = cards.count - display.count

        return HStack(spacing: 2) {
            ForEach(Array(display.enumerated()), id: \.offset) { _, card in
                Text(shortCardName(card))
                    .font(.stsLabel)
                    .foregroundStyle(Color.stsText)
                    .padding(.horizontal, 3)
                    .padding(.vertical, 1)
                    .background(Color.stsBorderDim)
                    .clipShape(RoundedRectangle(cornerRadius: 2))
            }
            if overflow > 0 {
                Text("+\(overflow)")
                    .font(.stsLabel)
                    .foregroundStyle(Color.stsTextMuted)
            }
        }
    }

    /// Shorten card names for compact display: "Eruption+" -> "Erupt+", etc.
    private func shortCardName(_ name: String) -> String {
        let upgraded = name.hasSuffix("+")
        let base = upgraded ? String(name.dropLast()) : name
        let short: String
        if base.count > 7 {
            short = String(base.prefix(6))
        } else {
            short = base
        }
        return upgraded ? short + "+" : short
    }
}
