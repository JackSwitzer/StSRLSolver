import Foundation

enum ArtifactDecoder {
    static let json: JSONDecoder = {
        let decoder = JSONDecoder()
        return decoder
    }()

    static func decode<T: Decodable>(_ type: T.Type, from url: URL) -> T? {
        guard let data = try? Data(contentsOf: url) else { return nil }
        return try? json.decode(type, from: data)
    }
}

enum ManifestLoader {
    static func load(from logsURL: URL) async -> TrainingRunArtifactManifest? {
        ArtifactDecoder.decode(
            TrainingRunArtifactManifest.self,
            from: logsURL.appending(path: "manifest.json")
        )
    }
}

enum FrontierReportLoader {
    static func load(from logsURL: URL) async -> FrontierReportArtifact? {
        ArtifactDecoder.decode(
            FrontierReportArtifact.self,
            from: logsURL.appending(path: "frontier_report.json")
        )
    }
}

enum BenchmarkReportLoader {
    static func loadAll(from logsURL: URL) async -> [LocatedBenchmarkReport] {
        let urls = candidateJSONFiles(namedPrefix: "benchmark_report", under: logsURL)
        return urls.compactMap { url in
            guard let report = ArtifactDecoder.decode(BenchmarkReportArtifact.self, from: url) else {
                return nil
            }
            return LocatedBenchmarkReport(url: url, report: report)
        }
        .sorted { $0.url.lastPathComponent < $1.url.lastPathComponent }
    }
}

enum ArtifactEpisodeLogLoader {
    static func loadAll(from logsURL: URL) async -> [LocatedEpisodeLog] {
        let urls = candidateJSONLFiles(namedPrefix: "episodes", under: logsURL)
        var results: [LocatedEpisodeLog] = []

        for url in urls {
            guard let content = try? String(contentsOf: url, encoding: .utf8) else { continue }
            for (lineIndex, rawLine) in content.split(separator: "\n").enumerated() {
                let line = String(rawLine)
                guard
                    let data = line.data(using: .utf8),
                    var decoded = try? ArtifactDecoder.json.decode(ArtifactEpisodeLog.self, from: data)
                else {
                    continue
                }

                // Filter out legacy episode lines by requiring explicit step payloads.
                guard !decoded.steps.isEmpty else { continue }
                decoded = ArtifactEpisodeLog(
                    manifest: decoded.manifest,
                    steps: decoded.steps,
                    lineNumber: lineIndex
                )
                results.append(LocatedEpisodeLog(url: url, episode: decoded))
            }
        }

        return results.sorted { lhs, rhs in
            if lhs.url == rhs.url {
                return lhs.episode.lineNumber < rhs.episode.lineNumber
            }
            return lhs.url.lastPathComponent < rhs.url.lastPathComponent
        }
    }
}

enum EventStreamLoader {
    static func load(from logsURL: URL) async -> [TrainingEventRecord] {
        let url = logsURL.appending(path: "events.jsonl")
        guard let content = try? String(contentsOf: url, encoding: .utf8) else { return [] }
        return content.split(separator: "\n").compactMap { rawLine in
            guard
                let data = String(rawLine).data(using: .utf8),
                let payload = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
            else {
                return nil
            }

            let timestamp = payload["timestamp"] as? String
            let eventType = payload["event_type"] as? String ?? "unknown"
            var extras: [String: JSONValue] = [:]
            for (key, value) in payload where key != "timestamp" && key != "event_type" {
                extras[key] = toJSONValue(value)
            }
            return TrainingEventRecord(timestamp: timestamp, eventType: eventType, payload: extras)
        }
    }
}

enum MetricStreamLoader {
    static func load(from logsURL: URL) async -> [TrainingMetricRecord] {
        let url = logsURL.appending(path: "metrics.jsonl")
        guard let content = try? String(contentsOf: url, encoding: .utf8) else { return [] }
        return content.split(separator: "\n").compactMap { rawLine in
            guard let data = String(rawLine).data(using: .utf8) else { return nil }
            return try? ArtifactDecoder.json.decode(TrainingMetricRecord.self, from: data)
        }
    }
}

private func candidateJSONFiles(namedPrefix prefix: String, under root: URL) -> [URL] {
    candidateFiles(
        under: root,
        predicate: { url in
            url.pathExtension == "json" && url.deletingPathExtension().lastPathComponent.hasPrefix(prefix)
        }
    )
}

private func candidateJSONLFiles(namedPrefix prefix: String, under root: URL) -> [URL] {
    candidateFiles(
        under: root,
        predicate: { url in
            url.pathExtension == "jsonl" && url.deletingPathExtension().lastPathComponent.hasPrefix(prefix)
        }
    )
}

private func candidateFiles(under root: URL, predicate: (URL) -> Bool) -> [URL] {
    guard let enumerator = FileManager.default.enumerator(
        at: root,
        includingPropertiesForKeys: [.isDirectoryKey],
        options: [.skipsHiddenFiles]
    ) else {
        return []
    }

    var files: [URL] = []
    for case let url as URL in enumerator {
        if predicate(url) {
            files.append(url)
        }
    }
    return files
}

private func toJSONValue(_ raw: Any) -> JSONValue {
    switch raw {
    case let value as String:
        return .string(value)
    case let value as NSNumber:
        if CFGetTypeID(value) == CFBooleanGetTypeID() {
            return .bool(value.boolValue)
        }
        return .number(value.doubleValue)
    case let value as [String: Any]:
        return .object(value.mapValues(toJSONValue))
    case let value as [Any]:
        return .array(value.map(toJSONValue))
    default:
        return .null
    }
}
