import SwiftUI

struct SeedValidationReportView: View {
    let seedValidationReports: [LocatedSeedValidationReport]
    let checkpointComparisons: [LocatedSeedValidationComparison]

    private var selectedReport: LocatedSeedValidationReport? {
        seedValidationReports.first
    }

    private var comparisonRows: [SeedValidationComparisonArtifact] {
        var rows: [SeedValidationComparisonArtifact] = selectedReport?.report.checkpointComparisons ?? []
        rows.append(contentsOf: checkpointComparisons.map(\.report))
        return rows
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            SectionHeader(title: "Seed Validation / PUCT")

            if let report = selectedReport {
                reportOverview(report)
                if !report.report.stopReasons.isEmpty {
                    stopReasonStrip(report.report.stopReasons)
                }
                HStack(spacing: 12) {
                    stabilityCard(report.report.rootVisitStability, fallbackTitle: "Root Visits")
                    stabilityCard(report.report.frontierStability, fallbackTitle: "Frontier Width")
                }
                if !report.report.seedRows.isEmpty {
                    seedRowsCard(report.report.seedRows)
                }
            } else if comparisonRows.isEmpty {
                emptyState
            }

            if !comparisonRows.isEmpty {
                comparisonCard(comparisonRows)
            }
        }
        .sectionCard()
    }

    private func reportOverview(_ located: LocatedSeedValidationReport) -> some View {
        let report = located.report
        return HStack(spacing: 12) {
            metricCard(
                title: "Report",
                value: report.displayName,
                subtitle: located.url.lastPathComponent
            )
            metricCard(
                title: "Seeds",
                value: "\(report.seedCount ?? report.validatedSeedCount ?? report.seedRows.count)",
                subtitle: report.checkpoint ?? report.generatedAt ?? "seed_validation_report.json"
            )
            metricCard(
                title: "Pass / Fail",
                value: "\(report.passedSeedCount ?? 0) / \(report.failedSeedCount ?? 0)",
                subtitle: report.benchmarkConfig ?? "PUCT summary"
            )
        }
    }

    private func stopReasonStrip(_ reasons: [PUCTStopReasonArtifact]) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("PUCT Stop Reasons")
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)

            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 8) {
                    ForEach(reasons.prefix(12)) { reason in
                        VStack(alignment: .leading, spacing: 2) {
                            Text(reason.reason)
                                .font(.system(size: 11, weight: .semibold, design: .monospaced))
                                .foregroundStyle(Color.stsAccent)
                            Text(reason.count.map(String.init) ?? "n/a")
                                .font(.system(size: 10, weight: .regular, design: .monospaced))
                                .foregroundStyle(Color.stsTextDim)
                        }
                        .padding(.horizontal, 8)
                        .padding(.vertical, 6)
                        .background(Color.stsAccent.opacity(0.10))
                        .clipShape(RoundedRectangle(cornerRadius: 6))
                    }
                }
            }
        }
    }

    private func stabilityCard(_ stability: PUCTStabilityArtifact?, fallbackTitle: String) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(stability?.label ?? fallbackTitle)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)
            if let stability {
                Text(stability.summaryText)
                    .font(.stsBody)
                    .foregroundStyle(Color.stsText)
                    .lineLimit(2)
            } else {
                Text("waiting for stability metrics")
                    .font(.stsBody)
                    .foregroundStyle(Color.stsTextDim)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(10)
        .background(Color.stsBg)
        .clipShape(RoundedRectangle(cornerRadius: 6))
    }

    private func seedRowsCard(_ rows: [SeedValidationSeedArtifact]) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Validation Seeds")
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)

            ForEach(rows.prefix(6)) { row in
                HStack(alignment: .top, spacing: 10) {
                    VStack(alignment: .leading, spacing: 3) {
                        Text(row.seed)
                            .font(.stsBody)
                            .foregroundStyle(Color.stsText)
                        Text(row.subtitle)
                            .font(.stsLabel)
                            .foregroundStyle(Color.stsTextDim)
                            .lineLimit(2)
                    }
                    Spacer()
                }
                .padding(8)
                .background(Color.stsBg)
                .clipShape(RoundedRectangle(cornerRadius: 6))
            }
        }
    }

    private func comparisonCard(_ rows: [SeedValidationComparisonArtifact]) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Checkpoint Comparisons")
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)

            ForEach(rows.prefix(6)) { row in
                VStack(alignment: .leading, spacing: 4) {
                    HStack {
                        Text(row.fromCheckpoint)
                            .font(.stsBody)
                            .foregroundStyle(Color.stsText)
                        Text("→")
                            .font(.stsBody)
                            .foregroundStyle(Color.stsTextDim)
                        Text(row.toCheckpoint)
                            .font(.stsBody)
                            .foregroundStyle(Color.stsText)
                        Spacer()
                        if let seed = row.seed {
                            Text(seed)
                                .font(.system(size: 11, weight: .regular, design: .monospaced))
                                .foregroundStyle(Color.stsTextMuted)
                                .lineLimit(1)
                        }
                    }

                    HStack(spacing: 10) {
                        chip("seeds", row.seedCount.map(String.init) ?? "n/a")
                        if let delta = row.winRateDelta {
                            chip("win Δ", Fmt.decimal(delta, places: 3))
                        }
                        if let delta = row.rootVisitDelta {
                            chip("root Δ", Fmt.decimal(delta, places: 2))
                        }
                        if let delta = row.frontierDelta {
                            chip("frontier Δ", Fmt.decimal(delta, places: 2))
                        }
                        if let stopReason = row.stopReason {
                            chip("stop", stopReason)
                        }
                    }
                }
                .padding(8)
                .background(Color.stsBg)
                .clipShape(RoundedRectangle(cornerRadius: 6))
            }
        }
    }

    private func metricCard(title: String, value: String, subtitle: String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)
            Text(value)
                .font(.stsBody)
                .foregroundStyle(Color.stsText)
                .lineLimit(2)
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

    private func chip(_ title: String, _ value: String) -> some View {
        HStack(spacing: 4) {
            Text(title)
                .font(.system(size: 10, weight: .medium))
                .foregroundStyle(Color.stsTextMuted)
            Text(value)
                .font(.system(size: 11, weight: .semibold, design: .monospaced))
                .foregroundStyle(Color.stsAccent)
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
        .background(Color.stsAccent.opacity(0.12))
        .clipShape(RoundedRectangle(cornerRadius: 6))
    }

    private var emptyState: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("No seed_validation_report.json yet")
                .font(.stsBody)
                .foregroundStyle(Color.stsText)
            Text("The monitor will populate once the training run writes validation and PUCT summary artifacts.")
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextDim)
        }
        .padding(10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color.stsBg)
        .clipShape(RoundedRectangle(cornerRadius: 6))
    }
}
