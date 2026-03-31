import SwiftUI
import Charts

struct ReplayView: View {
    @Environment(AppState.self) private var appState

    private var store: DataStore { appState.store }

    private var episodes: [Episode] {
        let all = store.topEpisodes + store.recentEpisodes
        var seen = Set<String>()
        return all.filter { seen.insert($0.seed).inserted }
            .sorted { $0.effectiveFloor > $1.effectiveFloor }
    }

    var body: some View {
        HSplitView {
            // LEFT: Episode list
            VStack(alignment: .leading, spacing: 0) {
                SectionHeader(title: "Game Replays")
                    .padding(12)

                if episodes.isEmpty {
                    VStack(spacing: 8) {
                        Image(systemName: "film")
                            .font(.system(size: 30))
                            .foregroundStyle(Color.stsTextMuted)
                        Text("No episodes loaded")
                            .font(.stsBody)
                            .foregroundStyle(Color.stsTextMuted)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else {
                    ScrollView {
                        LazyVStack(spacing: 0) {
                            ForEach(episodes) { ep in
                                EpisodeRow(episode: ep, isSelected: appState.selectedEpisode?.seed == ep.seed)
                                    .contentShape(Rectangle())
                                    .onTapGesture {
                                        appState.selectedEpisode = ep
                                    }
                                    .padding(.horizontal, 8)
                                    .padding(.vertical, 2)
                                    .background(
                                        appState.selectedEpisode?.seed == ep.seed
                                            ? Color.stsAccent.opacity(0.1)
                                            : Color.clear
                                    )

                                Divider().background(Color.stsBorderDim)
                            }
                        }
                    }
                }
            }
            .frame(minWidth: 280, maxWidth: 350)
            .background(Color.stsBg)

            // RIGHT: Detail
            if let selected = appState.selectedEpisode {
                EpisodeDetailView(episode: selected)
            } else {
                VStack(spacing: 12) {
                    Image(systemName: "hand.point.left.fill")
                        .font(.system(size: 30))
                        .foregroundStyle(Color.stsTextMuted)
                    Text("Select an episode to view its replay")
                        .font(.stsBody)
                        .foregroundStyle(Color.stsTextDim)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .background(Color.stsBg)
            }
        }
        .background(Color.stsBg)
    }
}

// MARK: - Episode Row

private struct EpisodeRow: View {
    let episode: Episode
    let isSelected: Bool

    var body: some View {
        HStack(spacing: 10) {
            // Floor badge
            Text("F\(episode.effectiveFloor)")
                .font(.stsValue)
                .foregroundStyle(floorColor(episode.effectiveFloor))
                .frame(width: 40, alignment: .trailing)

            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    Text(episode.seed.prefix(12))
                        .font(.stsBody)
                        .foregroundStyle(Color.stsText)

                    if episode.won {
                        Text("WIN")
                            .font(.system(size: 9, weight: .bold, design: .monospaced))
                            .foregroundStyle(Color.stsAccent)
                            .padding(.horizontal, 4)
                            .padding(.vertical, 1)
                            .background(Color.stsAccent.opacity(0.15))
                            .clipShape(RoundedRectangle(cornerRadius: 3))
                    }
                }

                HStack(spacing: 8) {
                    if !episode.won, let killer = episode.deathEnemy {
                        Text(killer)
                            .font(.stsLabel)
                            .foregroundStyle(Color.stsRed)
                            .lineLimit(1)
                    }
                    Text(Fmt.duration(episode.effectiveDuration))
                        .font(.stsLabel)
                        .foregroundStyle(Color.stsTextDim)
                }
            }

            Spacer()

            // HP
            Text("\(episode.effectiveHp)/\(episode.effectiveMaxHp)")
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextDim)
        }
        .padding(.vertical, 4)
    }

    private func floorColor(_ floor: Int) -> Color {
        if floor >= 51 { return Color.stsAccent }
        if floor >= 34 { return Color.stsBlue }
        if floor >= 17 { return Color.stsYellow }
        return Color.stsText
    }
}

// MARK: - Episode Detail

struct EpisodeDetailView: View {
    let episode: Episode

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                // Header
                headerSection

                // HP Timeline
                if let hpHistory = episode.hpHistory, hpHistory.count > 1 {
                    hpTimelineChart(hpHistory)
                }

                // Combats
                if let combats = episode.combats, !combats.isEmpty {
                    combatTimeline(combats)
                }

                // Deck
                if let deck = episode.deckFinal, !deck.isEmpty {
                    deckSection(deck)
                }

                // Relics
                if let relics = episode.relicsFinal, !relics.isEmpty {
                    relicSection(relics)
                }

                // Rewards
                rewardSection
            }
            .padding(16)
        }
        .background(Color.stsBg)
    }

    // MARK: - Header

    private var headerSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 12) {
                Text("Seed: \(episode.seed)")
                    .font(.stsTitle)
                    .foregroundStyle(Color.stsText)

                Spacer()

                if episode.won {
                    Text("VICTORY")
                        .font(.system(size: 14, weight: .bold, design: .monospaced))
                        .foregroundStyle(Color.stsAccent)
                        .padding(.horizontal, 10)
                        .padding(.vertical, 4)
                        .background(Color.stsAccent.opacity(0.15))
                        .clipShape(RoundedRectangle(cornerRadius: 6))
                } else {
                    Text("DEFEAT")
                        .font(.system(size: 14, weight: .bold, design: .monospaced))
                        .foregroundStyle(Color.stsRed)
                        .padding(.horizontal, 10)
                        .padding(.vertical, 4)
                        .background(Color.stsRed.opacity(0.15))
                        .clipShape(RoundedRectangle(cornerRadius: 6))
                }
            }

            HStack(spacing: 20) {
                statPill("Floor", value: "\(episode.effectiveFloor)")
                statPill("HP", value: "\(episode.effectiveHp)/\(episode.effectiveMaxHp)")
                statPill("Duration", value: Fmt.duration(episode.effectiveDuration))
                if let combats = episode.combats {
                    statPill("Combats", value: "\(combats.count)")
                }
            }

            if !episode.won, let killer = episode.deathEnemy {
                HStack(spacing: 6) {
                    Image(systemName: "xmark.circle.fill")
                        .foregroundStyle(Color.stsRed)
                    Text("Killed by \(killer)")
                        .font(.stsBody)
                        .foregroundStyle(Color.stsRed)
                    if let room = episode.deathRoom {
                        Text("(\(room))")
                            .font(.stsLabel)
                            .foregroundStyle(Color.stsTextDim)
                    }
                }
            }
        }
        .sectionCard()
    }

    private func statPill(_ label: String, value: String) -> some View {
        VStack(spacing: 2) {
            Text(label)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextDim)
            Text(value)
                .font(.stsValue)
                .foregroundStyle(Color.stsText)
        }
    }

    // MARK: - HP Timeline

    private func hpTimelineChart(_ hpHistory: [Int]) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            SectionHeader(title: "HP Timeline")

            Chart {
                ForEach(Array(hpHistory.enumerated()), id: \.offset) { i, hp in
                    AreaMark(
                        x: .value("Floor", i),
                        y: .value("HP", hp)
                    )
                    .foregroundStyle(Color.stsRed.opacity(0.15))
                    .interpolationMethod(.catmullRom)

                    LineMark(
                        x: .value("Floor", i),
                        y: .value("HP", hp)
                    )
                    .foregroundStyle(Color.stsRed)
                    .interpolationMethod(.catmullRom)
                }
            }
            .frame(height: 120)
            .chartYAxis {
                AxisMarks(position: .leading) { _ in
                    AxisValueLabel().font(.stsLabel).foregroundStyle(Color.stsTextDim)
                    AxisGridLine().foregroundStyle(Color.stsBorderDim)
                }
            }
            .chartXAxis {
                AxisMarks { _ in
                    AxisValueLabel().font(.stsLabel).foregroundStyle(Color.stsTextDim)
                }
            }
        }
        .sectionCard()
    }

    // MARK: - Combat Timeline

    private func combatTimeline(_ combats: [Combat]) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            SectionHeader(title: "Combat Timeline")

            ForEach(Array(combats.enumerated()), id: \.offset) { _, combat in
                CombatRow(combat: combat)
            }
        }
        .sectionCard()
    }

    // MARK: - Deck

    private func deckSection(_ deck: [String]) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            SectionHeader(title: "Final Deck (\(deck.count) cards)")

            let grouped = Dictionary(grouping: deck, by: { $0 })
                .map { (card: $0.key, count: $0.value.count) }
                .sorted { $0.count > $1.count }

            LazyVGrid(columns: [GridItem(.adaptive(minimum: 120), spacing: 6)], spacing: 6) {
                ForEach(grouped, id: \.card) { item in
                    HStack(spacing: 4) {
                        Text(item.card)
                            .font(.stsBody)
                            .foregroundStyle(Color.stsText)
                            .lineLimit(1)
                        if item.count > 1 {
                            Text("x\(item.count)")
                                .font(.stsLabel)
                                .foregroundStyle(Color.stsTextDim)
                        }
                    }
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.stsBorderDim)
                    .clipShape(RoundedRectangle(cornerRadius: 4))
                }
            }
        }
        .sectionCard()
    }

    // MARK: - Relics

    private func relicSection(_ relics: [String]) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            SectionHeader(title: "Relics (\(relics.count))")

            LazyVGrid(columns: [GridItem(.adaptive(minimum: 130), spacing: 6)], spacing: 6) {
                ForEach(relics, id: \.self) { relic in
                    Text(relic)
                        .font(.stsBody)
                        .foregroundStyle(Color.stsGold)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(Color.stsGold.opacity(0.1))
                        .clipShape(RoundedRectangle(cornerRadius: 4))
                }
            }
        }
        .sectionCard()
    }

    // MARK: - Rewards

    private var rewardSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            SectionHeader(title: "Rewards")

            HStack(spacing: 20) {
                if let total = episode.totalReward {
                    statPill("Total", value: String(format: "%.2f", total))
                }
                if let pbrs = episode.pbrsReward {
                    statPill("PBRS", value: String(format: "%.2f", pbrs))
                }
                if let event = episode.eventReward {
                    statPill("Event", value: String(format: "%.2f", event))
                }
            }
        }
        .sectionCard()
    }
}

// MARK: - Combat Row

private struct CombatRow: View {
    let combat: Combat
    @State private var isExpanded = false

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            // Summary row
            Button(action: { withAnimation(.easeInOut(duration: 0.2)) { isExpanded.toggle() } }) {
                HStack(spacing: 10) {
                    // Floor
                    Text("F\(combat.floor)")
                        .font(.stsValue)
                        .foregroundStyle(Color.stsTextDim)
                        .frame(width: 36, alignment: .trailing)

                    // Room type badge
                    Text(combat.roomCategory.rawValue)
                        .font(.system(size: 9, weight: .bold, design: .monospaced))
                        .foregroundStyle(roomColor(combat.roomCategory))
                        .padding(.horizontal, 5)
                        .padding(.vertical, 2)
                        .background(roomColor(combat.roomCategory).opacity(0.12))
                        .clipShape(RoundedRectangle(cornerRadius: 3))

                    // Enemy name
                    Text(combat.enemy)
                        .font(.stsBody)
                        .foregroundStyle(Color.stsText)
                        .lineLimit(1)

                    Spacer()

                    // Stats
                    if let hpLost = combat.hpLost {
                        HStack(spacing: 3) {
                            Image(systemName: "heart.fill")
                                .font(.system(size: 8))
                                .foregroundStyle(Color.stsRed)
                            Text("-\(hpLost)")
                                .font(.stsLabel)
                                .foregroundStyle(Color.stsRed)
                        }
                    }

                    if let turns = combat.turns {
                        Text("\(turns)T")
                            .font(.stsLabel)
                            .foregroundStyle(Color.stsTextDim)
                    }

                    if let cards = combat.cardsPlayed {
                        Text("\(cards)C")
                            .font(.stsLabel)
                            .foregroundStyle(Color.stsTextDim)
                    }

                    Image(systemName: isExpanded ? "chevron.up" : "chevron.down")
                        .font(.system(size: 9))
                        .foregroundStyle(Color.stsTextMuted)
                }
            }
            .buttonStyle(.plain)
            .padding(.vertical, 6)

            // Expanded turn detail
            if isExpanded, let turns = combat.turnsDetail, !turns.isEmpty {
                VStack(alignment: .leading, spacing: 4) {
                    ForEach(turns, id: \.turn) { turn in
                        TurnRow(turn: turn)
                    }
                }
                .padding(.leading, 46)
                .padding(.bottom, 6)
            }
        }

        Divider().background(Color.stsBorderDim)
    }

    private func roomColor(_ cat: RoomCategory) -> Color {
        switch cat {
        case .monster: return Color.stsTextDim
        case .elite: return Color.stsOrange
        case .boss: return Color.stsRed
        }
    }
}

// MARK: - Turn Row

private struct TurnRow: View {
    let turn: TurnDetail

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Text("T\(turn.turn ?? 0)")
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)
                .frame(width: 24, alignment: .trailing)

            if let cards = turn.cards, !cards.isEmpty {
                Text(cards.joined(separator: ", "))
                    .font(.stsBody)
                    .foregroundStyle(Color.stsText)
                    .lineLimit(2)
            } else {
                Text("(no cards)")
                    .font(.stsBody)
                    .foregroundStyle(Color.stsTextMuted)
            }

            Spacer()

            HStack(spacing: 8) {
                if let hp = turn.playerHp {
                    Text("\(hp)HP")
                        .font(.stsLabel)
                        .foregroundStyle(Color.stsRed)
                }
                if let block = turn.playerBlock, block > 0 {
                    Text("\(block)B")
                        .font(.stsLabel)
                        .foregroundStyle(Color.stsBlue)
                }
                if let stance = turn.stance, stance != "NONE" {
                    Text(stance)
                        .font(.system(size: 8, weight: .bold, design: .monospaced))
                        .foregroundStyle(stanceColor(stance))
                        .padding(.horizontal, 3)
                        .padding(.vertical, 1)
                        .background(stanceColor(stance).opacity(0.12))
                        .clipShape(RoundedRectangle(cornerRadius: 2))
                }
                if let energy = turn.energyLeft {
                    Text("\(energy)E")
                        .font(.stsLabel)
                        .foregroundStyle(Color.stsGold)
                }
            }
        }
    }

    private func stanceColor(_ stance: String) -> Color {
        switch stance.uppercased() {
        case "WRATH": return .stanceWrath
        case "CALM": return .stanceCalm
        case "DIVINITY": return .stanceDivinity
        default: return .stanceNeutral
        }
    }
}
