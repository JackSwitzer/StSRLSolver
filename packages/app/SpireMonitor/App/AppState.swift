import Foundation

enum AppView: String, CaseIterable, Identifiable {
    case live = "Run"
    case analysis = "Benchmarks"
    case frontier = "Frontier"

    var id: String { rawValue }

    var icon: String {
        switch self {
        case .live: "play.circle.fill"
        case .analysis: "chart.bar.xaxis"
        case .frontier: "doc.text.magnifyingglass"
        }
    }
}

@MainActor @Observable
final class AppState {
    var selectedView: AppView = .live
    var selectedArtifactEpisode: LocatedEpisodeLog?
    var selectedArtifactStepIndex: Int = 0

    let config = AppConfig()
    let store = DataStore()
    var poller: StatusPoller?
    var sysMonitor: SystemMonitor?

    func startPolling() {
        let p = StatusPoller(config: config, store: store)
        poller = p
        Task { await p.start() }

        let m = SystemMonitor(store: store)
        sysMonitor = m
        Task { await m.start() }

        Task { await loadArtifacts() }
    }

    func stopPolling() {
        Task {
            await poller?.stop()
            await sysMonitor?.stop()
        }
    }

    func loadArtifacts() async {
        async let manifest = ManifestLoader.load(from: config.logsPath)
        async let frontier = FrontierReportLoader.load(from: config.logsPath)
        async let benchmarkReports = BenchmarkReportLoader.loadAll(from: config.logsPath)
        async let artifactEpisodes = ArtifactEpisodeLogLoader.loadAll(from: config.logsPath)
        async let events = EventStreamLoader.load(from: config.logsPath)
        async let metrics = MetricStreamLoader.load(from: config.logsPath)

        store.runManifest = await manifest
        store.frontierReport = await frontier
        store.benchmarkReports = await benchmarkReports
        store.artifactEpisodes = await artifactEpisodes
        store.eventStream = await events
        store.metricStream = await metrics

        if selectedArtifactEpisode == nil, let first = store.artifactEpisodes.first {
            selectedArtifactEpisode = first
            selectedArtifactStepIndex = first.episode.frontierSteps.first?.stepIndex ?? 0
        }
    }

    func refresh() {
        Task {
            await loadArtifacts()
        }
    }
}
