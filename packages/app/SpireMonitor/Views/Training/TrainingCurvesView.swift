import SwiftUI
import Charts

/// A single data point for charting.
private struct ChartPoint: Identifiable {
    let id: Int  // use step as id within a series
    let step: Int
    let value: Double
    let series: String
}

/// Persistent training curves from metrics_history.jsonl.
/// Shows loss, floor progression, and entropy over the full training run.
struct TrainingCurvesView: View {
    @Environment(AppState.self) private var appState

    private var history: [MetricsSnapshot] {
        appState.store.metricsHistory
    }

    var body: some View {
        VStack(spacing: 16) {
            if history.count < 2 {
                placeholderView
            } else {
                lossCurve
                floorCurve
                entropyCurve
                evKlCurve
            }
        }
    }

    private var placeholderView: some View {
        VStack(spacing: 8) {
            Image(systemName: "chart.xyaxis.line")
                .font(.system(size: 24))
                .foregroundStyle(Color.stsTextMuted)
            Text("Waiting for metrics_history.jsonl...")
                .font(.stsBody)
                .foregroundStyle(Color.stsTextMuted)
            Text("Data appears after training writes periodic snapshots")
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextDim)
        }
        .frame(maxWidth: .infinity)
        .padding(20)
        .background(Color.stsCard)
        .clipShape(RoundedRectangle(cornerRadius: 8))
    }

    private var lossCurve: some View {
        let points = buildPoints([
            ("Total", history.compactMap { s in s.step.flatMap { step in s.totalLoss.map { (step, $0) } } }),
            ("Policy", history.compactMap { s in s.step.flatMap { step in s.policyLoss.map { (step, $0) } } }),
            ("Value", history.compactMap { s in s.step.flatMap { step in s.valueLoss.map { (step, $0) } } }),
        ])
        let colors: [String: Color] = ["Total": .stsRed, "Policy": .stsBlue, "Value": .stsGold]
        return CurveChartView(title: "Loss Over Time", points: points, colorMap: colors)
    }

    private var floorCurve: some View {
        let points = buildPoints([
            ("Avg Floor", history.compactMap { s in s.step.flatMap { step in s.avgFloor.map { (step, $0) } } }),
            ("Peak Floor", history.compactMap { s in s.step.flatMap { step in s.peakFloor.map { (step, Double($0)) } } }),
        ])
        let colors: [String: Color] = ["Avg Floor": .stsAccent, "Peak Floor": .stsBlue]
        return CurveChartView(title: "Floor Progression", points: points, colorMap: colors)
    }

    private var entropyCurve: some View {
        let points = buildPoints([
            ("Entropy", history.compactMap { s in s.step.flatMap { step in s.entropy.map { (step, $0) } } }),
        ])
        let colors: [String: Color] = ["Entropy": .stsOrange]
        return CurveChartView(title: "Entropy", points: points, colorMap: colors)
    }

    private var evKlCurve: some View {
        let points = buildPoints([
            ("EV", history.compactMap { s in s.step.flatMap { step in s.explainedVariance.map { (step, $0) } } }),
            ("KL", history.compactMap { s in s.step.flatMap { step in s.klDivergence.map { (step, $0) } } }),
        ])
        let colors: [String: Color] = ["EV": .stsAccent, "KL": .stsOrange]
        return CurveChartView(title: "Explained Variance & KL", points: points, colorMap: colors)
    }

    private func buildPoints(_ seriesData: [(String, [(Int, Double)])]) -> [ChartPoint] {
        var result: [ChartPoint] = []
        for (name, values) in seriesData {
            for (i, pair) in values.enumerated() {
                result.append(ChartPoint(id: i, step: pair.0, value: pair.1, series: name))
            }
        }
        return result
    }
}

/// Extracted chart view to help the Swift type checker.
private struct CurveChartView: View {
    let title: String
    let points: [ChartPoint]
    let colorMap: [String: Color]

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .font(.stsLabel)
                .foregroundStyle(Color.stsTextMuted)

            chartBody

            legendRow
        }
        .padding(12)
        .background(Color.stsCard)
        .clipShape(RoundedRectangle(cornerRadius: 8))
    }

    private var chartBody: some View {
        Chart(points) { pt in
            LineMark(
                x: .value("Step", pt.step),
                y: .value("Value", pt.value)
            )
            .foregroundStyle(colorMap[pt.series] ?? Color.stsText)
            .interpolationMethod(.catmullRom)
        }
        .frame(height: 140)
        .chartYAxis {
            AxisMarks(position: .leading) { _ in
                AxisValueLabel().font(.stsLabel).foregroundStyle(Color.stsTextDim)
                AxisGridLine().foregroundStyle(Color.stsBorderDim)
            }
        }
        .chartXAxis {
            AxisMarks { _ in
                AxisValueLabel().font(.stsLabel).foregroundStyle(Color.stsTextDim)
            }
        }
        .chartPlotStyle { plotArea in
            plotArea.background(Color.stsBg.opacity(0.5))
        }
    }

    private var legendRow: some View {
        HStack(spacing: 12) {
            ForEach(Array(colorMap.sorted(by: { $0.key < $1.key })), id: \.key) { name, color in
                HStack(spacing: 4) {
                    Circle().fill(color).frame(width: 6, height: 6)
                    Text(name).font(.stsLabel).foregroundStyle(Color.stsTextDim)
                }
            }
        }
        .padding(.top, 2)
    }
}
