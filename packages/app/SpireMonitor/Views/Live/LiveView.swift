import SwiftUI

enum StatsMode: String, CaseIterable {
    case last100 = "Last 100"
    case allTime = "All Time"
}

struct LiveView: View {
    @Environment(AppState.self) private var appState
    @State private var statsMode: StatsMode = .last100

    private var store: DataStore { appState.store }

    var body: some View {
        VStack(spacing: 0) {
            // Status bar + rolling average toggle
            HStack {
                StatusBarView(status: store.status, isLive: store.isLive,
                              peakFloor: store.peakFloor, mode: statsMode)
                Spacer()
                Picker("", selection: $statsMode) {
                    ForEach(StatsMode.allCases, id: \.self) { mode in
                        Text(mode.rawValue).tag(mode)
                    }
                }
                .pickerStyle(.segmented)
                .frame(width: 180)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 8)

            // Main content: left charts / right tables
            HSplitView {
                // LEFT: Charts stack
                ScrollView {
                    VStack(spacing: 12) {
                        FloorCurveChart(data: store.floorCurve)
                            .frame(minHeight: 180)
                            .sectionCard()

                        LossChartsView(history: store.lossHistory)
                            .frame(minHeight: 180)
                            .sectionCard()

                        HyperparamGridView(status: store.status)
                            .sectionCard()
                    }
                    .padding(12)
                }
                .frame(minWidth: 400)

                // RIGHT: Tables + Workers
                ScrollView {
                    VStack(spacing: 12) {
                        TopRunsTable(episodes: Array(store.topRunsSorted.prefix(10)))
                            .sectionCard()

                        if !store.workers.isEmpty {
                            WorkerGridView(workers: store.workers)
                                .sectionCard()
                        }

                        DeathAnalysisChart(deaths: Array(store.deathStats.prefix(8)))
                            .frame(minHeight: 160)
                            .sectionCard()

                        PerformancePanelView(performance: store.performanceByRoom)
                            .sectionCard()
                    }
                    .padding(12)
                }
                .frame(minWidth: 350)
            }

            // Bottom: System stats bar
            Divider().background(Color.stsBorder)
            SystemStatsBar(current: store.systemStats, history: store.systemHistory)
        }
    }
}
