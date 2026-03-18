import SwiftUI

enum StatsMode: String, CaseIterable {
    case last100 = "Last 100"
    case allTime = "Full Data"
}

enum FloorCurveMode: String {
    case average = "Avg"
    case max = "Max"
}

struct LiveView: View {
    @Environment(AppState.self) private var appState
    @State private var statsMode: StatsMode = .last100
    @State private var floorCurveMode: FloorCurveMode = .average

    private var store: DataStore { appState.store }

    // Mode-aware data sources
    private var activeEpisodes: [Episode] {
        if statsMode == .allTime {
            // Merge both sources, deduplicate by seed
            var seen = Set<String>()
            var merged: [Episode] = []
            for ep in store.topEpisodes + store.recentEpisodes {
                if seen.insert(ep.seed).inserted { merged.append(ep) }
            }
            return merged
        }
        return store.recentEpisodes.isEmpty ? store.topEpisodes : store.recentEpisodes
    }

    private var floorCurveData: [Double] {
        let source: [Double]
        if floorCurveMode == .max {
            source = activeEpisodes.map { Double($0.effectiveFloor) }
        } else {
            source = store.floorCurve
        }
        if statsMode == .last100 { return Array(source.suffix(100)) }
        return source
    }

    private var deathData: [(enemy: String, count: Int)] {
        var counts: [String: Int] = [:]
        for ep in activeEpisodes where !ep.won {
            counts[ep.deathEnemy ?? "Unknown", default: 0] += 1
        }
        return counts.sorted { $0.value > $1.value }.map { (enemy: $0.key, count: $0.value) }
    }

    private var topRuns: [Episode] {
        activeEpisodes.sorted { $0.effectiveFloor > $1.effectiveFloor }
    }

    private var performanceData: [RoomCategory: RoomPerformance] {
        var grouped: [RoomCategory: [Combat]] = [:]
        for ep in activeEpisodes {
            for combat in ep.combats ?? [] {
                grouped[combat.roomCategory, default: []].append(combat)
            }
        }
        var result: [RoomCategory: RoomPerformance] = [:]
        for (cat, combats) in grouped {
            let turns = combats.compactMap(\.turns)
            let hpLost = combats.compactMap(\.hpLost)
            let avgT = turns.isEmpty ? 0 : Double(turns.reduce(0, +)) / Double(turns.count)
            let avgHP = hpLost.isEmpty ? 0 : Double(hpLost.reduce(0, +)) / Double(hpLost.count)
            let potR = combats.isEmpty ? 0 :
                Double(combats.filter { ($0.potionsUsed ?? 0) > 0 }.count) / Double(combats.count)
            result[cat] = RoomPerformance(count: combats.count, avgTurns: avgT, avgHpLost: avgHP, potionRate: potR)
        }
        return result
    }

    var body: some View {
        VStack(spacing: 0) {
            // Status bar
            HStack(spacing: 8) {
                StatusBarView(status: store.status, isLive: store.isLive,
                              peakFloor: store.peakFloor, mode: statsMode)

                Button(action: toggleStats) {
                    Text(statsMode.rawValue)
                        .font(.stsBody)
                        .fontWeight(.medium)
                        .foregroundStyle(Color.stsAccent)
                        .padding(.horizontal, 12)
                        .padding(.vertical, 4)
                        .background(Color.stsAccent.opacity(0.15))
                        .clipShape(RoundedRectangle(cornerRadius: 4))
                }
                .buttonStyle(.plain)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 6)

            // Main content: left charts / right tables
            HSplitView {
                // LEFT: Charts
                ScrollView {
                    VStack(spacing: 10) {
                        FloorCurveChart(data: floorCurveData,
                                        title: "Floor Curve (\(floorCurveMode.rawValue))")
                            .frame(minHeight: 160)
                            .sectionCard()

                        LossChartsView(history: store.lossHistory)
                            .frame(minHeight: 160)
                            .sectionCard()

                        HyperparamGridView(status: store.status)
                            .sectionCard()

                        PerformancePanelView(performance: performanceData)
                            .sectionCard()
                    }
                    .padding(10)
                }
                .frame(minWidth: 380)

                // RIGHT: Tables + Workers + Deaths
                ScrollView {
                    VStack(spacing: 10) {
                        TopRunsTable(episodes: Array(topRuns.prefix(10)))
                            .sectionCard()

                        WorkerGridView(workers: store.workers)
                            .sectionCard()

                        DeathAnalysisChart(deaths: Array(deathData.prefix(5)))
                            .frame(minHeight: 140)
                            .sectionCard()
                    }
                    .padding(10)
                }
                .frame(minWidth: 320)
            }

            // Bottom: keybinds + system stats
            Divider().background(Color.stsBorder)
            HStack(spacing: 0) {
                HStack(spacing: 16) {
                    keybind("M", action: "Floor \(floorCurveMode == .average ? "Max" : "Avg")")
                    keybind("F", action: statsMode == .last100 ? "Full Data" : "Last 100")
                }
                .padding(.horizontal, 14)
                .padding(.vertical, 6)

                Divider().frame(height: 30).background(Color.stsBorderDim)

                SystemStatsBar(current: store.systemStats, history: store.systemHistory)
            }
            .background(Color.stsCard)
        }
        .focusable()
        .focusEffectDisabled()
        .onKeyPress("m", action: { toggleFloorMode(); return .handled })
        .onKeyPress("f", action: { toggleStats(); return .handled })
    }

    private func toggleFloorMode() {
        floorCurveMode = floorCurveMode == .average ? .max : .average
    }

    private func toggleStats() {
        statsMode = statsMode == .last100 ? .allTime : .last100
    }

    private func keybind(_ key: String, action: String) -> some View {
        HStack(spacing: 4) {
            Text(key)
                .font(.system(size: 10, weight: .bold, design: .monospaced))
                .foregroundStyle(Color.stsText)
                .padding(.horizontal, 4)
                .padding(.vertical, 1)
                .background(Color.stsBorderDim)
                .clipShape(RoundedRectangle(cornerRadius: 3))
            Text(action)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextDim)
        }
    }
}
