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

    private var floorCurveData: [Double] {
        let source: [Double]
        if floorCurveMode == .max {
            // Per-episode max floors from episodes
            source = store.topRunsSorted.map { Double($0.effectiveFloor) }
        } else {
            source = store.floorCurve
        }
        if statsMode == .last100 {
            return Array(source.suffix(100))
        }
        return source
    }

    private var floorCurveTitle: String {
        "Floor Curve (\(floorCurveMode.rawValue))"
    }

    var body: some View {
        VStack(spacing: 0) {
            // Status bar
            HStack(spacing: 8) {
                StatusBarView(status: store.status, isLive: store.isLive,
                              peakFloor: store.peakFloor, mode: statsMode)

                Button(action: {
                    statsMode = statsMode == .last100 ? .allTime : .last100
                }) {
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
                .help("Toggle between Last 100 games and Full Data")
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 6)

            // Main content: left charts / right tables
            HSplitView {
                // LEFT: Charts
                ScrollView {
                    VStack(spacing: 10) {
                        FloorCurveChart(data: floorCurveData, title: floorCurveTitle)
                            .frame(minHeight: 160)
                            .sectionCard()

                        LossChartsView(history: store.lossHistory)
                            .frame(minHeight: 160)
                            .sectionCard()

                        HyperparamGridView(status: store.status)
                            .sectionCard()

                        PerformancePanelView(performance: store.performanceByRoom)
                            .sectionCard()
                    }
                    .padding(10)
                }
                .frame(minWidth: 380)

                // RIGHT: Tables + Workers + Deaths + Performance in compact layout
                ScrollView {
                    VStack(spacing: 10) {
                        TopRunsTable(episodes: Array(store.topRunsSorted.prefix(10)))
                            .sectionCard()

                        WorkerGridView(workers: store.workers)
                            .sectionCard()

                        DeathAnalysisChart(deaths: Array(store.deathStats.prefix(8)))
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
                // Keybinds legend
                HStack(spacing: 16) {
                    keybind("M", action: "Floor \(floorCurveMode == .average ? "Max" : "Avg")")
                    keybind("R", action: "Refresh")
                    keybind("S", action: "Toggle Stats")
                }
                .padding(.horizontal, 14)
                .padding(.vertical, 6)

                Divider().frame(height: 30).background(Color.stsBorderDim)

                SystemStatsBar(current: store.systemStats, history: store.systemHistory)
            }
            .background(Color.stsCard)
        }
        .onKeyPress("m") {
            floorCurveMode = floorCurveMode == .average ? .max : .average
            return .handled
        }
        .onKeyPress("s") {
            statsMode = statsMode == .last100 ? .allTime : .last100
            return .handled
        }
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
