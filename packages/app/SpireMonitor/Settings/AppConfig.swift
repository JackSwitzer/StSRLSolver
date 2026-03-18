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
            // Auto-discover most recent active run
            self.logsPath = Self.findActiveRun(near: logs) ?? logs
            self.archivedRunsPath = runs
        } else {
            let fallback = FileManager.default.homeDirectoryForCurrentUser
                .appending(path: "Desktop/SlayTheSpireRL")
            let defaultLogs = fallback.appending(path: "logs/weekend-run")
            self.logsPath = Self.findActiveRun(near: defaultLogs) ?? defaultLogs
            self.archivedRunsPath = fallback.appending(path: "logs/runs")
        }
    }

    func setLogsPath(_ url: URL) {
        logsPath = url
        archivedRunsPath = url.deletingLastPathComponent().appending(path: "runs")
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

    /// Scan sibling directories for the most recently modified status.json
    private static func findActiveRun(near defaultPath: URL) -> URL? {
        let logsParent = defaultPath.deletingLastPathComponent() // e.g. logs/
        let fm = FileManager.default

        var candidates: [(url: URL, mtime: Date)] = []

        // Check the default path itself
        let defaultStatus = defaultPath.appending(path: "status.json")
        if let attrs = try? fm.attributesOfItem(atPath: defaultStatus.path()),
           let mtime = attrs[.modificationDate] as? Date {
            candidates.append((defaultPath, mtime))
        }

        // Scan logs/training/run_*/ and logs/runs/run_*/
        for subdir in ["training", "runs"] {
            let scanDir = logsParent.appending(path: subdir)
            guard let contents = try? fm.contentsOfDirectory(at: scanDir, includingPropertiesForKeys: nil) else { continue }
            for dir in contents where dir.hasDirectoryPath {
                let statusFile = dir.appending(path: "status.json")
                if let attrs = try? fm.attributesOfItem(atPath: statusFile.path()),
                   let mtime = attrs[.modificationDate] as? Date {
                    candidates.append((dir, mtime))
                }
            }
        }

        // Return the most recently modified
        guard let best = candidates.max(by: { $0.mtime < $1.mtime }) else { return nil }
        // Only use it if it's fresher than 5 minutes
        if best.mtime.timeIntervalSinceNow > -300 {
            return best.url
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
