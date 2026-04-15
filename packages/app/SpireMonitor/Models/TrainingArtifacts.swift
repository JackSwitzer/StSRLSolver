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
