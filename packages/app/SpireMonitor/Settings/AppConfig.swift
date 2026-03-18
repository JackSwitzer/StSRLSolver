import Foundation

@Observable
final class AppConfig {
    var logsPath: URL
    var archivedRunsPath: URL

    private static let logsPathKey = "SpireMonitor.logsPath"

    init() {
        // Resolution: UserDefaults > .spire-monitor.json > fallback
        if let saved = UserDefaults.standard.string(forKey: Self.logsPathKey) {
            let savedURL = URL(filePath: saved)
            self.logsPath = savedURL
            self.archivedRunsPath = savedURL.deletingLastPathComponent().appending(path: "runs")
        } else if let (logs, runs) = Self.loadConfigFile() {
            self.logsPath = logs
            self.archivedRunsPath = runs
        } else {
            let fallback = FileManager.default.homeDirectoryForCurrentUser
                .appending(path: "Desktop/SlayTheSpireRL")
            self.logsPath = fallback.appending(path: "logs/weekend-run")
            self.archivedRunsPath = fallback.appending(path: "logs/runs")
        }
    }

    func setLogsPath(_ url: URL) {
        logsPath = url
        UserDefaults.standard.set(url.path(), forKey: Self.logsPathKey)
    }

    func resetToDefault() {
        UserDefaults.standard.removeObject(forKey: Self.logsPathKey)
        if let (logs, runs) = Self.loadConfigFile() {
            self.logsPath = logs
            self.archivedRunsPath = runs
        }
    }

    private static func loadConfigFile() -> (URL, URL)? {
        // Check SPIRE_MONITOR_ROOT env var first
        let rootCandidates: [URL]
        if let envRoot = ProcessInfo.processInfo.environment["SPIRE_MONITOR_ROOT"] {
            rootCandidates = [URL(filePath: envRoot)]
        } else {
            // Walk up from executable to find .spire-monitor.json
            var candidates: [URL] = []
            var dir = URL(filePath: FileManager.default.currentDirectoryPath)
            for _ in 0..<10 {
                candidates.append(dir)
                dir = dir.deletingLastPathComponent()
            }
            rootCandidates = candidates
        }

        for root in rootCandidates {
            let configURL = root.appending(path: ".spire-monitor.json")
            guard let data = try? Data(contentsOf: configURL),
                  let config = try? JSONDecoder().decode(ConfigFile.self, from: data) else {
                continue
            }
            let projectRoot = config.projectRoot == "." ? root : URL(filePath: config.projectRoot, relativeTo: root)
            let logs = projectRoot.appending(path: config.logsPath)
            let runs = projectRoot.appending(path: config.archivedRunsPath)
            return (logs, runs)
        }
        return nil
    }
}

private struct ConfigFile: Codable {
    let projectRoot: String
    let logsPath: String
    let archivedRunsPath: String

    enum CodingKeys: String, CodingKey {
        case projectRoot = "project_root"
        case logsPath = "logs_path"
        case archivedRunsPath = "archived_runs_path"
    }
}
