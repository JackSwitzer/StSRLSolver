import Foundation

enum AppView: String, CaseIterable, Identifiable {
    case live = "Live"
    case analysis = "Analysis"
    case detail = "Detail"
    case replay = "Replay"

    var id: String { rawValue }

    var icon: String {
        switch self {
        case .live: "play.circle.fill"
        case .analysis: "chart.bar.xaxis"
        case .detail: "doc.text.magnifyingglass"
        case .replay: "film"
        }
    }
}

@MainActor @Observable
final class AppState {
    var selectedView: AppView = .live
    var selectedEpisode: Episode?

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

        Task { await loadEpisodes() }
    }

    func stopPolling() {
        Task {
            await poller?.stop()
            await sysMonitor?.stop()
        }
    }

    func loadEpisodes() async {
        let recent = await EpisodeLoader.loadRecent(from: config.logsPath)
        let top = await EpisodeLoader.loadTop(from: config.logsPath)
        store.recentEpisodes = recent
        store.topEpisodes = top
    }

    func refresh() {
        Task { await loadEpisodes() }
    }
}
