import SwiftUI

struct ActiveRunSummaryView: View {
    let manifest: TrainingRunArtifactManifest?
    let latestEvent: TrainingEventRecord?
    let latestMetric: TrainingMetricRecord?
    let frontierReport: FrontierReportArtifact?
    let benchmarkReport: BenchmarkReportArtifact?
    let seedValidationReport: SeedValidationReportArtifact?
    let checkpointComparison: SeedValidationComparisonArtifact?
    let seedValidationReportCount: Int
    let checkpointComparisonCount: Int

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            SectionHeader(title: "Artifact Run Summary")

            if let manifest {
                VStack(alignment: .leading, spacing: 10) {
                    HStack(alignment: .top) {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(manifest.runID)
                                .font(.stsTitle)
                                .foregroundStyle(Color.stsText)
                            Text(manifest.createdAt)
                                .font(.stsLabel)
                                .foregroundStyle(Color.stsTextMuted)
                        }
                        Spacer()
                        statusPill(
                            manifest.git.dirty ? "DIRTY" : "CLEAN",
                            color: manifest.git.dirty ? .stsYellow : .stsAccent
                        )
                    }

                    HStack(spacing: 12) {
                        summaryMetric("Branch", manifest.git.branch)
                        summaryMetric("Commit", String(manifest.git.commitSHA.prefix(10)))
                        summaryMetric("Config", String(manifest.config.configHash.prefix(10)))
                    }

                    if !manifest.tags.isEmpty {
                        flowTags(title: "Tags", values: manifest.tags)
                    }
                    if !manifest.notes.isEmpty {
                        flowTags(title: "Notes", values: manifest.notes)
                    }
                }
            } else {
                missingCard(
                    title: "No manifest.json yet",
                    subtitle: "Waiting for the rebuilt training logger to write an active run manifest."
                )
            }

            HStack(spacing: 12) {
                streamCard(
                    title: "Latest Event",
                    headline: latestEvent?.eventType ?? "No events",
                    subtitle: latestEvent?.timestamp ?? "events.jsonl not found",
                    detail: latestEvent?.payload.isEmpty == false
                        ? latestEvent?.payload
                            .sorted { $0.key < $1.key }
                            .prefix(2)
                            .map { "\($0.key)=\($0.value.displayString)" }
                            .joined(separator: " · ")
                        : nil
                )

                streamCard(
                    title: "Latest Metric",
                    headline: latestMetric.map { "\($0.name): \(Fmt.decimal($0.value, places: 3))" } ?? "No metrics",
                    subtitle: latestMetric.map { "step \($0.step) · \($0.config)" } ?? "metrics.jsonl not found",
                    detail: latestMetric?.timestamp
                )
            }

            HStack(spacing: 12) {
                smallArtifactStat(
                    title: "Frontier Lines",
                    value: "\(frontierReport?.frontier.count ?? 0)",
                    subtitle: frontierReport?.ranking.first ?? "No frontier"
                )
                smallArtifactStat(
                    title: "Benchmark Slices",
                    value: "\(benchmarkReport?.slices.count ?? 0)",
                    subtitle: benchmarkReport?.manifest?.benchmarkConfig ?? "No benchmark"
                )
            }

            HStack(spacing: 12) {
                smallArtifactStat(
                    title: "Seed Validation",
                    value: "\(seedValidationReportCount)",
                    subtitle: seedValidationReport?.displayName ?? "seed_validation_report.json"
                )
                smallArtifactStat(
                    title: "PUCT Comparisons",
                    value: "\(checkpointComparisonCount)",
                    subtitle: checkpointComparison?.subtitle ?? "checkpoint comparison stream"
                )
            }
        }
        .sectionCard()
    }

    private func summaryMetric(_ title: String, _ value: String) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)
            Text(value)
                .font(.stsBody)
                .foregroundStyle(Color.stsText)
                .textSelection(.enabled)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(10)
        .background(Color.stsBg)
        .clipShape(RoundedRectangle(cornerRadius: 6))
    }

    private func flowTags(title: String, values: [String]) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)
            FlowLayout(items: values)
        }
    }

    private func statusPill(_ text: String, color: Color) -> some View {
        Text(text)
            .font(.system(size: 11, weight: .bold, design: .monospaced))
            .foregroundStyle(color)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(color.opacity(0.15))
            .clipShape(RoundedRectangle(cornerRadius: 5))
    }

    private func smallArtifactStat(title: String, value: String, subtitle: String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)
            Text(value)
                .font(.stsTitle)
                .foregroundStyle(Color.stsText)
            Text(subtitle)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextDim)
                .lineLimit(1)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(10)
        .background(Color.stsBg)
        .clipShape(RoundedRectangle(cornerRadius: 6))
    }

    private func streamCard(title: String, headline: String, subtitle: String, detail: String?) -> some View {
        VStack(alignment: .leading, spacing: 5) {
            Text(title)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)
            Text(headline)
                .font(.stsBody)
                .foregroundStyle(Color.stsText)
            Text(subtitle)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextDim)
            if let detail, !detail.isEmpty {
                Text(detail)
                    .font(.system(size: 11, weight: .regular, design: .monospaced))
                    .foregroundStyle(Color.stsTextMuted)
                    .lineLimit(2)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(10)
        .background(Color.stsBg)
        .clipShape(RoundedRectangle(cornerRadius: 6))
    }

    private func missingCard(title: String, subtitle: String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .font(.stsBody)
                .foregroundStyle(Color.stsText)
            Text(subtitle)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextDim)
        }
        .padding(10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color.stsBg)
        .clipShape(RoundedRectangle(cornerRadius: 6))
    }
}

private struct FlowLayout: View {
    let items: [String]

    var body: some View {
        HStack(spacing: 6) {
            ForEach(items, id: \.self) { item in
                Text(item)
                    .font(.system(size: 11, weight: .medium, design: .monospaced))
                    .foregroundStyle(Color.stsAccent)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 3)
                    .background(Color.stsAccent.opacity(0.12))
                    .clipShape(RoundedRectangle(cornerRadius: 4))
            }
        }
    }
}
