import SwiftUI

struct ArtifactAnalysisView: View {
    @Environment(AppState.self) private var appState

    private var store: DataStore { appState.store }

    var body: some View {
        ScrollView {
            VStack(spacing: 16) {
                ActiveRunSummaryView(
                    manifest: store.runManifest,
                    latestEvent: store.latestEvent,
                    latestMetric: store.latestMetric,
                    frontierReport: store.frontierReport,
                    benchmarkReport: store.currentBenchmarkReport,
                    seedValidationReport: store.currentSeedValidationReport,
                    checkpointComparison: store.currentCheckpointComparison,
                    seedValidationReportCount: store.seedValidationReports.count,
                    checkpointComparisonCount: store.checkpointComparisons.count
                )
                .padding(.horizontal, 16)

                SeedValidationReportView(
                    seedValidationReports: store.seedValidationReports,
                    checkpointComparisons: store.checkpointComparisons
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
        .background(Color.stsBg)
    }
}
