import SwiftUI

enum StatsMode: String, CaseIterable {
    case last100 = "Last 100"
    case allTime = "Full Data"
}

struct LiveView: View {
    @Environment(AppState.self) private var appState
    @State private var statsMode: StatsMode = .last100

    private var store: DataStore { appState.store }

    private var floorCurveData: [Double] {
        if statsMode == .last100 {
            return Array(store.floorCurve.suffix(100))
        }
        return store.floorCurve
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
                        FloorCurveChart(data: floorCurveData)
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

            // Bottom: System stats (always visible, pinned)
            Divider().background(Color.stsBorder)
            SystemStatsBar(current: store.systemStats, history: store.systemHistory)
        }
    }
}
