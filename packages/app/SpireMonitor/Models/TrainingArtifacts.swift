import Foundation

enum JSONValue: Codable, Hashable {
    case string(String)
    case number(Double)
    case bool(Bool)
    case object([String: JSONValue])
    case array([JSONValue])
    case null

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if container.decodeNil() {
            self = .null
        } else if let value = try? container.decode(Bool.self) {
            self = .bool(value)
        } else if let value = try? container.decode(Double.self) {
            self = .number(value)
        } else if let value = try? container.decode(String.self) {
            self = .string(value)
        } else if let value = try? container.decode([String: JSONValue].self) {
            self = .object(value)
        } else if let value = try? container.decode([JSONValue].self) {
            self = .array(value)
        } else {
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Unsupported JSON value"
            )
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .string(let value):
            try container.encode(value)
        case .number(let value):
            try container.encode(value)
        case .bool(let value):
            try container.encode(value)
        case .object(let value):
            try container.encode(value)
        case .array(let value):
            try container.encode(value)
        case .null:
            try container.encodeNil()
        }
    }

    var stringValue: String? {
        if case let .string(value) = self {
            return value
        }
        return nil
    }

    var numberValue: Double? {
        if case let .number(value) = self {
            return value
        }
        return nil
    }

    var doubleValue: Double? {
        switch self {
        case .number(let value):
            return value
        case .string(let value):
            return Double(value)
        default:
            return nil
        }
    }

    var boolValue: Bool? {
        if case let .bool(value) = self {
            return value
        }
        return nil
    }

    var objectValue: [String: JSONValue]? {
        if case let .object(value) = self {
            return value
        }
        return nil
    }

    var arrayValue: [JSONValue]? {
        if case let .array(value) = self {
            return value
        }
        return nil
    }

    var displayString: String {
        switch self {
        case .string(let value):
            value
        case .number(let value):
            if value.rounded() == value {
                String(Int(value))
            } else {
                String(format: "%.3f", value)
            }
        case .bool(let value):
            value ? "true" : "false"
        case .object(let value):
            "{\(value.count)} fields"
        case .array(let value):
            "[\(value.count)]"
        case .null:
            "null"
        }
    }
}

struct ArtifactGitSnapshot: Codable {
    let commitSHA: String
    let branch: String
    let dirty: Bool

    enum CodingKeys: String, CodingKey {
        case commitSHA = "commit_sha"
        case branch, dirty
    }
}

struct ArtifactConfigSnapshot: Codable {
    let values: [String: JSONValue]
    let configHash: String

    enum CodingKeys: String, CodingKey {
        case values
        case configHash = "config_hash"
    }
}

struct TrainingRunArtifactManifest: Codable {
    let runID: String
    let createdAt: String
    let git: ArtifactGitSnapshot
    let config: ArtifactConfigSnapshot
    let tags: [String]
    let notes: [String]

    enum CodingKeys: String, CodingKey {
        case runID = "run_id"
        case createdAt = "created_at"
        case git, config, tags, notes
    }
}

struct RestrictionPolicyArtifact: Codable {
    let builtins: [String]
}

struct RuntimeContractManifest: Codable {
    let gitSHA: String
    let gitDirty: Bool
    let combatObservationSchemaVersion: Int
    let actionCandidateSchemaVersion: Int
    let gameplayExportSchemaVersion: Int
    let replayEventTraceSchemaVersion: Int
    let modelVersion: String
    let benchmarkConfig: String
    let seed: Int
    let restrictionPolicy: RestrictionPolicyArtifact
    let hardware: String
    let runtime: String

    enum CodingKeys: String, CodingKey {
        case gitSHA = "git_sha"
        case gitDirty = "git_dirty"
        case combatObservationSchemaVersion = "combat_observation_schema_version"
        case actionCandidateSchemaVersion = "action_candidate_schema_version"
        case gameplayExportSchemaVersion = "gameplay_export_schema_version"
        case replayEventTraceSchemaVersion = "replay_event_trace_schema_version"
        case modelVersion = "model_version"
        case benchmarkConfig = "benchmark_config"
        case seed
        case restrictionPolicy = "restriction_policy"
        case hardware, runtime
    }
}

struct FrontierPointArtifact: Codable, Identifiable {
    let label: String
    let winRate: Double
    let avgFloor: Double
    let throughputGPM: Double

    enum CodingKeys: String, CodingKey {
        case label
        case winRate = "win_rate"
        case avgFloor = "avg_floor"
        case throughputGPM = "throughput_gpm"
    }

    var id: String { label }
}

struct FrontierWeightsArtifact: Codable {
    let winRate: Double
    let avgFloor: Double
    let throughputGPM: Double

    enum CodingKeys: String, CodingKey {
        case winRate = "win_rate"
        case avgFloor = "avg_floor"
        case throughputGPM = "throughput_gpm"
    }
}

struct FrontierReportArtifact: Codable {
    let points: [FrontierPointArtifact]
    let frontier: [String]
    let ranking: [String]
    let bestByMetric: [String: String]
    let weights: FrontierWeightsArtifact

    enum CodingKeys: String, CodingKey {
        case points, frontier, ranking, weights
        case bestByMetric = "best_by_metric"
    }
}

struct BenchmarkSliceArtifact: Codable, Identifiable {
    let sliceName: String
    let cases: Int
    let solveRate: Double
    let expectedHPLoss: Double
    let expectedTurns: Double
    let oracleTopKAgreement: Double
    let p95ElapsedMS: Double
    let p95RSSGB: Double

    enum CodingKeys: String, CodingKey {
        case sliceName = "slice_name"
        case cases
        case solveRate = "solve_rate"
        case expectedHPLoss = "expected_hp_loss"
        case expectedTurns = "expected_turns"
        case oracleTopKAgreement = "oracle_top_k_agreement"
        case p95ElapsedMS = "p95_elapsed_ms"
        case p95RSSGB = "p95_rss_gb"
    }

    var id: String { sliceName }
}

struct BenchmarkReportArtifact: Codable {
    let manifest: RuntimeContractManifest?
    let slices: [BenchmarkSliceArtifact]
}

struct CombatOutcomeArtifact: Codable {
    let solveProbability: Double
    let expectedHPLoss: Double
    let expectedTurns: Double
    let potionCost: Double
    let setupValueDelta: Double
    let persistentScalingDelta: Double

    enum CodingKeys: String, CodingKey {
        case solveProbability = "solve_probability"
        case expectedHPLoss = "expected_hp_loss"
        case expectedTurns = "expected_turns"
        case potionCost = "potion_cost"
        case setupValueDelta = "setup_value_delta"
        case persistentScalingDelta = "persistent_scaling_delta"
    }
}

struct CombatFrontierLineArtifact: Codable, Identifiable {
    let lineIndex: Int
    let actionPrefix: [Int]
    let visits: Int
    let expandedNodes: Int
    let elapsedMS: Int
    let outcome: CombatOutcomeArtifact

    enum CodingKeys: String, CodingKey {
        case lineIndex = "line_index"
        case actionPrefix = "action_prefix"
        case visits
        case expandedNodes = "expanded_nodes"
        case elapsedMS = "elapsed_ms"
        case outcome
    }

    var id: Int { lineIndex }
}

struct CombatFrontierSummaryArtifact: Codable {
    let capacity: Int
    let lines: [CombatFrontierLineArtifact]
}

struct ArtifactEpisodeStep: Codable, Identifiable {
    let stepIndex: Int
    let actionID: Int
    let rewardDelta: Double
    let done: Bool
    let searchFrontier: CombatFrontierSummaryArtifact?
    let value: CombatOutcomeArtifact?

    enum CodingKeys: String, CodingKey {
        case stepIndex = "step_index"
        case actionID = "action_id"
        case rewardDelta = "reward_delta"
        case done
        case searchFrontier = "search_frontier"
        case value
    }

    var id: Int { stepIndex }
}

struct ArtifactEpisodeLog: Codable, Identifiable {
    let manifest: RuntimeContractManifest?
    let steps: [ArtifactEpisodeStep]
    let lineNumber: Int

    init(manifest: RuntimeContractManifest?, steps: [ArtifactEpisodeStep], lineNumber: Int) {
        self.manifest = manifest
        self.steps = steps
        self.lineNumber = lineNumber
    }

    enum CodingKeys: String, CodingKey {
        case manifest, steps
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        manifest = try container.decodeIfPresent(RuntimeContractManifest.self, forKey: .manifest)
        steps = try container.decodeIfPresent([ArtifactEpisodeStep].self, forKey: .steps) ?? []
        lineNumber = 0
    }

    var id: String {
        if let manifest {
            return "\(manifest.seed)-\(manifest.benchmarkConfig)-\(lineNumber)"
        }
        return "episode-\(lineNumber)"
    }

    var displayName: String {
        if let manifest {
            return "\(manifest.benchmarkConfig) · seed \(manifest.seed)"
        }
        return "Episode \(lineNumber + 1)"
    }

    var frontierSteps: [ArtifactEpisodeStep] {
        steps.filter { $0.searchFrontier?.lines.isEmpty == false }
    }
}

struct TrainingEventRecord: Identifiable, Hashable {
    let timestamp: String?
    let eventType: String
    let payload: [String: JSONValue]

    var id: String {
        "\(timestamp ?? "unknown")-\(eventType)"
    }
}

struct TrainingMetricRecord: Codable, Identifiable {
    let timestamp: String?
    let name: String
    let value: Double
    let step: Int
    let config: String

    var id: String {
        "\(name)-\(step)-\(config)"
    }
}

struct LocatedBenchmarkReport: Identifiable {
    let url: URL
    let report: BenchmarkReportArtifact

    var id: String { url.path() }
}

struct LocatedEpisodeLog: Identifiable {
    let url: URL
    let episode: ArtifactEpisodeLog

    var id: String { "\(url.path())#\(episode.id)" }
}

private extension Dictionary where Key == String, Value == JSONValue {
    func value(forKeys keys: [String]) -> JSONValue? {
        for key in keys {
            if let value = self[key] {
                return value
            }
        }
        return nil
    }

    func stringValue(forKeys keys: [String]) -> String? {
        value(forKeys: keys)?.stringValue
    }

    func intValue(forKeys keys: [String]) -> Int? {
        value(forKeys: keys)?.intValue
    }

    func doubleValue(forKeys keys: [String]) -> Double? {
        value(forKeys: keys)?.doubleValue
    }

    func objectValue(forKeys keys: [String]) -> [String: JSONValue]? {
        value(forKeys: keys)?.objectValue
    }

    func arrayValue(forKeys keys: [String]) -> [JSONValue]? {
        value(forKeys: keys)?.arrayValue
    }

    func stringArray(forKeys keys: [String]) -> [String] {
        arrayValue(forKeys: keys)?.compactMap(\.stringValue) ?? []
    }
}

private extension JSONValue {
    var intValue: Int? {
        switch self {
        case .number(let value) where value.rounded() == value:
            return Int(value)
        case .string(let value):
            return Int(value)
        default:
            return nil
        }
    }
}

private extension Array where Element == Double {
    var medianValue: Double? {
        guard !isEmpty else { return nil }
        let values = sorted()
        let mid = values.count / 2
        if values.count.isMultiple(of: 2) {
            return (values[mid - 1] + values[mid]) / 2
        }
        return values[mid]
    }

    func percentile(_ percentile: Double) -> Double? {
        guard !isEmpty else { return nil }
        let clamped = Swift.max(0, Swift.min(1, percentile))
        let values = sorted()
        let index = Int(round((Double(values.count) - 1) * clamped))
        return values[Swift.min(Swift.max(index, 0), values.count - 1)]
    }

    var standardDeviation: Double? {
        guard count > 1 else { return 0 }
        let mean = reduce(0, +) / Double(count)
        let variance = reduce(0) { $0 + pow($1 - mean, 2) } / Double(count - 1)
        return sqrt(variance)
    }
}

struct PUCTStopReasonArtifact: Identifiable {
    let reason: String
    let count: Int?
    let examples: [String]
    let note: String?

    var id: String { reason }

    init(reason: String, count: Int?, examples: [String] = [], note: String? = nil) {
        self.reason = reason
        self.count = count
        self.examples = examples
        self.note = note
    }

    init?(payload: [String: JSONValue]) {
        let reason = payload.stringValue(forKeys: ["reason", "name", "label", "stop_reason", "stopReason"]) ?? ""
        guard !reason.isEmpty else { return nil }
        self.reason = reason
        self.count = payload.intValue(forKeys: ["count", "total", "occurrences", "n", "value"])
        self.examples = payload.stringArray(forKeys: ["examples", "sample_seeds", "seed_examples", "seeds"])
        self.note = payload.stringValue(forKeys: ["note", "description", "details"])
    }

    var title: String {
        if let count {
            return "\(reason) · \(count)"
        }
        return reason
    }
}

struct PUCTStabilityArtifact {
    let label: String
    let mean: Double?
    let minimum: Double?
    let maximum: Double?
    let stdDev: Double?
    let median: Double?
    let p95: Double?
    let sampleCount: Int?

    init(label: String, payload: [String: JSONValue]) {
        self.label = label
        self.mean = payload.doubleValue(forKeys: ["mean", "avg", "average", "value"])
        self.minimum = payload.doubleValue(forKeys: ["min", "minimum", "low"])
        self.maximum = payload.doubleValue(forKeys: ["max", "maximum", "high"])
        self.stdDev = payload.doubleValue(forKeys: ["std_dev", "stdDev", "stdev", "std", "sigma"])
        self.median = payload.doubleValue(forKeys: ["median", "p50"])
        self.p95 = payload.doubleValue(forKeys: ["p95", "p_95", "percentile95"])
        self.sampleCount = payload.intValue(forKeys: ["sample_count", "samples", "count", "n"])
    }

    init?(label: String, samples: [Double]) {
        guard !samples.isEmpty else { return nil }
        self.label = label
        self.mean = samples.reduce(0, +) / Double(samples.count)
        self.minimum = samples.min()
        self.maximum = samples.max()
        self.stdDev = samples.standardDeviation
        self.median = samples.medianValue
        self.p95 = samples.percentile(0.95)
        self.sampleCount = samples.count
    }

    var summaryText: String {
        var pieces: [String] = []
        if let mean {
            pieces.append("mean \(Fmt.decimal(mean, places: 2))")
        }
        if let stdDev {
            pieces.append("sd \(Fmt.decimal(stdDev, places: 2))")
        }
        if let minimum, let maximum {
            pieces.append("range \(Fmt.decimal(minimum, places: 0))-\(Fmt.decimal(maximum, places: 0))")
        }
        if let sampleCount {
            pieces.append("\(sampleCount) samples")
        }
        return pieces.joined(separator: " · ")
    }
}

struct SeedValidationSeedArtifact: Identifiable {
    let seed: String
    let checkpoint: String?
    let stopReason: String?
    let rootVisits: Int?
    let frontierWidth: Int?
    let note: String?

    var id: String { "\(checkpoint ?? "current")-\(seed)" }

    init?(payload: [String: JSONValue]) {
        let seed = payload.stringValue(forKeys: ["seed", "seed_id", "seedString", "seed_string", "label"]) ?? ""
        guard !seed.isEmpty else { return nil }
        self.seed = seed
        self.checkpoint = payload.stringValue(forKeys: ["checkpoint", "checkpoint_name", "checkpoint_id", "from_checkpoint"])
        self.stopReason = payload.stringValue(forKeys: ["stop_reason", "stopReason", "reason", "termination_reason"])
        self.rootVisits = payload.intValue(forKeys: ["root_visits", "rootVisits", "visits", "visit_count"])
        self.frontierWidth = payload.intValue(forKeys: ["frontier_width", "frontierWidth", "frontier_size", "frontier_count", "frontier"])
        self.note = payload.stringValue(forKeys: ["note", "description", "details"])
    }

    var subtitle: String {
        var pieces: [String] = []
        if let checkpoint { pieces.append(checkpoint) }
        if let stopReason { pieces.append("stop \(stopReason)") }
        if let rootVisits { pieces.append("root \(rootVisits)") }
        if let frontierWidth { pieces.append("frontier \(frontierWidth)") }
        return pieces.isEmpty ? "seed validation row" : pieces.joined(separator: " · ")
    }
}

struct SeedValidationComparisonArtifact: Codable, Identifiable {
    let fromCheckpoint: String
    let toCheckpoint: String
    let seed: String?
    let seedCount: Int?
    let stopReason: String?
    let rootVisitDelta: Double?
    let frontierDelta: Double?
    let winRateDelta: Double?
    let note: String?

    var id: String { "\(fromCheckpoint)->\(toCheckpoint)-\(seed ?? "summary")" }

    init(from decoder: Decoder) throws {
        let value = try JSONValue(from: decoder)
        guard case let .object(object) = value, let parsed = SeedValidationComparisonArtifact(payload: object) else {
            throw DecodingError.dataCorrupted(
                .init(codingPath: [], debugDescription: "invalid seed validation comparison payload")
            )
        }
        self = parsed
    }

    init?(payload: [String: JSONValue]) {
        let fromCheckpoint = payload.stringValue(forKeys: ["from_checkpoint", "baseline_checkpoint", "checkpoint_a", "left_checkpoint"]) ?? ""
        let toCheckpoint = payload.stringValue(forKeys: ["to_checkpoint", "candidate_checkpoint", "checkpoint_b", "right_checkpoint"]) ?? ""
        guard !fromCheckpoint.isEmpty || !toCheckpoint.isEmpty else { return nil }
        self.fromCheckpoint = fromCheckpoint.isEmpty ? "baseline" : fromCheckpoint
        self.toCheckpoint = toCheckpoint.isEmpty ? "candidate" : toCheckpoint
        self.seed = payload.stringValue(forKeys: ["seed", "seed_id", "seed_string"])
        self.seedCount = payload.intValue(forKeys: ["seed_count", "seeds", "sample_count", "samples"])
        self.stopReason = payload.stringValue(forKeys: ["stop_reason", "reason", "termination_reason"])
        self.rootVisitDelta = payload.doubleValue(forKeys: ["root_visit_delta", "visit_delta", "root_delta"])
        self.frontierDelta = payload.doubleValue(forKeys: ["frontier_delta", "frontier_width_delta", "frontier_change"])
        self.winRateDelta = payload.doubleValue(forKeys: ["win_rate_delta", "solve_rate_delta", "rate_delta"])
        self.note = payload.stringValue(forKeys: ["note", "description", "details"])
    }

    var subtitle: String {
        var pieces: [String] = ["\(fromCheckpoint) → \(toCheckpoint)"]
        if let seed { pieces.append(seed) }
        if let seedCount { pieces.append("\(seedCount) seeds") }
        return pieces.joined(separator: " · ")
    }
}

struct SeedValidationReportArtifact: Decodable {
    let payload: [String: JSONValue]

    init(from decoder: Decoder) throws {
        let value = try JSONValue(from: decoder)
        guard case let .object(object) = value else {
            throw DecodingError.dataCorrupted(
                .init(codingPath: [], debugDescription: "seed validation report must be a JSON object")
            )
        }
        payload = object
    }

    var suiteName: String? {
        payload.stringValue(forKeys: ["suite_name", "suiteName", "name", "validation_suite", "suite"])
    }

    var generatedAt: String? {
        payload.stringValue(forKeys: ["generated_at", "generatedAt", "timestamp", "created_at"])
    }

    var checkpoint: String? {
        payload.stringValue(forKeys: ["checkpoint", "checkpoint_name", "checkpoint_id", "current_checkpoint"])
    }

    var benchmarkConfig: String? {
        payload.stringValue(forKeys: ["benchmark_config", "benchmarkConfig", "config", "profile"])
    }

    var seedCount: Int? {
        payload.intValue(forKeys: ["seed_count", "seeds_total", "total_seeds", "count"])
    }

    var validatedSeedCount: Int? {
        payload.intValue(forKeys: ["validated_seeds", "validated_count", "samples", "cases"])
    }

    var passedSeedCount: Int? {
        payload.intValue(forKeys: ["passed_seeds", "pass_count", "passed"])
    }

    var failedSeedCount: Int? {
        payload.intValue(forKeys: ["failed_seeds", "fail_count", "failed"])
    }

    var seedRows: [SeedValidationSeedArtifact] {
        guard let values = payload.arrayValue(forKeys: ["seeds", "seed_rows", "cases", "results"]) else { return [] }
        return values.compactMap { value in
            guard case let .object(object) = value else { return nil }
            return SeedValidationSeedArtifact(payload: object)
        }
    }

    var checkpointComparisons: [SeedValidationComparisonArtifact] {
        guard let values = payload.arrayValue(forKeys: ["checkpoint_comparisons", "comparisons", "checkpointComparison", "checkpoint_pairs"]) else { return [] }
        return values.compactMap { value in
            guard case let .object(object) = value else { return nil }
            return SeedValidationComparisonArtifact(payload: object)
        }
    }

    var stopReasons: [PUCTStopReasonArtifact] {
        if let reasons = payload.objectValue(forKeys: ["stop_reason_counts", "stopReasonsCounts", "stop_reason_counts_by_reason"]) {
            return reasons
                .compactMap { key, value in
                    PUCTStopReasonArtifact(
                        reason: key,
                        count: value.intValue,
                        examples: value.arrayValue?.compactMap(\.stringValue) ?? [],
                        note: nil
                    )
                }
                .sorted { lhs, rhs in
                    (lhs.count ?? 0) > (rhs.count ?? 0)
                }
        }

        if !seedRows.isEmpty {
            let counts = Dictionary(grouping: seedRows.compactMap(\.stopReason), by: { $0 })
                .mapValues(\.count)
            return counts
                .map { PUCTStopReasonArtifact(reason: $0.key, count: $0.value) }
                .sorted { lhs, rhs in
                    (lhs.count ?? 0) > (rhs.count ?? 0)
                }
        }

        return []
    }

    var rootVisitStability: PUCTStabilityArtifact? {
        if let payload = payload.objectValue(forKeys: ["root_visit_stability", "rootVisits", "root_visits", "root_visit_stats"]) {
            return PUCTStabilityArtifact(label: "Root Visits", payload: payload)
        }

        let samples = seedRows.compactMap(\.rootVisits).map(Double.init)
        return PUCTStabilityArtifact(label: "Root Visits", samples: samples)
    }

    var frontierStability: PUCTStabilityArtifact? {
        if let payload = payload.objectValue(forKeys: ["frontier_stability", "frontierWidth", "frontier_width", "frontier_stats"]) {
            return PUCTStabilityArtifact(label: "Frontier Width", payload: payload)
        }

        let samples = seedRows.compactMap(\.frontierWidth).map(Double.init)
        return PUCTStabilityArtifact(label: "Frontier Width", samples: samples)
    }

    var displayName: String {
        suiteName ?? benchmarkConfig ?? "Seed Validation"
    }
}

struct LocatedSeedValidationReport: Identifiable {
    let url: URL
    let report: SeedValidationReportArtifact

    var id: String { url.path() }
}

struct LocatedSeedValidationComparison: Identifiable {
    let url: URL
    let report: SeedValidationComparisonArtifact

    var id: String { url.path() }
}
