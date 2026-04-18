import Foundation

@MainActor @Observable
final class DataStore {
    var runManifest: TrainingRunArtifactManifest?
    var frontierReport: FrontierReportArtifact?
    var seedValidationReports: [LocatedSeedValidationReport] = []
    var checkpointComparisons: [LocatedSeedValidationComparison] = []
    var benchmarkReports: [LocatedBenchmarkReport] = []
    var artifactEpisodes: [LocatedEpisodeLog] = []
    var eventStream: [TrainingEventRecord] = []
    var metricStream: [TrainingMetricRecord] = []
    var systemStats: SystemStats?
    var systemHistory: [SystemStats] = []
    var recordedRunReplay: RecordedRunReplayReportArtifact?

    var hasManifest: Bool {
        runManifest != nil
    }

    var latestEvent: TrainingEventRecord? {
        eventStream.last
    }

    var latestMetric: TrainingMetricRecord? {
        metricStream.last
    }

    var currentBenchmarkReport: BenchmarkReportArtifact? {
        benchmarkReports.first?.report
    }

    var currentSeedValidationReport: SeedValidationReportArtifact? {
        seedValidationReports.first?.report
    }

    var currentCheckpointComparison: SeedValidationComparisonArtifact? {
        checkpointComparisons.first?.report
    }

    var currentFrontierRanking: [FrontierPointArtifact] {
        guard let frontierReport else { return [] }
        let lookup = Dictionary(uniqueKeysWithValues: frontierReport.points.map { ($0.label, $0) })
        return frontierReport.ranking.compactMap { lookup[$0] }
    }
}

struct SystemStats: Identifiable {
    let id = UUID()
    let timestamp: Date
    let cpuPercent: Double
    let memoryUsedGB: Double
    let memoryTotalGB: Double
    let gpuPercent: Double?
}
