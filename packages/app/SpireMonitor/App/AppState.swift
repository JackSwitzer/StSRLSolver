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
        let bundle = await MonitorArtifactLoader.load(from: config.logsPath)

        store.runManifest = bundle.manifest
        store.frontierReport = bundle.frontier
        store.seedValidationReports = bundle.seedValidationReports
        store.checkpointComparisons = bundle.checkpointComparisons
        store.benchmarkReports = bundle.benchmarkReports
        store.artifactEpisodes = bundle.artifactEpisodes
        store.eventStream = bundle.events
        store.metricStream = bundle.metricStream

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
