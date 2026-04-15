import SwiftUI

struct LiveView: View {
    @Environment(AppState.self) private var appState

    private var store: DataStore { appState.store }

    var body: some View {
        VStack(spacing: 0) {
            ScrollView {
                VStack(spacing: 16) {
                    ActiveRunSummaryView(
                        manifest: store.runManifest,
                        latestEvent: store.latestEvent,
                        latestMetric: store.latestMetric,
                        frontierReport: store.frontierReport,
                        benchmarkReport: store.currentBenchmarkReport
                    )
                    .padding(.horizontal, 16)

                    BenchmarkSliceDashboardView(
                        reports: store.benchmarkReports,
                        frontier: store.frontierReport
                    )
                    .padding(.horizontal, 16)

                    ArtifactStreamsView(
                        events: store.eventStream,
                        metrics: store.metricStream
                    )
                    .padding(.horizontal, 16)
                }
                .padding(.vertical, 16)
            }

            Divider().background(Color.stsBorder)
            SystemStatsBar(current: store.systemStats, history: store.systemHistory)
                .background(Color.stsCard)
        }
        .background(Color.stsBg)
    }
}
