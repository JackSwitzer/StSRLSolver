import SwiftUI
import Charts

struct BenchmarkSliceDashboardView: View {
    let reports: [LocatedBenchmarkReport]
    let frontier: FrontierReportArtifact?

    @State private var selectedReportID: String?

    private var selectedReport: LocatedBenchmarkReport? {
        if let selectedReportID {
            return reports.first(where: { $0.id == selectedReportID })
        }
        return reports.first
    }

    private var slices: [BenchmarkSliceArtifact] {
        selectedReport?.report.slices ?? []
    }

    private var avgSolveRate: Double {
        guard !slices.isEmpty else { return 0 }
        return slices.reduce(0) { $0 + $1.solveRate } / Double(slices.count)
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                SectionHeader(title: "Benchmark Slice Dashboard")

                if reports.isEmpty {
                    emptyState
                } else {
                    reportPicker
                    summaryCards

                    if !slices.isEmpty {
                        solveRateChart
                        latencyChart
                        sliceTable
                    }

                    if let frontier, !frontier.ranking.isEmpty {
                        frontierRankingCard(frontier)
                    }
                }
            }
            .padding(16)
        }
        .background(Color.stsBg)
        .onAppear {
            selectedReportID = selectedReport?.id
        }
        .onChange(of: reports.map(\.id)) {
            if selectedReport == nil {
                selectedReportID = reports.first?.id
            }
        }
    }

    private var emptyState: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("No benchmark_report.json found")
                .font(.stsBody)
                .foregroundStyle(Color.stsText)
            Text("The artifact-driven dashboard is ready; it will populate once benchmark reports are written into the active run directory.")
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextDim)
        }
        .sectionCard()
    }

    private var reportPicker: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Report")
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)
            Picker("Benchmark Report", selection: $selectedReportID) {
                ForEach(reports) { report in
                    Text(report.url.lastPathComponent)
                        .tag(Optional(report.id))
                }
            }
            .pickerStyle(.segmented)
        }
        .sectionCard()
    }

    private var summaryCards: some View {
        HStack(spacing: 12) {
            dashboardMetric("Reports", value: "\(reports.count)", subtitle: selectedReport?.report.manifest?.benchmarkConfig ?? "active")
            dashboardMetric("Slices", value: "\(slices.count)", subtitle: "\(Fmt.pct(avgSolveRate)) avg solve")
            dashboardMetric("Best Slice", value: bestSliceName, subtitle: bestSliceSubtitle)
        }
        .sectionCard()
    }

    private var bestSliceName: String {
        slices.max(by: { $0.solveRate < $1.solveRate })?.sliceName ?? "n/a"
    }

    private var bestSliceSubtitle: String {
        guard let best = slices.max(by: { $0.solveRate < $1.solveRate }) else { return "no slices" }
        return "solve \(Fmt.pct(best.solveRate))"
    }

    private var solveRateChart: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Solve Rate vs Oracle Agreement")
                .font(.stsBody)
                .foregroundStyle(Color.stsText)

            Chart(slices) { slice in
                BarMark(
                    x: .value("Slice", slice.sliceName),
                    y: .value("Solve Rate", slice.solveRate)
                )
                .foregroundStyle(Color.stsAccent.gradient)

                PointMark(
                    x: .value("Slice", slice.sliceName),
                    y: .value("Oracle", slice.oracleTopKAgreement)
                )
                .foregroundStyle(Color.stsYellow)
                .symbolSize(60)
            }
            .chartYScale(domain: 0...1)
            .frame(height: 220)
        }
        .sectionCard()
    }

    private var latencyChart: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Latency / RSS by Slice")
                .font(.stsBody)
                .foregroundStyle(Color.stsText)

            Chart(slices) { slice in
                BarMark(
                    x: .value("Slice", slice.sliceName),
                    y: .value("p95 elapsed ms", slice.p95ElapsedMS)
                )
                .foregroundStyle(Color.stsBlue.gradient)

                LineMark(
                    x: .value("Slice", slice.sliceName),
                    y: .value("p95 rss gb", slice.p95RSSGB)
                )
                .foregroundStyle(Color.stsRed)
                .lineStyle(.init(lineWidth: 2))
                .symbol(Circle())
            }
            .frame(height: 220)
        }
        .sectionCard()
    }

    private var sliceTable: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Slices")
                .font(.stsBody)
                .foregroundStyle(Color.stsText)

            ForEach(slices) { slice in
                VStack(alignment: .leading, spacing: 6) {
                    HStack {
                        Text(slice.sliceName)
                            .font(.stsBody)
                            .foregroundStyle(Color.stsText)
                        Spacer()
                        Text("\(slice.cases) cases")
                            .font(.stsLabel)
                            .foregroundStyle(Color.stsTextDim)
                    }

                    HStack(spacing: 12) {
                        miniMetric("Solve", Fmt.pct(slice.solveRate))
                        miniMetric("HP Loss", Fmt.decimal(slice.expectedHPLoss))
                        miniMetric("Turns", Fmt.decimal(slice.expectedTurns))
                        miniMetric("Oracle", Fmt.pct(slice.oracleTopKAgreement))
                        miniMetric("p95 ms", Fmt.decimal(slice.p95ElapsedMS))
                        miniMetric("RSS", Fmt.decimal(slice.p95RSSGB))
                    }
                }
                .padding(10)
                .background(Color.stsBg)
                .clipShape(RoundedRectangle(cornerRadius: 6))
            }
        }
        .sectionCard()
    }

    private func frontierRankingCard(_ frontier: FrontierReportArtifact) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Frontier Ranking")
                .font(.stsBody)
                .foregroundStyle(Color.stsText)

            ForEach(Array(frontier.ranking.prefix(5).enumerated()), id: \.offset) { index, label in
                HStack {
                    Text("#\(index + 1)")
                        .font(.system(size: 11, weight: .bold, design: .monospaced))
                        .foregroundStyle(Color.stsAccent)
                    Text(label)
                        .font(.stsBody)
                        .foregroundStyle(Color.stsText)
                    Spacer()
                    if frontier.frontier.contains(label) {
                        Text("FRONTIER")
                            .font(.system(size: 10, weight: .bold, design: .monospaced))
                            .foregroundStyle(Color.stsYellow)
                    }
                }
            }
        }
        .sectionCard()
    }

    private func dashboardMetric(_ title: String, value: String, subtitle: String) -> some View {
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

    private func miniMetric(_ title: String, _ value: String) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title)
                .font(.system(size: 10, weight: .medium))
                .foregroundStyle(Color.stsTextMuted)
            Text(value)
                .font(.system(size: 12, weight: .semibold, design: .monospaced))
                .foregroundStyle(Color.stsText)
        }
    }
}
