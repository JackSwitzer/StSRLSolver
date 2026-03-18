import SwiftUI

struct LiveView: View {
    @Environment(AppState.self) private var appState

    private var store: DataStore { appState.store }

    var body: some View {
        ScrollView {
            VStack(spacing: 12) {
                StatusBarView(status: store.status, isLive: store.isLive)

                // Charts row
                HStack(spacing: 12) {
                    FloorCurveChart(data: store.floorCurve)
                        .frame(minHeight: 200)
                        .sectionCard()

                    DeathAnalysisChart(deaths: Array(store.deathStats.prefix(8)))
                        .frame(minHeight: 200)
                        .sectionCard()
                }

                // Tables row
                HStack(alignment: .top, spacing: 12) {
                    TopRunsTable(episodes: Array(store.topRunsSorted.prefix(10)))
                        .sectionCard()

                    PerformancePanelView(performance: store.performanceByRoom)
                        .sectionCard()
                }

                // Workers (only if data exists)
                if !store.workers.isEmpty {
                    WorkerGridView(workers: store.workers)
                        .sectionCard()
                }
            }
            .padding(16)
        }
    }
}
